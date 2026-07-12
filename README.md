# churchrm
CRM for a parish

## Local development

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) with Compose
- [Rust](https://rustup.rs/)
- WebAssembly target: `rustup target add wasm32-unknown-unknown`
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos): `cargo install cargo-leptos`

### 1. Configure the database connection

Copy the example environment file:

```bash
cp .env.example .env
```

`.env` is used by both Docker Compose and the app. Defaults are:

| Variable | Default | Purpose |
| --- | --- | --- |
| `POSTGRES_USER` | `churchrm` | Postgres username |
| `POSTGRES_PASSWORD` | `churchrm` | Postgres password |
| `POSTGRES_DB` | `churchrm` | Database name |
| `POSTGRES_PORT` | `5432` | Host port mapped to Postgres |
| `DATABASE_URL` | `postgres://churchrm:churchrm@127.0.0.1:5432/churchrm` | App connection string |

If you change the Postgres user, password, database, or port, update `DATABASE_URL` to match.

### 2. Start PostgreSQL

```bash
docker compose up -d
```

Confirm the container is healthy:

```bash
docker compose ps
```

You should see `churchrm-postgres` with status `healthy`. The database is empty at this point; the app creates the schema on startup.

### 3. Run the app

```bash
cargo leptos watch
```

On startup the server:

1. Loads `DATABASE_URL` from `.env`
2. Connects to Postgres
3. Runs migrations (creates the `customer` table if needed)
4. Serves the UI at [http://127.0.0.1:3000](http://127.0.0.1:3000)

### 4. Use the app

1. Open [http://127.0.0.1:3000](http://127.0.0.1:3000) — the contacts list (empty until you add some).
2. Click **New contact**, enter a name (required), optional email and phone, then **Save contact**.
3. You are returned to the list; new contacts are stored in the Postgres `customer` table.
4. Use the Name / Email / Phone column filters to search.

### Stop the database

```bash
docker compose down
```

Data is kept in the `postgres_data` Docker volume. To wipe it:

```bash
docker compose down -v
```
