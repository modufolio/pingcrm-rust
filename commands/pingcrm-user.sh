#!/usr/bin/env bash
#
# PingCRM user-management shortcuts.

_pingcrm_console() {
    (cd "$(get_project_root)" && cargo run --quiet -p app --bin console -- "$@")
}

# List all users [user,list]
cmd_user_list() {
    _pingcrm_console user:list
}

# Add a user — usage: user:add --email <email> --name <name> [user,add]
cmd_user_add() {
    local -A args
    parse_arguments args "$@"
    local email="${args[email]:-}"
    local name="${args[name]:-}"

    if [[ -z "$email" || -z "$name" ]]; then
        error "Usage: user:add --email <email> --name <name>"
        exit 1
    fi

    _pingcrm_console user:add --email "$email" --name "$name"
}

# Quick-create the default admin user (admin@example.com) [user,admin]
cmd_user_admin() {
    info "Creating admin@example.com..."
    _pingcrm_console user:add --email admin@example.com --name "Admin User"
}
