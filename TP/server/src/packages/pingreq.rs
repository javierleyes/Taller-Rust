use crate::managers::credentialmanager::CredentialManager;
use crate::managers::messagemanager::MessageManager;
use crate::managers::sessionmanager::{SessionManager, Socket};
use crate::managers::topicmanager::TopicManager;
use crate::packages::server_packet::PacketError;
use crate::packages::server_packet::ServerPacket;
use shared::packages::pingreq::Pingreq;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

impl ServerPacket for Pingreq {
    fn handle_packet(
        &self,
        _stream: &mut TcpStream,
        _credentials: Arc<Mutex<CredentialManager>>,
        _sessions: Arc<Mutex<SessionManager>>,
        _topics: Arc<Mutex<TopicManager>>,
        _messages: Arc<Mutex<MessageManager>>,
        _actual_streams: Arc<Mutex<Vec<Socket>>>,
    ) -> Result<(), PacketError> {
        Ok(())
    }
}
