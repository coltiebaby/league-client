//! Core values

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]

/// The message that is sent from the client
pub struct Incoming(pub usize, pub String, pub Message);

impl Incoming {
    pub fn into_message(self) -> Message {
        self.2
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    #[default]
    Unknown,
    Create,
    Delete,
    Update,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub data: serde_json::Value,
    pub event_type: EventType,
    pub uri: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_message() {
        let raw = r#"[8,"OnJsonApiEvent",{
            "data":{},
            "eventType":"Update",
            "uri":"/lol-champ-select/v1/skin-selector-info"
            }]"#;

        let data: Incoming = serde_json::from_str(raw).expect("should have a new message");
        let msg = data.into_message();
        assert_eq!(msg.event_type, EventType::Update);
        assert_eq!(
            msg.uri,
            "/lol-champ-select/v1/skin-selector-info".to_string()
        );
    }
}
