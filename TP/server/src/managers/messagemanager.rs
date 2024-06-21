use shared::packages::publish::Publish;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct PendingMessage {
    /// A String containing the topic to publish to
    pub topic_name: String,
    /// A payload String that represents the message to be published
    pub payload: String,
    /// A numeric packet identifier
    pub packet_id: u16,
    /// The QoS level for this packet
    pub qos: u8,
    /// A retain_flag that indicates if the message should be retained or not
    pub retain_flag: u8,
}

impl PendingMessage {
    pub fn from_publish_packet(packet: &Publish) -> PendingMessage {
        PendingMessage {
            topic_name: packet.topic_name.to_owned(),
            payload: packet.payload.to_owned(),
            packet_id: packet.packet_id,
            qos: packet.qos,
            retain_flag: packet.retain_flag,
        }
    }

    pub fn to_publish_packet(&self) -> Publish {
        Publish {
            topic_name: self.topic_name.to_owned(),
            payload: self.payload.to_owned(),
            packet_id: self.packet_id,
            qos: self.qos,
            retain_flag: self.retain_flag,
            dup_flag: 1_u8, // Publish from a Pending Message is always a duplicate
        }
    }
}

/// This struct represents a storage of pending messages to send or re-send to clients
pub struct MessageManager {
    /// Messages are related to clients
    messages: HashMap<String, Vec<PendingMessage>>,
}

impl MessageManager {
    /// Returns an empty MessageManager
    pub fn new() -> MessageManager {
        MessageManager {
            messages: HashMap::new(),
        }
    }

    /// Queues a client to a topic and creates the topic if needed
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client_id to re-send the packet to
    /// * `message` - A pending message to re-send
    ///
    pub fn add_message(&mut self, client_id: &str, message: &PendingMessage) {
        let messages = self
            .messages
            .entry(client_id.to_string())
            .or_insert_with(Vec::new);
        messages.push(message.clone());
    }

    /// Checks if a given client exists in the MessageManager
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client to search
    ///
    fn _has_client(&self, client_id: &str) -> bool {
        self.messages.contains_key(client_id)
    }

    /// Returns pending messages for a given client
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client to get the messages for
    ///
    fn _get_messages(&self, client_id: &str) -> Vec<PendingMessage> {
        if self._has_client(client_id) {
            self.messages[client_id].clone()
        } else {
            Vec::new()
        }
    }

    /// Removes a message from pending messages to send
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client to delete the message from
    /// * `packet_id` - A string slice containing the packet_id to remove
    ///
    pub fn remove_message(&mut self, client_id: &str, packet_id: u16) {
        if let Some(messages) = self.messages.get_mut(client_id) {
            if let Some(index) = messages.iter().position(|msg| msg.packet_id == packet_id) {
                messages.remove(index);
            }
        }
    }

    /// Returns an iterator of clients and pending messages
    ///
    pub fn get_all(&mut self) -> std::collections::hash_map::Iter<String, Vec<PendingMessage>> {
        self.messages.iter()
    }

    /// Delete a client from the message manager
    /// # Arguments
    ///
    /// * `client_id` - A string slice containing the client id
    ///
    pub fn delete(&mut self, client_id: &str) {
        if self._has_client(client_id) {
            self.messages.remove(client_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::managers::messagemanager::MessageManager;
    use crate::managers::messagemanager::PendingMessage;
    #[test]
    fn test_add_message_successful() {
        let mut sut = MessageManager::new();
        let packet = get_dummy_publish();
        sut.add_message("some_client", &packet);
        assert_eq!(sut._get_messages("some_client"), vec![packet])
    }

    #[test]
    fn test_remove_message() {
        let mut sut = MessageManager::new();
        let packet = get_dummy_publish();
        let client = "some_client";
        let packet_id = packet.packet_id;
        sut.add_message(client, &packet);
        assert_eq!(sut._get_messages(client), vec![packet]);
        sut.remove_message(client, packet_id);
    }

    fn get_dummy_publish() -> PendingMessage {
        PendingMessage {
            topic_name: "some_topic".to_string(),
            payload: "some payload".to_string(),
            packet_id: 1_u16,
            qos: 1_u8,
            retain_flag: 0_u8,
        }
    }
}
