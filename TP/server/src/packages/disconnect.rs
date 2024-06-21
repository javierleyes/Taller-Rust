use crate::managers::credentialmanager::CredentialManager;
use crate::managers::messagemanager::MessageManager;
use crate::managers::sessionmanager::{SessionManager, Socket};
use crate::managers::topicmanager::TopicManager;
use crate::packages::server_packet::PacketError;
use crate::packages::server_packet::ServerPacket;
use shared::packages::disconnect::Disconnect;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

impl ServerPacket for Disconnect {
    fn handle_packet(
        &self,
        stream: &mut TcpStream,
        _credentials: Arc<Mutex<CredentialManager>>,
        sessions: Arc<Mutex<SessionManager>>,
        topics: Arc<Mutex<TopicManager>>,
        messages: Arc<Mutex<MessageManager>>,
        actual_streams: Arc<Mutex<Vec<Socket>>>,
    ) -> Result<(), PacketError> {
        // delete session
        let peer = stream.peer_addr().unwrap().port();

        let mut session_manager = sessions.lock().unwrap();
        let client_id = session_manager.get_client_id(&peer).unwrap();

        session_manager.delete(&client_id);
        drop(session_manager);

        // remove client from message manager
        let mut message_manager = messages.lock().unwrap();
        message_manager.delete(&client_id);
        drop(message_manager);

        // unsubscribe topics
        let mut topic_manager = topics.lock().unwrap();
        let topics = topic_manager.get_topics_available();

        for topic in topics.iter() {
            topic_manager.unsubscribe(topic, &client_id);
        }

        drop(topic_manager);

        // remove socket from actual_streams
        let peer = stream.peer_addr().unwrap().port();

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

        Ok(())
    }
}
