pub struct ConnackResponse {
    return_code: u8,
    session_present: u8,
}

impl ConnackResponse {
    /// Returns an inicialized ConnackResponse
    pub fn new(return_code: u8, session_present: u8) -> ConnackResponse {
        ConnackResponse {
            return_code,
            session_present,
        }
    }

    /// Returns the return_code value of the connack response
    pub fn get_return_code(&self) -> u8 {
        self.return_code
    }

    /// Returns the session_present value of the connack response
    pub fn get_session_present(&self) -> u8 {
        self.session_present
    }
}
