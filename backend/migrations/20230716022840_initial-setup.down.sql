DROP TABLE IF EXISTS actions;
DROP TABLE IF EXISTS task_labels;
DROP TABLE IF EXISTS labels;
DROP TABLE IF EXISTS task_relations;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS list_relations;
DROP TABLE IF EXISTS lists;
DROP TABLE IF EXISTS discord_user_login;
DROP TABLE IF EXISTS email_user_logins;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS users;



DROP FUNCTION IF EXISTS manage_updated_at(_tbl regclass);
DROP FUNCTION IF EXISTS set_updated_at();

DROP EXTENSION IF EXISTS "uuid-ossp";