use std::{mem, sync::Arc};

use eyre::Context as _;

use crate::{
    commands::help::{handle_help_basecommand, handle_help_subcommand},
    core::{events::EventLocation, Context},
    pagination::components::*,
    util::{interaction::InteractionComponent, Authored},
};

pub async fn handle_component(ctx: Arc<Context>, mut component: InteractionComponent) {
    let name = mem::take(&mut component.data.custom_id);

    {
        let username = component
            .user()
            .map(|u| u.name.as_str())
            .unwrap_or("<unknown user>");

        let location = EventLocation::new(&ctx, &component);
        info!("[{location}] {username} invoked component `{name}`");
    }

    let res = match name.as_str() {
        "help_basecommand" => handle_help_basecommand(&ctx, component).await,
        "help_subcommand" => handle_help_subcommand(&ctx, component).await,
        "pagination_start" => handle_pagination_start(ctx, component).await,
        "pagination_back" => handle_pagination_back(ctx, component).await,
        "pagination_custom" => handle_pagination_custom(ctx, component).await,
        "pagination_step" => handle_pagination_step(ctx, component).await,
        "pagination_end" => handle_pagination_end(ctx, component).await,
        "profile_compact" => handle_profile_compact(ctx, component).await,
        "profile_medium" => handle_profile_medium(ctx, component).await,
        "profile_full" => handle_profile_full(ctx, component).await,
        _ => return error!("unknown message component `{name}`"),
    };

    if let Err(err) = res.with_context(|| format!("failed to process component `{name}`")) {
        error!("{err:?}");
    }
}
