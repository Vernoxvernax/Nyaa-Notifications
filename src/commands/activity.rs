use serenity::builder::CreateApplicationCommand;
use serenity::prelude::Context;
use serenity::model::Permissions;
use serenity::model::prelude::Activity;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub async fn run(options: &[CommandDataOption], ctx: &Context) -> String {
  let discord_activity_type = &options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap();
  let discord_activity_text = &options.get(1).unwrap().value.as_ref().unwrap().as_str().unwrap();
  if *discord_activity_type == "listening" {
    ctx.set_activity(Activity::listening(discord_activity_text)).await;
  } else if *discord_activity_type == "playing" {
    ctx.set_activity(Activity::playing(discord_activity_text)).await;
  } else if *discord_activity_type == "watching" {
    ctx.set_activity(Activity::watching(discord_activity_text)).await;
  } else {
    ctx.set_activity(Activity::competing(discord_activity_text)).await;
  }
  "Activity changed.".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
  command
  .name("activity").description("Change the activity status of the bot.")
  .default_member_permissions(Permissions::ADMINISTRATOR)
  .create_option(|option| {
    option
      .name("activity-type")
      .description("type")
      .add_string_choice("playing", "playing")
      .add_string_choice("listening", "listening")
      .add_string_choice("watching", "watching")
      .add_string_choice("competing", "competing")
      .kind(CommandOptionType::String)
      .required(true)
  })
  .create_option(|option| {
    option
      .name("activity-text")
      .description("text")
      .kind(CommandOptionType::String)
      .min_length(2)
      .required(true)
  })
}
