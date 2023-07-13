use serenity::builder::CreateApplicationCommand;
use serenity::model::Permissions;
use serenity::model::prelude::ChannelType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use sqlx::{Sqlite, Pool};

use crate::database::Database;

pub async fn run(options: &[CommandDataOption], discord_bot_id: &String, database_pool: Pool<Sqlite>) -> String {
	let channel_id: u64 = options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap().parse().unwrap();

  let mut database: Database;
  if let Ok(database_) = Database::use_pool(database_pool).await {
    database = database_;
  } else {
    return "Failed to connect to database".to_string();
  }

  if ! database.discord_channel_exists(discord_bot_id, channel_id).await {
    return "This discord channel has never been configured. See `/create`.".to_string();
  }

  database.remove_discord_channel(discord_bot_id, channel_id).await;

	println!("Configuration removed for {:?}", channel_id);
	"Channel configuration successfully removed.".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	command
		.name("reset").description("Remove configurations for the current channel")
		.create_option(|option| {
      option
        .name("channel")
        .description("Channel to receive the notifications")
        .kind(CommandOptionType::Channel)
        .channel_types(&[ChannelType::Text])
        .required(true)
    })
		.default_member_permissions(Permissions::ADMINISTRATOR)
}
