use pumpkin::command::args::Arg;
use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::dispatcher::CommandError;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin::server::Server;
use pumpkin_util::text::TextComponent;

use crate::state::StateManager;

pub struct InviteCommandExecutor {
    pub state_manager: std::sync::Arc<StateManager>,
}

impl CommandExecutor for InviteCommandExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(target_name)) = args.get("target") else {
                return Err(CommandError::InvalidConsumption(Some("target".into())));
            };

            if let CommandSender::Player(source_player) = sender {
                if !source_player
                    .has_permission(server, "pumpkin_voice:groups")
                    .await
                {
                    sender
                        .send_message(TextComponent::text(
                            "You do not have permission to use voice groups.",
                        ))
                        .await;
                    return Ok(1);
                }

                let source_uuid = source_player.gameprofile.id;

                if let Some(player_state) = self.state_manager.get_player(&source_uuid).await {
                    if let Some(group_id) = player_state.group {
                        if let Some(group) = self.state_manager.get_group(&group_id).await {
                            if let Some(target_player) = server.get_player_by_name(target_name) {
                                let pwd_suffix = group
                                    .password
                                    .map(|p| format!(" {}", p))
                                    .unwrap_or_default();

                                target_player.send_system_message(
                                    &TextComponent::text(format!("{} invited you to group '{}'. Type: /voicechat join {}{}", source_player.gameprofile.name, group.name, group.id, pwd_suffix))
                                        .color_named(pumpkin_util::text::color::NamedColor::Green)
                                ).await;

                                sender
                                    .send_message(
                                        TextComponent::text(format!("Invited {}", target_name))
                                            .color_named(
                                                pumpkin_util::text::color::NamedColor::Green,
                                            ),
                                    )
                                    .await;
                            } else {
                                sender
                                    .send_message(
                                        TextComponent::text("Player not found").color_named(
                                            pumpkin_util::text::color::NamedColor::Red,
                                        ),
                                    )
                                    .await;
                            }
                        }
                    } else {
                        sender
                            .send_message(
                                TextComponent::text("You are not in a group")
                                    .color_named(pumpkin_util::text::color::NamedColor::Red),
                            )
                            .await;
                    }
                }
            }
            Ok(1)
        })
    }
}
