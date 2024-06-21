use crate::managers::credentialmanager::CredentialManager;
use crate::managers::messagemanager::MessageManager;
use crate::managers::sessionmanager::{SessionManager, Socket};
use crate::managers::topicmanager::TopicManager;
use crate::packages::server_packet::PacketError;
use crate::packages::server_packet::ServerPacket;
use shared::packages::puback::Puback;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::{event, Level};

impl ServerPacket for Puback {
    fn handle_packet(
        &self,
        stream: &mut TcpStream,
        _credentials: Arc<Mutex<CredentialManager>>,
        sessions: Arc<Mutex<SessionManager>>,
        _topics: Arc<Mutex<TopicManager>>,
        messages: Arc<Mutex<MessageManager>>,
        _actual_streams: Arc<Mutex<Vec<Socket>>>,
    ) -> Result<(), PacketError> {
        match stream.peer_addr() {
            Ok(peer) => {
                let session_manager = sessions.lock().unwrap();
                let mut message_manager = messages.lock().unwrap();
                let peer_port = peer.port();
                if session_manager.has_peer(&peer_port) {
                    let clientid = session_manager.get_client_id(&peer_port).unwrap();
                    message_manager.remove_message(&clientid, self.acknowledged_packet_id);
                }
                drop(session_manager);
                drop(message_manager);
                Ok(())
            }
            Err(e) => {
                event!(Level::ERROR, "Failed processing puback. Reason: {:?}", e);
                Err(PacketError::ExecuteError(e.to_string()))
            }
        }
    }
}
