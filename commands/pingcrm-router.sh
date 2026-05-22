#!/usr/bin/env bash
#
# PingCRM router debugging shortcuts.

_pingcrm_console() {
    (cd "$(get_project_root)" && cargo run --quiet -p app --bin console -- "$@")
}

# Show all registered routes [router,debug]
cmd_router_debug() {
    _pingcrm_console router:debug "$@"
}

# Show only GET routes [router]
cmd_router_get() {
    _pingcrm_console router:debug --method GET
}

# Show only POST routes [router]
cmd_router_post() {
    _pingcrm_console router:debug --method POST
}

# Show only routes under /api [router,api]
cmd_router_api() {
    _pingcrm_console router:debug --path /api
}
