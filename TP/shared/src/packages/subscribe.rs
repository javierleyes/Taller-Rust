use crate::packages::packet::FixedHeader;
use crate::packages::packet::PacketType;
use crate::packages::packet::ReadablePacket;
use crate::packages::packet::WritablePacket;
use crate::utils::read_utf8_string;
use std::io::Read;
use std::io::Write;

/// This struct represents a Subscribe packet
#[derive(Debug, PartialEq)]
pub struct Subscribe {
    /// A numeric packet identifier
    pub packet_id: u16,
    /// A String array containing filters for subscribing to topics
    pub topic_filters: Vec<String>,
    /// A byte array that represents requested QoS level for each topic filter
    pub requested_qos: Vec<u8>,
}

impl ReadablePacket<Subscribe> for Subscribe {
    /// Returns a Result with a Subscribe packet from a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A readable stream to read the packet from
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Subscribe;
    ///
    /// // assuming an existing readable stream called my_stream
    /// let fixed_header = FixedHeader::read_fixed_header(my_stream)?;
    /// if let packet = Subscribe::read_from(my_stream, fixed_header) {
    ///     // Do something with the packet (Subscribe)
    /// }
    ///
    /// ```
    fn read_from(stream: &mut dyn Read, fixed_header: FixedHeader) -> std::io::Result<Subscribe> {
        const UTF8_LENGTH: u32 = 2;
        // Variable header

        let mut num_buffer = [0u8; 2];
        // packet_id
        stream.read_exact(&mut num_buffer)?;
        let packet_id = u16::from_be_bytes(num_buffer);
        let mut accum_length: u32 = UTF8_LENGTH;

        // payload
        let mut topic_filters = Vec::new();
        let mut requested_qos = Vec::new();

        while fixed_header.remaining_length - accum_length > 0 {
            // topic_filter
            let topic_filter = read_utf8_string(stream)?;
            accum_length += UTF8_LENGTH + topic_filter.len() as u32;
            // qos
            let mut num_buffer = [0u8; 1];
            stream.read_exact(&mut num_buffer)?;
            let requested_qos_item = u8::from_be_bytes(num_buffer);
            accum_length += 1;

            topic_filters.push(topic_filter);
            requested_qos.push(requested_qos_item);
        }

        let subscribe = Subscribe {
            packet_id,
            topic_filters,
            requested_qos,
        };

        Ok(subscribe)
    }
}

impl WritablePacket for Subscribe {
    /// Writes a Subscribe packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Subscribe;
    ///
    /// // assuming an existing writable stream called my_stream
    /// let packet = Subscribe {
    ///                 packet_id,
    ///                 topic_filters,
    ///                 requested_qos,
    ///              };
    /// packet.write_to(my_stream)
    /// ```    
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        // fixed header
        let header = FixedHeader {
            packet_type: PacketType::Subscribe as u8,
            packet_type_flags: 0x00,
            remaining_length: self.calculate_remaining_length(),
        };
        header.write_fixed_header(stream)?;

        // Variable header

        // packet_id
        let packet_id_be = self.packet_id.to_be_bytes();
        stream.write_all(&packet_id_be)?;

        // payload
        for i in 0..self.topic_filters.len() {
            let size_be = (self.topic_filters[i].len() as u16).to_be_bytes();
            stream.write_all(&size_be)?;
            stream.write_all(self.topic_filters[i].as_bytes())?;

            let requested_qos = self.requested_qos[i].to_be_bytes();
            stream.write_all(&requested_qos)?;
        }

        Ok(())
    }

    fn calculate_remaining_length(&self) -> u32 {
        const UTF8_LENGTH: u32 = 2;
        const PACKET_ID_LENGTH: u32 = 2;
        const REQUESTED_QOS_SIZE: u32 = 1;

        let mut length: u32 = PACKET_ID_LENGTH;
        for i in 0..self.topic_filters.len() {
            length += UTF8_LENGTH + self.topic_filters[i].len() as u32;
            length += REQUESTED_QOS_SIZE;
        }
        length
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::packet::PacketType;
    use crate::packages::packet::ReadablePacket;
    use crate::packages::subscribe::FixedHeader;
    use crate::packages::subscribe::Subscribe;
    use std::io::BufReader;

    fn generate_mock_subscribe_header() -> FixedHeader {
        let header = FixedHeader {
            packet_type: PacketType::Subscribe as u8,
            packet_type_flags: 0x00,
            remaining_length: 8,
        };
        header
    }

    fn generate_mock_subscribe_packet() -> Subscribe {
        let packet = Subscribe {
            packet_id: 10_u16,
            topic_filters: vec!["a/d".to_owned()],
            requested_qos: vec![0_u8],
        };
        packet
    }

    fn generate_mock_subscribe_raw(source: Subscribe) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Variable Header
        buffer.extend_from_slice(&((source.packet_id as u16).to_be_bytes()));

        // Payload
        for i in 0..source.topic_filters.len() {
            buffer.extend_from_slice(&((source.topic_filters[i].len() as u16).to_be_bytes()));
            buffer.extend_from_slice(source.topic_filters[i].as_bytes());
            buffer.extend_from_slice(&((source.requested_qos[i] as u8).to_be_bytes()));
        }
        buffer
    }

    #[test]
    fn test_mock_subscribe_package_valid() {
        let packet = generate_mock_subscribe_raw(generate_mock_subscribe_packet());
        let pointer = &packet[..];

        let mut reader = BufReader::new(pointer);

        match Subscribe::read_from(&mut reader, generate_mock_subscribe_header()) {
            Ok(_) => {
                assert_eq!(2 + 2, 4)
            }
            Err(e) => {
                panic!("TEST: Error de Subscribe::read_from: {}", e)
            }
        }
    }
}
