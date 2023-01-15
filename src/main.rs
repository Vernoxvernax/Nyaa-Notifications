use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::prelude::*;
use std::time::Duration;
use std::fmt::{self, Debug};
use std::process::exit;
use std::thread;
use chrono::NaiveDateTime;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, message::{header, MultiPart, SinglePart},
    Tokio1Executor, Message
};
use serenity::framework::StandardFramework;
use serenity::{json::json, utils::Color, async_trait, prelude::*, model::prelude::*};
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::command::Command;
use http::StatusCode;
use isahc::{Request, RequestExt, ReadResponseExt, prelude::Configurable};
use serde_derive::{Deserialize, Serialize};

pub mod database;
pub mod html;
mod commands;

use database::{get_main_database, updates_to_main_database, get_discord_channels, get_channel_database, update_channel_db};
use html::{serizalize_torrent_page, serizalize_search_page, get_uploader_avatar, get_uploader_name};


#[derive(Clone, Debug)]
pub struct NyaaComment {
    pub html: String,
    pub message: String,
    pub user: String,
    pub gravatar: String,
    pub timestamp: String
}


#[derive(Clone, Debug)]
pub struct NyaaPage {
    pub torrent_count: u64,
    pub torrents: Vec<NyaaTorrent>,
    pub incomplete: bool
}


#[derive(Debug, Clone)]
pub struct NyaaTorrent {
    pub title: String,
    pub category: String,
    pub comments: u64,
    pub size: String,
    pub torrent_file: String,
    pub magnet: String,
    pub date: String,
    pub seeders: u64,
    pub leechers: u64,
    pub completed: u64,
    pub timestamp: u64,
    pub uploader_avatar: Option<String>
}


#[derive(Debug, Clone)]
pub struct Update {
    pub nyaa_comments: Vec<NyaaComment>,
    pub nyaa_url: String,
    pub new_comments: u64,
    pub nyaa_torrent: NyaaTorrent,
    pub new_torrent: bool
}


#[derive(Debug, Deserialize, Serialize, Clone)]
struct ConfigFile {
    main: Main,
    discord_bot: DiscordInstance,
    smtp: Smtp,
    gotfiy: Gotify,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
struct DiscordInstance {
    enabled: bool,
    discord_token: String,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DiscordChannel {
    pub activated: bool,
    pub releases: bool,
    pub comments: bool,
    pub channel_id: i64,
    pub urls: Vec<String>
}


#[derive(Debug, Deserialize, Serialize, Clone)]
struct Smtp {
    enabled: bool,
    comment_notifications: bool,
    smtp_username: String,
    smtp_password: String,
    smtp_address: String,
    smtp_subject: String,
    smtp_port: u64,
    smtp_receiver: String
}


#[derive(Debug, Deserialize, Serialize, Clone)]
struct Gotify {
    enabled: bool,
    domain: String,
    token: String,
    comment_notifications: bool,
    comment_priority: u8,
    release_priority: u8
}


#[derive(Debug, Deserialize, Serialize, Clone)]
struct Main {
    nyaa_url: Vec<String>,
    complete_result: bool,
    update_delay: u64
}


impl std::fmt::Display for NyaaPage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Torrents: {}\nMultiple pages: {}", self.torrent_count, self.incomplete).expect("failed to fmt");
        writeln!(f, "Torrents:").expect("failed to fmt");
        self.torrents.iter().fold(Ok(()), |result, nyaatorrent| {
            result.and_then(|_| writeln!(f, "{}", nyaatorrent))
        })
    }
}


impl std::fmt::Display for NyaaTorrent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " Title: {}\n  Category: {}\n  Comments: {}\n  Size: {}\n  Torrent-file link: {}\n  Magnet-Link: {}\n  Upload-Date: {}\n  Seeders: {}\n  Leechers: {}\n  Completed: {}", 
        self.title, self.category, self.comments, self.size, self.torrent_file, self.magnet, self.date, self.seeders, self.leechers, self.completed)
    }
}

struct Handler {
    config_clone: ConfigFile,
    running_loop: AtomicBool,
}


