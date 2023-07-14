use crate::web::{NyaaTorrent, NyaaComment, NyaaUser};

pub fn serialize_feed(html: String, domain: &str) -> Result<(Vec<NyaaTorrent>, bool), String> {
  let id_reg = regex::Regex::new(r"/download/([0-9]{5,})").unwrap();
  if ! html.starts_with(&"<!DOCTYPE html>".to_string()) {
    return Err("This is not plaintext html code!".to_string());
  }
  let lines = html.split('\n');
  let mut worthy_text: usize = 0;

  // Skipping trash like the menu and footer
  for (index, line) in lines.clone().enumerate() {
    if line.trim() == "</nav>" {
      // as ez as that
      worthy_text = index;
      break
    } else if line.trim() == "</html>" {
      return Err("Couldn't split html code.".to_string())
    }
  };
  let mut body: Vec<String> = [].to_vec();
  for line in lines.collect::<Vec<&str>>().split_at(worthy_text).1 {
    body.append(&mut [line.to_string()].to_vec());
  };
  let mut body_iterator = body.iter();
  let mut torrent_list_end: bool = false;
  let mut torrents: Vec<NyaaTorrent> = [].to_vec();
  let mut continuing: bool = false;
  while let Some(line) = body_iterator.next() {
    let x = line.trim();
    if ! torrent_list_end && x == "<tbody>" {
      let mut category: String = String::new();
      let mut comments: String = String::new();
      let mut title: String = String::new();
      let mut magnet: String = String::new();
      let mut id: u64 = 0;
      let mut size: String = String::new();
      let mut date: String = String::new();
      let mut seeders: String = String::new();
      let mut leechers: String = String::new();
      let mut completed: String = String::new();
      let mut temp: String = String::new();
      let mut timestamp: String = String::new();
      let mut comments_found: bool = false;
      for line in body_iterator.by_ref() {
        let x = line.trim();
        // Gathering category string
        if x.contains(r#"class="category-icon""#) {
          let category_iterator = x.chars();
          for x in category_iterator {
            if x == '"' {
              if category.ends_with(r#"<img src="#) || category.ends_with(r#"png"#) || category.ends_with(r#"alt="#) {
                category.clear();
              } else {
                break
              }
            } else {
              category.push_str(x.to_string().as_str());
            }
          }
        } else if x.contains(r#"<i class="fa fa-comments"#) {
          let category_iterator = x.chars();
          for x in category_iterator {
            if x == '>' {
              if comments.ends_with('"') || comments.ends_with(r#"</i"#) {
                comments.clear();
              } else {
                break
              }
            } else {
              comments.push_str(x.to_string().as_str());
            }
          }
          comments = comments.trim_end_matches(r#"</a"#).to_string();
          comments_found = true;
        } else if x.starts_with(r#"<a href="/view/"#) && x.ends_with(r#"</a>"#) && title == *"" {
          if ! comments_found {
            comments = "0".to_string()
          };
          let category_iterator = x.chars();
          for ch in category_iterator {
            if ch == '>' {
              if title.ends_with('"') {
                title.clear();
              } else {
                break
              }
            } else {
              title.push_str(ch.to_string().as_str());
            }
          }
          title = title.trim_end_matches(r#"</a"#).to_string();
        } else if x.ends_with(r#"fa-download"></i></a>"#) {
          let mut result = vec![];
          for (_, [id]) in id_reg.captures_iter(x).map(|c| c.extract()) {
            result.push(id);
          }
          id = result.first().unwrap().parse::<u64>().unwrap();
        } else if x.starts_with(r#"<a href="magnet:?xt"#) {
          let iterator = x.chars();
          for x in iterator {
            if x == '"' {
              if magnet.ends_with(r#"href="#) {
                magnet.clear();
              } else {
                break
              }
            } else {
              magnet.push_str(x.to_string().as_str());
            }
          }
        } else if x.starts_with(r#"<td class="text-center""#) && x.ends_with(r#"</td>"#) {
          let iterator = x.chars();
          for x in iterator {
            if x == '>' {
              if temp.ends_with('"') {
                temp.clear();
              } else if size == *"" {
                size.push_str(temp.to_string().as_str().trim_end_matches(r#"</td"#));
              } else if date == *"" {
                date.push_str(temp.to_string().as_str().trim_end_matches(r#"</td"#));
              } else if seeders == *"" {
                seeders.push_str(temp.to_string().as_str().trim_end_matches(r#"</td"#));
              } else if leechers == *"" {
                leechers.push_str(temp.to_string().as_str().trim_end_matches(r#"</td"#));
              } else if completed == *"" {
                completed.push_str(temp.to_string().as_str().trim_end_matches(r#"</td"#));
                torrents.append(&mut [NyaaTorrent {
                  category: category.clone(),
                  title: html_escape::decode_html_entities(&title).to_string(),
                  comments_amount: comments.parse::<u64>().unwrap(),
                  magnet_link: magnet.clone(),
                  size: size.clone(),
                  id,
                  domain: domain.to_owned(),
                  uploader: None,
                  upload_date_str: date.clone(),
                  seeders: seeders.parse::<u64>().unwrap(),
                  leechers: leechers.parse::<u64>().unwrap(),
                  completed: completed.parse::<u64>().unwrap(),
                  upload_date_timestamp: timestamp.parse::<f64>().unwrap(),
                  comments: vec![]
                }].to_vec());
                category = String::new();
                comments = String::new();
                title = String::new();
                magnet = String::new();
                size = String::new();
                date = String::new();
                seeders = String::new();
                leechers = String::new();
                completed = String::new();
                temp = String::new();
                timestamp = String::new();
                comments_found = false;
              }
            } else {
              temp.push_str(x.to_string().as_str());
            }
          }
        } else if x.starts_with(r#"<li><a rel="next"#) && x.ends_with(r#">&raquo;</a></li>"#) {
          continuing = true;
        };
        if x.contains("data-timestamp=") && x.ends_with(r#"</td>"#) {
          let iterator = x.chars();
          for ch in iterator {
            timestamp.push_str(ch.to_string().as_str());
            if ch == '"' && timestamp.contains(r#"data-timestamp"#) {
              timestamp.clear()
            } else if ch == '"' && ! timestamp.starts_with('<') {
              timestamp.pop();
              break
            };
          }
        };
      };
      torrent_list_end = true;
    }
  }
  Ok((torrents, continuing))
}

pub fn serialize_torrent(html: &str, page_url: String, domain: &str) -> (Option<NyaaUser>, Vec<NyaaComment>) {
  let mut short_domain = domain.to_owned();
  short_domain.pop();
  let lines = html.split('\n');
  let mut worthy_text: usize = 0;
  // Skipping trash like the menu and footer
  for (index, line) in lines.clone().enumerate() {
    if line.trim() == r#"<div id="comments" class="panel panel-default">"# {
      // as shrimple as that
      worthy_text = index;
      break
    }
  };
  let mut body: Vec<String> = [].to_vec();
  for line in lines.collect::<Vec<&str>>().split_at(worthy_text).1 {
    body.append(&mut [line.to_string()].to_vec());
  };
  let mut comments: Vec<NyaaComment> = [].to_vec();
  let mut body_iterator = body.iter();
  let mut torrent_page_end: bool = false;
  while let Some(line) = body_iterator.next() {
    let x = line.trim();
    if ! torrent_page_end && x.contains("comments") {
      let mut username = String::new();
      let mut user_role = String::new();
      let mut banned = false;
      let mut avatar = String::new();
      let mut uploader = false;
      let mut message = String::new();
      let mut date_timestamp: f64 = 0.0;
      let mut edited_timestamp: Option<f64> = None;
      let mut direct_link: String = String::new();
      for line in body_iterator.by_ref() {
        let x = line.trim();
        if x.contains("data-timestamp=") && ! x.contains(">(edited)</small>") {
          let mut last_part = "";
          for part in x.split('"') {
            if last_part == "<a href=" {
              direct_link = page_url.clone()+part;
              break;
            } else {
              last_part = part;
            }
          }
          let iterator = x.chars();
          let mut date_timestamp_temp = String::new();
          for ch in iterator {
            date_timestamp_temp.push_str(ch.to_string().as_str());
            if ch == '"' && date_timestamp_temp.ends_with(r#"data-timestamp=""#) {
              date_timestamp_temp.clear()
            } else if ch == '"' && ! date_timestamp_temp.starts_with('<') {
              date_timestamp_temp.pop();
              date_timestamp = date_timestamp_temp.parse::<f64>().unwrap();
              break
            }
          }
        } else if x.contains(">(edited)</small>") {
          let mut last_part = "";
          for part in x.split('"') {
            if last_part == " data-timestamp=" {
              edited_timestamp = Some(part.parse::<f64>().unwrap());
              break;
            } else {
              last_part = part;
            }
          }
        } else if x.contains(r#"href="/user/"#) {
          let characters = x.chars();
          let mut last_part = "";
          for part in x.split('\"') {
            if last_part == " title=" {
              user_role = part.to_string();
              if user_role.contains("BANNED") {
                banned = true;
              }
              break;
            } else {
              last_part = part;
            }
          }
          for ch in characters {
            if ch == '>' {
              username.clear()
            } else if ch == '<' && ! username.is_empty() {
              break
            } else {
              username.push_str(ch.to_string().as_str());
            }
          }
        } else if x.ends_with("(uploader)") {
          uploader = true;
        } else if x.starts_with(r#"<img class="avatar" src=""#) {
          let mut last_part = "";
          for part in x.split('"') {
            if last_part == " src=" {
              avatar = part.to_string();
              break;
            } else {
              last_part = part;
            }
          }
          if avatar.starts_with('/') {
            avatar = short_domain.clone()+&avatar;
          }
        } else if x.contains(r#"comment-content"#) {
          for part in x.split('>') {
            if part.ends_with("</div") {
              message = part.trim_end_matches("</div").to_string();
              break;
            }
          }
          if (message.contains("![](") || message.contains("![") && message.contains("](")) && message.contains(')') {
            let mut remove_these: Vec<(usize, usize, usize, usize, usize)> = vec![];
            let mut exclamation: bool = false;
            let mut open_sq_br: bool = false;
            let mut closed_sq_br: bool = false;
            let mut open_ro_br: bool = false;
            let mut values = (0, 0, 0, 0, 0);
            
            for (index, ch) in message.char_indices() {
              if ch == '!' {
                values.0 = index;
                exclamation = true;
              } else if ch == '[' && exclamation {
                open_sq_br = true;
                values.1 = index;
              } else if ch == ']' && open_sq_br && exclamation {
                closed_sq_br = true;
                values.2 = index;
              } else if ch == '(' && closed_sq_br && open_sq_br && exclamation {
                values.3 = index;
                open_ro_br = true;
              } else if ch == ')' && open_ro_br {
                exclamation = false;
                open_sq_br = false;
                closed_sq_br = false;
                open_ro_br = false;
                values.4 = index;
                remove_these.append(&mut vec![(values)]);
              } else if ! (open_ro_br || open_sq_br) {
                exclamation = false;
                open_sq_br = false;
                closed_sq_br = false;
                open_ro_br = false;
              }
            };

            for (excl, start, mid, start2, end) in remove_these.iter().rev() {
              message.remove(*end);
              message.remove(*start2);
              message.remove(*mid);
              message.remove(*start);
              message.remove(*excl);
            }
          }

          comments.append(&mut [NyaaComment {
            user: NyaaUser {
              anonymous: false,
              role: user_role.clone(),
              username,
              avatar: Some(avatar),
              banned
            },
            message: html_escape::decode_html_entities(message.trim_end_matches(r#"</div"#)).to_string(),
            old_message: None,
            uploader,
            date_timestamp,
            edited_timestamp,
            old_edited_timestamp: None,
            direct_link: direct_link.clone(),
            update_type: crate::web::NyaaCommentUpdateType::UNDECIDED
          }].to_vec());
          user_role = String::new();
          username = String::new();
          avatar = String::new();
          banned = false;
          message = String::new();
          uploader = false;
          date_timestamp = 0.0;
          edited_timestamp = None;
          direct_link = String::new();
        }
      };
      torrent_page_end = true;
    };
  };
  (get_uploader_name(html), comments)
}

pub fn get_uploader_name(html: &str) -> Option<NyaaUser> {
  if ! html.starts_with("<!DOCTYPE html>") {
    return None;
  };
  let lines = html.split('\n');
  let mut uploader = String::new();
  let mut role = String::new();
  let mut banned = false;
  let mut submitter_line: bool = false;
  for line in lines {
    if line.ends_with(r#"<div class="panel panel-success">"#) {
      role = "Trusted".to_string();
    } else if line.ends_with(r#"<div class="col-md-1">Submitter:</div>"#) {
      submitter_line = true;
    } else if submitter_line {
      let mut last_part = "";
      if line.contains(r#"href="/user/"#) {
        for part in line.split('"') {
          if last_part == " title=" {
            role = part.to_string();
            if role.contains("BANNED") {
              banned = true;
            }
            last_part = ">.< -!- >~<";
            continue;
          } else if last_part == ">.< -!- >~<" {
            uploader = part.trim_start_matches('>').trim_end_matches("</a>\t\t\t</div>").to_string();
            break;
          } else {
            last_part = part;
          }
        }
        if !uploader.is_empty() {
          break;
        }
      }
    } else if line.contains(r#"<div class="col-md-1">Seeders:</div>"#) {
      return Some(NyaaUser {
        anonymous: true,
        role,
        username: "Anonymous".to_string(),
        avatar: None,
        banned: false
      });
    }
  }

  Some(NyaaUser {
    anonymous: false,
    username: uploader,
    role,
    avatar: None,
    banned,
  })
}

pub fn serialize_user_page(html: &str, domain: &String) -> String {
  if ! html.starts_with(&"<!DOCTYPE html>".to_string()) {
    return "".to_string();
  };
  let lines = html.split('\n');
  let mut worthy_text = "";
  
  for line in lines.clone() {
    if line.contains(r#"<meta property="og:image" content=""#) {
      worthy_text = line;
      break
    } else if line.trim() == "</html>" {
      return "".to_string();
    }
  };
  
  let mut avatar: String = String::new();
  let mut delim = 0;
  for ch in worthy_text.chars() {
    if ch == '"' {
      if delim == 3 {
        continue;
      }
      delim += 1;
      continue;
    }
    if delim == 3 && ch != '>' {
      avatar.push(ch);
    }
  }
  if avatar.starts_with('/') {
    avatar = domain.to_owned()+&avatar;
  }
  avatar
}
