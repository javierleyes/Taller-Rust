use crate::packages::packet::FixedHeader;
use crate::packages::packet::PacketType;
use crate::packages::packet::ReadablePacket;
use crate::packages::packet::WritablePacket;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
/// This struct represents a Disconnect packet
pub struct Disconnect {}

impl ReadablePacket<Disconnect> for Disconnect {
    /// Returns a Result with a Disconnect from a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A readable stream to read the packet from
    ///
    /// # Examples
    ///
    /// ```ignore
    ///
    /// // assuming an existing readable stream called my_stream
    /// if let packet = Disconnect::read_from(my_stream) {
    ///     // Do something with the packet (Disconnect)
    /// }
    ///
    /// ```
    fn read_from(
        _stream: &mut dyn Read,
        _fixed_header: FixedHeader,
    ) -> std::io::Result<Disconnect> {
        let disconnect = Disconnect {};

        Ok(disconnect)
    }
}

impl WritablePacket for Disconnect {
    /// Writes a Disconnect packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Disconnect;
    ///
    /// // assuming an existing writable stream called my_stream
    /// let packet = Disconnect {
    ///              };
    /// packet.write_to(my_stream)
    /// ```
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        // fixed header
        let header = FixedHeader {
            packet_type: PacketType::Disconnect as u8,
            packet_type_flags: 0x00,
            remaining_length: self.calculate_remaining_length(),
        };
        header.write_fixed_header(stream)?;

        Ok(())
    }

    fn calculate_remaining_length(&self) -> u32 {
        0 // Disconnect always has no variable header and no payload
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::disconnect::Disconnect;
    use crate::packages::packet::FixedHeader;
    use crate::packages::packet::PacketType;
    use crate::packages::packet::ReadablePacket;
    use std::io::BufReader;

    fn generate_mock_disconnect_header() -> FixedHeader {
        let header = FixedHeader {
            packet_type: PacketType::Disconnect as u8,
            packet_type_flags: 0x00,
            remaining_length: 10,
        };
        header
    }

    fn generate_mock_contentless_packet_raw() -> Vec<u8> {
        // This packet has no variable Header or payload
        Vec::new()
    }

    #[test]
    fn test_mock_disconnect_package_valid() {
        let packet = generate_mock_contentless_packet_raw();
        let pointer = &packet[..];

        let mut reader = BufReader::new(pointer);

        match Disconnect::read_from(&mut reader, generate_mock_disconnect_header()) {
            Ok(_) => {
                assert_eq!(2 + 2, 4)
            }
            Err(e) => {
                panic!("TEST: Error de Disconnect::read_from: {}", e)
            }
        }
    }
}
