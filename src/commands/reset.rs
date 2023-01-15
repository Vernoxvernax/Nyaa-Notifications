use serenity::builder::CreateApplicationCommand;
use serenity::model::Permissions;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

use crate::database::{check_for_channel_id, update_discord_bot};

pub async fn run(_options: &[CommandDataOption], channel_id: ChannelId) -> String {
    let channel_id =  channel_id.0 as i64;
    if check_for_channel_id(channel_id).await.unwrap().is_empty()
    {
        return "This discord channel has not been configured. Type `/create` to set it up.".to_string();
    }
    update_discord_bot(channel_id, false, true).await.unwrap();
    println!("Configuration removed for {:?}", channel_id);
    "Channel configuration successfully removed.".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("reset").description("Remove configurations for the current channel")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
