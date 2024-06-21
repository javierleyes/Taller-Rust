use crate::managers::measuresmanager::MeasuresManager;
use crate::packages::client_packet::ClientPacket;
use shared::packages::connack::Connack;
use std::sync::{Arc, Mutex};

impl ClientPacket for Connack {
    fn handle_packet(&self, _measures_manager: Arc<Mutex<MeasuresManager>>) -> std::io::Result<()> {
        Ok(())
    }
}