#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // println!("Received command interaction: {:#?}", command);
            let content = match command.data.name.as_str() {
                "help" => commands::help::run(&command.data.options),
                "create" => commands::create::run(&command.data.options, command.channel_id).await,
                "reset" => commands::reset::run(&command.data.options, command.channel_id).await,
                "pause" => commands::pause_unpause::run(&command.data.options, command.channel_id).await,
                "activity" => commands::activity::run(&command.data.options, &ctx).await,
                _ => "not implemented :(".to_string(),
            };
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        ctx.set_activity(Activity::listening("japanese cats.")).await;
        Command::create_global_application_command(&ctx.http, |command| {
            commands::help::register(command)
        }).await.unwrap();
        Command::create_global_application_command(&ctx.http, |command| {
            commands::create::register(command)
        }).await.unwrap();
        Command::create_global_application_command(&ctx.http, |command| {
            commands::reset::register(command)
        }).await.unwrap();
        Command::create_global_application_command(&ctx.http, |command| {
            commands::pause_unpause::register(command)
        }).await.unwrap();
        Command::create_global_application_command(&ctx.http, |command| {
            commands::activity::register(command)
        }).await.unwrap();
        let config_clone = self.config_clone.clone();
        let ctx = Arc::new(ctx);
        if self.running_loop.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            tokio::spawn(async move {
                println!("Starting Loop");
                loop {
                    println!("Checking at: {}", chrono::Local::now());
                    let channels = get_discord_channels().await.unwrap();
                    for channel in channels {
                        if channel.activated {
                            let database = get_channel_database(channel.channel_id).await.unwrap();
                            for url in channel.urls {
                                if url.is_empty() {
                                    continue;
                                }
                                if database.is_empty() {
                                    let mut edited_config = config_clone.clone();
                                    edited_config.gotfiy.comment_notifications = false;
                                    edited_config.smtp.comment_notifications = false;
                                    edited_config.discord_bot.enabled = false;
                                    let updates = nyaa_check(&edited_config, &url, database.clone(), false).await;
                                    update_channel_db(channel.channel_id, &updates).await.unwrap();
                                    continue;
                                }
                                let updates = nyaa_check(&config_clone, &url, database.clone(), true).await;
                                if updates.is_empty() {
                                    println!("NO UPDATES");
                                } else {
                                    for update in updates.clone() {
                                        if update.new_torrent && channel.releases {
                                            let channel_id = channel.channel_id as u64;
                                            let timestamp = &update.nyaa_torrent.timestamp.to_string()[0..update.nyaa_torrent.timestamp.to_string().len()].parse::<i64>().unwrap();
                                            let nanosec = &update.nyaa_torrent.timestamp.to_string()[update.nyaa_torrent.timestamp.to_string().len() - 3..update.nyaa_torrent.timestamp.to_string().len()].parse::<u32>().unwrap();
                                            let utc_time = chrono::DateTime::<chrono::Utc>::from_utc(NaiveDateTime::from_timestamp_opt(*timestamp, *nanosec).unwrap(), chrono::Utc);
                                            let avatar = if update.nyaa_torrent.uploader_avatar.is_some() {
                                                update.nyaa_torrent.uploader_avatar.clone().unwrap().replace("amp;", "")
                                            } else {
                                                "https://avatars3.githubusercontent.com/u/28658394?v=4&s=400".to_owned()
                                            };
                                            let discord_message = ChannelId(channel_id)
                                            .send_message(&ctx1, |m| {
                                                m.embed(|e| {
                                                    e.title(update.nyaa_torrent.title.clone())
                                                    .color(Color::BLITZ_BLUE)
                                                    .thumbnail(avatar)
                                                    .author(|a| {
                                                        a.name("Found in this feed!")
                                                        .url(update.nyaa_url.clone())
                                                    })
                                                    .description("A new release!")
                                                    .fields(vec![
                                                        ("Category", update.nyaa_torrent.category.clone(), false),
                                                        ("Size", update.nyaa_torrent.size.clone(), false)
                                                    ])
                                                    .timestamp(utc_time)
                                                }).components(|c| {
                                                    c.create_action_row(|r| {
                                                        r.create_button(|b| {
                                                            b.label("Nyaa.si")
                                                            .url(update.nyaa_torrent.torrent_file.replace("download", "view").strip_suffix(".torrent").unwrap())
                                                            .style(serenity::model::prelude::component::ButtonStyle::Link)
                                                        })
                                                        .create_button(|b| {
                                                            b.label("Torrent-File")
                                                            .url(update.nyaa_torrent.torrent_file.clone())
                                                            .style(serenity::model::prelude::component::ButtonStyle::Link)
                                                        })
                                                    })
                                                })
                                            }).await;
                                            if let Err(w) = discord_message {
                                                eprintln!("Failed to create message: {:?}", w)
                                            };
                                        }
                                        if update.new_comments > 0 && channel.comments {
                                            let channel_id = channel.channel_id as u64;
                                            for comment_index in update.nyaa_comments.len() as u64 - update.new_comments..update.nyaa_comments.len() as u64 {
                                                let nyaa_comment = update.nyaa_comments.get(comment_index as usize).unwrap();
                                                let timestamp_op1 = if nyaa_comment.timestamp.contains('.') {
                                                    let mut temp: String = String::new();
                                                    for ch in nyaa_comment.timestamp.chars() {
                                                        if ch == '.' {
                                                            break
                                                        } else {
                                                            temp.push_str(ch.to_string().as_str())
                                                        }
                                                    };
                                                    temp
                                                } else {
                                                    nyaa_comment.timestamp.to_string()
                                                };
                                                let seconds = timestamp_op1[0..timestamp_op1.len()].parse::<i64>().unwrap();
                                                let nanosec = timestamp_op1[timestamp_op1.len() - 3..timestamp_op1.len()].parse::<u32>().unwrap();
                                                let utc_time_comment = chrono::DateTime::<chrono::Utc>::from_utc(NaiveDateTime::from_timestamp_opt(seconds, nanosec).unwrap(), chrono::Utc);
                                                if nyaa_comment.user.len() + nyaa_comment.message.len() <= 1024 {
                                                    let discord_message = ChannelId(channel_id)
                                                    .send_message(&ctx1, |m| {
                                                        m.embed(|e| {
                                                            e.title(update.nyaa_torrent.title.clone())
                                                            .color(Color::BLITZ_BLUE)
                                                            .thumbnail(nyaa_comment.gravatar.clone().replace("amp;", ""))
                                                            .author(|a| {
                                                                a.name("Found in this feed!")
                                                                .url(update.nyaa_url.clone())
                                                            })
                                                            .fields(vec![
                                                                (nyaa_comment.user.clone() + ":", nyaa_comment.message.clone(), false),
                                                            ])
                                                            .timestamp(utc_time_comment)
                                                        }).components(|c| {
                                                            c.create_action_row(|r| {
                                                                r.create_button(|b| {
                                                                    b.label("Nyaa.si")
                                                                    .url(update.nyaa_torrent.torrent_file.replace("download", "view").strip_suffix(".torrent").unwrap())
                                                                    .style(serenity::model::prelude::component::ButtonStyle::Link)
                                                                })
                                                                .create_button(|b| {
                                                                    b.label(nyaa_comment.user.clone())
                                                                    .url(format!("https://nyaa.si/user/{}", nyaa_comment.user.clone()))
                                                                    .style(serenity::model::prelude::component::ButtonStyle::Link)
                                                                })
                                                            })
                                                        })
                                                    }).await;
                                                    if let Err(w) = discord_message {
                                                        eprintln!("Failed to create message: {:?}", w)
                                                    };
                                                } else {
                                                    let amount = ((nyaa_comment.user.len() as f64 + nyaa_comment.message.len() as f64) / 500.0).ceil() as u32;
                                                    let mut comment = nyaa_comment.message.clone();
                                                    for index in 1..amount {
                                                        let cut = if comment.len() > 500 {
                                                            500
                                                        } else {
                                                            comment.len()
                                                        };
                                                        let discord_message = ChannelId(channel_id)
                                                        .send_message(&ctx1, |m| {
                                                            m.embed(|e| {
                                                                e.title(update.nyaa_torrent.title.clone())
                                                                .color(Color::BLITZ_BLUE)
                                                                .thumbnail(nyaa_comment.gravatar.clone().replace("amp;", ""))
                                                                .author(|a| {
                                                                    a.name("Found in this feed!")
                                                                    .url(update.nyaa_url.clone())
                                                                })
                                                                .fields(vec![
                                                                    (nyaa_comment.user.clone() + " (" + &index.to_string() + "/" + &(amount-1).to_string() + ")" + ":", &comment[..cut], false),
                                                                ])
                                                                .timestamp(utc_time_comment)
                                                            }).components(|c| {
                                                                c.create_action_row(|r| {
                                                                    r.create_button(|b| {
                                                                        b.label("Nyaa.si")
                                                                        .url(update.nyaa_torrent.torrent_file.replace("download", "view").strip_suffix(".torrent").unwrap())
                                                                        .style(serenity::model::prelude::component::ButtonStyle::Link)
                                                                    })
                                                                    .create_button(|b| {
                                                                        b.label(nyaa_comment.user.clone())
                                                                        .url(format!("https://nyaa.si/user/{}", nyaa_comment.user.clone()))
                                                                        .style(serenity::model::prelude::component::ButtonStyle::Link)
                                                                    })
                                                                })
                                                            })
                                                        }).await;
                                                        if let Err(w) = discord_message {
                                                            eprintln!("Failed to create message: {:?}", w)
                                                        } else {
                                                            comment = comment[500..comment.len()].to_string();
                                                        }
                                                    }
                                                }
                                            };
                                        }
                                    }
                                    update_channel_db(channel.channel_id, &updates).await.unwrap();
                                }
                            }
                        }
                    }
                    for nyaa_url in config_clone.clone().main.nyaa_url {
                        if config_clone.gotfiy.enabled || config_clone.smtp.enabled {
                            let database = get_main_database().await.unwrap();
                            let updates = nyaa_check(&config_clone, &nyaa_url, database, false).await;
                            if updates.is_empty() {
                                println!("NO UPDATES");
                            } else {
                                updates_to_main_database(&updates).await.unwrap();
                                send_notification(&config_clone, &updates).await.unwrap();
                            }
                        };
                    }
                    println!("Finished update-check.");
                    tokio::time::sleep(Duration::from_secs(config_clone.main.update_delay)).await;
                }
            });
            self.running_loop.swap(true, Ordering::Relaxed);
        }
    }
}


