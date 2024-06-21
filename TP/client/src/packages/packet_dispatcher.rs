use crate::packages::client_packet::ClientPacket;
use shared::packages::connack::Connack;
use shared::packages::packet::FixedHeader;
use shared::packages::packet::PacketType;
use shared::packages::packet::ReadablePacket;
use shared::packages::pingresp::Pingresp;
use shared::packages::puback::Puback;
use shared::packages::publish::Publish;
use shared::packages::suback::Suback;
use shared::packages::unsuback::Unsuback;
use std::io::Read;

/// Returns an heap-allocated mqtt packet from a readable object
/// # Arguments
///
/// * `stream` - a readable object
///
/// # Examples
///
/// ```
/// // This gets a Box with a ClientPacket
/// let packet = read_utf8_string(my_stream)?;
/// ```
pub fn dispatch_packet(stream: &mut dyn Read) -> std::io::Result<Box<dyn ClientPacket>> {
    let fixed_header = FixedHeader::read_fixed_header(stream)?;

    match PacketType::from_u8(fixed_header.packet_type) {
        Some(PacketType::Publish) => Ok(Box::new(Publish::read_from(stream, fixed_header)?)),
        Some(PacketType::Connack) => Ok(Box::new(Connack::read_from(stream, fixed_header)?)),
        Some(PacketType::Puback) => Ok(Box::new(Puback::read_from(stream, fixed_header)?)),
        Some(PacketType::Suback) => Ok(Box::new(Suback::read_from(stream, fixed_header)?)),
        Some(PacketType::Unsuback) => Ok(Box::new(Unsuback::read_from(stream, fixed_header)?)),
        Some(PacketType::Pingresp) => Ok(Box::new(Pingresp::read_from(stream, fixed_header)?)),
        _ => Ok(Box::new(Publish::read_from(stream, fixed_header)?)),
    }
}
