-- Add migration script here
create table guild_preferences (
  guild_id BIGINT PRIMARY KEY,
  volume   INTEGER DEFAULT 100
);
