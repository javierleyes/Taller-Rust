use std::collections::HashMap;

/// This struct represents a storage packet_id and topic
///
/// * `subscriptions`: Hashmap with a packet_id as key and the topic of that packet as value
///
/// * `active_subscriptions`: number of topics in the subscription tab table
pub struct SubscriptionManager {
    subscriptions: HashMap<u16, String>,
    active_subscriptions: i32,
}

impl SubscriptionManager {
    /// Returns an empty SubscriptionManager
    pub fn new() -> SubscriptionManager {
        SubscriptionManager {
            subscriptions: HashMap::new(),
            active_subscriptions: 1,
        }
    }

    /// Add a subscription to subscriptions
    /// # Arguments
    ///
    /// * `packet_id` - An u16
    /// * `topic` - A &str
    ///
    pub fn add_subscription(&mut self, packet_id: u16, topic: &str) {
        self.subscriptions.insert(packet_id, topic.to_string());
    }

    /// Get the topic associated to that packet_id
    /// # Arguments
    ///
    /// * `packet_id` - An u16
    pub fn get_packet_topic(&self, packet_id: u16) -> Result<String, String> {
        if self.has_topic(packet_id) {
            Ok((self.subscriptions[&packet_id]).to_owned())
        } else {
            Err("Error getting the subscription...".to_string())
        }
    }

    /// Get if the subscriptions collection has an especific packet_id
    /// # Arguments
    ///
    /// * `packet_id` - An u16
    pub fn has_topic(&self, packet_id: u16) -> bool {
        self.subscriptions.contains_key(&packet_id)
    }

    /// Delete a subscription
    /// # Arguments
    ///
    /// * `packet_id` - An u16
    pub fn delete_subscription(&mut self, packet_id: u16) {
        if self.has_topic(packet_id) {
            self.subscriptions.remove(&packet_id);
        };
    }

    /// Get the number of active_subscriptions
    pub fn get_active_subscriptions(&self) -> i32 {
        self.active_subscriptions
    }

    /// Add an active_subscriptions
    pub fn add_active_subscription(&mut self) {
        self.active_subscriptions += 1;
    }

    /// Remove an active_subscriptions
    pub fn remove_active_subscription(&mut self) {
        self.active_subscriptions -= 1;
    }

    /// Clear the subscriptions collection
    pub fn delete_all_subscription(&mut self) {
        self.subscriptions.clear();
        self.active_subscriptions = 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::managers::subscriptionmanager::SubscriptionManager;

    #[test]
    fn test_add_subscription_successful() {
        let mut subscription_manager = SubscriptionManager::new();
        subscription_manager.add_subscription(1234, "mi_topico");

        assert_eq!(
            subscription_manager.get_packet_topic(1234).unwrap(),
            "mi_topico"
        )
    }

    #[test]
    fn test_delete_subscription_successful() {
        let mut subscription_manager = SubscriptionManager::new();
        subscription_manager.add_subscription(1234, "mi_topico");
        subscription_manager.delete_subscription(1234);

        match subscription_manager.get_packet_topic(1234) {
            Ok(_) => {}
            Err(e) => assert_eq!(e, "Error getting the subscription...".to_string()),
        };
    }
}
