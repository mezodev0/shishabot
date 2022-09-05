use std::{mem, sync::Arc};

use eyre::Context as _;

use crate::{
    core::{events::EventLocation, Context},
    pagination::components::handle_pagination_modal,
    util::{interaction::InteractionModal, Authored},
};

pub async fn handle_modal(ctx: Arc<Context>, mut modal: InteractionModal) {
    let name = mem::take(&mut modal.data.custom_id);

    {
        let username = modal
            .user()
            .map(|u| u.name.as_str())
            .unwrap_or("<unknown user>");

        let location = EventLocation::new(&ctx, &modal);
        info!("[{location}] {username} invoked modal `{name}`");
    }

    let res = match name.as_str() {
        "pagination_page" => handle_pagination_modal(ctx, modal).await,
        _ => return error!("unknown modal `{name}`: {modal:#?}"),
    };

    if let Err(err) = res.with_context(|| format!("failed to process modal `{name}`")) {
        error!("{err:?}");
    }
}
