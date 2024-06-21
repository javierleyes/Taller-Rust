use crate::managers::idmanager::IDManager;
use crate::packages::client_packet::ClientPacket;
use crate::packagesresponses::connackresponse::ConnackResponse;
use crate::packagesresponses::subackresponse::SubackResponse;
use crate::packagesresponses::unsubackresponse::UnsubackResponse;
use shared::packages::packet::WritablePacket;
use shared::packages::puback::Puback;
use shared::packages::publish::Publish;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

impl ClientPacket for Publish {
    fn handle_packet(
        &self,
        stream: &mut TcpStream,
        client_sender: glib::Sender<String>,
        _connack_status_sender: glib::Sender<ConnackResponse>,
        _suback_return_codes_sender: glib::Sender<SubackResponse>,
        _unsuback_status_sender: glib::Sender<UnsubackResponse>,
        _id_manager: Arc<Mutex<IDManager>>,
    ) -> std::io::Result<()> {
        let mut publish_info = String::new();

        let separator = ("|").to_string();

        publish_info += &self.topic_name;
        publish_info += &separator;
        publish_info += &self.payload;

        client_sender
            .send(publish_info)
            .expect("Error al enviar el resultado de la conexion.");

        if self.qos != 0 {
            let puback = Puback {
                acknowledged_packet_id: self.packet_id,
            };
            if puback.write_to(stream).is_ok() {};
        }
        Ok(())
    }
}
