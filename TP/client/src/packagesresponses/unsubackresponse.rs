pub struct UnsubackResponse {
    packet_id: u16,
    status_code: u8,
}

impl UnsubackResponse {
    /// Returns an inicialized UnsubackResponse
    pub fn new(packet_id: u16, status_code: u8) -> UnsubackResponse {
        UnsubackResponse {
            packet_id,
            status_code,
        }
    }

    /// Returns the return_code value of the connack response
    pub fn get_status_code(&self) -> u8 {
        self.status_code
    }

    /// Returns the session_present value of the connack response
    pub fn get_packet_id(&self) -> u16 {
        self.packet_id
    }
}
