use crate::{NyaaComment, NyaaPage, NyaaTorrent};


pub fn serizalize_torrent_page(website: &String) -> Result<Vec<NyaaComment>, String> {
    if ! website.starts_with(&"<!DOCTYPE html>".to_string()) {
        return Err("This is not plaintext html code!".to_string())
    };
    let lines = website.split('\n');
    let mut worthy_text: usize = 0;
    // Skipping trash like the menu and footer
    for (index, line) in lines.clone().enumerate() {
        if line.trim() == r#"<div id="comments" class="panel panel-default">"# {
            // as easy as that
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
    let mut comments: Vec<NyaaComment> = [].to_vec();
    let mut body_iterator = body.iter();
    let mut torrent_page_end: bool = false;
    loop {
        let x = match body_iterator.next() {
            Some(lime) => {
                lime.trim()
            },
            None => {
                break
            }
        };
        if ! torrent_page_end && x.contains("comments") {
            let mut user = String::new();
            let mut gravatar = String::new();
            let mut text = String::new();
            let mut timestamp: String = String::new();
            let mut time_str: String = String::new();
        loop {
            let x = match body_iterator.next() {
                Some(lime) => {
                    lime.trim()
                },
                None => {
                    break
                }
            };
            if x.contains("data-timestamp=") {
                let iterator = x.chars();
                for ch in iterator {
                    timestamp.push_str(ch.to_string().as_str());
                    if ch == '"' && timestamp.ends_with(r#"data-timestamp=""#) {
                        timestamp.clear()
                    } else if ch == '"' && ! timestamp.starts_with('<') {
                        timestamp.pop();
                        break
                    }
                }
            };
            if x.contains(r#"<a class="text-default" href="/user/"#) {
                let characters = x.chars();
                for ch in characters {
                    if ch == '>' {
                        user.clear()
                    } else if ch == '<' && ! user.is_empty() {
                        break
                    } else {
                        user.push_str(ch.to_string().as_str());
                    }
                }
            } else if x.starts_with(r#"<img class="avatar" src=""#) {
                gravatar = x.trim_start_matches(r#"<img class="avatar" src=""#).trim_end_matches(r#"" alt="User">"#).to_string();
            } else if x.contains(r#"data-timestamp"#) {
                if ! time_str.is_empty() {
                    continue;
                }
                let mut a_tag: bool = false;
                let mut small_tag: bool = false;
                let mut now: bool = false;
                let characters = x.chars();
                for ch in characters {
                    if ch == '>' && ! a_tag {
                        a_tag = true;
                    } else if ch == '>' && ! small_tag {
                        small_tag = true;
                        time_str.clear()
                    } else if ch == '<' && now {
                        break
                    } else if small_tag {
                        time_str.push_str(ch.to_string().as_str());
                        now = true;
                    }
                }
            } else if x.contains(r#"comment-content"#) {
                let characters = x.chars();
                let mut on: bool = false;
                for ch in characters {
                    if ch == '>' {
                        on = true;
                    } else if on {
                        text.push_str(ch.to_string().as_str());
                    }
                };
                let html = format!(r#"<div class="panel panel-default comment-panel" id="com-1">
                        <div class="panel-body">
                            <div class="col-md-2">
                                <p>
                                    <a class="text-default" href="https://nyaa.si/user/{}" data-toggle="tooltip" title="User">{}</a>
                                </p>
                                <img class="avatar" src="{}" alt="User">
                            </div>
                            <div class="col-md-10 comment">
                                <div class="row comment-details">
                                    <a href="https://nyaa.si/user/{}"><small data-timestamp-swap>{}</small></a>
                                    <div class="comment-actions">
                                    </div>
                                </div>
                                <div class="row comment-body">
                                    <div markdown-text class="comment-content" id="comment">{}</div>
                                </div>
                            </div>
                        </div>
                        </div>"#,
                        user, user, gravatar.clone(), user, time_str, text.trim_end_matches(r#"</div"#));
                comments.append(&mut [NyaaComment {
                    html,
                    user: user.clone(),
                    message: html_escape::decode_html_entities(text.trim_end_matches(r#"</div"#)).to_string(),
                    timestamp: timestamp.clone(),
                    gravatar: gravatar.clone()
                }].to_vec());
                user = String::new();
                text = String::new();
                time_str = String::new();
                timestamp = String::new();
                gravatar = String::new();
                }
            };
        torrent_page_end = true;
        };
    };
    Ok(comments)
}


pub fn serizalize_user_page(website: &String) -> Result<NyaaPage, String> {
    if ! website.starts_with(&"<!DOCTYPE html>".to_string()) {
        return Err("This is not plaintext html code!".to_string())
    };
    let lines = website.split('\n');
    let mut worthy_text: usize = 0;

    // Skipping trash like the menu and footer
    for (index, line) in lines.clone().enumerate() {
        if line.trim() == "</nav>" {
            // as easy as that
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
    let mut user = String::from("None");
    let mut torrent_count = 0;
    let mut body_iterator = body.iter();
    let mut torrent_list_end: bool = false;
    let mut torrents: Vec<NyaaTorrent> = [].to_vec();
    let mut incomplete: bool = false;
    if website.contains("Browsing <span class=\"") {
        loop {
            let x = match body_iterator.next() {
                Some(lime) => {
                    lime.trim()
                },
                None => {
                    break
                }
            };
            if x.contains("Browsing <span class=\"") {
                let mut text = x.trim();
                text = text.strip_prefix(r#"Browsing <span class="text-default" data-toggle="tooltip" title="User">"#).unwrap();
                text = text.strip_suffix(r#"</span>'s torrents"#).unwrap();
                user = text.to_string();
                let x = body_iterator.next().unwrap();
                torrent_count = x.trim()[1..x.trim().len() - 1].parse::<u64>().unwrap();
                break
            }
        };
    };
    loop {
        let x = match body_iterator.next() {
            Some(lime) => {
                lime.trim()
            },
            None => {
                break
            }
        };
        if ! torrent_list_end && x == "<tbody>" {
            let mut category: String = String::new();
            let mut comments: String = String::new();
            let mut torrent_file: String = String::new();
            let mut title: String = String::new();
            let mut magnet: String = String::new();
            let mut size: String = String::new();
            let mut date: String = String::new();
            let mut seeders: String = String::new();
            let mut leechers: String = String::new();
            let mut completed: String = String::new();
            let mut temp: String = String::new();
            let mut timestamp: String = String::new();
            let mut comments_found: bool = false;
            loop {
                let x = match body_iterator.next() {
                    Some(lime) => {
                        lime.trim()
                    },
                    None => {
                        break
                    }
                };
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
                } else if x.starts_with(r#"<a href=""#) && x.ends_with(r#"</a>"#) && title == *"" {
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
                    let iterator = x.chars();
                    for x in iterator {
                        if x == '"' {
                            if torrent_file.ends_with(r#"href="#) {
                                torrent_file.clear();
                            } else {
                                break
                            }
                        } else {
                            torrent_file.push_str(x.to_string().as_str());
                        }
                    }
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
                                    title: title.clone(),
                                    comments: comments.parse::<u64>().unwrap(),
                                    magnet: magnet.clone(),
                                    torrent_file: "https://nyaa.si".to_owned() + &torrent_file,
                                    size: size.clone(),
                                    date: date.clone(),
                                    seeders: seeders.parse::<u64>().unwrap(),
                                    leechers: leechers.parse::<u64>().unwrap(),
                                    completed: completed.parse::<u64>().unwrap(),
                                    timestamp: timestamp.parse::<u64>().unwrap()
                                }].to_vec());
                                category = String::new();
                                comments = String::new();
                                torrent_file = String::new();
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
                    incomplete = true;
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
    Ok(NyaaPage {
        user,
        torrent_count,
        torrents,
        incomplete
    })
}