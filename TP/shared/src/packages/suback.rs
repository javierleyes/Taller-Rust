use crate::packages::packet::FixedHeader;
use crate::packages::packet::PacketType;
use crate::packages::packet::ReadablePacket;
use crate::packages::packet::WritablePacket;
use std::io::Read;
use std::io::Write;

/// This struct represents a Suback packet
#[derive(Debug, PartialEq)]
pub struct Suback {
    /// A numeric packet identifier
    pub packet_id: u16,
    /// A byte array containing the return codes according to subackd topic filters
    pub return_codes: Vec<u8>,
}

impl ReadablePacket<Suback> for Suback {
    /// Returns a Result with a Suback packet from a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A readable stream to read the packet from
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Suback;
    ///
    /// // assuming an existing readable stream called my_stream
    /// let fixed_header = FixedHeader::read_fixed_header(my_stream)?;
    /// if let packet = Suback::read_from(my_stream, fixed_header) {
    ///     // Do something with the packet (Suback)
    /// }
    ///
    /// ```
    fn read_from(stream: &mut dyn Read, fixed_header: FixedHeader) -> std::io::Result<Suback> {
        const UTF8_LENGTH: u32 = 2;
        // Variable header

        let mut num_buffer = [0u8; 2];
        // packet_id
        stream.read_exact(&mut num_buffer)?;
        let packet_id = u16::from_be_bytes(num_buffer);
        let mut accum_length: u32 = UTF8_LENGTH;

        // payload

        let mut return_codes = Vec::new();

        while fixed_header.remaining_length - accum_length > 0 {
            // return_code
            let mut num_buffer = [0u8; 1];
            stream.read_exact(&mut num_buffer)?;
            let return_code = u8::from_be_bytes(num_buffer);
            accum_length += 1;

            return_codes.push(return_code);
        }

        let suback = Suback {
            packet_id,
            return_codes,
        };

        Ok(suback)
    }
}

impl WritablePacket for Suback {
    /// Writes a Suback packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Suback;
    ///
    /// // assuming an existing writable stream called my_stream
    /// let packet = Suback {
    ///                 packet_id,
    ///                 return_codes,
    ///              };
    /// packet.write_to(my_stream)
    /// ```    
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        // fixed header
        let header = FixedHeader {
            packet_type: PacketType::Suback as u8,
            packet_type_flags: 0x00,
            remaining_length: self.calculate_remaining_length(),
        };
        header.write_fixed_header(stream)?;

        // Variable header

        // packet_id
        let packet_id_be = self.packet_id.to_be_bytes();
        stream.write_all(&packet_id_be)?;

        // payload

        for i in 0..self.return_codes.len() {
            let return_code = self.return_codes[i].to_be_bytes();
            stream.write_all(&return_code)?;
        }

        Ok(())
    }

    fn calculate_remaining_length(&self) -> u32 {
        const PACKET_ID_LENGTH: u32 = 2;
        const RETURN_CODE_SIZE: u32 = 1;

        let mut length: u32 = PACKET_ID_LENGTH;
        length += RETURN_CODE_SIZE * self.return_codes.len() as u32;
        length
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::packet::PacketType;
    use crate::packages::packet::ReadablePacket;
    use crate::packages::suback::FixedHeader;
    use crate::packages::suback::Suback;
    use std::io::BufReader;

    fn generate_mock_suback_header() -> FixedHeader {
        let header = FixedHeader {
            packet_type: PacketType::Suback as u8,
            packet_type_flags: 0x00,
            remaining_length: 3,
        };
        header
    }

    fn generate_mock_suback_packet() -> Suback {
        let packet = Suback {
            packet_id: 10_u16,
            return_codes: vec![0_u8],
        };
        packet
    }

    fn generate_mock_suback_raw(source: Suback) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Variable Header
        buffer.extend_from_slice(&((source.packet_id as u16).to_be_bytes()));

        // Payload
        for i in 0..source.return_codes.len() {
            buffer.extend_from_slice(&((source.return_codes[i] as u8).to_be_bytes()));
        }
        buffer
    }

    #[test]
    fn test_mock_suback_package_valid() {
        let packet = generate_mock_suback_raw(generate_mock_suback_packet());
        let pointer = &packet[..];

        let mut reader = BufReader::new(pointer);

        match Suback::read_from(&mut reader, generate_mock_suback_header()) {
            Ok(_) => {
                assert_eq!(2 + 2, 4)
            }
            Err(e) => {
                panic!("TEST: Error de Suback::read_from: {}", e)
            }
        }
    }
}
