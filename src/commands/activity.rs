use serenity::model::prelude::command::CommandOptionType;
use serenity::{builder::CreateApplicationCommand, prelude::Context};
use serenity::model::Permissions;
use serenity::model::prelude::Activity;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub async fn run(options: &[CommandDataOption], ctx: &Context) -> String {
  let act = &options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap();
  ctx.set_activity(Activity::listening(act)).await;
  "Activity changed.".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
  command
  .name("activity").description("Change the activity status of the bot.")
  .default_member_permissions(Permissions::ADMINISTRATOR)
  .create_option(|option| {
    option
      .name("listening")
      .description("listening to what")
      .kind(CommandOptionType::String)
      .min_length(2)
      .required(true)
  })
}
