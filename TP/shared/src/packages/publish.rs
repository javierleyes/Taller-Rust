use crate::packages::packet::FixedHeader;
use crate::packages::packet::PacketType;
use crate::packages::packet::ReadablePacket;
use crate::packages::packet::WritablePacket;
use crate::utils::read_utf8_string;
use std::io::Read;
use std::io::Write;

/// This struct represents a Publish packet
#[derive(Debug, PartialEq)]
pub struct Publish {
    /// A String containing the topic to publish to
    pub topic_name: String,
    /// A payload String that represents the message to be published
    pub payload: String,
    /// A numeric packet identifier
    pub packet_id: u16,
    /// The QoS level for this packet
    pub qos: u8,
    /// A retain_flag that indicates if the message should be retained or not
    pub retain_flag: u8,
    /// This flag indicates that the message is a duplicate and was resent because the intended recipient (client or broker) did not acknowledge the original message   
    pub dup_flag: u8,
}

impl ReadablePacket<Publish> for Publish {
    /// Returns a Result with a Publish packet from a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A readable stream to read the packet from
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Publish;
    ///
    /// // assuming an existing readable stream called my_stream
    /// if let packet = Publish::read_from(my_stream) {
    ///     // Do something with the packet (Publish)
    /// }
    ///
    /// ```
    fn read_from(stream: &mut dyn Read, fixed_header: FixedHeader) -> std::io::Result<Publish> {
        const UTF8_LENGTH: u32 = 2;

        // flags from fixed header
        let retain_flag = fixed_header.packet_type_flags & 0b00000001_u8;
        let dup_flag = (fixed_header.packet_type_flags & 0b00001000_u8) >> 3;
        let qos = (fixed_header.packet_type_flags & 0b00000110_u8) >> 1;

        // data from stream
        let mut accum_length: u32 = 0;
        let mut num_buffer = [0u8; 2];
        // topic_name
        let topic_name = read_utf8_string(stream)?;
        accum_length += UTF8_LENGTH + topic_name.len() as u32;
        // packet_id
        let mut packet_id = 0_u16;
        if qos > 0 {
            stream.read_exact(&mut num_buffer)?;
            packet_id = u16::from_be_bytes(num_buffer);
            accum_length += 2_u32;
        }

        // payload
        let payload_length = (fixed_header.remaining_length - accum_length) as usize;
        let mut payload = String::new();
        if payload_length > 0 {
            let mut num_buffer = vec![0u8; payload_length];
            stream.read_exact(&mut num_buffer)?;
            payload = std::str::from_utf8(&num_buffer)
                .expect("Error al leer el campo")
                .to_owned();
        }

        let publish = Publish {
            topic_name,
            payload,
            packet_id,
            qos,
            retain_flag,
            dup_flag,
        };

        Ok(publish)
    }
}

impl WritablePacket for Publish {
    /// Writes a Publish packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Publish;
    ///
    /// // assuming an existing writable stream called my_stream
    /// let packet = Publish {
    ///                 topic_name,
    ///                 payload,
    ///                 packet_id,
    ///                 qos,
    ///                 retain_flag,
    ///                 dup_flag,
    ///              };
    /// packet.write_to(my_stream)
    /// ```    
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        // fixed header
        let mut packet_type_flags = self.retain_flag & 0b00000001;
        packet_type_flags |= (self.qos << 1) & 0b00000110;
        packet_type_flags |= (self.dup_flag << 3) & 0b00001000;
        let header = FixedHeader {
            packet_type: PacketType::Publish as u8,
            packet_type_flags,
            remaining_length: self.calculate_remaining_length(),
        };
        header.write_fixed_header(stream)?;

        // topic_name
        let size_be = (self.topic_name.len() as u16).to_be_bytes();
        stream.write_all(&size_be)?;
        stream.write_all(self.topic_name.as_bytes())?;
        // packet_id
        if self.qos > 0 {
            let packet_id_be = self.packet_id.to_be_bytes();
            stream.write_all(&packet_id_be)?;
        }
        // payload
        stream.write_all(self.payload.as_bytes())?;

