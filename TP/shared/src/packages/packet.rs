use std::io::Read;
use std::io::Write;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub enum PacketType {
    Connect = 1,
    Connack = 2,
    Publish = 3,
    Puback = 4,
    Subscribe = 8,
    Suback = 9,
    Unsubscribe = 10,
    Unsuback = 11,
    Pingreq = 12,
    Pingresp = 13,
    Disconnect = 14,
}

impl PacketType {
    pub fn from_u8(n: u8) -> Option<PacketType> {
        match n {
            1 => Some(PacketType::Connect),
            2 => Some(PacketType::Connack),
            3 => Some(PacketType::Publish),
            4 => Some(PacketType::Puback),
            8 => Some(PacketType::Subscribe),
            9 => Some(PacketType::Suback),
            10 => Some(PacketType::Unsubscribe),
            11 => Some(PacketType::Unsuback),
            12 => Some(PacketType::Pingreq),
            13 => Some(PacketType::Pingresp),
            14 => Some(PacketType::Disconnect),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FixedHeader {
    pub packet_type: u8,
    pub packet_type_flags: u8,
    pub remaining_length: u32,
}

impl FixedHeader {
    pub fn read_fixed_header(stream: &mut dyn Read) -> std::io::Result<FixedHeader> {
        let mut num_buffer = [0u8; 1];
        stream.read_exact(&mut num_buffer)?;
        let packet_type = num_buffer[0] >> 4;
        let packet_type_flags = num_buffer[0] & 0x0F;
        let remaining_length = FixedHeader::decode_remaining_length(stream)?;

        let fixed_header = FixedHeader {
            packet_type,
            packet_type_flags,
            remaining_length,
        };

        Ok(fixed_header)
    }

    pub fn write_fixed_header(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        let control_packet_byte = (self.packet_type << 4) | ((self.packet_type_flags & 0x0F) as u8);
        let buffer = control_packet_byte.to_be_bytes();
        stream.write_all(&buffer)?;

        FixedHeader::encode_remaining_length(stream, self.remaining_length)?;

        Ok(())
    }

    pub fn decode_remaining_length(stream: &mut dyn Read) -> std::io::Result<u32> {
        let mut num_buffer = [0u8; 1];
        let mut remaining_length = 0_u32;
        let mut multiplier = 1_u32;

        loop {
            stream.read_exact(&mut num_buffer)?;
            remaining_length += (num_buffer[0] as u32 & 0x7F) * multiplier;
            if multiplier > (1 << 21) {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid Remaining length",
                ));
            }
            multiplier <<= 7;

            if (num_buffer[0] & 0x80) == 0 {
                break;
            }
        }
        Ok(remaining_length)
    }

    pub fn encode_remaining_length(
        stream: &mut dyn Write,
        remaining_length: u32,
    ) -> std::io::Result<()> {
        let mut num_buffer: [u8; 1];
        let mut pending_remaining_length: u32 = remaining_length;

        loop {
            num_buffer = ((pending_remaining_length & 0x7F) as u8).to_be_bytes();
            pending_remaining_length >>= 7;

            // if there are more data to encode, set the top bit of this byte
            if pending_remaining_length > 0 {
                num_buffer[0] |= 0x80;
            }

            stream.write_all(&num_buffer)?;

            if !pending_remaining_length > 0 {
                break;
            }
        }
        Ok(())
    }
}

pub trait ReadablePacket<T> {
    /// Returns a Result with a T from a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A readable stream to read the packet from
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // assuming an existing readable stream called my_stream
    /// if let packet = T::read_from(my_stream) {
    ///     // Do something with the packet (T)
    /// }
    /// ```
    fn read_from(stream: &mut dyn Read, fixed_header: FixedHeader) -> std::io::Result<T>;
}

pub trait WritablePacket {
    /// Writes a packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()>;

    fn calculate_remaining_length(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use crate::packages::packet::FixedHeader;
    use std::io::BufReader;

    #[test]
    fn test_decode_remaining_length_1byte_valid() {
        const EXPECTED_LENGTH: u8 = 10;

        let buffer: [u8; 1] = [EXPECTED_LENGTH];
        let pointer = &buffer[..];

        let mut reader = BufReader::new(pointer);

        match FixedHeader::decode_remaining_length(&mut reader) {
            Ok(remaining_length) => {
                assert_eq!(remaining_length, EXPECTED_LENGTH as u32)
            }
            Err(e) => {
                panic!("TEST: Error al decodificar longitud: {}", e)
            }
        }
    }
}
