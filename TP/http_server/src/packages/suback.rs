use crate::managers::measuresmanager::MeasuresManager;
use crate::packages::client_packet::ClientPacket;
use shared::packages::suback::Suback;
use std::sync::{Arc, Mutex};

impl ClientPacket for Suback {
    fn handle_packet(&self, _measures_manager: Arc<Mutex<MeasuresManager>>) -> std::io::Result<()> {
        Ok(())
    }
}
