use serenity::builder::CreateApplicationCommand;
use serenity::model::Permissions;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::ChannelType;
use sqlx::{Pool, Sqlite};

use crate::database::Database;

pub async fn run(options: &[CommandDataOption], discord_bot_id: &String, database_pool: Pool<Sqlite>) -> String {
  let channel_id: u64 = options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap().parse().unwrap();
  let url_input = options.get(1).unwrap().value.as_ref().unwrap().as_str().unwrap();
  let uploads = options.get(2).unwrap().value.as_ref().unwrap().as_bool().unwrap();
  let comments = options.get(3).unwrap().value.as_ref().unwrap().as_bool().unwrap();
  let complete = options.get(4).unwrap().value.as_ref().unwrap().as_bool().unwrap();
  let pinged_role: String = match &options.get(5) {
    Some(arg) => {
      arg.value.as_ref().unwrap().as_str().unwrap().replace('"', "")
    },
    None => {
      "0".to_string()
    }
  };

  let mut database: Database;
  if let Ok(database_) = Database::use_pool(database_pool).await {
    database = database_;
  } else {
    return "Failed to connect to database".to_string();
  }

  if database.discord_channel_exists(discord_bot_id, channel_id).await {
    return "This discord channel has already been configured, please make sure to `/reset` it before creating new settings.".to_string();
  }
  
  let urls: Vec<String> = url_input.split(',').map(|str| str.trim().to_string()).collect();

  println!("[INF] {:?} configured with {:?} | {} {} {}", channel_id, urls, uploads, comments, complete);
  
  database.add_discord_channel(discord_bot_id, channel_id, urls, (comments, uploads, complete), pinged_role).await;
  "Channel successfully configured.".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
  command
    .name("create").description("Setup notifications for the current channel")
    .create_option(|option| {
      option
        .name("channel")
        .description("Channel to receive the notifications")
        .kind(CommandOptionType::Channel)
        .channel_types(&[ChannelType::Text])
        .required(true)
    })
    .create_option(|option| {
      option
        .name("url")
        .description("Nyaa URL separated by `,` (f.e.: `https://nyaa.si/user/neoborn, https://nyaa.si/user/djatom`)")
        .kind(CommandOptionType::String)
        .min_length(5)
        .required(true)
    })
    .create_option(|option| {
      option
        .name("uploads")
        .description("Notifications for uploads")
        .kind(CommandOptionType::Boolean)
        .required(true)
    })
    .create_option(|option| {
      option
        .name("comments")
        .description("Notifications for comments")
        .kind(CommandOptionType::Boolean)
        .required(true)
    })
    .create_option(|option| {
      option
        .name("complete")
        .description("Check every page of the query, or all of them?")
        .kind(CommandOptionType::Boolean)
        .required(true)
    })
    .create_option(|option| {
      option
        .name("pinged-role")
        .description("Ping this role when sending the notifications")
        .kind(CommandOptionType::Role)
        .required(false)
    })
    .default_member_permissions(Permissions::ADMINISTRATOR)
}
