use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Incoming(usize, String, Message);

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
    data: serde_json::Value,
    event_type: EventType,
    uri: String,
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

        let data: Incoming = serde_json::from_str(&raw).expect("should have a new message");
        let msg = data.2.clone();
    }
}
