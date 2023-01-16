use crate::{NyaaTorrent, Update, DiscordChannel};
use sqlx::Row;

pub async fn updates_to_main_database(updates: &[Update]) -> Result<(), sqlx::Error> {
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./data/nyaa-notifs.sqlite")
                .create_if_missing(true),
    ).await.unwrap();
    for update in updates.iter().cloned() {
        let comments = update.nyaa_torrent.comment_amount as i64;
        let seeders = update.nyaa_torrent.seeders as i64;
        let leechers = update.nyaa_torrent.leechers as i64;
        let completed = update.nyaa_torrent.completed as i64;
        let timestamp = update.nyaa_torrent.timestamp as i64;
        if update.new_torrent {
            sqlx::query!("INSERT INTO MAIN (Category, Title, Comments, Magnet, Torrent_File, Seeders, Leechers, Completed, Timestamp) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            update.nyaa_torrent.category, update.nyaa_torrent.title, comments, update.nyaa_torrent.magnet, update.nyaa_torrent.torrent_file, 
            seeders, leechers, completed, timestamp
            ).execute(&database).await.expect("insert error");
        } else {
            sqlx::query!("UPDATE Main SET Category=?, Title=?, Comments=?, Seeders=?, Leechers=?, Completed=? WHERE Torrent_File=?",
            update.nyaa_torrent.category, update.nyaa_torrent.title, comments, seeders, leechers, completed, update.nyaa_torrent.torrent_file
            ).execute(&database).await.expect("insert error");
        }
    };
    println!("Updated database");
    database.close().await;
    Ok(())
}


pub async fn get_main_database() -> Result<Vec<NyaaTorrent>, sqlx::Error> {
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./data/nyaa-notifs.sqlite")
                .create_if_missing(true),
        ).await.unwrap();
    sqlx::migrate!("./migrations").run(&database).await.unwrap();
    if sqlx::query!("SELECT * FROM Main").fetch_one(&database).await.is_ok() {
        let rows: Vec<NyaaTorrent> = sqlx::query!("SELECT * FROM Main").fetch_all(&database).await.unwrap().iter().map(|row| NyaaTorrent {
            title: row.Title.clone(),
            category: row.Category.as_ref().unwrap().to_string(),
            comment_amount: row.Comments.unwrap() as u64,
            size: "NULL".to_string(),
            torrent_file: row.Torrent_File.clone(),
            magnet: row.Magnet.clone(),
            date: "NULL".to_string(),
            seeders: row.Seeders.unwrap() as u64,
            leechers: row.Leechers.unwrap() as u64,
            completed: row.Completed.unwrap() as u64,
            timestamp: row.Timestamp.unwrap() as u64,
            uploader_avatar: None,
            comments: None
        } ).collect();
        database.close().await;
        Ok(rows)
    } else {
        database.close().await;
        Ok([].to_vec())
    }
}

pub async fn add_discord_channel(channel: DiscordChannel) -> Result<(), sqlx::Error> {
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./data/nyaa-notifs.sqlite")
                .create_if_missing(true),
    ).await.unwrap();
    let url_string =
    {
        let mut str: String = String::new();
        for url in channel.urls
        {
            str.push_str(&(url+","));
        }
        str = str.trim_end_matches(", ").to_string();
        str
    };
    let activated = channel.activated.to_string();
    let release = channel.releases.to_string();
    let comments = channel.comments.to_string();

    sqlx::query!("INSERT INTO FRONT (activated, releases, comments, channel_id, urls) VALUES(?1, ?2, ?3, ?4, ?5)",
    activated, release, comments, channel.channel_id, url_string
    ).execute(&database).await.expect("insert error");
    sqlx::query(format!("CREATE TABLE _{:?} (
        Category TEXT NO NULL,
        Title TEXT NOT NULL,
        Comments INTEGER,
        Magnet TEXT NOT NULL,
        Torrent_File TEXT NOT NULL,
        Seeders INTERGER,
        Leechers INTEGER,
        Completed INTEGER,
        Timestamp INTEGER
    )", channel.channel_id).as_str()).execute(&database).await.unwrap();
    println!("Added new discord channel");
    database.close().await;
    Ok(())
}

pub async fn check_for_channel_id(channel_id: i64) -> Result<Vec<DiscordChannel>, sqlx::Error> {
    let channels = get_discord_channels().await.unwrap();
    for channel in channels
    {
        if channel.channel_id == channel_id
        {
            return Ok(vec![channel]);
        }
    }
    Ok(vec![])
}

