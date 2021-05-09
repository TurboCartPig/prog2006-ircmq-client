use serde::{Deserialize, Serialize};

/// Messages to be serialized and sent to the server.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "tag")]
pub enum MessageType {
    /// Initial message sent when a client first connects.
    Hello { name: String, channel: String },
    /// A text message sent from a client to all the other clients, through the server.
    Message {
        name: String,
        channel: String,
        content: String,
    },
    /// Response from the server, containing a list of members of the current channel.
    ResponseMembers { members: Vec<String> },
    /// Response from the server, containing a list of channels on the server.
    ResponseChannels { channels: Vec<String> },
    /// Final message from client to server, notifying the server, that the client is disconnecting.
    Goodbye { name: String, channel: String },
}

#[cfg(test)]
mod test {
    //! Test that MessageType serializes and deserializes to and from the expected strings and
    //! structs

    #[test]
    fn test_serialize_message() {
        let message = crate::message::MessageType::Message {
            name: "Sebern".into(),
            channel: "A".into(),
            content: "Heihei".into(),
        };

        let message = serde_json::to_string(&message)
            .expect("Serde failed to serialize MessageType::Message");

        assert_eq!(
            message,
            "{\"tag\":\"Message\",\"name\":\"Sebern\",\"channel\":\"A\",\"content\":\"Heihei\"}"
        );
    }

    #[test]
    fn test_deserialize_message() {
        let message: &str =
            "{\"tag\":\"Message\",\"name\":\"Sebern\",\"channel\":\"A\",\"content\":\"Heihei\"}";

        let message: crate::message::MessageType = serde_json::from_str(message)
            .expect("Serde failed to deserialize MessageType::Message");

        assert_eq!(
            message,
            crate::message::MessageType::Message {
                name: "Sebern".into(),
                channel: "A".into(),
                content: "Heihei".into(),
            }
        );
    }

    #[test]
    fn test_serialize_hello() {
        let message = crate::message::MessageType::Hello {
            name: String::from("Name"),
            channel: String::from("Channel"),
        };

        let message =
            serde_json::to_string(&message).expect("Serde failed to serialize MessageType::Hello");

        assert_eq!(
            message,
            "{\"tag\":\"Hello\",\"name\":\"Name\",\"channel\":\"Channel\"}"
        );
    }

    #[test]
    fn test_deserialize_hello() {
        let message: &str = "{\"tag\":\"Hello\",\"name\":\"Name\",\"channel\":\"Channel\"}";

        let message: crate::message::MessageType =
            serde_json::from_str(message).expect("Serde failed to deserialize MessageType::Hello");

        assert_eq!(
            message,
            crate::message::MessageType::Hello {
                name: String::from("Name"),
                channel: String::from("Channel"),
            }
        );
    }

    #[test]
    fn test_serialize_goodbye() {
        let message = crate::message::MessageType::Goodbye {
            name: String::from("Name"),
            channel: String::from("Channel"),
        };

        let message = serde_json::to_string(&message)
            .expect("Serde failed to serialize MessageType::Goodbye");

        assert_eq!(
            message,
            "{\"tag\":\"Goodbye\",\"name\":\"Name\",\"channel\":\"Channel\"}"
        );
    }

    #[test]
    fn test_deserialize_goodbye() {
        let message: &str = "{\"tag\":\"Goodbye\",\"name\":\"Name\",\"channel\":\"Channel\"}";

        let message: crate::message::MessageType = serde_json::from_str(message)
            .expect("Serde failed to deserialize MessageType::Goodbye");

        assert_eq!(
            message,
            crate::message::MessageType::Goodbye {
                name: String::from("Name"),
                channel: String::from("Channel"),
            }
        );
    }

    #[test]
    fn test_serialize_response_members() {
        let message = crate::message::MessageType::ResponseMembers {
            members: vec![String::from("Member")],
        };

        let message = serde_json::to_string(&message)
            .expect("Serde failed to serialize MessageType::ResponseMembers");

        assert_eq!(
            message,
            "{\"tag\":\"ResponseMembers\",\"members\":[\"Member\"]}"
        );
    }

    #[test]
    fn test_deserialize_response_members() {
        let message: &str = "{\"tag\":\"ResponseMembers\",\"members\":[\"Member\"]}";

        let message: crate::message::MessageType = serde_json::from_str(message)
            .expect("Serde failed to deserialize MessageType::ResponseMembers");

        assert_eq!(
            message,
            crate::message::MessageType::ResponseMembers {
                members: vec![String::from("Member")],
            }
        );
    }

    #[test]
    fn test_serialize_response_channels() {
        let message = crate::message::MessageType::ResponseChannels {
            channels: vec![String::from("Channel")],
        };

        let message = serde_json::to_string(&message)
            .expect("Serde failed to serialize MessageType::ResponseChannels");

        assert_eq!(
            message,
            "{\"tag\":\"ResponseChannels\",\"channels\":[\"Channel\"]}"
        );
    }

    #[test]
    fn test_deserialize_response_channel() {
        let message: &str = "{\"tag\":\"ResponseChannels\",\"channels\":[\"Channel\"]}";

        let message: crate::message::MessageType = serde_json::from_str(message)
            .expect("Serde failed to deserialize MessageType::ResponseChannels");

        assert_eq!(
            message,
            crate::message::MessageType::ResponseChannels {
                channels: vec![String::from("Channel")],
            },
        );
    }
}
