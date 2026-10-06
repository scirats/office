use crate::domain::{
    ExpectedWorkspace, Inconsistency, ReconciliationReport, WorkspaceObservation,
    WorkspaceReconciliation, WorkspaceStatus,
};
use crate::ports::WorkspaceInspector;

pub struct ReconciliationService {
    inspector: Box<dyn WorkspaceInspector>,
}

impl ReconciliationService {
    pub fn new(inspector: Box<dyn WorkspaceInspector>) -> Self {
        Self { inspector }
    }

    pub fn run(&self, expected: &[ExpectedWorkspace]) -> ReconciliationReport {
        ReconciliationReport {
            workspaces: expected
                .iter()
                .map(|workspace| self.reconcile(workspace))
                .collect(),
        }
    }

    fn reconcile(&self, expected: &ExpectedWorkspace) -> WorkspaceReconciliation {
        let observation = match self.inspector.inspect(&expected.user, &expected.home) {
            Ok(observation) => observation,
            Err(_) => {
                return WorkspaceReconciliation {
                    slug: expected.slug.clone(),
                    status: WorkspaceStatus::Inconsistent,
                    inconsistencies: vec![Inconsistency::InspectionUnavailable],
                };
            }
        };

        let inconsistencies = self.detect(&observation);
        let status = if inconsistencies.is_empty() {
            WorkspaceStatus::Active
        } else {
            WorkspaceStatus::Inconsistent
        };

        WorkspaceReconciliation {
            slug: expected.slug.clone(),
            status,
            inconsistencies,
        }
    }

    fn detect(&self, observation: &WorkspaceObservation) -> Vec<Inconsistency> {
        let mut inconsistencies = Vec::new();

        if !observation.user_exists {
            inconsistencies.push(Inconsistency::MissingUser);
        }
        if !observation.home_exists {
            inconsistencies.push(Inconsistency::MissingHome);
        }
        if observation.home_exists {
            if let (Some(owner), Some(expected_uid)) = (observation.home_uid, observation.user_uid)
                && owner != expected_uid
            {
                inconsistencies.push(Inconsistency::WrongOwner);
            }
            if observation.home_mode.map(|mode| mode & 0o777) != Some(0o700) {
                inconsistencies.push(Inconsistency::WrongMode);
            }
        }

        inconsistencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::InspectError;

    fn observation(
        user_exists: bool,
        home_exists: bool,
        owner: u32,
        mode: u32,
    ) -> WorkspaceObservation {
        WorkspaceObservation {
            user_exists,
            user_uid: if user_exists { Some(owner) } else { None },
            home_exists,
            home_uid: if home_exists { Some(owner) } else { None },
            home_mode: if home_exists { Some(mode) } else { None },
        }
    }

    struct FakeInspector(WorkspaceObservation);

    impl WorkspaceInspector for FakeInspector {
        fn inspect(&self, _user: &str, _home: &str) -> Result<WorkspaceObservation, InspectError> {
            Ok(self.0.clone())
        }
    }

    struct BrokenInspector;

    impl WorkspaceInspector for BrokenInspector {
        fn inspect(&self, _user: &str, _home: &str) -> Result<WorkspaceObservation, InspectError> {
            Err(InspectError::Unsupported)
        }
    }

    fn expected() -> ExpectedWorkspace {
        ExpectedWorkspace {
            slug: "acme".to_owned(),
            user: "acme".to_owned(),
            home: "/home/acme".to_owned(),
        }
    }

    #[test]
    fn active_when_user_and_home_are_consistent() {
        let service = ReconciliationService::new(Box::new(FakeInspector(observation(
            true, true, 1000, 0o700,
        ))));
        let report = service.run(&[expected()]);
        assert_eq!(report.workspaces[0].status, WorkspaceStatus::Active);
        assert!(report.workspaces[0].inconsistencies.is_empty());
    }

    #[test]
    fn detects_missing_user_and_home() {
        let service =
            ReconciliationService::new(Box::new(FakeInspector(observation(false, false, 0, 0))));
        let report = service.run(&[expected()]);
        assert_eq!(report.workspaces[0].status, WorkspaceStatus::Inconsistent);
        assert_eq!(
            report.workspaces[0].inconsistencies,
            vec![Inconsistency::MissingUser, Inconsistency::MissingHome]
        );
    }

    #[test]
    fn detects_wrong_owner_and_mode() {
        let service = ReconciliationService::new(Box::new(FakeInspector(WorkspaceObservation {
            user_exists: true,
            user_uid: Some(1000),
            home_exists: true,
            home_uid: Some(2000),
            home_mode: Some(0o755),
        })));
        let report = service.run(&[expected()]);
        assert_eq!(
            report.workspaces[0].inconsistencies,
            vec![Inconsistency::WrongOwner, Inconsistency::WrongMode]
        );
    }

    #[test]
    fn flags_inspection_failures() {
        let service = ReconciliationService::new(Box::new(BrokenInspector));
        let report = service.run(&[expected()]);
        assert_eq!(
            report.workspaces[0].inconsistencies,
            vec![Inconsistency::InspectionUnavailable]
        );
    }
}
