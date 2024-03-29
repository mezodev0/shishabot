use std::{sync::Arc, time::Duration};

use eyre::Result;
use tokio::{
    sync::watch::{self, Receiver, Sender},
    time::sleep,
};
use twilight_model::{
    application::component::{button::ButtonStyle, ActionRow, Button, Component},
    channel::{embed::Embed, ReactionType},
    id::{
        marker::{ChannelMarker, MessageMarker, UserMarker},
        Id,
    },
};

use crate::{
    core::Context,
    util::{
        builder::MessageBuilder, interaction::InteractionCommand, numbers::last_multiple, Authored,
        InteractionCommandExt, MessageExt,
    },
};

pub use self::skin_list::*;

mod skin_list;

pub mod components;

pub enum PaginationKind {
    SkinList(Box<SkinListPagination>),
}

impl PaginationKind {
    async fn build_page(&mut self, _ctx: &Context, pages: &Pages) -> Result<Embed> {
        match self {
            Self::SkinList(kind) => Ok(kind.build_page(pages)),
        }
    }
}

pub struct Pagination {
    pub defer_components: bool,
    pub pages: Pages,
    author: Id<UserMarker>,
    kind: PaginationKind,
    component_kind: ComponentKind,
    tx: Sender<()>,
}

impl Pagination {
    async fn start(
        ctx: Arc<Context>,
        command: InteractionCommand,
        builder: PaginationBuilder,
    ) -> Result<()> {
        let PaginationBuilder {
            mut kind,
            pages,
            attachment,
            content,
            start_by_callback,
            defer_components,
            component_kind,
        } = builder;

        let embed = kind.build_page(&ctx, &pages).await?;
        let components = pages.components(component_kind);

        let mut builder = MessageBuilder::new().embed(embed).components(components);

        if let Some((name, bytes)) = attachment {
            builder = builder.attachment(name, bytes);
        }

        if let Some(content) = content {
            builder = builder.content(content);
        }

        let response_raw = if start_by_callback {
            command.callback(&ctx, builder, false).await?;

            ctx.interaction().response(&command.token).exec().await?
        } else {
            command.update(&ctx, &builder).await?
        };

        if pages.last_index == 0 {
            return Ok(());
        }

        let response = response_raw.model().await?;
        let channel = response.channel_id;
        let msg = response.id;

        let (tx, rx) = watch::channel(());
        Self::spawn_timeout(Arc::clone(&ctx), rx, msg, channel);

        let pagination = Pagination {
            author: command.user_id()?,
            component_kind,
            defer_components,
            kind,
            pages,
            tx,
        };

        ctx.paginations.own(msg).await.insert(pagination);

        Ok(())
    }

    fn is_author(&self, user: Id<UserMarker>) -> bool {
        self.author == user
    }

    fn reset_timeout(&self) {
        let _ = self.tx.send(());
    }

    async fn build(&mut self, ctx: &Context) -> Result<MessageBuilder<'static>> {
        let embed = self.build_page(ctx).await?;
        let components = self.pages.components(self.component_kind);

        Ok(MessageBuilder::new().embed(embed).components(components))
    }

    async fn build_page(&mut self, ctx: &Context) -> Result<Embed> {
        self.kind.build_page(ctx, &self.pages).await
    }

    fn spawn_timeout(
        ctx: Arc<Context>,
        mut rx: Receiver<()>,
        msg: Id<MessageMarker>,
        channel: Id<ChannelMarker>,
    ) {
        static MINUTE: Duration = Duration::from_secs(60);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    res = rx.changed() => if res.is_ok() { continue } else { return },
                    _ = sleep(MINUTE) => {
                        let pagination_active = ctx.paginations.lock(&msg).await.remove().is_some();

                        if pagination_active  {
                            let builder = MessageBuilder::new().components(Vec::new());

                            if let Err(err) = (msg, channel).update(&ctx, &builder).await {
                                warn!("failed to remove components: {err:?}");
                            }
                        }

                        return;
                    },
                }
            }
        });
    }
}

pub struct PaginationBuilder {
    kind: PaginationKind,
    pages: Pages,
    attachment: Option<(String, Vec<u8>)>,
    content: Option<String>,
    start_by_callback: bool,
    defer_components: bool,
    component_kind: ComponentKind,
}

