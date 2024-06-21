use shared::packages::publish::Publish;
use std::collections::HashMap;
use std::fmt;

/// This struct represents an client subscription
#[derive(Clone, Debug, PartialEq)]
pub struct ClientSubscription {
    /// ClientSubscription belongs to a client
    pub client_id: String,
    /// ClientSubscription has a Quality of Service
    pub qos: u8,
}

impl fmt::Display for ClientSubscription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(Subscription for client {}, qos {})",
            self.client_id, self.qos
        )
    }
}

impl ClientSubscription {
    /// Creates a new client subscription
    pub fn new(client_id: &str, qos: u8) -> ClientSubscription {
        ClientSubscription {
            client_id: client_id.to_string(),
            qos,
        }
    }
}

/// This struct represents a retained message
#[derive(Clone, Debug, PartialEq)]
pub struct RetainedMessage {
    /// A String containing the retained message topic
    pub topic_name: String,
    /// Retained message content
    pub message: String,
    /// A numeric packet identifier
    pub packet_id: u16,
}

impl RetainedMessage {
    pub fn from_publish_packet(packet: &Publish) -> RetainedMessage {
        RetainedMessage {
            topic_name: packet.topic_name.to_owned(),
            message: packet.payload.to_owned(),
            packet_id: packet.packet_id,
        }
    }

    pub fn to_publish_packet(&self, qos: u8) -> Publish {
        Publish {
            topic_name: self.topic_name.to_owned(),
            payload: self.message.to_owned(),
            packet_id: self.packet_id,
            qos,
            retain_flag: 1_u8, // Publishing from a retained message
            dup_flag: 0_u8,
        }
    }
}

/// This struct represents an MQTT topic
#[derive(Clone, Debug, PartialEq)]
pub struct Topic {
    /// Topic has its name
    pub name: String,
    /// Topic has ClientSubscriptions that represent clients subscribed to it
    pub subscriptions: Vec<ClientSubscription>,
    /// Topic might have a retained message
    pub retained_message: Option<RetainedMessage>,
}

impl Topic {
    /// Returns a topic
    pub fn new(name: String) -> Topic {
        Topic {
            name,
            subscriptions: Vec::new(),
            retained_message: None,
        }
    }
}

/// This struct represents a storage of topics and their subscribed clients
pub struct TopicManager {
    /// Topics have their name and their clients
    topics: HashMap<String, Topic>,
}

impl TopicManager {
    /// Returns an empty TopicManager
    pub fn new() -> TopicManager {
        TopicManager {
            topics: HashMap::new(),
        }
    }

    /// Subscribes a client to a topic or multiple topics if the client used wildcards
    /// Creates each topic if needed
    ///
    /// # Arguments
    ///
    /// * `topic` - A string slice containing the topic to subscribe
    /// * `client` - A string slice containing the client that will subscribe to the topic
    ///
    pub fn subscribe(&mut self, topic: &str, subscription: &ClientSubscription) {
        if topic == "#" || topic == "/#" {
            for available_topic in &self.get_topics_available() {
                self.subscribe_to_topic(available_topic, subscription);
            }
        } else if topic.contains('#') {
            let available_topics = self.get_topics_available();
            let topic_substring = &mut topic.split('#');
            let topic = topic_substring.next().unwrap();

            for available_topic in &available_topics {
                if available_topic.starts_with(topic) {
                    self.subscribe_to_topic(available_topic, subscription);
                }
            }
        } else if topic.contains('+') {
            let available_topics = self.get_topics_available();
            let topic_substring = &mut topic.split('+');
            let topic_first_half = topic_substring.next().unwrap();
            let topic_second_half = topic_substring.next().unwrap();

            for available_topic in &available_topics {
                if available_topic.starts_with(topic_first_half)
                    && available_topic.ends_with(topic_second_half)
                {
                    self.subscribe_to_topic(available_topic, subscription);
                }
            }
        } else {
            self.subscribe_to_topic(topic, subscription);
        }
    }

    /// Private function that subscribes a client to a specific topic and creates the topic if needed
    /// # Arguments
    ///
    /// * `topic` - A string slice containing the topic to subscribe
    /// * `client` - A string slice containing the client that will subscribe to the topic
    ///
    fn subscribe_to_topic(&mut self, topic: &str, subscription: &ClientSubscription) {
        let clients = self
            .topics
            .entry(topic.to_string())
            .or_insert_with(|| Topic::new(topic.to_string()));

        if let Some(position) = clients
            .subscriptions
            .iter()
            .position(|sub| sub.client_id == subscription.client_id)
        {
            clients.subscriptions.remove(position);
        }
        clients.subscriptions.push(subscription.clone());
    }

