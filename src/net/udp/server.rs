use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{error, info};
use uuid::Uuid;

use super::crypto::send_packet;
use crate::net::voice_packets::{
    AuthenticateAckPacket, AuthenticatePacket, ConnectionCheckAckPacket, VoicePacket,
};
use crate::state::StateManager;
use crate::util::buf_ext::BufExt;

pub struct UdpServer {
    state_manager: Arc<StateManager>,
    server: Arc<pumpkin::server::Server>,
}

// TODO: Make this less ugly

impl UdpServer {
    pub fn new(state_manager: Arc<StateManager>, server: Arc<pumpkin::server::Server>) -> Self {
        Self {
            state_manager,
            server,
        }
    }

    pub async fn start(&self, addr: &str) -> Result<(), std::io::Error> {
        let socket = Arc::new(UdpSocket::bind(addr).await?);
        info!("Voice chat UDP server listening on {}", addr);

        // Spawn a keep-alive background task
        let keep_alive_socket = socket.clone();
        let keep_alive_state = self.state_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
            loop {
                interval.tick().await;
                let targets = keep_alive_state.get_keep_alive_targets().await;
                for (target, secret) in targets {
                    let _ = send_packet(
                        &keep_alive_socket,
                        target,
                        VoicePacket::KeepAlive(crate::net::voice_packets::KeepAlivePacket),
                        &secret,
                    )
                    .await;
                }
            }
        });

        let mut buf = [0; 2048];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, src)) => {
                    let data = buf[..len].to_vec();
                    if data.len() < 17 {
                        continue;
                    }
                    if data[0] != 0xFF {
                        continue;
                    }

                    let state_manager = self.state_manager.clone();
                    let server = self.server.clone();
                    let socket = socket.clone();

                    tokio::spawn(async move {
                        let mut uuid_bytes = [0u8; 16];
                        uuid_bytes.copy_from_slice(&data[1..17]);
                        let player_id = Uuid::from_bytes(uuid_bytes);

                        if let Some(player_state) = state_manager.get_player(&player_id).await {
                            // In Java, payload is read via readByteArray which reads a VarInt len then bytes.
                            let mut payload_buf = &data[17..];
                            let payload_bytes = payload_buf.get_byte_array();

                            if let Ok(decrypted) = player_state.secret.decrypt(&payload_bytes) {
                                if decrypted.is_empty() {
                                    return;
                                }
                                let packet_type = decrypted[0];
                                let mut packet_data = &decrypted[1..];

                                match packet_type {
                                    0x1 => {
                                        let mic_packet =
                                            crate::net::voice_packets::MicPacket::from_bytes(
                                                &mut packet_data,
                                            );
                                        let all_players = state_manager.get_all_players().await;

                                        // Verify Speak Permission
                                        let sender_pl = match server.get_player_by_uuid(player_id) {
                                            Some(p) => p,
                                            None => return,
                                        };

                                        if !sender_pl
                                            .has_permission(&server, "pumpkin_voice:speak")
                                            .await
                                        {
                                            return;
                                        }

                                        // Gamemode and spectator check
                                        let is_spectator = matches!(
                                            sender_pl.gamemode.load(),
                                            pumpkin_util::GameMode::Spectator
                                        );

                                        if is_spectator
                                            && !crate::config::CONFIG.spectator_interaction
                                        {
                                            return;
                                        }

                                        if let Some(group_id) = player_state.group {
                                            let group_packet =
                                                crate::net::voice_packets::GroupSoundPacket {
                                                    channel_id: group_id,
                                                    sender: player_id,
                                                    data: mic_packet.data.clone(),
                                                    sequence_number: mic_packet.sequence_number,
                                                    category: None,
                                                };
                                            for receiver in all_players {
                                                if receiver.uuid == player_id {
                                                    continue;
                                                }
                                                if receiver.group == Some(group_id) {
                                                    if let Some(addr) = receiver.socket_addr {
                                                        if let Some(recv_pl) =
                                                            server.get_player_by_uuid(receiver.uuid)
                                                        {
                                                            if recv_pl
                                                                .has_permission(
                                                                    &server,
                                                                    "pumpkin_voice:listen",
                                                                )
                                                                .await
                                                            {
                                                                let _ = send_packet(
                                                                    &socket,
                                                                    addr,
                                                                    VoicePacket::GroupSound(
                                                                        group_packet.clone(),
                                                                    ),
                                                                    &receiver.secret,
                                                                )
                                                                .await;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            let pos_a = sender_pl.living_entity.entity.pos.load();

                                            let distance_config = if mic_packet.whispering {
                                                crate::config::CONFIG.whisper_distance
                                            } else {
                                                crate::config::CONFIG.max_voice_distance
                                            };

                                            let broadcast_range =
                                                if crate::config::CONFIG.broadcast_range < 0.0 {
                                                    crate::config::CONFIG.max_voice_distance + 1.0
                                                } else {
                                                    crate::config::CONFIG.broadcast_range
                                                }
                                                .max(distance_config);

                                            let distance_sq = broadcast_range.powi(2);

                                            for receiver in all_players {
                                                if receiver.uuid == player_id {
                                                    continue;
                                                }
                                                if let Some(addr) = receiver.socket_addr {
                                                    if let Some(recv_pl) =
                                                        server.get_player_by_uuid(receiver.uuid)
                                                    {
                                                        // Same world check
                                                        if std::sync::Arc::ptr_eq(
                                                            &sender_pl.world(),
                                                            &recv_pl.world(),
                                                        ) && recv_pl
                                                            .has_permission(
                                                                &server,
                                                                "pumpkin_voice:listen",
                                                            )
                                                            .await
                                                        {
                                                            let pos_b = recv_pl
                                                                .living_entity
                                                                .entity
                                                                .pos
                                                                .load();
                                                            let dist = (pos_a.x - pos_b.x).powi(2)
                                                                + (pos_a.y - pos_b.y).powi(2)
                                                                + (pos_a.z - pos_b.z).powi(2);

                                                            if dist <= distance_sq {
                                                                let sound_packet = crate::net::voice_packets::PlayerSoundPacket {
                                                                        channel_id: player_id,
                                                                        sender: player_id,
                                                                        data: mic_packet.data.clone(),
                                                                        sequence_number: mic_packet.sequence_number,
                                                                        distance: distance_config as f32,
                                                                        whispering: mic_packet.whispering,
                                                                        category: None,
                                                                    };
                                                                let _ = send_packet(
                                                                    &socket,
                                                                    addr,
                                                                    VoicePacket::PlayerSound(
                                                                        sound_packet,
                                                                    ),
                                                                    &receiver.secret,
                                                                )
                                                                .await;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    0x5 => {
                                        // AuthenticatePacket
                                        let auth_packet =
                                            AuthenticatePacket::from_bytes(&mut packet_data);
                                        if auth_packet.secret.to_bytes()
                                            == player_state.secret.to_bytes()
                                        {
                                            info!(
                                                "Successfully authenticated player {}",
                                                auth_packet.player_uuid
                                            );
                                            state_manager.update_player_addr(&player_id, src).await;

                                            let _ = send_packet(
                                                &socket,
                                                src,
                                                VoicePacket::AuthenticateAck(AuthenticateAckPacket),
                                                &player_state.secret,
                                            )
                                            .await;
                                        }
                                    }
                                    0x7 => {
                                        // Ping packet, server just echoes back Pong
                                        // which is PingPacket back directly
                                        let _ = send_packet(
                                            &socket,
                                            src,
                                            VoicePacket::Ping(
                                                crate::net::voice_packets::PingPacket::from_bytes(
                                                    &mut packet_data,
                                                ),
                                            ),
                                            &player_state.secret,
                                        )
                                        .await;
                                    }
                                    0x8 => {
                                        // KeepAlivePacket
                                    }
                                    0x9 => {
                                        // ConnectionCheckPacket
                                        info!("Validated connection of player {}", player_id);
                                        let _ = send_packet(
                                            &socket,
                                            src,
                                            VoicePacket::ConnectionCheckAck(
                                                ConnectionCheckAckPacket,
                                            ),
                                            &player_state.secret,
                                        )
                                        .await;
                                    }
                                    _ => {}
                                }
                            } else {
                                error!("Failed to decrypt packet from {:?}", src);
                            }
                        } else {
                            // Secret not found, could handle ping here if ping handler needs it
                        }
                    });
                }
                Err(e) => {
                    error!("Error receiving from UDP socket: {}", e);
                }
            }
        }
    }
}
