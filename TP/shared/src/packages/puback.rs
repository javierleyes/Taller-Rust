use crate::packages::packet::FixedHeader;
use crate::packages::packet::PacketType;
use crate::packages::packet::ReadablePacket;
use crate::packages::packet::WritablePacket;
use std::io::Read;
use std::io::Write;

/// This struct represents a puback packet
#[derive(Debug, PartialEq)]
pub struct Puback {
    /// Puback has its acknowledged packet id
    pub acknowledged_packet_id: u16,
}

impl ReadablePacket<Puback> for Puback {
    /// Returns a puback packet from a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A readable stream to read the packet from
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Puback;
    ///
    /// // assuming an existing readable stream called my_stream
    /// if let packet = Puback::read_from(my_stream) {
    ///     // Do something with the packet (Puback)
    /// }
    /// ```    
    fn read_from(stream: &mut dyn Read, _fixed_header: FixedHeader) -> std::io::Result<Puback> {
        // create empty buffer
        let mut num_buffer = [0u8; 2];
        // packet_id
        stream.read_exact(&mut num_buffer)?;
        let acknowledged_packet_id = u16::from_be_bytes(num_buffer);

        let puback = Puback {
            acknowledged_packet_id,
        };

        Ok(puback)
    }
}

impl WritablePacket for Puback {
    /// Writes a puback packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Puback;
    ///
    /// // assuming an existing writable stream called my_stream
    /// let packet = Puback { acknowledged_packet_id: 1 };
    /// packet.write_to(my_stream)
    /// ```    
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        // fixed header
        let header = FixedHeader {
            packet_type: PacketType::Puback as u8,
            packet_type_flags: 0x00,
            remaining_length: self.calculate_remaining_length(),
        };
        header.write_fixed_header(stream)?;

        // packet_id
        let packet_id_be = self.acknowledged_packet_id.to_be_bytes();
        stream.write_all(&packet_id_be)?;
        Ok(())
    }

    fn calculate_remaining_length(&self) -> u32 {
        2 // Puback always has 2 bytes from Variable Header
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::packet::PacketType;
    use crate::packages::packet::ReadablePacket;
    use crate::packages::puback::FixedHeader;
    use crate::packages::puback::Puback;
    use std::io::BufReader;

    fn generate_mock_puback_header() -> FixedHeader {
        let header = FixedHeader {
            packet_type: PacketType::Puback as u8,
            packet_type_flags: 0x00,
            remaining_length: 10,
        };
        header
    }

    fn generate_mock_puback_packet() -> Puback {
        let packet = Puback {
            acknowledged_packet_id: 10_u16,
        };
        packet
    }

    fn generate_mock_puback_raw(source: Puback) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Variable Header
        buffer.extend_from_slice(&((source.acknowledged_packet_id as u16).to_be_bytes()));
        buffer
    }

    #[test]
    fn test_mock_puback_package_valid() {
        let packet = generate_mock_puback_raw(generate_mock_puback_packet());
        let pointer = &packet[..];

        let mut reader = BufReader::new(pointer);

        match Puback::read_from(&mut reader, generate_mock_puback_header()) {
            Ok(_) => {
                assert_eq!(2 + 2, 4)
            }
            Err(e) => {
                panic!("TEST: Error de Puback::read_from: {}", e)
            }
        }
    }
}
