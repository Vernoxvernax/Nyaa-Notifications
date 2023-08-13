use std::{sync::Arc, time::Duration, thread};
use chrono::TimeZone;
use isahc::{prelude::Configurable, RequestExt, http::StatusCode};
use lettre::{transport::smtp::authentication::Credentials, Message, message::{MultiPart, SinglePart, header}, AsyncSmtpTransport, Tokio1Executor, AsyncTransport};
use serde_json::json;
use serenity::{prelude::GatewayIntents, http::Http, Client, framework::StandardFramework};

use crate::config::{ModuleConfig, ModuleType};
use crate::discord::{Handler, discord_send_updates, limit_string_length};
use crate::database::Database;
use crate::web::{NyaaUpdate, NyaaCommentUpdateType, NyaaComment};

pub struct Notifications {
  http: Option<Arc<Http>>
}

impl Notifications {
  pub async fn new(modules: Vec<ModuleConfig>, database: &mut Database) -> Result<Self, ()> {
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let http: Arc<Http>;
    for module in modules {
      if module.active && (module.module_type == ModuleType::Discord) {
        let token = module.discord_token.unwrap();
        if let Ok(mut client) = Client::builder(token.clone(), intents)
        .event_handler(Handler {
          database_pool: database.database.clone(),
          discord_bot_id: module.discord_bot_id.unwrap(),
          discord_activity_type: module.discord_bot_activity_type.unwrap(),
          discord_activity_text: module.discord_bot_activity_text.unwrap()
        }).framework(StandardFramework::new())
        .await {
          http = client.cache_and_http.http.clone();
          tokio::spawn(async move {
            loop {
              if client.start().await.is_err() {
                eprintln!("Failed to start discord bot, trying again.");
              };
              tokio::time::sleep(Duration::from_secs(300)).await;
            }
          });
          return Ok(Notifications { http: Some(http) })
        }
      }
    }
    Ok(Notifications { http: None })
  }

  pub async fn process_updates(&mut self, module: &ModuleConfig, database: &mut Database, updates: Vec<NyaaUpdate>) -> Vec<NyaaUpdate> {
    match module.module_type {
      ModuleType::Email => {
        return email_send_updates(module, updates).await;
      },
      ModuleType::Gotify => {
        if let Ok(updates) = gotify_create_updates(module, updates).await {
          return updates
        }
      },
      ModuleType::Discord => {
        if module.active {
          if let Ok(updates) = discord_send_updates(self.http.clone().unwrap().to_owned(), module, updates).await {
            return updates
          } else {
            database.pause_discord_channel(&module.discord_bot_id.clone().unwrap(), module.discord_channel_id.unwrap(), false).await;
          }
        }
      }
    }
    vec![]
  }
}

