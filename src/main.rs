use database::Database;
use lazy_static::lazy_static;
use log::debug;
use std::{
  process::ExitCode, thread, time::Duration
};

use web::Web;
use config::{
  Config, ModuleType
};
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
  env_logger::init();
  debug!("Reading configuration.");
  let config_res = Config::new();
  let mut config: Config;
  if config_res.is_err() {
    return ExitCode::FAILURE;
  } else {
    config = config_res.unwrap();
  };

  debug!("Generating and opening database.");
  let database_res = Database::new();
  let mut database: Database;
  if let Ok(database_) = database_res.await {
    database = database_;
  } else {
    return ExitCode::FAILURE;
  }

  debug!("Initializing notifications class.");
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

    debug!("Refreshing discord modules.");
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
        debug!("Getting updates from nyaa.");
        let mut updates = web.get_updates(module, &id, &mut database).await;
        updates.reverse();
        debug!("Sending updates:\n{:?}", updates);
        for update in notifications.process_updates(module, &mut database, updates).await {
          database.update_db_table(module.module_type.to_string(), &id, update).await;
        }
      }
    }

    println!("Waiting {} minutes...", config.clone().update_interval);
    thread::sleep(Duration::from_secs(config.clone().update_interval * 60));
  }
}
