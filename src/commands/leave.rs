use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;

use crate::state::StateManager;

pub struct LeaveCommandExecutor {
    pub state_manager: std::sync::Arc<StateManager>,
}

impl CommandExecutor for LeaveCommandExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            if let CommandSender::Player(player) = sender {
                if !player.has_permission(_server, "pumpkin_voice:groups").await {
                    sender
                        .send_message(TextComponent::text(
                            "You do not have permission to use voice groups.",
                        ))
                        .await;
                    return Ok(1);
                }

                let player_uuid = player.gameprofile.id;
                let old_group = self
                    .state_manager
                    .get_player(&player_uuid)
                    .await
                    .and_then(|p| p.group);

                self.state_manager
                    .set_player_group(&player_uuid, None)
                    .await;

                let joined_packet = crate::net::JoinedGroupPacket {
                    group: None,
                    wrong_password: false,
                };
                player
                    .send_custom_payload("voicechat:joined_group", &joined_packet.to_bytes())
                    .await;

                if let Some(state) = self.state_manager.get_player(&player_uuid).await {
                    let bc_packet = crate::net::PlayerStatePacket {
                        player_state: &state,
                    };
                    let bc_bytes = bc_packet.to_bytes();
                    for client in _server.get_all_players() {
                        client
                            .send_custom_payload("voicechat:state", &bc_bytes)
                            .await;
                    }
                }

                if let Some(old_id) = old_group {
                    if self.state_manager.remove_if_empty(&old_id).await {
                        let rm_packet = crate::net::RemoveGroupPacket { group: old_id };
                        let rm_bytes = rm_packet.to_bytes();
                        for client in _server.get_all_players() {
                            client
                                .send_custom_payload("voicechat:remove_group", &rm_bytes)
                                .await;
                        }
                    }
                }

                sender
                    .send_message(
                        TextComponent::text("Left group")
                            .color_named(pumpkin_util::text::color::NamedColor::Green),
                    )
                    .await;
            }

            Ok(1)
        })
    }
}
