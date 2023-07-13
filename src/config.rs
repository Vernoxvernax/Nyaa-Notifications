use std::{path::Path, fs::{File, self}, io::Write};
use serde::Deserialize;

use crate::{NYAA_FOLDER_PATH, NYAA_CONFIG_PATH, database::Database};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub update_interval: u64,
  pub module: Vec<ModuleConfig>
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ModuleType {
  Email,
  Gotify,
  Discord
}

impl std::fmt::Display for ModuleType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ModuleType::Gotify => {
        write!(f, "Gotify")
      },
      ModuleType::Email => {
        write!(f, "Email")
      },
      ModuleType::Discord => {
        write!(f, "Discord")
      }
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleConfig {
  pub module_type: ModuleType,
  pub active: bool,
  pub feeds: Vec<String>,
  pub comments: bool,
  pub uploads: bool,
  pub retrieve_all_pages: bool,
  pub smtp_username: Option<String>,
  pub smtp_password: Option<String>,
  pub smtp_domain: Option<String>,
  pub smtp_port: Option<u32>,
  pub smtp_subject: Option<String>,
  pub smtp_recipients: Option<Vec<String>>,
  pub gotify_domain: Option<String>,
  pub gotify_token: Option<String>,
  pub gotify_comment_priority: Option<u32>,
  pub gotify_upload_priority: Option<u32>,
  pub discord_token: Option<String>,
  pub discord_bot_id: Option<String>,
  pub discord_channel_id: Option<u64>,
  pub discord_pinged_role: Option<u64>
}

impl Config {
  pub fn new() -> Result<Self, ()> {
    if Path::is_dir(Path::new(&NYAA_FOLDER_PATH.to_string())) &&
    Path::is_file(Path::new(&NYAA_CONFIG_PATH.to_string())) {
      if let Ok(file) = &fs::read_to_string(Path::new(&NYAA_CONFIG_PATH.to_string())) {
        match toml::from_str::<Config>(file) {
          Ok(config) => {
            if config.module.iter().all(|module| ! module.active) {
              println!("[INF] None of the modules have been activated.\nPlease edit {}.", *NYAA_CONFIG_PATH);
              return Err(());
            }

            if config.module.iter().any(|module| module.module_type == ModuleType::Discord) &&
            config.module.iter().filter(|module| module.active && (module.module_type == ModuleType::Discord)).count() > 1 {
              eprintln!("[ERR] More than one discord module is activated. Serenity only allows one client per instance!");
              return Err(());
            }

            return Ok(Config {
              update_interval: config.update_interval,
              module: config.module
            });
          }
          Err(e) => {
            eprintln!("Failed to read {}.\n{}", *NYAA_CONFIG_PATH, e);
          }
        }
      }
      Err(())
    } else {
      let template: &str = r#"update_interval = 5 # minutes

[[module]]
active = false
module_type = "Email"
feeds = ["https://nyaa.si/user/neoborn"]
comments = false
uploads = false
retrieve_all_pages = false
smtp_username = "example@mail.com"
smtp_password = "password123"
smtp_domain = "gmail.com"
smtp_subject = "Nyaa-Notifications"
smtp_port = 587
smtp_recipients = ["example@mail.com", "example1@mail.com"]

[[module]]
active = false
module_type = "Gotify"
feeds = ["https://nyaa.si/"]
comments = false
uploads = false
retrieve_all_pages = true
gotify_domain = "<GOTIFY-SERVER>"
gotify_token = "<GOTIFY-TOKEN>"
gotify_comment_priority = 1
gotify_upload_priority = 10

[[module]]
active = false
module_type = "Discord"
discord_token = "<DISCORD-BOT-TOKEN>"
discord_bot_id = "just something variable to name the database"
feeds = []  # not used
comments = true # not used
uploads = true  # not used
retrieve_all_pages = false  # not used
"#;

      if Path::is_dir(Path::new(&NYAA_FOLDER_PATH.to_string())) ||
      std::fs::create_dir(NYAA_FOLDER_PATH.to_string()).is_ok() {
        if let Ok(mut config_file) = File::create(NYAA_CONFIG_PATH.to_string()) {
          if config_file.write_all(template.as_bytes()).is_err() {
            eprintln!("Failed to write to {}", *NYAA_CONFIG_PATH);
          } else {
            eprintln!("Please edit {}.", *NYAA_CONFIG_PATH)
          }
        } else {
          eprintln!("Failed to create ./{}/config.toml", *NYAA_FOLDER_PATH);
        }
      } else {
        eprintln!("Failed to create ./{}", *NYAA_FOLDER_PATH);
      }
      
      Err(())
    }
  }

  pub async fn refresh_discord_modules(&mut self, database: &mut Database, discord_bot_id: String) {
    for (index, module) in self.module.clone().iter().enumerate() {
      if module.active && (module.module_type == ModuleType::Discord) {
        self.module.remove(index);
      }
    }
    self.module.append(&mut database.get_discord_channels(&discord_bot_id).await);
  }
}
