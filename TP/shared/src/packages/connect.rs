use crate::packages::packet::FixedHeader;
use crate::packages::packet::PacketType;
use crate::packages::packet::ReadablePacket;
use crate::packages::packet::WritablePacket;
use crate::utils::read_utf8_string;
use std::io::Read;
use std::io::Write;

/// This struct represents a Connect packet
#[derive(Debug, PartialEq)]
pub struct Connect {
    /// The client identifier identifies each MQTT client that connects to a broker
    pub client_id: String,
    /// User name for client authentication and authorization
    pub username: String,
    /// Password for client authentication and authorization
    pub password: String,
    /// Topic to notify other clients when a client disconnects ungracefully.
    pub last_will_topic: String,
    /// Message that notifies other clients when a client disconnects ungracefully.
    pub last_will_message: String,
    /// Time interval in seconds that defines the longest period of time that the broker and client can endure without sending a message
    pub keep_alive: u16,
    /// QoS for last will message
    pub last_will_qos: u8,
    /// It tells the broker whether the client wants to establish a persistent session or not.
    pub clean_session: u8,
    /// Flag to indicate if the last will message has to be retained when it is published.
    pub last_will_retain: u8,
    /// Flag to indicate if last will information is included in the packet
    pub last_will_flag: u8,
}

impl ReadablePacket<Connect> for Connect {
    /// Returns a Result with a Connect from a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A readable stream to read the packet from
    ///
    /// # Examples
    ///
    /// ``` ignore
    ///
    /// // assuming an existing readable stream called my_stream
    /// if let packet = Connect::read_from(my_stream) {
    ///     // Do something with the packet (Connect)
    /// }
    ///
    /// ```
    fn read_from(stream: &mut dyn Read, _fixed_header: FixedHeader) -> std::io::Result<Connect> {
        // Variable Header
        let mut buffer = vec![0; 7_usize];
        stream.read_exact(&mut buffer)?; // Ignore first 7 bytes
        let mut buffer = [0u8; 1];
        stream.read_exact(&mut buffer)?;
        let flags = buffer[0];
        let has_username = (flags & 0b10000000) > 0;
        let has_password = (flags & 0b01000000) > 0;
        let last_will_retain = ((flags & 0b00100000) > 0) as u8;
        let last_will_qos = (flags & 0b00011000) >> 3;
        let last_will_flag = ((flags & 0b00000100) > 0) as u8;
        let clean_session = ((flags & 0b00000010) > 0) as u8;
        let mut num_buffer = [0u8; 2];
        stream.read_exact(&mut num_buffer)?;
        let keep_alive = u16::from_be_bytes(num_buffer);

        // Payload

        // clientId
        let client_id = read_utf8_string(stream)?;
        // lastWillTopic
        let mut last_will_topic = String::new();
        if last_will_flag > 0 {
            last_will_topic = read_utf8_string(stream)?;
        }
        // lastWillMesseage
        let mut last_will_message = String::new();
        if last_will_flag > 0 {
            last_will_message = read_utf8_string(stream)?;
        }
        // username
        let mut username = String::new();
        if has_username {
            username = read_utf8_string(stream)?;
        }
        // password
        let mut password = String::new();
        if has_password {
            password = read_utf8_string(stream)?;
        }

        let connect = Connect {
            client_id,
            username,
            password,
            last_will_topic,
            last_will_message,
            keep_alive,
            last_will_qos,
            clean_session,
            last_will_retain,
            last_will_flag,
        };

        Ok(connect)
    }
}

impl WritablePacket for Connect {
    /// Writes a Connect packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    /// # Examples
    ///
    /// ``` ignore
    /// use packet::Connect;
    ///
    /// // assuming an existing writable stream called my_stream
    /// let packet = Connect {
    ///                 client_id,
    ///                 username,
    ///                 password,
    ///                 last_will_topic,
    ///                 last_will_message,
    ///                 keep_alive,
    ///                 last_will_qos,
    ///                 clean_session,
    ///                 last_will_retain,
    ///                 last_will_flag,
    ///              };
    /// packet.write_to(my_stream)
    /// ```
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        // fixed header
        let header = FixedHeader {
            packet_type: PacketType::Connect as u8,
            packet_type_flags: 0x00,
            remaining_length: self.calculate_remaining_length(),
        };
        header.write_fixed_header(stream)?;

        // Variable Header
        let protocol_name_length = 4_u16;
        stream.write_all(&protocol_name_length.to_be_bytes())?;
        let protocol_name = "MQTT".to_owned();
        stream.write_all(protocol_name.as_bytes())?;
        let protocol_level = 4_u8; // MQTT 3.1.1
        stream.write_all(&protocol_level.to_be_bytes())?;
        let connect_flags = self.get_flags() as u8;
        stream.write_all(&connect_flags.to_be_bytes())?;
        let keep_alive = self.keep_alive as u16;
        stream.write_all(&keep_alive.to_be_bytes())?;

        // clientId
        let size_be = (self.client_id.len() as u16).to_be_bytes();
        stream.write_all(&size_be)?;
        stream.write_all(self.client_id.as_bytes())?;
        if self.last_will_flag > 0 {
            // lastWillTopic
            let size_be = (self.last_will_topic.len() as u16).to_be_bytes();
            stream.write_all(&size_be)?;
            stream.write_all(self.last_will_topic.as_bytes())?;
            // lastWillMesseage
            let size_be = (self.last_will_message.len() as u16).to_be_bytes();
            stream.write_all(&size_be)?;
            stream.write_all(self.last_will_message.as_bytes())?;
        }
        if !self.username.is_empty() {
            // username
            let size_be = (self.username.len() as u16).to_be_bytes();
            stream.write_all(&size_be)?;
            stream.write_all(self.username.as_bytes())?;
        }
        if !self.password.is_empty() {
            // password
            let size_be = (self.password.len() as u16).to_be_bytes();
            stream.write_all(&size_be)?;
            stream.write_all(self.password.as_bytes())?;
        }

