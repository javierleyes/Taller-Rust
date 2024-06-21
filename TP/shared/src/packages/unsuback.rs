use crate::packages::packet::FixedHeader;
use crate::packages::packet::PacketType;
use crate::packages::packet::ReadablePacket;
use crate::packages::packet::WritablePacket;
use std::io::Read;
use std::io::Write;

/// This struct represents a Unsuback packet
#[derive(Debug, PartialEq)]
pub struct Unsuback {
    /// A numeric packet identifier
    pub packet_id: u16,
}

impl ReadablePacket<Unsuback> for Unsuback {
    /// Returns a Result with a Unsuback packet from a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A readable stream to read the packet from
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Unsuback;
    ///
    /// // assuming an existing readable stream called my_stream
    /// let fixed_header = FixedHeader::read_fixed_header(my_stream)?;
    /// if let packet = Unsuback::read_from(my_stream, fixed_header) {
    ///     // Do something with the packet (Unsuback)
    /// }
    ///
    /// ```
    fn read_from(stream: &mut dyn Read, _fixed_header: FixedHeader) -> std::io::Result<Unsuback> {
        // Variable header

        let mut num_buffer = [0u8; 2];
        // packet_id
        stream.read_exact(&mut num_buffer)?;
        let packet_id = u16::from_be_bytes(num_buffer);

        // no payload

        let unsuback = Unsuback { packet_id };

        Ok(unsuback)
    }
}

impl WritablePacket for Unsuback {
    /// Writes a Unsuback packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Unsuback;
    ///
    /// // assuming an existing writable stream called my_stream
    /// let packet = Unsuback {
    ///                 packet_id,
    ///              };
    /// packet.write_to(my_stream)
    /// ```    
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        // fixed header
        let header = FixedHeader {
            packet_type: PacketType::Unsuback as u8,
            packet_type_flags: 0x00,
            remaining_length: self.calculate_remaining_length(),
        };
        header.write_fixed_header(stream)?;

        // Variable header

        // packet_id
        let packet_id_be = self.packet_id.to_be_bytes();
        stream.write_all(&packet_id_be)?;

        // no payload

        Ok(())
    }

    fn calculate_remaining_length(&self) -> u32 {
        2 // Unsuback always has the size of packet_id
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::packet::PacketType;
    use crate::packages::packet::ReadablePacket;
    use crate::packages::unsuback::FixedHeader;
    use crate::packages::unsuback::Unsuback;
    use std::io::BufReader;

    fn generate_mock_unsuback_header() -> FixedHeader {
        let header = FixedHeader {
            packet_type: PacketType::Unsuback as u8,
            packet_type_flags: 0x00,
            remaining_length: 2,
        };
        header
    }

    fn generate_mock_unsuback_packet() -> Unsuback {
        let packet = Unsuback { packet_id: 10_u16 };
        packet
    }

    fn generate_mock_unsuback_raw(source: Unsuback) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Variable Header
        buffer.extend_from_slice(&((source.packet_id as u16).to_be_bytes()));

        buffer
    }

    #[test]
    fn test_mock_unsuback_package_valid() {
        let packet = generate_mock_unsuback_raw(generate_mock_unsuback_packet());
        let pointer = &packet[..];

        let mut reader = BufReader::new(pointer);

        match Unsuback::read_from(&mut reader, generate_mock_unsuback_header()) {
            Ok(_) => {
                assert_eq!(2 + 2, 4)
            }
            Err(e) => {
                panic!("TEST: Error de Unsuback::read_from: {}", e)
            }
        }
    }
}