async fn gotify_create_updates(module: &ModuleConfig, updates: Vec<NyaaUpdate>) -> Result<Vec<NyaaUpdate>, ()> {
  let mut successful_updates: Vec<NyaaUpdate> = vec![];
  for update in updates {
    let title = limit_string_length(&update.torrent.title, 75);
    let mut only_upload = update.torrent.clone();
    only_upload.comments = vec![];
    if update.new_upload && module.uploads.unwrap(){
      let message = format!("{} | {} | #{}", update.torrent.category, update.torrent.size, update.torrent.id);
      if let Ok(()) = gotify_send_message(module, &title, message, module.gotify_upload_priority.unwrap()).await {
        successful_updates.append(&mut vec![NyaaUpdate {
          new_upload: true,
          torrent: only_upload
        }]);
      } else {
        continue;
      }
    } else if update.new_upload && ! module.uploads.unwrap() {
      successful_updates.append(&mut vec![NyaaUpdate {
        new_upload: true,
        torrent: only_upload
      }]);
    }

    let mut only_comment_updates = update.torrent.clone();
    only_comment_updates.comments = vec![];
    if !update.torrent.comments.is_empty() && module.comments.unwrap() {
      for comment in update.torrent.comments {
        match comment.update_type {
          NyaaCommentUpdateType::DELETED => {
            let message = format!("{} [DELETED]: {}", comment.user.username, comment.message);
            if let Err(()) = gotify_send_message(module, &title, message, module.gotify_comment_priority.unwrap()).await {
              only_comment_updates.comments.append(&mut vec![NyaaComment {
                user: comment.user,
                message: comment.message,
                old_message: comment.old_message,
                uploader: comment.uploader,
                date_timestamp: comment.date_timestamp,
                edited_timestamp: comment.edited_timestamp,
                old_edited_timestamp: comment.old_edited_timestamp,
                direct_link: comment.direct_link,
                update_type: NyaaCommentUpdateType::UNCHECKED
              }]);
            };
          },
          NyaaCommentUpdateType::EDITED => {
            let message = format!("{} [EDITED]: {}", comment.user.username, comment.message);
            if let Ok(()) = gotify_send_message(module, &title, message, module.gotify_comment_priority.unwrap()).await {
              only_comment_updates.comments.append(&mut vec![comment]);
            } else {
              only_comment_updates.comments.append(&mut vec![NyaaComment {
                user: comment.user,
                message: comment.old_message.unwrap(),
                old_message: None,
                uploader: comment.uploader,
                date_timestamp: comment.date_timestamp,
                edited_timestamp: comment.old_edited_timestamp,
                old_edited_timestamp: None,
                direct_link: comment.direct_link,
                update_type: NyaaCommentUpdateType::UNCHECKED
              }]);
            };
          },
          NyaaCommentUpdateType::NEW => {
            let message = format!("{} [NEW]: {}", comment.user.username, comment.message);
            if let Ok(()) = gotify_send_message(module, &title, message, module.gotify_comment_priority.unwrap()).await {
              let mut finished_comment = comment.clone();
              finished_comment.update_type = NyaaCommentUpdateType::UNCHECKED;
              only_comment_updates.comments.append(&mut vec![finished_comment]);
            };
          },
          NyaaCommentUpdateType::UNDECIDED => {
            only_comment_updates.comments.append(&mut vec![comment]);
          },
          NyaaCommentUpdateType::UNCHECKED => {
            only_comment_updates.comments.append(&mut vec![comment]);
          }
        }
      }
      successful_updates.append(&mut vec![NyaaUpdate {
        new_upload: false,
        torrent: only_comment_updates
      }]);
    } else if ! module.comments.unwrap() {
      for comment in update.torrent.comments.clone() {
        if comment.update_type == NyaaCommentUpdateType::DELETED {
          continue;
        }
        only_comment_updates.comments.append(&mut vec![comment]);
      }
      successful_updates.append(&mut vec![NyaaUpdate {
        new_upload: false,
        torrent: only_comment_updates
      }]);
    }
  }
  Ok(successful_updates)
}

async fn gotify_send_message(module: &ModuleConfig, title: &str, message: String, priority: u32) -> Result<(), ()> {
  let json_body = serde_json::to_string(
    &json!({
      "title": title,
      "message": message,
      "priority": priority
    })
  ).unwrap();

  let url = format!("{}/message?token={}", module.gotify_domain.clone().unwrap(), module.gotify_token.clone().unwrap());
  let post_request = isahc::Request::post(url)
    .header("Content-Type", "application/json")
    .timeout(Duration::from_secs(5))
    .body(json_body).expect("Failed to create request.")
  .send();

  thread::sleep(Duration::from_secs(2));

  if let Ok(request) = post_request {
    if request.status() == StatusCode::OK {
      Ok(())
    } else {
      Err(())
    }
  } else {
    Err(())
  }
}

