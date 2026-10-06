use std::collections::{HashMap, VecDeque};

use crate::domain::{ConnectionId, Filter, ReplayError, StoredEvent};

pub struct Broker {
    capacity_per_subject: usize,
    next_sequence: HashMap<String, u64>,
    history: HashMap<String, VecDeque<StoredEvent>>,
    subscriptions: HashMap<ConnectionId, Filter>,
}

impl Broker {
    pub fn new(capacity_per_subject: usize) -> Self {
        assert!(
            capacity_per_subject > 0,
            "broker capacity per subject must be greater than zero"
        );
        Self {
            capacity_per_subject,
            next_sequence: HashMap::new(),
            history: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, connection: ConnectionId, filter: Filter) {
        self.subscriptions.insert(connection, filter);
    }

    pub fn unsubscribe(&mut self, connection: &ConnectionId) {
        self.subscriptions.remove(connection);
    }

    pub fn reserve_sequence(&mut self, subject: &str) -> u64 {
        let next = self.next_sequence.entry(subject.to_owned()).or_insert(1);
        let sequence = *next;
        *next += 1;
        sequence
    }

    pub fn commit(&mut self, subject: &str, sequence: u64, payload: Vec<u8>) -> Vec<ConnectionId> {
        let history = self.history.entry(subject.to_owned()).or_default();
        history.push_back(StoredEvent {
            sequence,
            subject: subject.to_owned(),
            payload,
        });
        while history.len() > self.capacity_per_subject {
            history.pop_front();
        }

        self.subscriptions
            .iter()
            .filter(|(_, filter)| filter.matches(subject))
            .map(|(connection, _)| connection.clone())
            .collect()
    }

    pub fn replay(
        &self,
        subject: &str,
        last_sequence: u64,
    ) -> Result<Vec<StoredEvent>, ReplayError> {
        let history = match self.history.get(subject) {
            Some(history) => history,
            None => return Ok(Vec::new()),
        };
        if history.is_empty() {
            return Ok(Vec::new());
        }

        let oldest = history.front().expect("non-empty").sequence;
        if last_sequence.saturating_add(1) < oldest {
            return Err(ReplayError::ResyncRequired {
                subject: subject.to_owned(),
                oldest,
            });
        }

        Ok(history
            .iter()
            .filter(|event| event.sequence > last_sequence)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn(id: &str) -> ConnectionId {
        ConnectionId(id.to_owned())
    }

    #[test]
    fn assigns_monotonic_sequences_per_subject() {
        let mut broker = Broker::new(10);
        let s1 = broker.reserve_sequence("execution:a");
        let s2 = broker.reserve_sequence("execution:a");
        let s3 = broker.reserve_sequence("execution:b");

        assert_eq!((s1, s2, s3), (1, 2, 1));
    }

    #[test]
    fn delivers_only_to_matching_subscribers() {
        let mut broker = Broker::new(10);
        broker.subscribe(conn("c1"), Filter::prefix("execution:"));
        broker.subscribe(conn("c2"), Filter::prefix("task:"));

        let seq = broker.reserve_sequence("execution:a");
        let recipients = broker.commit("execution:a", seq, vec![1]);

        assert_eq!(recipients, vec![conn("c1")]);
    }

    #[test]
    fn unsubscribed_connections_stop_receiving() {
        let mut broker = Broker::new(10);
        broker.subscribe(conn("c1"), Filter::any());
        broker.unsubscribe(&conn("c1"));

        let seq = broker.reserve_sequence("execution:a");
        assert!(broker.commit("execution:a", seq, vec![1]).is_empty());
    }

    #[test]
    fn replays_events_after_cursor() {
        let mut broker = Broker::new(10);
        for i in 0..5 {
            let seq = broker.reserve_sequence("execution:a");
            broker.commit("execution:a", seq, vec![i]);
        }

        let replayed = broker.replay("execution:a", 2).unwrap();
        assert_eq!(replayed.len(), 3);
        assert_eq!(replayed[0].sequence, 3);
        assert_eq!(replayed[2].sequence, 5);
    }

    #[test]
    fn resync_required_when_cursor_is_too_old() {
        let mut broker = Broker::new(3);
        for i in 0..5 {
            let seq = broker.reserve_sequence("execution:a");
            broker.commit("execution:a", seq, vec![i]);
        }

        let error = broker.replay("execution:a", 1).unwrap_err();
        assert_eq!(
            error,
            ReplayError::ResyncRequired {
                subject: "execution:a".into(),
                oldest: 3
            }
        );
    }

    #[test]
    fn replay_of_unknown_subject_is_empty() {
        let broker = Broker::new(10);
        assert!(broker.replay("execution:missing", 0).unwrap().is_empty());
    }
}
