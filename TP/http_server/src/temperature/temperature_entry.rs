use std::time::SystemTime;

#[derive(Clone)]
pub struct TemperatureEntry {
    pub measured_at: SystemTime,
    pub value: f32,
}
