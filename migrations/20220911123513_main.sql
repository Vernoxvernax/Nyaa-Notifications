-- Add migration script here
CREATE TABLE MAIN (
    Category TEXT NO NULL,
    Title TEXT NOT NULL,
    Comments INTEGER,
    Magnet TEXT NOT NULL,
    Torrent_File TEXT NOT NULL,
    Seeders INTERGER,
    Leechers INTEGER,
    Completed INTEGER,
    Timestamp INTEGER
)