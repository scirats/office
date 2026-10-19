use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ProtocolError;
use crate::{CONTENT_TYPE, SPEC_VERSION};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudEvent {
    pub specversion: String,
    pub id: String,
    pub source: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datacontenttype: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataschema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlationid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causationid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,
    pub data: Value,
}

impl CloudEvent {
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        type_: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            specversion: SPEC_VERSION.to_owned(),
            id: id.into(),
            source: source.into(),
            type_: type_.into(),
            time: None,
            subject: None,
            datacontenttype: Some(CONTENT_TYPE.to_owned()),
            dataschema: None,
            correlationid: None,
            causationid: None,
            sequence: None,
            data,
        }
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn with_correlation(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlationid = Some(correlation_id.into());
        self
    }

    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }

    pub fn from_json(input: &str) -> Result<Self, ProtocolError> {
        let event: CloudEvent = serde_json::from_str(input)?;
        event.validate()?;
        Ok(event)
    }

    pub fn to_json(&self) -> Result<String, ProtocolError> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn data_as<T: DeserializeOwned>(&self) -> Result<T, ProtocolError> {
        Ok(T::deserialize(&self.data)?)
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.specversion != SPEC_VERSION {
            return Err(ProtocolError::UnsupportedSpecVersion(
                self.specversion.clone(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trips_through_json() {
        let event = CloudEvent::new(
            "id-1",
            "/office/daemon",
            "office.test.v1",
            json!({ "a": 1 }),
        )
        .with_subject("execution:exec_1")
        .with_sequence(7)
        .with_correlation("cmd-1");

        let encoded = event.to_json().expect("serializes");
        let decoded = CloudEvent::from_json(&encoded).expect("deserializes");

        assert_eq!(decoded, event);
        assert_eq!(decoded.sequence, Some(7));
    }

    #[test]
    fn rejects_unsupported_spec_version() {
        let raw = r#"{"specversion":"0.3","id":"1","source":"/x","type":"t","data":{}}"#;
        let error = CloudEvent::from_json(raw).unwrap_err();
        assert!(matches!(error, ProtocolError::UnsupportedSpecVersion(v) if v == "0.3"));
    }

    #[test]
    fn omitted_optional_attributes_are_not_serialized() {
        let event = CloudEvent::new("id-1", "/office/daemon", "office.test.v1", json!({}));
        let encoded = event.to_json().expect("serializes");
        assert!(!encoded.contains("subject"));
        assert!(!encoded.contains("sequence"));
    }
}
