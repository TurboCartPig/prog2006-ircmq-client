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
    #[test]
    fn test_serialize_message() {
        let message = crate::message::MessageType::Message {
            name: "Sebern".into(),
            channel: "A".into(),
            content: "Heihei".into(),
        };

        let message =
            serde_json::to_string(&message).expect("Serde failed to serialize MessageType::Mesage");

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
}
