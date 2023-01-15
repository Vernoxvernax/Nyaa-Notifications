-- Add migration script here
CREATE TABLE FRONT (
    activated TEXT NO NULL,
    comments TEXT NO NULL,
    releases TEXT NO NULL,
    channel_id INTEGER,
    urls TEXT NO NULL
)