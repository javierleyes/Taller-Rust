use crate::managers::idmanager::IDManager;
use crate::packages::client_packet::ClientPacket;
use crate::packagesresponses::connackresponse::ConnackResponse;
use crate::packagesresponses::subackresponse::SubackResponse;
use crate::packagesresponses::unsubackresponse::UnsubackResponse;
use shared::packages::puback::Puback;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

impl ClientPacket for Puback {
    fn handle_packet(
        &self,
        _stream: &mut TcpStream,
        _client_sender: glib::Sender<String>,
        _connack_status_sender: glib::Sender<ConnackResponse>,
        _suback_return_codes_sender: glib::Sender<SubackResponse>,
        _unsuback_status_sender: glib::Sender<UnsubackResponse>,
        id_manager: Arc<Mutex<IDManager>>,
    ) -> std::io::Result<()> {
        let mut packet_id_manager = id_manager.lock().unwrap();
        packet_id_manager.free_id(self.acknowledged_packet_id);
        Ok(())
    }
}
