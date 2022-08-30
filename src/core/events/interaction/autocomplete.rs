use std::{mem, sync::Arc};

use crate::{
    commands::help::slash_help,
    core::{events::log_command, Context},
    util::interaction::InteractionCommand,
};

pub async fn handle_autocomplete(ctx: Arc<Context>, mut command: InteractionCommand) {
    let name = mem::take(&mut command.data.name);
    log_command(&ctx, &command, &name);

    let res = match name.as_str() {
        "help" => slash_help(ctx, command).await,
        _ => return error!("unknown autocomplete command `{name}`"),
    };

    if let Err(err) = res {
        error!("failed to process autocomplete `{name}`: {err:?}");
    }
}
