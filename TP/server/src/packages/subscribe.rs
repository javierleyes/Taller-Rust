use crate::managers::credentialmanager::CredentialManager;
use crate::managers::messagemanager::MessageManager;
use crate::managers::sessionmanager::{SessionManager, Socket};
use crate::managers::topicmanager::{ClientSubscription, TopicManager};
use crate::packages::server_packet::PacketError;
use crate::packages::server_packet::ServerPacket;
use shared::packages::packet::WritablePacket;
use shared::packages::suback::Suback;
use shared::packages::subscribe::Subscribe;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::{event, Level};

impl ServerPacket for Subscribe {
    fn handle_packet(
        &self,
        stream: &mut TcpStream,
        _credentials: Arc<Mutex<CredentialManager>>,
        sessions: Arc<Mutex<SessionManager>>,
        topics: Arc<Mutex<TopicManager>>,
        _messages: Arc<Mutex<MessageManager>>,
        _actual_streams: Arc<Mutex<Vec<Socket>>>,
    ) -> Result<(), PacketError> {
        let session_manager = sessions.lock().unwrap();
        let peer = stream.peer_addr().unwrap().port();

        let mut response_qos = Vec::new();
        if session_manager.has_peer(&peer) {
            let mut topic_manager = topics.lock().unwrap();
            let client_id = session_manager.get_client_id(&peer).unwrap();

            let topic_amount = self.topic_filters.len();

            for index in 0..topic_amount {
                let requested_qos = self.requested_qos[index];
                let subscription = ClientSubscription::new(&client_id, requested_qos);
                topic_manager.subscribe(&self.topic_filters[index], &subscription);

                let final_subscriptions = topic_manager.get_client_subscriptions(&client_id);

                for sub in final_subscriptions.iter() {
                    match topic_manager.get_retained_message(sub) {
                        Some(retained_message) => {
                            event!(Level::INFO, "LLEGUE con retained {:?}", &retained_message);
                            let publish_packet =
                                retained_message.to_publish_packet(self.requested_qos[index]);
                            match publish_packet.write_to(stream) {
                                Ok(_) => {
                                    event!(
                                        Level::DEBUG,
                                        "SEND retained message {:?}",
                                        &retained_message
                                    );
                                }
                                Err(e) => {
                                    event!(
                                        Level::WARN,
                                        "FAIL sending retained message {:?}. Reason {:?}",
                                        &retained_message,
                                        e
                                    );
                                }
                            }
                        }
                        None => {
                            event!(
                                Level::DEBUG,
                                "No retained message for topic {:?}",
                                &self.topic_filters[index]
                            );
                        }
                    }
                }
                response_qos.push(self.requested_qos[index]);
            }
            drop(topic_manager);
        }

        drop(session_manager);

        let response = Suback {
            packet_id: self.packet_id,
            return_codes: response_qos,
        };
        response.write_to(stream)?;

        Ok(())
    }
}