#[tokio::main]
async fn main() {
    let default_config = ConfigFile {
        main: Main {
            nyaa_url: ["https://nyaa.si/user/neoborn".to_string()].to_vec(),
            complete_result: false,
            update_delay: 500
        },
        discord_bot: DiscordInstance {
            enabled: false,
            discord_token: "<DISCORD-BOT-TOKEN>".to_string()
        },
        smtp: Smtp {
            enabled: false,
            comment_notifications: false,
            smtp_username: "<SENDER - EMAIL>".to_string(),
            smtp_password: "<SENDER - PASSWORD>".to_string(),
            smtp_address: "<SMTP SERVER>".to_string(),
            smtp_subject: "<SUBJECT>".to_string(),
            smtp_receiver: "<RECEIVER - EMAIL>".to_string(),
            smtp_port: 587
        },
        gotfiy: Gotify {
            enabled: false,
            comment_notifications: false,
            domain: "<POINTING TO GOTIFY WEBUI>".to_string(),
            token: "<APPLICATION TOKEN>".to_string(),
            comment_priority: 10,
            release_priority: 5
        }
    };
    if ! Path::new("./data").exists() {
        fs::create_dir("./data").expect("Failed to create config path.");
        let mut file = fs::File::create("./data/config.toml").expect("Failed to create config file.");
        file.write_all(toml::to_string_pretty(&default_config).expect("Failed to create config template.").as_bytes()).expect("Failed writing config template");
        println!("Please edit the config file and restart the application.");
        exit(0x0100);
    } else if ! Path::new("./data/config.toml").exists() {
        let mut file = fs::File::create("./data/config.toml").expect("Failed to create config file.");
        file.write_all(toml::to_string_pretty(&default_config).expect("Failed to create config template.").as_bytes()).expect("Failed writing config template");
        println!("Please edit the config file and restart the application.");
        exit(0x0100);
    };
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("nyaa-notifs.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");
    sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");
    database.close().await;
    let config_file = toml::from_str::<ConfigFile>(&fs::read_to_string(Path::new("./data/config.toml")).expect("Failed reading config file.")).expect("Failed to deserialize config file.");
    let config_clone = config_file.clone();
    if config_file.discord_bot.enabled {
        tokio::spawn(async move {
            loop {
                if discord_bot(&config_clone).await.is_err() {
                    println!("Failed to start discord bot, trying again.")
                };
            }
        }).await.unwrap();
    } else if config_file.gotfiy.enabled || config_file.smtp.enabled {
        tokio::spawn(async move {
            loop {
                println!("Checking at: {}", chrono::Local::now());
                for nyaa_url in &config_clone.main.nyaa_url {
                    let database = get_main_database().await.unwrap();
                    let updates = &nyaa_check(&config_clone, nyaa_url, database, false).await;
                    if updates.is_empty() {
                        println!("NO UPDATES");
                    } else {
                        updates_to_main_database(updates).await.unwrap();
                    }
                    send_notification(&config_clone, updates).await.unwrap();
                };
                thread::sleep(Duration::from_secs(config_clone.main.update_delay));
            }
        }).await.expect("Thread failed to run.");
    } else {
        println!("No notification service has been activated. I will index all torrents without sending any notifications.");
        tokio::spawn(async move {
            println!("Checking at: {}", chrono::Local::now());
            for nyaa_url in &config_clone.main.nyaa_url {
                let database = get_main_database().await.unwrap();
                let updates = &nyaa_check(&config_clone, nyaa_url, database, false).await;
                if updates.is_empty() {
                    println!("NO UPDATES");
                } else {
                    updates_to_main_database(updates).await.unwrap();
                }
                send_notification(&config_clone, updates).await.unwrap();
            };
            println!("Done.")
        }).await.expect("Thread failed to run.");
        exit(0x0100);
    }
}


