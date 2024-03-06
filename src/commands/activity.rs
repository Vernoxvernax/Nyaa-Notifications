use serenity::all::{
  Context, Permissions, CreateCommand, CreateCommandOption, CommandOptionType, CommandDataOptionValue, CommandDataOption, ActivityData
};

pub async fn run(options: &[CommandDataOption], ctx: &Context) -> String {
  let discord_activity_type = match &options.get(0).unwrap().value {
    CommandDataOptionValue::String(text) => text,
    _ => {
      panic!("Discord returned invalid command options.")
    }
  };
  let discord_activity_text = match &options.get(1).unwrap().value {
    CommandDataOptionValue::String(text) => text,
    _ => {
      panic!("Discord returned invalid command options.")
    }
  };

  if discord_activity_type == "listening" {
    ctx.set_activity(Some(ActivityData::listening(discord_activity_text)));
  } else if discord_activity_type == "playing" {
    ctx.set_activity(Some(ActivityData::playing(discord_activity_text)));
  } else if discord_activity_type == "watching" {
    ctx.set_activity(Some(ActivityData::watching(discord_activity_text)));
  } else {
    ctx.set_activity(Some(ActivityData::competing(discord_activity_text)));
  }
  "Activity changed.".to_string()
}

pub fn register() -> CreateCommand {
  CreateCommand::new("activity")
  .description("Change the activity status of the bot.")
  .default_member_permissions(Permissions::ADMINISTRATOR)
  .add_option(
    CreateCommandOption::new(
      CommandOptionType::String,
      "activity-type",
      "type"
    )
    .add_string_choice("playing", "playing")
    .add_string_choice("listening", "listening")
    .add_string_choice("watching", "watching")
    .add_string_choice("competing", "competing")
    .required(true)
    )
  .add_option(
    CreateCommandOption::new(
      CommandOptionType::String,
      "activity-text",
      "text"
    )
    .min_length(2)
    .required(true)
  )
}
