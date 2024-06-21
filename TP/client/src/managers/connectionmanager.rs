use std::net::TcpStream;

/// This struct represents the connection state of the client
pub struct ConnectionManager {
    streams: Vec<TcpStream>,
}

impl ConnectionManager {
    /// Returns an empty ConnectionManager
    pub fn new() -> ConnectionManager {
        ConnectionManager {
            streams: Vec::new(),
        }
    }

    /// Add a stream to streams
    /// # Arguments
    ///
    /// * `stream` - A stream
    ///
    pub fn add_client_stream(&mut self, stream: TcpStream) {
        self.streams.push(stream);
    }

    /// Returns the stream
    /// # Arguments    
    ///
    pub fn get_stream(&self) -> Result<TcpStream, u8> {
        if self.has_stream() {
            match self.streams[0].try_clone() {
                Ok(stream) => Ok(stream),
                Err(_e) => Err(1),
            }
        } else {
            Err(1)
        }
    }

    /// Clear the streams vec
    /// # Arguments    
    ///
    pub fn drop_stream(&mut self) {
        self.streams.clear();
    }

    /// Get if the streams vec has elements
    /// # Arguments
    ///
    pub fn has_stream(&self) -> bool {
        !self.streams.is_empty()
    }
}
