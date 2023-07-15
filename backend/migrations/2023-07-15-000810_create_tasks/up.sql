-- Your SQL goes here
CREATE TABLE users (
  id SERIAL PRIMARY KEY
);

CREATE TABLE discord_user_login (
  id SERIAL PRIMARY KEY,
  user_id INTEGER REFERENCES users,
  client_id TEXT NOT NULL
);

CREATE TABLE tasks (
  id SERIAL PRIMARY KEY,
  created_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  edited_date TIMESTAMP NOT NULL,
  due_date TIMESTAMP NOT NULL,
  due_text TEXT NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT FALSE,
  title TEXT NOT NULL,
  description TEXT
);

CREATE TABLE tasks_hierarchy (
  child_task_id INTEGER REFERENCES tasks,
  parent_task_id INTEGER REFERENCES tasks,
  PRIMARY KEY (child_task_id, parent_task_id)
);

CREATE TABLE lists (
  id SERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  color VARCHAR(7) NOT NULL,
  user_id INTEGER REFERENCES users,
  CONSTRAINT color_hex_constraint
    CHECK (color ~* '^#[a-f0-9]{6}$')
);

CREATE TABLE lists_hierarchy (
  child_list_id INTEGER REFERENCES lists,
  parent_list_id INTEGER REFERENCES lists,
  PRIMARY KEY (child_list_id, parent_list_id)
);

CREATE TABLE labels (
  id SERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  color VARCHAR(7) NOT NULL,
  CONSTRAINT color_hex_constraint
    CHECK (color ~* '^#[a-f0-9]{6}$')
);

CREATE TABLE task_labels (
  task_id INTEGER REFERENCES tasks,
  label_id INTEGER REFERENCES labels,
  PRIMARY KEY (task_id, label_id)
);