use pumpkin::command::args::Arg;
use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::dispatcher::CommandError;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;

use crate::state::StateManager;

pub struct JoinCommandExecutor {
    pub state_manager: std::sync::Arc<StateManager>,
}

impl CommandExecutor for JoinCommandExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(group_name)) = args.get("group_name") else {
                return Err(CommandError::InvalidConsumption(Some("group_name".into())));
            };

            let password = args.get("password").and_then(|a| {
                if let Arg::Simple(pwd) = a {
                    Some(pwd.to_string())
                } else {
                    None
                }
            });

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

                // Look up group securely
                if let Some(group) = self.state_manager.get_group_by_name(group_name).await {
                    let password_ok = match &group.password {
                        None => true,
                        Some(expected) => password.as_deref() == Some(expected.as_str()),
                    };

                    if password_ok {
                        let old_group = self
                            .state_manager
                            .get_player(&player_uuid)
                            .await
                            .and_then(|p| p.group);

                        self.state_manager
                            .set_player_group(&player_uuid, Some(group.id))
                            .await;

                        let joined_packet = crate::net::JoinedGroupPacket {
                            group: Some(group.id),
                            wrong_password: false,
                        };
                        player
                            .send_custom_payload(
                                "voicechat:joined_group",
                                &joined_packet.to_bytes(),
                            )
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
                                TextComponent::text(format!("Joined group {}", group_name))
                                    .color_named(pumpkin_util::text::color::NamedColor::Green),
                            )
                            .await;
                    } else {
                        let joined_packet = crate::net::JoinedGroupPacket {
                            group: None,
                            wrong_password: true,
                        };
                        player
                            .send_custom_payload(
                                "voicechat:joined_group",
                                &joined_packet.to_bytes(),
                            )
                            .await;

                        let error_msg = if password.is_none() {
                            "Missing password"
                        } else {
                            "Incorrect password"
                        };
                        sender
                            .send_message(
                                TextComponent::text(error_msg)
                                    .color_named(pumpkin_util::text::color::NamedColor::Red),
                            )
                            .await;
                    }
                } else {
                    sender
                        .send_message(
                            TextComponent::text("Group does not exist")
                                .color_named(pumpkin_util::text::color::NamedColor::Red),
                        )
                        .await;
                }
            }

            Ok(1)
        })
    }
}