async fn nyaa_check(config_file: &ConfigFile, nyaa_url: &String, database: Vec<NyaaTorrent>, discord: bool) -> Vec<Update> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    let mut updates: Vec<Update> = [].to_vec();
    let mut nyaa_page_res = get_nyaa(nyaa_url);
    if nyaa_page_res.is_err() {
        println!("Web requests are failing.");
        return updates
    }
    let mut nyaa_page = nyaa_page_res.unwrap();
    let mut page_array: Vec<NyaaPage> = [].to_vec();
    let mut page_number = 2;
    loop {
        match serizalize_search_page(&nyaa_page) {
            Ok(page) => {
                page_array.append(&mut [page.clone()].to_vec());
                if page.incomplete && (config_file.main.complete_result || nyaa_url.contains('?')) {
                    println!("Waiting 2 seconds");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    nyaa_page_res = get_nyaa(&format!("{}{}{}", &nyaa_url,
                    if nyaa_url.contains('?') {
                        "&p="
                    } else {
                        "?p="
                    },
                    page_number));
                    if nyaa_page_res.is_err() {
                        println!("Web requests are failing.");
                        tokio::time::sleep(Duration::from_secs(20)).await;
                        return vec![];
                    }
                    nyaa_page = nyaa_page_res.unwrap();
                    page_number += 1;
                } else {
                    break
                }
            },
            Err(e) => {
                panic!("Serizalization failed. {}", e)
            }
        }
    };
    // This might seem stupid, but considering some torrent lists could grow into thousands, checking for a new comment is a lot more effective this way.
    let mut torrent_file_links: String = String::new();
    let database_iterator = database.iter();
    for torrent in database.clone() {
        torrent_file_links.push_str(&(torrent.torrent_file.as_str().to_owned() + " "));
    };
    for page in page_array {
        for mut torrent in page.torrents {
            if ! torrent_file_links.contains(&torrent.torrent_file) {
                let nyaa_comments_res: Result<Vec<NyaaComment>, ()> = if torrent.comments > 0 &&
                    (config_file.discord_bot.enabled ||
                    config_file.smtp.enabled && config_file.smtp.comment_notifications ||
                    config_file.gotfiy.enabled && config_file.gotfiy.comment_notifications) {
                    println!("Waiting 2 seconds");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    get_nyaa_comments(&torrent).await
                } else {
                    Ok([].to_vec())
                };
                if nyaa_comments_res.is_err() {
                    continue
                };
                let nyaa_comments = nyaa_comments_res.unwrap();
                
                if discord {
                    let torrent_page_unv = get_nyaa(&torrent.torrent_file.replace("download", "view").trim_end_matches(".torrent").to_string());
                    if let Ok(torrent_page) = torrent_page_unv {
                        let uploader: Option<String> = get_uploader_name(torrent_page); // This time as an option, since anonymous uploades
                        if let Some(name) = uploader {
                            println!("Waiting 2 seconds");
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            let torrent_page_unv = get_nyaa(&("https://nyaa.si/user/".to_owned()+&name));
                            if let Ok(user_page) = torrent_page_unv {
                                torrent.uploader_avatar = Some(get_uploader_avatar(user_page));
                            };
                        };
                    };
                }
                updates.append(&mut [Update {
                    nyaa_comments,
                    nyaa_url: nyaa_url.to_string(),
                    new_comments: torrent.comments,
                    nyaa_torrent: torrent,
                    new_torrent: true
                }].to_vec());
            } else {
                let database_match_opt = database_iterator.clone().find(|&x| x.torrent_file.contains(&torrent.torrent_file));
                let database_match = database_match_opt.unwrap();
                if database_match.comments < torrent.comments {
                    println!("I found a new comment.");
                    let amount_new_comments = torrent.comments - database_match.comments;
                    let nyaa_comments_res = get_nyaa_comments(&torrent).await;
                    if nyaa_comments_res.is_err() {
                        continue
                    }
                    let nyaa_comments = nyaa_comments_res.unwrap();
                    updates.append(&mut [Update {
                        nyaa_comments,
                        nyaa_url: nyaa_url.to_string(),
                        new_comments: amount_new_comments,
                        nyaa_torrent: torrent,
                        new_torrent: false
                    }].to_vec());
                };
            }
        }
    }
    updates
}