impl PaginationBuilder {
    fn new(kind: PaginationKind, pages: Pages) -> Self {
        Self {
            kind,
            pages,
            attachment: None,
            content: None,
            start_by_callback: true,
            defer_components: false,
            component_kind: ComponentKind::Default,
        }
    }

    /// Start the pagination
    pub async fn start(self, ctx: Arc<Context>, command: InteractionCommand) -> Result<()> {
        Pagination::start(ctx, command, self).await
    }

    #[allow(unused)]
    /// Add an attachment to the initial message which
    /// will stick throughout all pages.
    pub fn attachment(mut self, name: impl Into<String>, bytes: Vec<u8>) -> Self {
        self.attachment = Some((name.into(), bytes));

        self
    }

    #[allow(unused)]
    /// Add content to the initial message which
    /// will stick throughout all pages.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());

        self
    }

    #[allow(unused)]
    /// By default, the initial message will be sent by callback.
    /// This only works if the invoke originates either from a message,
    /// or from an interaction that was **not** deferred.
    ///
    /// If this method is called, the initial message will be sent
    /// through updating meaning it will work for interactions
    /// that have been deferred already.
    pub fn start_by_update(mut self) -> Self {
        self.start_by_callback = false;

        self
    }

    #[allow(unused)]
    /// By default, the page-update message will be sent by callback.
    /// This only works if the page generation is quick enough i.e. <300ms.
    ///
    /// If this method is called, pagination components will be deferred
    /// and then after the page generation updated.
    pub fn defer_components(mut self) -> Self {
        self.defer_components = true;

        self
    }
}

#[derive(Clone, Debug)]
pub struct Pages {
    pub index: usize,
    last_index: usize,
    pub per_page: usize,
}

impl Pages {
    /// `per_page`: How many entries per page
    ///
    /// `amount`: How many entries in total
    pub fn new(per_page: usize, amount: usize) -> Self {
        Self {
            index: 0,
            per_page,
            last_index: last_multiple(per_page, amount),
        }
    }

    pub fn curr_page(&self) -> usize {
        self.index / self.per_page + 1
    }

    pub fn last_page(&self) -> usize {
        self.last_index / self.per_page + 1
    }

    fn components(&self, kind: ComponentKind) -> Vec<Component> {
        match kind {
            ComponentKind::Default => self.default_components(),
        }
    }

    fn default_components(&self) -> Vec<Component> {
        if self.last_index == 0 {
            return Vec::new();
        }

        let jump_start = Button {
            custom_id: Some("pagination_start".to_owned()),
            disabled: self.index == 0,
            emoji: Some(ReactionType::Unicode {
                name: "⏮️".to_owned(),
            }),
            label: None,
            style: ButtonStyle::Secondary,
            url: None,
        };

        let single_step_back = Button {
            custom_id: Some("pagination_back".to_owned()),
            disabled: self.index == 0,
            emoji: Some(ReactionType::Unicode {
                name: "⏪".to_owned(),
            }),
            label: None,
            style: ButtonStyle::Secondary,
            url: None,
        };

        let jump_custom = Button {
            custom_id: Some("pagination_custom".to_owned()),
            disabled: false,
            emoji: Some(ReactionType::Unicode {
                name: "*️⃣".to_owned(),
            }),
            label: None,
            style: ButtonStyle::Secondary,
            url: None,
        };

        let single_step = Button {
            custom_id: Some("pagination_step".to_owned()),
            disabled: self.index == self.last_index,
            emoji: Some(ReactionType::Unicode {
                name: "⏩".to_owned(),
            }),
            label: None,
            style: ButtonStyle::Secondary,
            url: None,
        };

        let jump_end = Button {
            custom_id: Some("pagination_end".to_owned()),
            disabled: self.index == self.last_index,
            emoji: Some(ReactionType::Unicode {
                name: "⏭️".to_owned(),
            }),
            label: None,
            style: ButtonStyle::Secondary,
            url: None,
        };

        let components = vec![
            Component::Button(jump_start),
            Component::Button(single_step_back),
            Component::Button(jump_custom),
            Component::Button(single_step),
            Component::Button(jump_end),
        ];

        vec![Component::ActionRow(ActionRow { components })]
    }
}

#[derive(Copy, Clone)]
enum ComponentKind {
    Default,
}
