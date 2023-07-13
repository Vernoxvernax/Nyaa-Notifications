use serenity::builder::CreateApplicationCommand;
use serenity::model::Permissions;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub fn run(_options: &[CommandDataOption]) -> String {
  "```\
[Nyaa-Notifications]

Commands:
  \"help\" - Print this help message
  \"create\" - Setup notifications for the current channel
  \"reset\" - Remove notifications for the current channel
  \"pause\" - Pause all notifications for this channel
  \"resume\" - Resume all notifications for this channel```".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
  command.name("help").description("Small description of available commands")
  .default_member_permissions(Permissions::ADMINISTRATOR)
}