        Ok(())
    }

    fn calculate_remaining_length(&self) -> u32 {
        const UTF8_LENGTH: u32 = 2;

        let mut length: u32 = UTF8_LENGTH + self.topic_name.len() as u32;
        if self.qos > 0 {
            length += 2; // Packet identifier is a 16 bit number
        }
        length += self.payload.len() as u32;
        length
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::packet::PacketType;
    use crate::packages::packet::ReadablePacket;
    use crate::packages::packet::WritablePacket;
    use crate::packages::publish::FixedHeader;
    use crate::packages::publish::Publish;
    use std::io::BufReader;
    use std::io::BufWriter;
    use std::io::Read;

    fn generate_mock_publish_header() -> FixedHeader {
        let header = FixedHeader {
            packet_type: PacketType::Publish as u8,
            packet_type_flags: 0b00001000_u8,
            remaining_length: 16,
        };
        header
    }

    fn generate_mock_publish_packet() -> Publish {
        let packet = Publish {
            topic_name: "topic".to_owned(),
            payload: "a message".to_owned(),
            packet_id: 0_u16,
            qos: 0_u8,
            retain_flag: 0_u8,
            dup_flag: 1_u8,
        };
        packet
    }

    fn generate_mock_publish_raw(source: Publish, include_header: bool) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Fixed header
        if include_header {
            let header = generate_mock_publish_header();
            let control_packet_byte =
                (header.packet_type << 4) | header.packet_type_flags & 0x0F as u8;
            buffer.extend_from_slice(&((control_packet_byte as u8).to_be_bytes()));
            buffer.extend_from_slice(&((header.remaining_length as u8).to_be_bytes()));
        }

        // Variable Header
        buffer.extend_from_slice(&((source.topic_name.len() as u16).to_be_bytes()));
        buffer.extend_from_slice(source.topic_name.as_bytes());
        if source.qos > 0 {
            buffer.extend_from_slice(&((source.packet_id as u16).to_be_bytes()));
        }

        // Payload
        buffer.extend_from_slice(source.payload.as_bytes());
        buffer
    }

    #[test]
    fn test_mock_publish_package_valid() {
        let packet = generate_mock_publish_raw(generate_mock_publish_packet(), false);
        let pointer = &packet[..];

        let mut reader = BufReader::new(pointer);

        match Publish::read_from(&mut reader, generate_mock_publish_header()) {
            Ok(_) => {
                assert_eq!(2 + 2, 4)
            }
            Err(e) => {
                panic!("TEST: Error de Publish::read_from: {}", e)
            }
        }
    }

    #[test]
    fn test_mock_publish_package_read_valid() {
        let publish_in = generate_mock_publish_packet();
        let packet = generate_mock_publish_raw(publish_in, false);
        let pointer = &packet[..];
        let mut reader = BufReader::new(pointer);

        match Publish::read_from(&mut reader, generate_mock_publish_header()) {
            Ok(publish_out) => {
                assert_eq!(generate_mock_publish_packet(), publish_out);
            }
            Err(e) => {
                panic!("TEST: Error de Publish::read_from : {}", e)
            }
        }
    }

    #[test]
    fn test_mock_publish_package_write_valid() {
        let publish_in = generate_mock_publish_packet();
        let mut writer = BufWriter::new(Vec::new());

        match publish_in.write_to(&mut writer) {
            Ok(_) => {
                assert_eq!(
                    generate_mock_publish_raw(generate_mock_publish_packet(), true),
                    writer.into_inner().unwrap()
                );
            }
            Err(e) => {
                panic!("TEST: Error de Publish::write_to : {}", e)
            }
        }
    }

    #[test]
    fn test_mock_publish_package_echo_valid() {
        let publish_in = generate_mock_publish_packet();
        let mut writer = BufWriter::new(Vec::new());

        match publish_in.write_to(&mut writer) {
            Ok(_) => {
                const FIXED_HEADER_SIZE: usize = 2;

                let packet = writer.into_inner().unwrap();
                let pointer = &packet[..];
                let mut reader = BufReader::new(pointer);

                println!("len: {}, Content: {:?}", packet.len(), packet);
                // Remove Fixed Header
                reader.read_exact(&mut [0u8; FIXED_HEADER_SIZE]).unwrap();

                match Publish::read_from(&mut reader, generate_mock_publish_header()) {
                    Ok(publish_out) => {
                        assert_eq!(generate_mock_publish_packet(), publish_out);
                    }
                    Err(e) => {
                        panic!("TEST: Error de Publish::read_from : {}", e)
                    }
                }
            }
            Err(e) => {
                panic!("TEST: Error de Publish::write_to : {}", e)
            }
        }
    }
}
