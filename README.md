# PingCRM Rust - Dev Setup

## Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)
- Node.js 18+ and npm (for the Vue frontend)
- SQLite (bundled with the project)
- Bash ≥ 4 (for `./cli.sh`)

### macOS: install a newer bash

macOS ships bash 3.2 (GPL licensing), which is too old — `cli.sh` uses
associative arrays (`declare -A`) introduced in bash 4. Install via Homebrew:

```bash
brew install bash
```

Make sure the Homebrew bash comes before `/bin` on your `PATH` so
`#!/usr/bin/env bash` resolves to it. `cli.sh` performs a runtime version check
and prints a hint if the bash is still too old.

## Quick Start

All operational tasks are wrapped by `./cli.sh` (see `./cli.sh list` for the full
catalog). Each shortcut below ultimately invokes `cargo` or `npm` under the hood.

### 1. Environment Configuration

```bash
cp .env.example .env
```

**Defaults:**
- **Database:** SQLite (file: `appkit.db`)
- **Session Storage:** In-memory (development only)

If you want to use Redis for sessions, uncomment the `REDIS_URL` line in `.env` and ensure Redis is running.

### 2. Frontend assets

Install JS dependencies and produce a production Vite bundle:

```bash
npm install
npm run build         # writes public/build/{assets/, .vite/manifest.json}
```

For hot-reload during development, run `npm run dev` in a separate terminal —
Vite's plugin writes `public/build/.vite-dev` and the Rust template detects it
to switch into dev mode automatically.

### 3. Database setup + seed

Reset, migrate, and seed in one shot:

```bash
./cli.sh db:reset
```

Or step-by-step:

```bash
./cli.sh db:migrate   # run pending migrations
./cli.sh db:seed --purge   # seed (use --purge to wipe first)
```

### 4. Start the web server

```bash
./cli.sh server:start       # backgrounded, PID tracked
./cli.sh server:status      # check status
./cli.sh server:logs        # tail logs
./cli.sh server:stop        # stop
```

The server listens on `http://127.0.0.1:3000`. Log in with
`johndoe@example.com` / `secret` (the form is prefilled).

## Available Commands

`./cli.sh` is the entry point. Commands live in `commands/*.sh` (auto-discovered);
add your own by dropping a `cmd_<group>_<name>()` function into a `.sh` file
there. Run `./cli.sh list` for the full catalog.

### Database

```bash
./cli.sh db:reset       # drop + migrate + seed
./cli.sh db:fresh       # drop + migrate (empty)
./cli.sh db:migrate     # run pending migrations
./cli.sh db:seed        # seed (pass --purge to wipe first)
./cli.sh db:info        # show database info
./cli.sh db:tables      # list tables
./cli.sh db:status      # migration status
./cli.sh db:sql "SELECT * FROM users LIMIT 5"
```

### Users

```bash
./cli.sh user:list
./cli.sh user:add --email new@example.com --name "New User"
./cli.sh user:admin     # quick-create admin@example.com
```

### Server lifecycle

```bash
./cli.sh server:start          # background, PID + logs in .pingcrm-server.{pid,log}
./cli.sh server:stop
./cli.sh server:restart
./cli.sh server:status
./cli.sh server:logs           # tail -f the log
./cli.sh server:kill           # force-kill anything on port 3000
# PINGCRM_PORT=8080 ./cli.sh server:start  # use a different port
```

### Dev loop

```bash
./cli.sh dev:build      # cargo build --workspace
./cli.sh dev:check      # cargo check --workspace
./cli.sh dev:test       # cargo test
./cli.sh dev:itest      # integration tests (single-threaded)
./cli.sh dev:fmt        # cargo fmt --all
./cli.sh dev:clippy     # cargo clippy --workspace --all-targets
./cli.sh dev:clean      # cargo clean
./cli.sh dev:watch      # cargo-watch around app-server (needs cargo-watch)
```

### Router debug

```bash
./cli.sh router:debug                # all routes
./cli.sh router:get                  # GET only
./cli.sh router:post                 # POST only
./cli.sh router:api                  # api firewall only
```

## Test Users

After seeding, the following test users are available (password listed where
non-default):

| Email                          | Password       | Role        |
| ------------------------------ | -------------- | ----------- |
| `johndoe@example.com`          | `secret`       | demo (prefilled in Login.vue) |
| `admin@example.com`            | `admin123`     | super admin |
| `manager@example.com`          | `password123`  | admin       |
| `admin.john@example.com`       | `password123`  | admin       |
| `jane.smith@example.com`       | `password123`  | user        |
| `disabled@example.com`         | `password123`  | disabled    |
| `locked@example.com`           | `password123`  | locked      |
| `secure@example.com`           | `password123`  | 2FA enabled |

## Architecture

```
.
├── app/
│   ├── bin/              # Binary entry points (app-server, console, seed)
│   ├── config/           # Route and resource configuration
│   ├── src/              # Library: database, router, middleware, fixtures, ...
│   └── tests/            # Integration tests
├── crates/core/          # Security, inertia, jsonapi (need to move it to an separate repo in the future)
├── commands/             # bash CLI commands (auto-discovered by cli.sh)
├── cli.sh                # CLI entry point
├── resources/            # Vue frontend (Pages, Shared, app.js, css)
├── public/build/         # Vite build output
├── templates/            # Tera HTML templates
└── migrations/           # Diesel migrations
```

