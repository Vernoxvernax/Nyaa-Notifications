use crate::{NyaaTorrent, Update};


pub async fn updates_to_database(updates: &Vec<Update>) -> Result<(), sqlx::Error> {
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./data/nyaa-notifs.sqlite")
                .create_if_missing(true),
    ).await.unwrap();
    for update in updates.clone() {
        let comments = update.nyaa_torrent.comments as i64;
        let seeders = update.nyaa_torrent.seeders as i64;
        let leechers = update.nyaa_torrent.leechers as i64;
        let completed = update.nyaa_torrent.completed as i64;
        let timestamp = update.nyaa_torrent.timestamp as i64;
        sqlx::query!("INSERT INTO MAIN (Category, Title, Comments, Magnet, Torrent_File, Seeders, Leechers, Completed, Timestamp) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        update.nyaa_torrent.category, update.nyaa_torrent.title, comments, update.nyaa_torrent.magnet, update.nyaa_torrent.torrent_file, 
        seeders, leechers, completed, timestamp
        ).execute(&database).await.expect("insert error");
    };
    database.close().await;
    Ok(())
}


pub async fn get_database() -> Result<Vec<NyaaTorrent>, sqlx::Error> {
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
            comments: row.Comments.unwrap() as u64,
            size: "NULL".to_string(),
            torrent_file: row.Torrent_File.clone(),
            magnet: row.Magnet.clone(),
            date: "NULL".to_string(),
            seeders: row.Seeders.unwrap() as u64,
            leechers: row.Leechers.unwrap() as u64,
            completed: row.Completed.unwrap() as u64,
            timestamp: row.Timestamp.unwrap() as u64
        } ).collect();
        database.close().await;
        return Ok(rows);
    } else {
        database.close().await;
        return Ok([].to_vec());
    }
}