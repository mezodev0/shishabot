use std::{mem, sync::Arc};

use eyre::Context as _;

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

    if let Err(err) = res.with_context(|| format!("failed to process autocomplete `{name}`")) {
        error!("{err:?}");
    }
}
