use bytes::{BufMut, BytesMut};
use tokio::net::UdpSocket;

use crate::net::voice_packets::VoicePacket;
use crate::state::Secret;
use crate::util::buf_ext::BufMutExt;

pub async fn send_packet(
    socket: &UdpSocket,
    target: std::net::SocketAddr,
    packet: VoicePacket,
    secret: &Secret,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut inner_buf = BytesMut::new();
    inner_buf.put_u8(packet.get_type_id());
    match &packet {
        VoicePacket::AuthenticateAck(p) => p.to_bytes(&mut inner_buf),
        VoicePacket::ConnectionCheckAck(p) => p.to_bytes(&mut inner_buf),
        VoicePacket::PlayerSound(p) => p.to_bytes(&mut inner_buf),
        VoicePacket::GroupSound(p) => p.to_bytes(&mut inner_buf),
        VoicePacket::LocationSound(p) => p.to_bytes(&mut inner_buf),
        VoicePacket::Ping(p) => p.to_bytes(&mut inner_buf),
        VoicePacket::KeepAlive(p) => p.to_bytes(&mut inner_buf),
        _ => {} // Other packets
    }

    let encrypted = secret.encrypt(&inner_buf).map_err(|_| "Encryption error")?;

    let mut final_buf = BytesMut::new();
    final_buf.put_u8(0xFF);
    final_buf.put_byte_array(&encrypted);

    socket.send_to(&final_buf, target).await?;
    Ok(())
}
