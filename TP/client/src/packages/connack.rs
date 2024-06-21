use crate::managers::idmanager::IDManager;
use crate::packages::client_packet::ClientPacket;
use crate::packagesresponses::connackresponse::ConnackResponse;
use crate::packagesresponses::subackresponse::SubackResponse;
use crate::packagesresponses::unsubackresponse::UnsubackResponse;
use shared::packages::connack::Connack;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

impl ClientPacket for Connack {
    fn handle_packet(
        &self,
        _stream: &mut TcpStream,
        _client_sender: glib::Sender<String>,
        connack_status_sender: glib::Sender<ConnackResponse>,
        _suback_return_codes_sender: glib::Sender<SubackResponse>,
        _unsuback_status_sender: glib::Sender<UnsubackResponse>,
        _id_manager: Arc<Mutex<IDManager>>,
    ) -> std::io::Result<()> {
        let connack_response =
            ConnackResponse::new(self.return_code as u8, self.session_present as u8);

        connack_status_sender
            .send(connack_response)
            .expect("Error al enviar el status de la conexion.");

        Ok(())
    }
}
