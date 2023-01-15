use serenity::builder::CreateApplicationCommand;
use serenity::model::Permissions;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::prelude::command::CommandOptionType;

use crate::database::{check_for_channel_id, add_discord_channel};
use crate::DiscordChannel;

pub async fn run(options: &[CommandDataOption], channel_id: ChannelId) -> String {
    let url_input = &options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap();
    let releases = &options.get(1).unwrap().value.as_ref().unwrap().as_bool().unwrap();
    let comments = &options.get(2).unwrap().value.as_ref().unwrap().as_bool().unwrap();
    let channel_id =  channel_id.0 as i64;
    if ! check_for_channel_id(channel_id).await.unwrap().is_empty()
    {
        return "This discord channel has already been configured, please make sure to `/reset` it before adding new settings.".to_string();
    }
    let urls: Vec<String> = url_input.split(", ").map(|str| str.to_string()).collect();
    add_discord_channel(DiscordChannel {
        activated: true,
        releases: *releases,
        comments: *comments,
        channel_id,
        urls: urls.clone()
    }).await.unwrap();
    println!("{:?} configured with {:?} | {} {}", channel_id, urls, releases, comments);
    "Channel successfully configured.".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("create").description("Setup notifications for the current channel")
        .create_option(|option| {
            option
                .name("url")
                .description("List of nyaa url's")
                .kind(CommandOptionType::String)
                .min_length(15)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("releases")
                .description("Notifications for releases")
                .kind(CommandOptionType::Boolean)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("comments")
                .description("Notifications for comments")
                .kind(CommandOptionType::Boolean)
                .required(true)
        }).default_member_permissions(Permissions::ADMINISTRATOR)
}
