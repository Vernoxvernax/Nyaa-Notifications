use std::{process::ExitCode, thread, time::Duration};
use database::Database;
use lazy_static::lazy_static;

use web::Web;
use config::{Config, ModuleType};
use notifications::Notifications;

mod commands;
pub mod config;
pub mod notifications;
pub mod database;
pub mod web;
pub mod discord;
pub mod html;

lazy_static! {
  static ref NYAA_FOLDER_PATH: &'static str = "./nyaa_notifications";
  static ref NYAA_CONFIG_PATH: String = format!("{}/config.toml", *NYAA_FOLDER_PATH);
  static ref NYAA_DATABASE_PATH: String = format!("{}/nyaa-notifications.sqlite", *NYAA_FOLDER_PATH);
}

#[tokio::main]
async fn main() -> ExitCode {
  let config_res = Config::new();
  let mut config: Config;
  if config_res.is_err() {
    return ExitCode::FAILURE;
  } else {
    config = config_res.unwrap();
  };

  let database_res = Database::new();
  let mut database: Database;
  if let Ok(database_) = database_res.await {
    database = database_;
  } else {
    return ExitCode::FAILURE;
  }

  let notifications_res = Notifications::new(config.module.clone(), &mut database).await;
  let mut notifications: Notifications;
  if let Ok(notifications_) = notifications_res {
    notifications = notifications_;
  } else {
    return ExitCode::FAILURE;
  }

  loop {
    let mut web = Web::default();
    println!("Checking at: {}", chrono::Local::now());

    for module in config.module.clone() {
      if module.active && module.discord_token.is_some() && (module.module_type == ModuleType::Discord) {
        config.refresh_discord_modules(&mut database, module.discord_bot_id.unwrap()).await;
        break;
      }
    }

    for (index, module) in config.module.iter().enumerate() {
      if module.active && module.discord_token.is_none() {
        let id = if module.module_type == ModuleType::Discord {
          module.discord_bot_id.clone().unwrap()+"_"+&module.discord_channel_id.unwrap().to_string()
        } else {
          index.to_string()
        };
        let mut updates = web.get_updates(module, &id, &mut database).await;
        updates.reverse();
        for update in notifications.process_updates(module, &mut database, updates).await {
          database.update_db_table(module.module_type.to_string(), &id, update).await;
        }
      }
    }

    println!("Waiting {} minutes...", config.clone().update_interval);
    thread::sleep(Duration::from_secs(config.clone().update_interval * 60));
  }
}
