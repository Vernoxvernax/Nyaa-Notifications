use serenity::all::{
  Permissions, CreateCommand, CommandDataOption
};

pub async fn run(_options: &[CommandDataOption]) -> String {
  "```\
[Nyaa-Notifications]

Commands:
  \"help\" - Print this help message
  \"create\" - Setup notifications for the current channel
  \"reset\" - Remove notifications for the current channel
  \"pause\" - Pause/Resume all notifications for this channel
  \"activity\" - Change current activity-text of the discord bot```".to_string()
}

pub fn register() -> CreateCommand {
  CreateCommand::new("help")
  .description("Small description of available commands")
  .default_member_permissions(Permissions::ADMINISTRATOR)
}
