-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS task_labels;
DROP TABLE IF EXISTS labels;
DROP TABLE IF EXISTS lists_hierarchy;
DROP TABLE IF EXISTS lists;
DROP TABLE IF EXISTS tasks_hierarchy;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS discord_user_login;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS users;

DROP EXTENSION IF EXISTS "uuid-ossp";