-- Your SQL goes here
CREATE TABLE IF NOT EXISTS folders (
  id INTEGER NOT NULL PRIMARY KEY,
  uuid VARCHAR NOT NULL UNIQUE,
  name VARCHAR NOT NULL,
  sort INTEGER NOT NULL DEFAULT 0,
  create_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  update_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS folder_channel_relations (
  id INTEGER NOT NULL PRIMARY KEY,
  folder_uuid VARCHAR NOT NULL,
  channel_uuid VARCHAR NOT NULL,
  create_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE("folder_uuid", "channel_uuid")
);

CREATE TABLE IF NOT EXISTS feed_metas (
  id INTEGER NOT NULL PRIMARY KEY,
  uuid VARCHAR NOT NULL UNIQUE,
  channel_uuid VARCHAR NOT NULL,
  parent_uuid VARCHAR NOT NULL,
  sort INTEGER NOT NULL DEFAULT 0,
  create_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  update_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_channel_uuid_and_read_status on articles(channel_uuid, read_status);
