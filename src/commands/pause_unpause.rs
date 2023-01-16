use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};
use serenity::model::Permissions;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

use crate::database::{check_for_channel_id, update_discord_bot};

pub async fn run(options: &[CommandDataOption], channel_id: ChannelId) -> String {
  let channel_id =  channel_id.0 as i64;
  let input = &options.get(0).unwrap().value.as_ref().unwrap().as_bool().unwrap();
  let check = check_for_channel_id(channel_id).await.unwrap();
  if check.is_empty()
  {
    return "This discord channel has not been configured yet. Type `/create` to set it up.".to_string();
  }
  else if check.get(0).unwrap().activated && ! *input
  {
    return "This discord channel is not paused to begin with.\nYou might want to check out `/pause True`.".to_string();
  }
  else if ! check.get(0).unwrap().activated && *input
  {
    return "This discord channel is already paused.\nYou might want to check out `/pause False`.".to_string();
  }
  if *input
  {
    update_discord_bot(channel_id, true, false).await.unwrap();
  }
  else
  {
    update_discord_bot(channel_id, false, false).await.unwrap();
  }
  println!("Notifications now {:?} for {:?}", input, channel_id);
  "Channel configuration successfully edited.".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
  command
    .name("pause").description("Change the notification-state for the current channel")
    .default_member_permissions(Permissions::ADMINISTRATOR)
    .create_option(|option| {
      option
        .name("yesno")
        .description("Pause notifications or resume them")
        .kind(CommandOptionType::Boolean)
        .required(true)
    })
}
