use crate::managers::credentialmanager::CredentialManager;
use crate::managers::messagemanager::MessageManager;
use crate::managers::sessionmanager::{LastWillTestament, SessionManager, Socket};
use crate::managers::topicmanager::TopicManager;
use crate::packages::server_packet::PacketError;
use crate::packages::server_packet::ServerPacket;
use shared::packages::connack::Connack;
use shared::packages::connect::Connect;
use shared::packages::packet::WritablePacket;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::{event, Level};

enum ConnectReturnCode {
    ConnectionAccepted = 0,
    ConnectionRefusedBadUsernameOrPassword = 4,
}

enum SessionPresent {
    Yes = 1,
    No = 0,
}

impl ServerPacket for Connect {
    fn handle_packet(
        &self,
        stream: &mut TcpStream,
        credentials: Arc<Mutex<CredentialManager>>,
        sessions: Arc<Mutex<SessionManager>>,
        topics: Arc<Mutex<TopicManager>>,
        _messages: Arc<Mutex<MessageManager>>,
        actual_streams: Arc<Mutex<Vec<Socket>>>,
    ) -> Result<(), PacketError> {
        let credential_manager = credentials.lock().unwrap();
        let is_valid = credential_manager.is_valid(&self.username, &self.password);
        drop(credential_manager);

        let mut return_code = ConnectReturnCode::ConnectionRefusedBadUsernameOrPassword as u8;
        let mut session_present = SessionPresent::No as u8;

        if is_valid {
            let mut session_manager = sessions.lock().unwrap();
            let lwt = match self.last_will_flag {
                0 => None,
                1 => Some(LastWillTestament {
                    topic_name: self.last_will_topic.to_string(),
                    payload: self.last_will_message.to_string(),
                    qos: self.last_will_qos,
                    retain_flag: self.last_will_retain,
                }),
                _ => panic!("Invalid last will flag!"),
            };
            if !session_manager.has_client(&self.client_id) {
                session_manager.add_client(&self.client_id, stream.try_clone().unwrap(), lwt);
                session_present = SessionPresent::No as u8;
            } else {
                // persistent session

                // delete actual streams
                match session_manager.get_old_peer(&self.client_id) {
                    Ok(previous_peer) => {
                        remove_stream(previous_peer, actual_streams);
                    }
                    Err(e) => {
                        event!(Level::ERROR, "Failed replacing socket: reason {:?}", e);
                    }
                }

                if self.clean_session == 0 {
                    session_present = SessionPresent::Yes as u8;

                    // session manager
                    session_manager.replace_stream(&self.client_id, stream.try_clone().unwrap());
                } else {
                    // Non persistent session
                    session_present = SessionPresent::No as u8;

                    // delete old session
                    session_manager.delete(&self.client_id);

                    // delete subscriptions
                    let mut topic_manager = topics.lock().unwrap();
                    let topics = topic_manager.get_topics_available();
                    for topic in topics.iter() {
                        topic_manager.unsubscribe(topic, &self.client_id);
                    }
                    drop(topic_manager);

                    // add new client
                    session_manager.add_client(&self.client_id, stream.try_clone().unwrap(), lwt);
                };
            }

            drop(session_manager);

            return_code = ConnectReturnCode::ConnectionAccepted as u8;
        } else {
            remove_stream(stream.peer_addr().unwrap().port(), actual_streams);
        };

        let connack = Connack {
            return_code,
            session_present,
        };

        connack.write_to(stream)?;

        Ok(())
    }
}

fn remove_stream(peer: u16, actual_streams: Arc<Mutex<Vec<Socket>>>) {
    let mut index = 0;
    let mut active_streams = actual_streams.lock().unwrap();

    for _ in 0..active_streams.len() {
        if peer == active_streams[index].peer {
            break;
        }

        index += 1;
    }

    active_streams.remove(index);

    drop(active_streams);
}
