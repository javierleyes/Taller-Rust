use crate::managers::idmanager::IDManager;
use crate::packagesresponses::connackresponse::ConnackResponse;
use crate::packagesresponses::subackresponse::SubackResponse;
use crate::packagesresponses::unsubackresponse::UnsubackResponse;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub trait ClientPacket: std::fmt::Debug {
    fn handle_packet(
        &self,
        stream: &mut TcpStream,
        client_sender: glib::Sender<String>,
        connack_status_sender: glib::Sender<ConnackResponse>,
        suback_return_codes_sender: glib::Sender<SubackResponse>,
        unsuback_status_sender: glib::Sender<UnsubackResponse>,
        id_manager: Arc<Mutex<IDManager>>,
    ) -> std::io::Result<()>;
}