        Ok(())
    }

    fn calculate_remaining_length(&self) -> u32 {
        const VARIABLE_HEADER_LENGTH: u32 = 10;
        const UTF8_LENGTH: u32 = 2;

        // Variable Header
        let mut length: u32 = VARIABLE_HEADER_LENGTH;
        // Payload
        length += UTF8_LENGTH + self.client_id.len() as u32;
        if self.last_will_flag > 0 {
            length += UTF8_LENGTH + self.last_will_topic.len() as u32;
            length += UTF8_LENGTH + self.last_will_message.len() as u32;
        }
        if !self.username.is_empty() {
            length += UTF8_LENGTH + self.username.len() as u32;
        }
        if !self.password.is_empty() {
            length += UTF8_LENGTH + self.password.len() as u32;
        }
        length
    }
}

impl Connect {
    pub fn get_flags(&self) -> u8 {
        let mut flags = 0_u8;
        if self.clean_session > 0 {
            flags |= 0b00000010;
        }
        if self.last_will_flag > 0 {
            flags |= 0b00000100;
        }
        if self.last_will_qos > 0 && self.last_will_qos < 3 {
            flags |= self.last_will_qos << 3;
        }
        if self.last_will_retain > 0 {
            flags |= 0b00100000;
        }
        if !self.password.is_empty() {
            flags |= 0b01000000;
        }
        if !self.username.is_empty() {
            flags |= 0b10000000;
        }

        flags
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::connect::Connect;
    use crate::packages::packet::FixedHeader;
    use crate::packages::packet::PacketType;
    use crate::packages::packet::ReadablePacket;
    use std::io::BufReader;

    fn generate_mock_connect_header() -> FixedHeader {
        let header = FixedHeader {
            packet_type: PacketType::Connect as u8,
            packet_type_flags: 0x00,
            remaining_length: 10,
        };
        header
    }

    fn generate_mock_connect_packet() -> Connect {
        let connect = Connect {
            client_id: "ID".to_owned(),
            username: "user".to_owned(),
            password: "passwd".to_owned(),
            last_will_topic: "lastWillTopic".to_owned(),
            last_will_message: "lastWillMessage".to_owned(),
            keep_alive: 10 as u16,
            last_will_qos: 0 as u8,
            clean_session: 0 as u8,
            last_will_retain: 0 as u8,
            last_will_flag: 1 as u8,
        };
        connect
    }

    fn generate_mock_connect_raw(source: Connect) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Variable Header
        let protocol_name_length = 4 as u16;
        buffer.extend_from_slice(&protocol_name_length.to_be_bytes());
        let protocol_name = "MQTT".to_owned();
        buffer.extend_from_slice(protocol_name.as_bytes());
        let protocol_level = 4 as u8; // MQTT 3.1.1
        buffer.extend_from_slice(&protocol_level.to_be_bytes());
        let connect_flags = 0b11000100 as u8;
        buffer.extend_from_slice(&connect_flags.to_be_bytes());
        let keep_alive = 10 as u16; // time in seconds
        buffer.extend_from_slice(&keep_alive.to_be_bytes());

        // Payload
        buffer.extend_from_slice(&((source.client_id.len() as u16).to_be_bytes()));
        buffer.extend_from_slice(source.client_id.as_bytes());
        buffer.extend_from_slice(&((source.last_will_topic.len() as u16).to_be_bytes()));
        buffer.extend_from_slice(source.last_will_topic.as_bytes());
        buffer.extend_from_slice(&((source.last_will_message.len() as u16).to_be_bytes()));
        buffer.extend_from_slice(source.last_will_message.as_bytes());
        buffer.extend_from_slice(&((source.username.len() as u16).to_be_bytes()));
        buffer.extend_from_slice(source.username.as_bytes());
        buffer.extend_from_slice(&((source.password.len() as u16).to_be_bytes()));
        buffer.extend_from_slice(source.password.as_bytes());
        buffer
    }

    #[test]
    fn test_mock_connect_package_valid() {
        let packet = generate_mock_connect_raw(generate_mock_connect_packet());
        let pointer = &packet[..];

        let mut reader = BufReader::new(pointer);

        match Connect::read_from(&mut reader, generate_mock_connect_header()) {
            Ok(_) => {
                assert_eq!(2 + 2, 4)
            }
            Err(e) => {
                panic!("TEST: Error de handler client: {}", e)
            }
        }
    }

    #[test]
    fn test_mock_connect_package_read_valid() {
        let connect_in = generate_mock_connect_packet();
        let packet = generate_mock_connect_raw(connect_in);
        let pointer = &packet[..];
        let mut reader = BufReader::new(pointer);

        match Connect::read_from(&mut reader, generate_mock_connect_header()) {
            Ok(connect_out) => {
                assert_eq!(generate_mock_connect_packet(), connect_out);
            }
            Err(e) => {
                panic!("TEST: Error de Connect::read_from : {}", e)
            }
        }
    }

    #[test]
    fn test_mock_connect_package_invalid() {
        let packet = generate_mock_connect_raw(generate_mock_connect_packet());
        let pointer = &packet[1..];

        let mut reader = BufReader::new(pointer);

        match Connect::read_from(&mut reader, generate_mock_connect_header()) {
            Ok(_) => {
                panic!("TEST: paquete Connect mal formado deberia fallar")
            }
            Err(_) => {
                assert_eq!(2 + 2, 4);
            }
        }
    }
}
