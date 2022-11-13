use crate::core::Context as TwilightContext;
use crate::pagination::SkinListPagination;
use crate::util::interaction::InteractionCommand;
use eyre::Result;
use std::sync::Arc;

pub async fn list(ctx: Arc<TwilightContext>, command: InteractionCommand) -> Result<()> {
    let skins = ctx
        .skin_list()
        .get()?
        .iter()
        .map(|skin| skin.to_string_lossy().replace('_', " "))
        .collect();

    SkinListPagination::builder(skins).start(ctx, command).await
}
