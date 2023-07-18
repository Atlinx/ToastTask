![Banner](web/public/banner.png)

# Toast Task 🍞

Todo/task tracking webapp build in Vue with a backend in Rust using a Postgres DB.

# Building

> **NOTE:**
> 
> Make sure to develop in WSL, Linux, or MacOS. `cargo watch` seems to not work when running on a Windows machine,
> which makes development painfully slow because the docker container must be rebuilt every time to view new changes.

1) Make a copy of the `.env.template` file and replace items with `<angle-brackets>` with actual data.

2) Run the command:
   
    ```bash
    ./run_dev.sh
    ```

# Testing

Run the command:

```
./run_test.sh
```

> **NOTE:**
> 
> If docker gives you `error getting credentials` errors whenever
> you build on WSL, please run:
> 
> ```bash
> rm ~/.docker/config.json
> ```

## Cleanup

Run the command:

```bash
docker-compose down -v
```


## Resources

- [Accessing Postgresql Database](https://www.cherryservers.com/blog/how-to-install-and-setup-postgresql-server-on-ubuntu-20-04)
  - Logging into Postgres 
    
    ```bash
    sudo -u postgres psql
    ```

  - Commands
  
    - `\q` - Quit
    - `\l` - List databases
    - `\c {database}` - Connect to `{database}`
    - `\dt` - Lists tables
    
  - Configuration file is stored at

    ```
    /etc/postgresql/12/main/postgresql.conf
    ```

    > **NOTE:**
    >
    > Use an alternative port to `5432` on windows,
    > because you might already have a Windows Postgres DB
    > taking up that port.

    > **NOTE:**
    > 
    > Add `listen_addresses = '*'` to your `postgresql.conf`
    > file to ensure you can connect from windows