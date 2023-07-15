-- Your SQL goes here
CREATE EXTENSION "uuid-ossp";

CREATE TABLE users (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  username TEXT NOT NULL
);

CREATE TABLE sessions (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  ip CIDR NOT NULL,
  created_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expire_date TIMESTAMP NOT NULL,
  user_id UUID REFERENCES users
);

CREATE TABLE discord_user_login (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  user_id UUID REFERENCES users,
  client_id TEXT NOT NULL
);

CREATE TABLE tasks (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  created_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  edited_date TIMESTAMP NOT NULL,
  due_date TIMESTAMP NOT NULL,
  due_text TEXT NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT FALSE,
  title TEXT NOT NULL,
  description TEXT
);

CREATE TABLE task_relations (
  child_task_id UUID REFERENCES tasks,
  parent_task_id UUID REFERENCES tasks,
  PRIMARY KEY (child_task_id, parent_task_id)
);

CREATE TABLE lists (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  title TEXT NOT NULL,
  description TEXT,
  color VARCHAR(7) NOT NULL,
  user_id UUID REFERENCES users,
  CONSTRAINT color_hex_constraint
    CHECK (color ~* '^#[a-f0-9]{6}$')
);

CREATE TABLE list_relations (
  child_list_id UUID REFERENCES lists,
  parent_list_id UUID REFERENCES lists,
  PRIMARY KEY (child_list_id, parent_list_id)
);

CREATE TABLE labels (
  id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  color VARCHAR(7) NOT NULL,
  CONSTRAINT color_hex_constraint
    CHECK (color ~* '^#[a-f0-9]{6}$')
);

CREATE TABLE task_labels (
  task_id UUID REFERENCES tasks,
  label_id UUID REFERENCES labels,
  PRIMARY KEY (task_id, label_id)
);