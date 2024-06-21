pub struct SubackResponse {
    status_codes: Vec<u8>,
}

impl SubackResponse {
    /// Returns an inicialized SubackResponse
    pub fn new(status_codes: Vec<u8>) -> SubackResponse {
        SubackResponse { status_codes }
    }

    /// Returns the return_code value of the connack response
    pub fn get_status_codes(&self) -> Vec<u8> {
        self.status_codes.clone()
    }
}
