use crate::CredentialManager;
use crate::MessageManager;
use crate::SessionManager;
use crate::Socket;
use crate::TopicManager;
use std::fmt;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

/// This struct represents an error in Packet Serialization/Deserialization
#[derive(Debug)]
pub enum PacketError {
    IOError(std::io::Error),
    ExecuteError(String),
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PacketError::IOError(ref err) => write!(f, "IO error: {}", err),
            PacketError::ExecuteError(ref err) => write!(f, "Packet Execution Error: {}", err),
        }
    }
}

impl std::error::Error for PacketError {}

impl From<std::io::Error> for PacketError {
    fn from(err: std::io::Error) -> PacketError {
        PacketError::IOError(err)
    }
}

pub trait ServerPacket: std::fmt::Debug {
    fn handle_packet(
        &self,
        stream: &mut TcpStream,
        credentials: Arc<Mutex<CredentialManager>>,
        sessions: Arc<Mutex<SessionManager>>,
        topics: Arc<Mutex<TopicManager>>,
        messages: Arc<Mutex<MessageManager>>,
        actual_streams: Arc<Mutex<Vec<Socket>>>,
    ) -> Result<(), PacketError>;
}