async fn email_send_updates(module: &ModuleConfig, updates: Vec<NyaaUpdate>) -> Vec<NyaaUpdate> {
  let mut successful_updates: Vec<NyaaUpdate> = vec![];
  for update in updates {
    if ! update.new_upload && update.torrent.comments.iter().all(|c| {
      c.update_type == NyaaCommentUpdateType::UNDECIDED || c.update_type == NyaaCommentUpdateType::UNCHECKED
    }) {
      successful_updates.append(&mut vec![update]);
      continue;
    }

    let mut html = HTML_HEAD.to_string();
    let title = html_escape::encode_quoted_attribute(&update.torrent.title).to_string();
    if update.new_upload && module.uploads.unwrap() {
      html.push_str(format!(
        r#"<div class="panel panel-default info-panel new_release">
        <div style="text-align: center;">
          <a class="new_release" href="{}">{}</a>
        </div>
        <p class="info">{}</p>
        <p class="info">{}</p>
        <p class="info">{}</p>
        <a href="{}" class="info">Download .torrent</a>
        </div>"#,
        format!("{}view/{}", update.torrent.domain, update.torrent.id),
        title,
        update.torrent.category,
        update.torrent.upload_date_str,
        update.torrent.size,
        format!("{}download/{}.torrent", update.torrent.domain, update.torrent.id)
      ).as_str());
    } else {
      html.push_str(format!(
        r#"<div class="panel panel-default info-panel">
        <div style="text-align: center;">
          <a href="{}">{}</a>
        </div>
        <p class="info">{}</p>
        <p class="info">{}</p>
        <p class="info">{}</p>
        </div>"#,
        format!("{}view/{}", update.torrent.domain, update.torrent.id),
        title,
        update.torrent.category,
        update.torrent.upload_date_str,
        update.torrent.size
      ).as_str());
    }

    if module.comments.unwrap() {
      for comment in update.torrent.comments.clone() {
        let timestamp: String;
        let message: String;
        match comment.update_type {
          NyaaCommentUpdateType::DELETED => {
            timestamp = chrono::offset::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            message = comment.message;
          },
          NyaaCommentUpdateType::EDITED => {
            timestamp = chrono::Utc.timestamp_opt(comment.edited_timestamp.unwrap() as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S").to_string();
            message = comment.message;
          },
          NyaaCommentUpdateType::NEW => {
            timestamp = chrono::Utc.timestamp_opt(comment.date_timestamp as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S").to_string();
            message = comment.message;
          },
          NyaaCommentUpdateType::UNDECIDED => {
            continue;
          },
          NyaaCommentUpdateType::UNCHECKED => {
            continue;
          }
        }
  
        let text_color = text_color_from_role(comment.user.role);
        let text_style = if comment.user.banned {
          " strike"
        } else {
          ""
        };
  
        html.push_str(format!(
          r#"<div class="panel panel-default comment-panel" id="com-1">
          <div class="panel-body">
            <div class="col-md-2">
              <p>
                <a class="text-{}{}" href="{}" data-toggle="tooltip" title="User">{}</a>
              </p>
              <img class="avatar" src="{}" alt="User">
            </div>
            <div class="col-md-10 comment">
              <div class="row comment-details">
                <a href="{}"><small data-timestamp-swap>{}</small></a>
                <div class="comment-actions">
                </div>
              </div>
              <div class="row comment-body">
                <div markdown-text class="comment-content" id="comment">{}</div>
              </div>
            </div>
          </div>
          </div>"#,
          text_color, text_style,
          format!("{}user/{}", update.torrent.domain, comment.user.username.clone()),
          comment.user.username.clone(),
          comment.user.avatar.clone().unwrap(),
          comment.direct_link.clone(),
          timestamp,
          message
        ).as_str());
      }
    }

    html.push_str(r#"</div></body></html>"#);

    let smtp_creds = Credentials::new(module.smtp_username.clone().unwrap(), module.smtp_password.clone().unwrap());
    let domain = module.smtp_domain.clone().unwrap();
    let already_successful = false;
    for recipient in module.smtp_recipients.clone().unwrap() {
      let email = Message::builder()
        .from(module.smtp_username.clone().unwrap().parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject(module.smtp_subject.clone().unwrap())
        .multipart(MultiPart::alternative()
          .singlepart(SinglePart::builder()
            .header(header::ContentType::TEXT_HTML)
            .body(html.clone())
          )
        )
      .expect("Failed to create message.");
      let mail_transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&domain);
      if mail_transport.is_ok() && ! already_successful {
        let mail = mail_transport.unwrap().credentials(smtp_creds.clone()).build();
        if mail.send(email).await.is_err() {
          eprintln!("Failed to send message");
          continue
        } else {
          let mut prepared_update = update.clone();
          for comment in prepared_update.torrent.comments.iter_mut() {
            if (comment.update_type != NyaaCommentUpdateType::UNCHECKED) || (comment.update_type != NyaaCommentUpdateType::UNDECIDED) {
              comment.update_type = NyaaCommentUpdateType::UNCHECKED;
            }
          }
          successful_updates.append(&mut vec![prepared_update]);
        }
      }
    }
  }
  successful_updates
}

fn text_color_from_role(role: String) -> &'static str {
  match role.as_str() {
    "Administrator" => {
      "purple"
    },
    "Trusted" => {
      "success"
    },
    _ => {
      "default"
    }
  }
}

static HTML_HEAD: &str = r#"<!DOCTYPE html>
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
</head><body>"#;