async fn get_nyaa_comments(torrent: &NyaaTorrent) -> Result<Vec<NyaaComment>, ()> {
    let url = &torrent.torrent_file.trim_end_matches(".torrent").replace("download", "view");
    let nyaa_page_res = get_nyaa(url);
    println!("Waiting 2 seconds");
    tokio::time::sleep(Duration::from_secs(2)).await;
    if nyaa_page_res.is_err() {
        println!("Web requests are failing.");
        return Err(());
    };
    let nyaa_page = nyaa_page_res.unwrap();
    match serizalize_torrent_page(&nyaa_page) {
        Ok(update) => {
            Ok(update)
        },
        Err(e) => {
            panic!("Serizalization failed on comments. {}", e)
        }
    }
}


async fn discord_bot(config_file: &ConfigFile) -> Result<(), SerenityError> {
    let config_clone = config_file.to_owned();
    let framework = StandardFramework::new();
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(config_clone.discord_bot.discord_token.clone(), intents)
        .event_handler(Handler {
            config_clone,
            running_loop: AtomicBool::new(false),
        })
        .framework(framework)
        .await?;
    client.start().await?;
    Ok(())
}


async fn send_notification(config_file: &ConfigFile, updates: &Vec<Update>) -> Result<(), Box<dyn std::error::Error>> {
    for update in updates {
        if config_file.smtp.enabled {
            let mut html = String::from(r#"<!DOCTYPE html>
                <html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
                <style>
                .comments {padding-left: 10px;padding-right: 10px;}div.title {font-size: 20px;text-align: center;}
                html {font-family: sans-serif;-ms-text-size-adjust: 100%;-webkit-text-size-adjust: 100%}body {margin: 0}a {background-color: transparent}
                a:active, a:hover {outline: 0}small {font-size: 80%}img {border: 0}
                @media print {*,*:before,*:after {background: transparent !important;color: #000 !important;-webkit-box-shadow: none !important;
                box-shadow: none !important;text-shadow: none !important}a,a:visited {text-decoration: underline}a[href]:after {content: " ("attr(href) ")"}
                img {page-break-inside: avoid}img {max-width: 100% !important}p {orphans: 3;widows: 3}}
                * {-webkit-box-sizing: border-box;-moz-box-sizing: border-box;box-sizing: border-box}
                *:before,*:after {-webkit-box-sizing: border-box;-moz-box-sizing: border-box;box-sizing: border-box}
                html {font-size: 10px;-webkit-tap-highlight-color: rgba(0, 0, 0, 0)}.row {margin-left: -15px;margin-right: -15px}
                body {font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;font-size: 14px;line-height: 1.42857143;color: #afafaf;background-color: #262626}
                a {color: #337ab7;text-decoration: none}a:hover, a:focus {color: #19578b;text-decoration: underline}
                a:focus {outline: 5px auto -webkit-focus-ring-color;outline-offset: -2px}img {vertical-align: middle}p {margin: 0 0 10px}small {font-size: 85%}
                .col-md-2, .col-md-10 {position: relative;min-height: 1px;padding-left: 15px;padding-right: 15px}
                @media (min-width:992px) {.col-md-2,.col-md-10 {float: left}.col-md-10 {width: 83.33333333%}.col-md-2 {width: 16.66666667%}}
                .panel {margin-top: 10px; margin-bottom: 10px;background-color: #323232;border: 1px solid transparent;border-radius: 4px;
                    -webkit-box-shadow: 0 1px 1px rgba(0, 0, 0, 0.05);box-shadow: 0 1px 1px rgba(0, 0, 0, 0.05)}.panel-body {padding: 15px}
                .panel-default {border-color: #111}.row:before, .row:after, .panel-body:before, .panel-body:after {content: " ";display: table}
                .row:after, .panel-body:after {clear: both}@-ms-viewport {width: device-width}.info {margin-bottom: 0px;padding-left: 10px;padding-left: 10px;}
                .info-panel {padding: 10px;}div.new_release {border-color:magenta;box-shadow: 0 0 10px rgba(255, 0, 255, 0.777);}
                a.new_release {text-align: center !important;font-size: 20px;}
                </style>
            </head><body>"#);
            if update.new_torrent {
                html.push_str(format!(
                    r#"<div class="panel panel-default info-panel new_release">
                        <div style="text-align: center;">
                            <a class="new_release" href="{}">{}</a>
                        </div>
                        <p class="info">{}</p>
                        <p class="info">{}</p>
                        <p class="info">{}</p>
                        <a href="{}" class="info">Download .torrent</a>
                        </div>
                    <div class="panel panel-default comments">"#,
                    update.nyaa_torrent.torrent_file.trim_end_matches(".torrent").replace("download", "view"),
                    update.nyaa_torrent.title,
                    update.nyaa_torrent.category,
                    update.nyaa_torrent.date,
                    update.nyaa_torrent.size,
                    update.nyaa_torrent.torrent_file
                ).as_str());
                if update.new_comments > 0 && config_file.smtp.comment_notifications {
                    for comment_index in update.nyaa_comments.len() as u64 - update.new_comments..update.nyaa_comments.len() as u64 {
                        html.push_str(update.nyaa_comments.get(comment_index as usize).unwrap().html.as_str());
                    };
                }
            } else if config_file.smtp.comment_notifications {
                html.push_str(format!(
                    r#"<div class="panel panel-default info-panel">
                        <div style="text-align: center;">
                            <a href="{}">{}</a>
                        </div>
                        <p class="info">{}</p>
                        <p class="info">{}</p>
                        <p class="info">{}</p>
                        </div>
                    <div class="panel panel-default comments">"#,
                    update.nyaa_torrent.torrent_file.trim_end_matches(".torrent").replace("download", "view"),
                    update.nyaa_torrent.title,
                    update.nyaa_torrent.category,
                    update.nyaa_torrent.date,
                    update.nyaa_torrent.size
                ).as_str());
                for comment_index in update.nyaa_comments.len() as u64 - update.new_comments..update.nyaa_comments.len() as u64 {
                    html.push_str(update.nyaa_comments.get(comment_index as usize).unwrap().html.as_str());
                };
            };
            html.push_str(r#"</div></body></html>"#);
            let smtp_creds = Credentials::new(config_file.smtp.smtp_username.clone(), config_file.smtp.smtp_password.clone());
            let email = Message::builder()
                .from(config_file.smtp.smtp_username.parse()?)
                .to(config_file.smtp.smtp_receiver.parse()?)
                .subject(config_file.smtp.smtp_subject.clone())
                .multipart(MultiPart::alternative()
                    .singlepart(SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(update.nyaa_torrent.date.clone()))
                    .singlepart(SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html.clone()),
                    ),
            ).expect("Failed to create message");
            let mail_transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&config_file.smtp.smtp_address);
            if mail_transport.is_ok() {
                let mail = mail_transport.unwrap().credentials(smtp_creds).build();
                if mail.send(email).await.is_err() {
                    println!("Failed to send message");
                    continue
                };
                println!("A new email has been sent.");
            }
        };
        if config_file.gotfiy.enabled {
            if update.new_torrent {
                let post_request = json!({
                    "message": "Has just been found.",
                    "priority": config_file.gotfiy.comment_priority,
                    "title": update.nyaa_torrent.title,
                });
                if send_gotify(config_file, post_request).is_err() {
                    println!("Failed to send a gotify message.");
                } else {
                    println!("Sent a gotify message.");
                }
            };
            if update.new_comments > 0 && config_file.gotfiy.comment_notifications {
                for comment_index in update.nyaa_comments.len() as u64 - update.new_comments..update.nyaa_comments.len() as u64 {
                    let comment = update.nyaa_comments.get(comment_index as usize).unwrap();
                    let post_request = json!({
                        "message": comment.user.clone() + ": " + &comment.message,
                        "priority": config_file.gotfiy.comment_priority,
                        "title": update.nyaa_torrent.title,
                    });
                    if send_gotify(config_file, post_request).is_err() {
                        println!("Failed to send a gotify message.");
                    } else {
                        println!("Sent a gotify message.");
                    }
                }
            }
        };
    };
    Ok(())
}


fn send_gotify(config_file: &ConfigFile, json: serde_json::Value) -> Result<(), ()> {
    let json_string = serde_json::to_string_pretty(&json).unwrap();
    let message = Request::post(config_file.gotfiy.domain.clone() + "/message?token=" + config_file.gotfiy.token.as_str())
    .header("accept", "application/json")
    .header("Content-Type", "application/json")
    .timeout(Duration::from_secs(10))
    .body(json_string);
    if message.is_ok() {
        if let Ok(request) = message {
            if request.send().is_ok() {
                return Ok(());
            };
        }
    }
    Err(())
}


fn get_nyaa(nyaa_url: &String) -> Result<String, ()> {
    println!("Requesting: {:?}", nyaa_url);
    let sending_request = Request::get(nyaa_url)
        .timeout(Duration::from_secs(10))
        .body(()).expect("Failed to create request.").send();
    if sending_request.is_err() {
        return Err(());
    }
    let mut get_response = sending_request.unwrap();
    let response = match get_response.status() {
        StatusCode::OK => {
            match get_response.text() {
                Ok(yay) => yay,
                Err(_) => {
                    return Err(())
                }
            }
        },
        _ => {
            return Err(())
        }
    };
    Ok(response)
}
