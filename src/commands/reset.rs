use serenity::builder::CreateApplicationCommand;
use serenity::model::Permissions;
use serenity::model::prelude::ChannelType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

use crate::database::{check_for_channel_id, update_discord_bot};

pub async fn run(options: &[CommandDataOption]) -> String {
	let channel_id: &i64 = &options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap().parse().unwrap();
	if check_for_channel_id(*channel_id).await.unwrap().is_empty()
	{
		return "This discord channel has not been configured. Type `/create` to set it up.".to_string();
	}
	update_discord_bot(*channel_id, false, true).await.unwrap();
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