    /// Checks if a given topic exists in the TopicManager
    /// # Arguments
    ///
    /// * `topic` - A string slice containing the topic to search
    ///
    pub fn has_topic(&self, topic: &str) -> bool {
        self.topics.contains_key(topic)
    }

    /// Returns the clients subscribed to a given topic
    /// # Arguments
    ///
    /// * `topic` - A string slice containing the topic to get the clients for
    ///
    pub fn get_subscriptions(&self, topic: &str) -> Vec<ClientSubscription> {
        if self.has_topic(topic) {
            self.topics[topic].subscriptions.clone()
        } else {
            Vec::new()
        }
    }

    /// Returns all topics that the specified client is subscribed to
    /// # Arguments
    ///
    /// * `client_id` - client identifier
    ///
    pub fn get_client_subscriptions(&self, client_id: &str) -> Vec<String> {
        let mut client_topics = Vec::new();
        for (topic_name, topic) in self.topics.iter() {
            if topic.subscriptions.iter().any(|s| s.client_id == client_id) {
                client_topics.push(topic_name.to_owned());
            }
        }

        client_topics
    }

    /// Unsubscribes a client from the given topic
    /// # Arguments
    ///
    /// * `topic` - A string slice containing the topic to unsubscribe a client
    /// * `client_to_unsubscribe` - A string slice containing the client to unsubscribe
    ///
    pub fn unsubscribe(&mut self, topic: &str, client_to_unsubscribe: &str) {
        // aca puede ir un Result o un enum que represente el resultado de la operacion
        if let Some(clients) = self.topics.get_mut(topic) {
            if let Some(index) = clients
                .subscriptions
                .iter()
                .position(|cl| cl.client_id == client_to_unsubscribe)
            {
                clients.subscriptions.remove(index);
            }
        }
    }

    pub fn get_topics_available(&mut self) -> Vec<String> {
        self.topics.keys().cloned().collect::<Vec<String>>()
    }

    /// Update available topics based on a publish message
    /// This allows the manager to keep track of topics that received a publish
    /// regardless of the subscriptions they have. When publish has enabled its
    /// retain flag, this method saves the retained message in the topic
    ///
    /// # Arguments
    ///
    /// * `msg` - A Publish packet containing the information to update available topics
    ///
    pub fn update_topic(&mut self, msg: &Publish) {
        let topic = self
            .topics
            .entry(msg.topic_name.to_string())
            .or_insert_with(|| Topic::new(msg.topic_name.to_string()));
        if msg.retain_flag == 1 {
            topic.retained_message = Some(RetainedMessage::from_publish_packet(msg));
        }
    }

