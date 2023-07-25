CREATE OR REPLACE FUNCTION manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;



CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  username TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT current_timestamp NOT NULL,
  updated_at TIMESTAMP DEFAULT current_timestamp NOT NULL
);
SELECT manage_updated_at('users');

CREATE TABLE sessions (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  ip CIDR NOT NULL,
  platform TEXT NOT NULL,
  user_agent TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expire_at TIMESTAMP NOT NULL,
  user_id UUID NOT NULL REFERENCES users
);

CREATE TABLE email_user_logins (
  user_id UUID PRIMARY KEY NOT NULL REFERENCES users,
  email VARCHAR(120) UNIQUE NOT NULL,
  password_hash BYTEA NOT NULL
);

CREATE TABLE discord_user_logins (
  user_id UUID PRIMARY KEY NOT NULL REFERENCES users,
  client_id TEXT UNIQUE NOT NULL
);

CREATE TABLE lists (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  parent_id UUID REFERENCES lists ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users,
  title TEXT NOT NULL,
  description TEXT,
  color VARCHAR(7) NOT NULL,
  CONSTRAINT color_hex_constraint
    CHECK (color ~* '^#[a-f0-9]{6}$')
);
CREATE INDEX list_user_idx ON lists(user_id);
CREATE INDEX list_parent_idx ON lists(parent_id);

CREATE TABLE tasks (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  parent_id UUID REFERENCES tasks ON DELETE CASCADE,
  list_id UUID NOT NULL REFERENCES lists,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  due_at TIMESTAMP NOT NULL,
  due_text TEXT NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT FALSE,
  title TEXT NOT NULL,
  description TEXT
);
SELECT manage_updated_at('tasks');
CREATE INDEX task_list_idx ON tasks(list_id);
CREATE INDEX task_parent_idx ON tasks(parent_id);

CREATE TABLE labels (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES users,
  title TEXT NOT NULL,
  description TEXT,
  color VARCHAR(7) NOT NULL,
  CONSTRAINT color_hex_constraint
    CHECK (color ~* '^#[a-f0-9]{6}$')
);
CREATE INDEX label_user_idx ON labels(user_id);

CREATE TABLE task_labels (
  task_id UUID NOT NULL REFERENCES tasks ON DELETE CASCADE,
  label_id UUID NOT NULL REFERENCES labels ON DELETE CASCADE,
  PRIMARY KEY (task_id, label_id)
);
CREATE INDEX task_label_idx ON task_labels(label_id);

CREATE TABLE actions (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  user_id UUID NOT NULL REFERENCES users,
  created_at TIMESTAMP DEFAULT current_timestamp NOT NULL,
  action_type TEXT NOT NULL,
  data JSON NOT NULL
);
CREATE INDEX actions_user_idx ON actions(user_id);