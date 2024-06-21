use crate::managers::credentialmanager::CredentialManager;
use crate::managers::messagemanager::{MessageManager, PendingMessage};
use crate::managers::sessionmanager::{SessionManager, Socket};
use crate::managers::topicmanager::{ClientSubscription, TopicManager};
use crate::packages::server_packet::PacketError;
use crate::packages::server_packet::ServerPacket;
use shared::packages::packet::WritablePacket;
use shared::packages::puback::Puback;
use shared::packages::publish::Publish;
use std::cmp;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::{event, Level};

impl ServerPacket for Publish {
    fn handle_packet(
        &self,
        stream: &mut TcpStream,
        _credentials: Arc<Mutex<CredentialManager>>,
        sessions: Arc<Mutex<SessionManager>>,
        topics: Arc<Mutex<TopicManager>>,
        messages: Arc<Mutex<MessageManager>>,
        _actual_streams: Arc<Mutex<Vec<Socket>>>,
    ) -> Result<(), PacketError> {
        let mut topic_manager = topics.lock().unwrap();
        let subscriptions: Vec<ClientSubscription> =
            topic_manager.get_subscriptions(&self.topic_name);

        let session_manager = sessions.lock().unwrap();
        let mut message_manager = messages.lock().unwrap();
        topic_manager.update_topic(self);
        drop(topic_manager);

        for sub in subscriptions.iter() {
            let client_id = sub.client_id.to_string();
            let mut session = match session_manager.get_client(&client_id) {
                Some(result) => result,
                None => {
                    event!(Level::WARN, "Could not get client {:?}", client_id);
                    continue;
                }
            };

            let qos_publish = cmp::min(sub.qos, self.qos);

            let publish = Publish {
                topic_name: self.topic_name.to_owned(),
                payload: self.payload.to_owned(),
                packet_id: self.packet_id,
                qos: qos_publish,
                retain_flag: self.retain_flag,
                dup_flag: self.dup_flag,
            };

            if qos_publish != 0 {
                let pending_message = PendingMessage::from_publish_packet(self);
                message_manager.add_message(&client_id, &pending_message);
            }

            match publish.write_to(&mut session.socket.stream) {
                Ok(_) => event!(Level::INFO, "{:?} sent to client {}", publish, client_id),
                Err(e) => event!(
                    Level::WARN,
                    "{:?} could not be sent to client {:?}. Reason: {:?}",
                    publish,
                    client_id,
                    e
                ),
            }
        }
        drop(session_manager);
        drop(message_manager);

        if self.qos == 1 {
            let puback = Puback {
                acknowledged_packet_id: self.packet_id,
            };
            match puback.write_to(stream) {
                Ok(_) => {
                    event!(
                        Level::INFO,
                        "Puback for packet id {} was succesfully sent",
                        self.packet_id
                    )
                }
                Err(e) => {
                    event!(
                        Level::ERROR,
                        "Puback for packet id {} failed. Reason: {:?}",
                        self.packet_id,
                        e
                    );
                    return Err(PacketError::ExecuteError(e.to_string()));
                }
            }
        }

        Ok(())
    }
}