pub async fn get_discord_channels() -> Result<Vec<DiscordChannel>, sqlx::Error> {
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./data/nyaa-notifs.sqlite")
                .create_if_missing(true),
    ).await.unwrap();
    let channels: Vec<DiscordChannel> = sqlx::query!("SELECT * FROM FRONT").fetch_all(&database).await.unwrap().iter().map(|record| DiscordChannel {
        activated: record.activated.clone().unwrap() == "true",
        releases: record.releases.clone().unwrap() == "true",
        comments: record.comments.clone().unwrap() == "true",
        channel_id: record.channel_id.unwrap() as i64,
        urls: record.urls.clone().unwrap().split(',').map(|str| str.to_string()).collect()
    }).collect();
    database.close().await;
    Ok(channels)
}

pub async fn update_discord_bot(channel_id: i64, pause: bool, reset: bool) -> Result<(), sqlx::Error> {
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./data/nyaa-notifs.sqlite")
                .create_if_missing(true),
    ).await.unwrap();
    if reset {
        sqlx::query!("DELETE FROM FRONT WHERE channel_id=?",
        channel_id).execute(&database).await.expect("insert error");
        sqlx::query(format!("DROP TABLE _{}", channel_id).as_str()).execute(&database).await.unwrap();
    }
    else
    {
        let activated = if pause {
            false.to_string()
        } else {
            true.to_string()
        };
        sqlx::query!("UPDATE FRONT SET activated=? WHERE channel_id=?",
        activated, channel_id).execute(&database).await.expect("insert error");
    }
    database.close().await;
    Ok(())
}

pub async fn get_channel_database(channel_id: i64) -> Result<Vec<NyaaTorrent>, sqlx::Error> {
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./data/nyaa-notifs.sqlite")
                .create_if_missing(true),
    ).await.unwrap();
    let db = sqlx::query(format!("SELECT * FROM _{}", &channel_id).as_str()).fetch_all(&database).await.unwrap();
    let mut torrents: Vec<NyaaTorrent> = vec![];

    for row in db {
        let comments: f64 = row.get_unchecked(2);
        let seeders: f64 = row.get_unchecked(5);
        let leechers: f64 = row.get_unchecked(6);
        let completed: f64 = row.get_unchecked(7);
        let timestamp: f64 = row.get_unchecked(8);

        let torrent = NyaaTorrent {
            category: row.get(0),
            title: row.get(1),
            magnet: row.get(3),
            torrent_file: row.get(4),
            size: "NULL".to_string(),
            date: "NULL".to_string(),
            uploader_avatar: None,
            seeders: seeders as u64,
            comment_amount: comments as u64,
            leechers: leechers as u64,
            completed: completed as u64,
            timestamp: timestamp as u64,
            comments: None
        };
        torrents.append(&mut vec![torrent]);
    }
    Ok(torrents)
}

pub async fn update_channel_db(channel_id: i64, updates: &[Update]) -> Result<(), sqlx::Error> {
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./data/nyaa-notifs.sqlite")
                .create_if_missing(true),
    ).await.unwrap();
    for update in updates.iter().cloned() {
        let comment_amount = update.nyaa_torrent.comment_amount as i64;
        let seeders = update.nyaa_torrent.seeders as i64;
        let leechers = update.nyaa_torrent.leechers as i64;
        let completed = update.nyaa_torrent.completed as i64;
        let timestamp = update.nyaa_torrent.timestamp as i64;
        if update.new_torrent {
            sqlx::query(format!("INSERT INTO _{} (Category, Title, Comments, Magnet, Torrent_File, Seeders, Leechers, Completed, Timestamp) 
            VALUES ({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?})",
            channel_id, update.nyaa_torrent.category, update.nyaa_torrent.title, comment_amount, update.nyaa_torrent.magnet, update.nyaa_torrent.torrent_file, 
            seeders, leechers, completed, timestamp).as_str()
            ).execute(&database).await.expect("insert error");
        } else {
            sqlx::query(format!("UPDATE _{} SET Category={:?}, Title={:?}, Comments={:?}, Seeders={:?}, Leechers={:?}, Completed={:?} WHERE Torrent_File={:?}",
            channel_id, update.nyaa_torrent.category, update.nyaa_torrent.title, comment_amount, seeders, leechers, completed, update.nyaa_torrent.torrent_file).as_str()
            ).execute(&database).await.expect("insert error");
        }
    };
    println!("Updated discord database");
    database.close().await;
    Ok(())
}
