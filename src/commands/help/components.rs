use std::{fmt::Write, mem};

use eyre::{ContextCompat, Result};
use twilight_interactions::command::{
    ApplicationCommandData, CommandOptionExt, CommandOptionExtInner,
};

use crate::{
    core::{
        commands::slash::{Command, Commands, SlashCommand},
        Context,
    },
    util::{
        builder::{EmbedBuilder, FooterBuilder, MessageBuilder},
        interaction::InteractionComponent,
        Authored, ComponentExt,
    },
};

use super::{generate_menus, option_fields};

const AUTHORITY_STATUS: &str = "Requires authority status";

type PartResult = Result<(Parts, bool)>;

struct Parts {
    name: String,
    help: String,
    options: Vec<CommandOptionExt>,
}

impl From<&'static SlashCommand> for Parts {
    fn from(command: &'static SlashCommand) -> Self {
        let command = (command.create)();

        Self {
            name: command.name,
            help: command.help.unwrap_or(command.description),
            options: command.options,
        }
    }
}

impl From<CommandOptionExt> for Parts {
    fn from(option: CommandOptionExt) -> Self {
        let (name, description, options) = match option.inner {
            CommandOptionExtInner::SubCommand(o) | CommandOptionExtInner::SubCommandGroup(o) => {
                (o.name, o.description, o.options)
            }
            CommandOptionExtInner::String(d) => (d.name, d.description, Vec::new()),
            CommandOptionExtInner::Integer(d) => (d.name, d.description, Vec::new()),
            CommandOptionExtInner::Number(d) => (d.name, d.description, Vec::new()),
            CommandOptionExtInner::Boolean(d) => (d.name, d.description, Vec::new()),
            CommandOptionExtInner::User(d) => (d.name, d.description, Vec::new()),
            CommandOptionExtInner::Channel(d) => (d.name, d.description, Vec::new()),
            CommandOptionExtInner::Role(d) => (d.name, d.description, Vec::new()),
            CommandOptionExtInner::Mentionable(d) => (d.name, d.description, Vec::new()),
            CommandOptionExtInner::Attachment(d) => (d.name, d.description, Vec::new()),
        };

        Self {
            name,
            help: option.help.unwrap_or(description),
            options,
        }
    }
}

impl From<EitherCommand> for Parts {
    fn from(either: EitherCommand) -> Self {
        match either {
            EitherCommand::Base(command) => command.into(),
            EitherCommand::Option(option) => (*option).into(),
        }
    }
}

impl From<CommandIter> for Parts {
    fn from(iter: CommandIter) -> Self {
        match iter.next {
            Some(option) => option.into(),
            None => iter.curr.into(),
        }
    }
}

enum EitherCommand {
    Base(&'static SlashCommand),
    Option(Box<CommandOptionExt>),
}

struct CommandIter {
    curr: EitherCommand,
    next: Option<CommandOptionExt>,
}

impl From<&'static SlashCommand> for CommandIter {
    fn from(command: &'static SlashCommand) -> Self {
        Self {
            curr: EitherCommand::Base(command),
            next: None,
        }
    }
}

impl CommandIter {
    fn next(&mut self, name: &str) -> bool {
        let options = match &mut self.next {
            Some(option) => match &mut option.inner {
                CommandOptionExtInner::SubCommand(o)
                | CommandOptionExtInner::SubCommandGroup(o) => mem::take(&mut o.options),
                _ => return true,
            },
            None => match &mut self.curr {
                EitherCommand::Base(command) => (command.create)().options,
                EitherCommand::Option(option) => match &mut option.inner {
                    CommandOptionExtInner::SubCommand(o)
                    | CommandOptionExtInner::SubCommandGroup(o) => mem::take(&mut o.options),
                    _ => return true,
                },
            },
        };

        let next = match options.into_iter().find(|o| o.inner.name() == name) {
            Some(option) => option,
            None => return true,
        };

        if let Some(curr) = self.next.replace(next) {
            self.curr = EitherCommand::Option(Box::new(curr));
        }

        false
    }
}

pub async fn handle_help_basecommand(ctx: &Context, component: InteractionComponent) -> Result<()> {
    let name = component
        .data
        .values
        .first()
        .context("no menu option was selected")?;

    let cmd = Commands::get()
        .command(name)
        .and_then(|cmd| match cmd {
            Command::Slash(cmd) => Some(cmd),
            Command::Message(_) => None,
        })
        .with_context(|| format!("missing slash command `{name}`"))?;

    let ApplicationCommandData {
        name,
        description,
        help,
        options,
        ..
    } = (cmd.create)();

    let description = help.unwrap_or(description);

    let mut embed = EmbedBuilder::new()
        .title(name)
        .description(description)
        .fields(option_fields(&options));

    if cmd.flags.authority() {
        let footer = FooterBuilder::new(AUTHORITY_STATUS);
        embed = embed.footer(footer);
    }

    let menus = generate_menus(component.user_id()?, &options);
    let builder = MessageBuilder::new().embed(embed).components(menus);

    component.callback(ctx, builder).await?;

    Ok(())
}

pub async fn handle_help_subcommand(
    ctx: &Context,
    mut component: InteractionComponent,
) -> Result<()> {
    let mut title = component
        .message
        .embeds
        .pop()
        .context("missing embed")?
        .title
        .context("missing embed title")?;

    let name = component
        .data
        .values
        .first()
        .with_context(|| format!("missing subcommand for `{title}`"))?;

    let (command, authority) = continue_subcommand(&mut title, name)?;

    // Prepare embed and components
    let mut embed_builder = EmbedBuilder::new()
        .title(title)
        .description(command.help)
        .fields(option_fields(&command.options));

    if authority {
        embed_builder = embed_builder.footer(FooterBuilder::new(AUTHORITY_STATUS));
    }

    let components = generate_menus(component.user_id()?, &command.options);

    let builder = MessageBuilder::new()
        .embed(embed_builder)
        .components(components);

    component.callback(ctx, builder).await?;

    Ok(())
}

fn continue_subcommand(title: &mut String, name: &str) -> PartResult {
    let mut names = title.split(' ');
    let base = names.next().context("missing embed title")?;

    let command = Commands::get()
        .command(base)
        .and_then(|cmd| match cmd {
            Command::Slash(cmd) => Some(cmd),
            Command::Message(_) => None,
        })
        .context("unknown command")?;

    let authority = command.flags.authority();
    let mut iter = CommandIter::from(command);

    for name in names {
        if iter.next(name) {
            bail!("unknown command");
        }
    }

    if iter.next(name) {
        bail!("unknown command");
    }

    let command = Parts::from(iter);
    let _ = write!(title, " {}", command.name);

    Ok((command, authority))
}
