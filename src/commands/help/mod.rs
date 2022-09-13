use twilight_interactions::command::{CommandOptionExt, CommandOptionExtInner};
use twilight_model::{
    application::{
        command::Command,
        component::{select_menu::SelectMenuOption, ActionRow, Component, SelectMenu},
    },
    channel::embed::EmbedField,
    id::{marker::UserMarker, Id},
};

use crate::core::{commands::slash::Commands, BotConfig};

pub use self::{
    components::{handle_help_basecommand, handle_help_subcommand},
    interaction::{slash_help, Help, HELP_SLASH},
};

mod components;
mod interaction;

fn generate_menus(user: Id<UserMarker>, options: &[CommandOptionExt]) -> Vec<Component> {
    let base_options: Vec<_> = Commands::get().filter_collect(|c| {
        let Command {
            name, description, ..
        } = c.create();

        if description.is_empty() || (name == "owner" && !BotConfig::get().owners.contains(&user)) {
            None
        } else {
            Some(SelectMenuOption {
                default: false,
                description: Some(description),
                emoji: None,
                label: name.clone(),
                value: name,
            })
        }
    });

    let select_menu = SelectMenu {
        custom_id: "help_basecommand".to_owned(),
        disabled: false,
        max_values: None,
        min_values: None,
        options: base_options,
        placeholder: Some("Select a base command".to_owned()),
    };

    let row = ActionRow {
        components: vec![Component::SelectMenu(select_menu)],
    };

    let base_menu = Component::ActionRow(row);

    match parse_subcommand_menu(options) {
        Some(sub_menu) => vec![base_menu, sub_menu],
        None => vec![base_menu],
    }
}

fn parse_subcommand_menu(options: &[CommandOptionExt]) -> Option<Component> {
    if options.is_empty() {
        return None;
    }

    let options: Vec<_> = options
        .iter()
        .filter_map(|option| match &option.inner {
            CommandOptionExtInner::SubCommand(d) => Some((&d.name, &d.description)),
            CommandOptionExtInner::SubCommandGroup(d) => Some((&d.name, &d.description)),
            _ => None,
        })
        .map(|(name, description)| SelectMenuOption {
            default: false,
            description: Some(description.to_owned()),
            emoji: None,
            label: name.to_owned(),
            value: name.to_owned(),
        })
        .collect();

    if options.is_empty() {
        return None;
    }

    let select_menu = SelectMenu {
        custom_id: "help_subcommand".to_owned(),
        disabled: false,
        max_values: None,
        min_values: None,
        options,
        placeholder: Some("Select a subcommand".to_owned()),
    };

    let row = ActionRow {
        components: vec![Component::SelectMenu(select_menu)],
    };

    Some(Component::ActionRow(row))
}

fn option_fields(children: &[CommandOptionExt]) -> Vec<EmbedField> {
    children
        .iter()
        .filter_map(|child| {
            let (required, name, description) = match &child.inner {
                CommandOptionExtInner::SubCommand(_)
                | CommandOptionExtInner::SubCommandGroup(_) => return None,
                CommandOptionExtInner::String(d) => (d.required, &d.name, &d.description),
                CommandOptionExtInner::Integer(d) => (d.required, &d.name, &d.description),
                CommandOptionExtInner::Boolean(d) => (d.required, &d.name, &d.description),
                CommandOptionExtInner::User(d) => (d.required, &d.name, &d.description),
                CommandOptionExtInner::Channel(d) => (d.required, &d.name, &d.description),
                CommandOptionExtInner::Role(d) => (d.required, &d.name, &d.description),
                CommandOptionExtInner::Mentionable(d) => (d.required, &d.name, &d.description),
                CommandOptionExtInner::Number(d) => (d.required, &d.name, &d.description),
                CommandOptionExtInner::Attachment(d) => (d.required, &d.name, &d.description),
            };

            let mut name = name.to_owned();

            if required {
                name.push_str(" (required)");
            }

            let value = child
                .help
                .as_ref()
                .map_or_else(|| description.to_owned(), |help| help.to_owned());

            let field = EmbedField {
                inline: value.len() <= 37,
                name,
                value,
            };

            Some(field)
        })
        .collect()
}
