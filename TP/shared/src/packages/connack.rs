use crate::packages::packet::FixedHeader;
use crate::packages::packet::PacketType;
use crate::packages::packet::ReadablePacket;
use crate::packages::packet::WritablePacket;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
/// This struct represents a Connack packet
pub struct Connack {
    /// This flag contains a return code that tells the client whether the connection attempt was successful or not.
    pub return_code: u8,
    /// This flag tells the client whether the broker already has a persistent session available from previous interactions.
    pub session_present: u8,
}

impl ReadablePacket<Connack> for Connack {
    /// Returns a Result with a Connack from a given stream
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
    /// if let packet = Connack::read_from(my_stream) {
    ///     // Do something with the packet (Connack)
    /// }
    ///
    /// ```
    fn read_from(stream: &mut dyn Read, _fixed_header: FixedHeader) -> std::io::Result<Connack> {
        let mut num_buffer = [0u8; 1];
        // session_present
        stream.read_exact(&mut num_buffer)?;
        let session_present = u8::from_be_bytes(num_buffer);
        // returnCode
        stream.read_exact(&mut num_buffer)?;
        let return_code = u8::from_be_bytes(num_buffer);

        let connack = Connack {
            return_code,
            session_present,
        };

        Ok(connack)
    }
}

impl WritablePacket for Connack {
    /// Writes a Connack packet to a given stream
    ///
    /// # Arguments
    ///
    /// * `stream` - A writable stream to write the packet into
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use packet::Connack;
    ///
    /// // assuming an existing writable stream called my_stream
    /// let packet = Connack {
    ///                 return_code: 0,
    ///                 session_present: 0,
    ///              };
    /// packet.write_to(my_stream)
    /// ```
    fn write_to(&self, stream: &mut dyn Write) -> std::io::Result<()> {
        // fixed header
        let header = FixedHeader {
            packet_type: PacketType::Connack as u8,
            packet_type_flags: 0x00,
            remaining_length: self.calculate_remaining_length(),
        };
        header.write_fixed_header(stream)?;

        // Variable header

        // sessionPresent
        let session_present_be = (self.session_present & 0b00000001_u8).to_be_bytes();
        stream.write_all(&session_present_be)?;
        // returnCode
        let return_code_be = self.return_code.to_be_bytes();
        stream.write_all(&return_code_be)?;
        Ok(())
    }

    fn calculate_remaining_length(&self) -> u32 {
        2 // Connack always has 2 bytes from Variable Header
    }
}
