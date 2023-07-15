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
    docker-compose up --build
    ```

## Cleanup

Run the command:

```bash
docker-compose down -v
```
