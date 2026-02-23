pub mod invite;
pub mod join;
pub mod leave;

use pumpkin::command::args::simple::SimpleArgConsumer;
use pumpkin::command::tree::builder::{argument, literal};
use pumpkin::command::tree::CommandTree;

use crate::state::StateManager;

const NAMES: [&str; 2] = ["voicechat", "vc"];
const DESCRIPTION: &str = "Manage simple voice chat settings.";

pub fn init_command_tree(state_manager: std::sync::Arc<StateManager>) -> CommandTree {
    let join_executor = join::JoinCommandExecutor {
        state_manager: state_manager.clone(),
    };
    let join_executor_with_pwd = join::JoinCommandExecutor {
        state_manager: state_manager.clone(),
    };
    let leave_executor = leave::LeaveCommandExecutor {
        state_manager: state_manager.clone(),
    };
    let invite_executor = invite::InviteCommandExecutor { state_manager };

    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("join").then(
                argument("group_name", SimpleArgConsumer)
                    .execute(join_executor)
                    .then(argument("password", SimpleArgConsumer).execute(join_executor_with_pwd)),
            ),
        )
        .then(literal("leave").execute(leave_executor))
        .then(
            literal("invite").then(argument("target", SimpleArgConsumer).execute(invite_executor)),
        )
}
