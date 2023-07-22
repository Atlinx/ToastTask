# Architecture

App should be very responsive so users can pull it up on any platform and quickly add/browse tasks.

## Web

Since browsers don't have large persistent storage, the web version should fetch data as it needs.

## Standalone

Since standalone installations give access to a filesystem, we can store user data in a local sqlite database.

# UX Flow

## Entry

### User creates account

```
user account is added to DB
user login information is added to DB
default data is added
  - Inbox list (undeletable)
  - Personal list
```

### User logs in / Opens app

```
perform sync
navigates to "Today tab"
```

## Layout

### Left tab

- Inbox
- Today page
- Upcoming page
- List tab

### List tab

Displays active lists.
You can add/remove lists.

### Add task button

Adds a task

Task has a

- Title
- Description
- Due date

### Settings menu

Opened from setting button

### Today page

Shows tasks due today in list format

Overdue tasks have their own group at the start

```
if web:
    if last synced task is still today:
        show a download button
            can be clicked to attempt to download another batch of 300 tasks
```

### Upcoming page

Shows upcoming tasks. This is a big list that starts with today and ends a year from today. Tasks are grouped by individual days.

Overdue tasks have their own group at the start

```
if web:
    if last synced task is still today:
        show a download button
            can be clicked to attempt to download another batch of 300 tasks
```

## Sync

```
if standalone and frontend currently has actions saved:
    download all actions since the last online action saved locally
    if exists new online actions:
        perform a "merge sync"
    apply downloaded actions onto local DB
else
    perform "full sync" with backend, and stores the last online action

if online:
    websocket event listeners are attached to server to listen for updates
```

### Full Sync

```
download user data
  - login data
  - settings
  - identity
download all lists recursively
  - list responses also tell us the # of completed tasks in the list
  - if web:
        <= 20 sublists downloaded
download all active tasks recursively
  - no completed tasks are downloaded
  - if web:
        <= 100 upcoming active tasks downloaded
```

### Merge Sync

```
iterate through local actions
    if exists future action that conflicts with this action
        remove this local action
upload remaining actions
```

## Not Synced

### Completed tasks

Completed tasks are **NOT** synced. This is to avoid data bloat of having to sync more and more tasks as the user continues using the app.

However, users can click on `show completed` toggle on a list to view the completed tasks for that list.

```
download most recent 20 completed tasks
```
