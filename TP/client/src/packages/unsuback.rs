use crate::managers::idmanager::IDManager;
use crate::packages::client_packet::ClientPacket;
use crate::packagesresponses::connackresponse::ConnackResponse;
use crate::packagesresponses::subackresponse::SubackResponse;
use crate::packagesresponses::unsubackresponse::UnsubackResponse;
use shared::packages::unsuback::Unsuback;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

impl ClientPacket for Unsuback {
    fn handle_packet(
        &self,
        _stream: &mut TcpStream,
        _client_sender: glib::Sender<String>,
        _connack_status_sender: glib::Sender<ConnackResponse>,
        _suback_return_codes_sender: glib::Sender<SubackResponse>,
        unsuback_status_sender: glib::Sender<UnsubackResponse>,
        _id_manager: Arc<Mutex<IDManager>>,
    ) -> std::io::Result<()> {
        let mut status = 1_u8;

        if self.packet_id > 0 {
            status = 0_u8;
        };

        let unsuback_response = UnsubackResponse::new(self.packet_id, status);

        unsuback_status_sender
            .send(unsuback_response)
            .expect("Error al enviar el status del unsuback.");

        Ok(())
    }
}
