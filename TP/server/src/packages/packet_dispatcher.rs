use crate::packages::server_packet::PacketError;
use crate::packages::server_packet::ServerPacket;
use shared::packages::connect::Connect;
use shared::packages::disconnect::Disconnect;
use shared::packages::packet::FixedHeader;
use shared::packages::packet::PacketType;
use shared::packages::packet::ReadablePacket;
use shared::packages::pingreq::Pingreq;
use shared::packages::puback::Puback;
use shared::packages::publish::Publish;
use shared::packages::subscribe::Subscribe;
use shared::packages::unsubscribe::Unsubscribe;
use std::io::Read;

/// Returns an heap-allocated mqtt packet from a readable object
/// # Arguments
///
/// * `stream` - a readable object
///
/// # Examples
///
/// ```
/// // This gets a Box with a ServerPacket
/// let packet = read_utf8_string(my_stream)?;
/// ```
pub fn dispatch_packet(stream: &mut dyn Read) -> Result<Box<dyn ServerPacket>, PacketError> {
    let fixed_header = FixedHeader::read_fixed_header(stream)?;

    match PacketType::from_u8(fixed_header.packet_type) {
        Some(PacketType::Connect) => Ok(Box::new(Connect::read_from(stream, fixed_header)?)),
        Some(PacketType::Publish) => Ok(Box::new(Publish::read_from(stream, fixed_header)?)),
        Some(PacketType::Puback) => Ok(Box::new(Puback::read_from(stream, fixed_header)?)),
        Some(PacketType::Subscribe) => Ok(Box::new(Subscribe::read_from(stream, fixed_header)?)),
        Some(PacketType::Unsubscribe) => {
            Ok(Box::new(Unsubscribe::read_from(stream, fixed_header)?))
        }
        Some(PacketType::Pingreq) => Ok(Box::new(Pingreq::read_from(stream, fixed_header)?)),
        Some(PacketType::Disconnect) => Ok(Box::new(Disconnect::read_from(stream, fixed_header)?)),
        _ => Ok(Box::new(Connect::read_from(stream, fixed_header)?)),
    }
}