    /// Get the retained message for a topic
    ///
    /// # Arguments
    ///
    /// * `topic` - A Publish packet that represents the retained message for a given topic
    ///
    pub fn get_retained_message(&mut self, topic: &str) -> Option<RetainedMessage> {
        match self.topics.get(topic) {
            Some(topic) => topic.retained_message.clone(),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::managers::topicmanager;
    use crate::managers::topicmanager::RetainedMessage;
    use shared::packages::publish::Publish;

    #[test]
    fn test_subscribe_succesful() {
        let mut sut = topicmanager::TopicManager::new();
        let client_sub = topicmanager::ClientSubscription::new("someclient", 0);
        sut.subscribe("sometopic", &client_sub);
        assert_eq!(sut.has_topic("sometopic"), true);
        assert_eq!(sut.get_subscriptions("sometopic"), vec![client_sub])
    }

    #[test]
    fn test_subscribe_subscribes_just_once() {
        let mut sut = topicmanager::TopicManager::new();
        let client_sub = topicmanager::ClientSubscription::new("someclient", 0);
        sut.subscribe("sometopic", &client_sub);
        sut.subscribe("sometopic", &client_sub);
        assert_eq!(sut.has_topic("sometopic"), true);
        assert_eq!(sut.get_subscriptions("sometopic"), vec![client_sub])
    }

    #[test]
    fn test_subscribe_changes_qos() {
        let mut sut = topicmanager::TopicManager::new();
        let client_sub = topicmanager::ClientSubscription::new("someclient", 0);
        sut.subscribe("sometopic", &client_sub);
        let updated_client_sub = topicmanager::ClientSubscription::new("someclient", 1);
        sut.subscribe("sometopic", &updated_client_sub);
        let subscriptions = sut.get_subscriptions("sometopic");
        assert_eq!(subscriptions, vec![updated_client_sub]);
    }

    #[test]
    fn test_unsubscribe_succesful() {
        let mut sut = topicmanager::TopicManager::new();
        let client_sub = topicmanager::ClientSubscription::new("someclient", 0);
        let another_client_sub = topicmanager::ClientSubscription::new("anotherclient", 0);
        sut.subscribe("sometopic", &client_sub);
        sut.subscribe("sometopic", &another_client_sub);
        sut.unsubscribe("sometopic", "someclient");
        assert_eq!(sut.get_subscriptions("sometopic"), vec![another_client_sub])
    }

    #[test]
    fn test_has_topic() {
        let mut sut = topicmanager::TopicManager::new();
        let client_sub = topicmanager::ClientSubscription::new("someclient", 0);
        assert_eq!(sut.has_topic("sometopic"), false);
        sut.subscribe("sometopic", &client_sub);
        assert_eq!(sut.has_topic("sometopic"), true);
    }

    #[test]
    fn test_get_subscribed_clients() {
        let mut sut = topicmanager::TopicManager::new();
        let client_sub = topicmanager::ClientSubscription::new("someclient", 0);
        let another_client_sub = topicmanager::ClientSubscription::new("anotherclient", 0);
        sut.subscribe("sometopic", &client_sub);
        sut.subscribe("sometopic", &another_client_sub);
        assert_eq!(
            sut.get_subscriptions("sometopic"),
            vec![client_sub, another_client_sub]
        )
    }

    #[test]
    fn test_update_topic_without_retain_flag() {
        let mut sut = topicmanager::TopicManager::new();
        let msg = Publish {
            topic_name: "/foo".to_string(),
            payload: "est".to_string(),
            packet_id: 1_u16,
            qos: 0_u8,
            retain_flag: 0_u8,
            dup_flag: 0_u8,
        };
        sut.update_topic(&msg);
        assert_eq!(sut.get_retained_message("/foo"), None)
    }

    #[test]
    fn test_update_topic_with_retain_flag() {
        let mut topic_manager = topicmanager::TopicManager::new();
        const TOPIC_NAME: &str = "/foo";
        const MESSAGE: &str = "est";
        const PACKET_ID: u16 = 1_u16;

        let source_packet = Publish {
            topic_name: TOPIC_NAME.to_owned(),
            payload: MESSAGE.to_owned(),
            packet_id: PACKET_ID,
            qos: 0_u8,
            retain_flag: 1_u8,
            dup_flag: 0_u8,
        };
        topic_manager.update_topic(&source_packet);
        let expected_retained_message = RetainedMessage {
            topic_name: TOPIC_NAME.to_owned(),
            message: MESSAGE.to_owned(),
            packet_id: PACKET_ID,
        };
        assert_eq!(
            topic_manager.get_retained_message(TOPIC_NAME),
            Some(expected_retained_message)
        )
    }

    #[test]
    fn test_subcribe_using_wildcard_multi_level() {
        let mut sut = topicmanager::TopicManager::new();
        let client_sub = topicmanager::ClientSubscription::new("someclient", 0);
        let another_client_sub = topicmanager::ClientSubscription::new("anotherclient", 0);
        sut.subscribe("topic/subtopic/topicone", &client_sub);
        sut.subscribe("topic/subtopic/topictwo", &client_sub);
        sut.subscribe("topic/subtopic/topicthree", &client_sub);

        sut.unsubscribe("topic/subtopic/topicone", "someclient");
        sut.unsubscribe("topic/subtopic/topictwo", "someclient");
        sut.unsubscribe("topic/subtopic/topicthree", "someclient");

        sut.subscribe("topic/subtopic/#", &another_client_sub);

        assert_eq!(
            sut.get_subscriptions("topic/subtopic/topicone"),
            vec![another_client_sub.clone()]
        );
        assert_eq!(
            sut.get_subscriptions("topic/subtopic/topictwo"),
            vec![another_client_sub.clone()]
        );
        assert_eq!(
            sut.get_subscriptions("topic/subtopic/topicthree"),
            vec![another_client_sub]
        );
    }

    #[test]
    fn test_subcribe_using_wildcard_single_level() {
        let mut sut = topicmanager::TopicManager::new();
        let client_sub = topicmanager::ClientSubscription::new("someclient", 0);
        let another_client_sub = topicmanager::ClientSubscription::new("anotherclient", 0);
        sut.subscribe("topic/livingroom/temperature", &client_sub);
        sut.subscribe("topic/kitchen/temperature", &client_sub);
        sut.subscribe("topic/bedroom/ligth", &client_sub);

        sut.unsubscribe("topic/livingroom/temperature", "someclient");
        sut.unsubscribe("topic/kitchen/temperature", "someclient");
        sut.unsubscribe("topic/bedroom/ligth", "someclient");

        sut.subscribe("topic/+/temperature", &another_client_sub);

        assert_eq!(
            sut.get_subscriptions("topic/livingroom/temperature"),
            vec![another_client_sub.clone()]
        );
        assert_eq!(
            sut.get_subscriptions("topic/kitchen/temperature"),
            vec![another_client_sub.clone()]
        );
        assert_eq!(sut.get_subscriptions("topic/bedroom/ligth"), vec![]);
    }
}
