use crate::managers::measuresmanager::MeasuresManager;
use crate::packages::client_packet::ClientPacket;
use crate::TemperatureEntry;
use shared::packages::publish::Publish;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

impl ClientPacket for Publish {
    fn handle_packet(&self, measures_manager: Arc<Mutex<MeasuresManager>>) -> std::io::Result<()> {
        let mut measures_manager_local = measures_manager.lock().unwrap();
        let new_measure = TemperatureEntry {
            measured_at: SystemTime::now(),
            value: self.payload.parse::<f32>().unwrap(),
        };
        measures_manager_local.add_new_measure(new_measure);

        drop(measures_manager_local);
        Ok(())
    }
}
