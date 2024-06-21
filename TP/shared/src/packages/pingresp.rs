use crate::packages::packet::FixedHeader;
use crate::packages::packet::PacketType;
use crate::packages::packet::ReadablePacket;
use crate::packages::packet::WritablePacket;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
/// This struct represents a Pingresp packet
pub struct Pingresp {}

impl ReadablePacket<Pingresp> for Pingresp {
    /// Returns a Result with a Pingresp from a given stream
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
    /// if let packet = Pingresp::read_from(my_stream) {
    ///     // Do something with the packet (Pingresp)
    /// }
    ///
    /// ```
    fn read_from(_stream: &mut dyn Read, _fixed_header: FixedHeader) -> std::io::Result<Pingresp> {
        let pingresp = Pingresp {};

        Ok(pingresp)
    }
}

impl WritablePacket for Pingresp {
    /// Writes a Pingresp packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Pingresp;
    ///
    /// // assuming an existing writable stream called my_stream
    /// let packet = Pingresp {
    ///              };
    /// packet.write_to(my_stream)
    /// ```
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        // fixed header
        let header = FixedHeader {
            packet_type: PacketType::Pingresp as u8,
            packet_type_flags: 0x00,
            remaining_length: self.calculate_remaining_length(),
        };
        header.write_fixed_header(stream)?;

        Ok(())
    }

    fn calculate_remaining_length(&self) -> u32 {
        0 // Pingresp always has no variable header and no payload
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::packet::FixedHeader;
    use crate::packages::packet::PacketType;
    use crate::packages::packet::ReadablePacket;
    use crate::packages::pingresp::Pingresp;
    use std::io::BufReader;

    fn generate_mock_pingresp_header() -> FixedHeader {
        let header = FixedHeader {
            packet_type: PacketType::Pingresp as u8,
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
    fn test_mock_pingresp_package_valid() {
        let packet = generate_mock_contentless_packet_raw();
        let pointer = &packet[..];

        let mut reader = BufReader::new(pointer);

        match Pingresp::read_from(&mut reader, generate_mock_pingresp_header()) {
            Ok(_) => {
                assert_eq!(2 + 2, 4)
            }
            Err(e) => {
                panic!("TEST: Error de Pingresp::read_from: {}", e)
            }
        }
    }
}
