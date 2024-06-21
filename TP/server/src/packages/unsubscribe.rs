use crate::managers::credentialmanager::CredentialManager;
use crate::managers::messagemanager::MessageManager;
use crate::managers::sessionmanager::{SessionManager, Socket};
use crate::managers::topicmanager::TopicManager;
use crate::packages::server_packet::PacketError;
use crate::packages::server_packet::ServerPacket;
use shared::packages::packet::WritablePacket;
use shared::packages::unsuback::Unsuback;
use shared::packages::unsubscribe::Unsubscribe;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

impl ServerPacket for Unsubscribe {
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

        if session_manager.has_peer(&peer) {
            let mut topic_manager = topics.lock().unwrap();
            let clientid = session_manager.get_client_id(&peer).unwrap();

            for index in 0..self.topic_filters.len() {
                topic_manager.unsubscribe(&self.topic_filters[index], &clientid);
            }

            drop(topic_manager);
        }

        drop(session_manager);

        let response = Unsuback {
            packet_id: self.packet_id,
        };
        response.write_to(stream)?;

        Ok(())
    }
}
