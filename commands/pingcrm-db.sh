#!/usr/bin/env bash
#
# PingCRM database shortcuts.
# Wraps `cargo run -p app --bin console` and `--bin seed`.

# Internal: run the rust console binary
_pingcrm_console() {
    (cd "$(get_project_root)" && cargo run --quiet -p app --bin console -- "$@")
}

# Internal: run the rust seed binary
_pingcrm_seed() {
    (cd "$(get_project_root)" && cargo run --quiet -p app --bin seed -- "$@")
}

# If a console-tracked app-server is running, restart it.
_pingcrm_restart_server_if_running() {
    local pid_file
    pid_file="$(get_project_root)/.pingcrm-server.pid"
    if [[ -f "$pid_file" ]] && kill -0 "$(cat "$pid_file")" 2>/dev/null; then
        info "Restarting tracked app-server so it sees the new database state..."
        "$(get_project_root)/cli.sh" server:restart
    fi
}

# Drop, migrate and re-seed the SQLite database [db,reset]
cmd_db_reset() {
    local project_root
    project_root="$(get_project_root)"
    info "Resetting database (drop + migrate + seed)..."
    rm -f "$project_root/appkit.db" "$project_root/appkit.db-shm" "$project_root/appkit.db-wal"
    _pingcrm_console migrate || { error "Migration failed"; exit 1; }
    _pingcrm_seed --purge || { error "Seed failed"; exit 1; }
    success "Database reset complete"
    _pingcrm_restart_server_if_running
}

# Drop and re-create the schema without seed data [db]
cmd_db_fresh() {
    local project_root
    project_root="$(get_project_root)"
    info "Re-creating empty database..."
    rm -f "$project_root/appkit.db" "$project_root/appkit.db-shm" "$project_root/appkit.db-wal"
    _pingcrm_console migrate || { error "Migration failed"; exit 1; }
    success "Database re-created (no seed data)"
    _pingcrm_restart_server_if_running
}

# Run all pending Diesel migrations [db,migrate]
cmd_db_migrate() {
    _pingcrm_console migrate
}

# Seed the database (pass --purge to wipe first) [db,seed]
cmd_db_seed() {
    _pingcrm_seed "$@" || { error "Seed failed"; exit 1; }
    _pingcrm_restart_server_if_running
}

# Show database information [db,info]
cmd_db_info() {
    _pingcrm_console database:info
}

# List database tables [db]
cmd_db_tables() {
    _pingcrm_console query:tables
}

# Run an arbitrary SQL query — usage: db:sql "SELECT * FROM users" [db,sql]
cmd_db_sql() {
    if [[ $# -eq 0 ]]; then
        error 'Usage: db:sql "SELECT * FROM users LIMIT 5"'
        exit 1
    fi
    _pingcrm_console query:sql "$@"
}

# Show migration status [db,migrate]
cmd_db_status() {
    _pingcrm_console migration:status
}
