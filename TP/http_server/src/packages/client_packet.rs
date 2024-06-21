use crate::managers::measuresmanager::MeasuresManager;
use std::sync::{Arc, Mutex};

pub trait ClientPacket: std::fmt::Debug {
    fn handle_packet(&self, measures_manager: Arc<Mutex<MeasuresManager>>) -> std::io::Result<()>;
}
