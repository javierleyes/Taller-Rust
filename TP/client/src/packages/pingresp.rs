use crate::managers::idmanager::IDManager;
use crate::packages::client_packet::ClientPacket;
use crate::packagesresponses::connackresponse::ConnackResponse;
use crate::packagesresponses::subackresponse::SubackResponse;
use crate::packagesresponses::unsubackresponse::UnsubackResponse;
use shared::packages::pingresp::Pingresp;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

impl ClientPacket for Pingresp {
    fn handle_packet(
        &self,
        _stream: &mut TcpStream,
        _client_sender: glib::Sender<String>,
        _connack_status_sender: glib::Sender<ConnackResponse>,
        _suback_return_codes_sender: glib::Sender<SubackResponse>,
        _unsuback_status_sender: glib::Sender<UnsubackResponse>,
        _id_manager: Arc<Mutex<IDManager>>,
    ) -> std::io::Result<()> {
        Ok(())
    }
}
