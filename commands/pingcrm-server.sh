#!/usr/bin/env bash
#
# PingCRM app-server lifecycle shortcuts.
# Manages a backgrounded `cargo run --bin app-server` via PID + log files.

_pingcrm_server_pid_file()  { echo "$(get_project_root)/.pingcrm-server.pid"; }
_pingcrm_server_log_file()  { echo "$(get_project_root)/.pingcrm-server.log"; }
_pingcrm_server_port()      { echo "${PINGCRM_PORT:-3000}"; }

_pingcrm_server_is_running() {
    local pid_file
    pid_file="$(_pingcrm_server_pid_file)"
    [[ -f "$pid_file" ]] && kill -0 "$(cat "$pid_file")" 2>/dev/null
}

_pingcrm_port_pids() {
    # -sTCP:LISTEN avoids matching client sockets (e.g. browser tabs) that
    # happen to have a stale connection to this port.
    lsof -ti ":$(_pingcrm_server_port)" -sTCP:LISTEN 2>/dev/null | grep -v '^$' || true
}

# Start the app-server in the background [server,start]
cmd_server_start() {
    local pid_file log_file port
    pid_file="$(_pingcrm_server_pid_file)"
    log_file="$(_pingcrm_server_log_file)"
    port="$(_pingcrm_server_port)"

    if _pingcrm_server_is_running; then
        warning "Server already running (PID $(cat "$pid_file"))"
        return 0
    fi

    local existing
    existing="$(_pingcrm_port_pids)"
    if [[ -n "$existing" ]]; then
        warning "Port $port already in use by PID(s): $existing"
        warning "Run 'server:kill' first, or set PINGCRM_PORT to another port."
        exit 1
    fi

    info "Starting app-server in background on port $port..."
    local project_root
    project_root="$(get_project_root)"
    # `exec` replaces the subshell with cargo so $! becomes cargo's PID.
    # `disown` removes the job from the shell's table so the parent
    # doesn't wait on it when it exits.
    (
        cd "$project_root"
        exec nohup cargo run --quiet -p app --bin app-server > "$log_file" 2>&1 < /dev/null
    ) &
    local server_pid=$!
    echo "$server_pid" > "$pid_file"
    disown 2>/dev/null || true
    sleep 2

    if _pingcrm_server_is_running; then
        success "Server started (PID $(cat "$pid_file")) — logs: $log_file"
    else
        error "Server failed to start. Inspect $log_file"
        rm -f "$pid_file"
        exit 1
    fi
}

# Stop the tracked app-server [server,stop]
cmd_server_stop() {
    local pid_file
    pid_file="$(_pingcrm_server_pid_file)"
    local pid=""
    if _pingcrm_server_is_running; then
        pid="$(cat "$pid_file")"
        # Kill cargo's whole process group so the spawned app-server child
        # (target/debug/app-server) doesn't survive as an orphan.
        kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null || true
    fi
    rm -f "$pid_file"

    # Belt-and-braces: sweep any leftover listener on the configured port.
    sleep 0.5
    local orphans
    orphans="$(_pingcrm_port_pids)"
    if [[ -n "$orphans" ]]; then
        echo "$orphans" | xargs kill 2>/dev/null || true
    fi

    if [[ -n "$pid" ]]; then
        success "Stopped server (PID $pid)"
    else
        warning "No console-tracked server was running"
    fi
}

# Restart the app-server [server]
cmd_server_restart() {
    cmd_server_stop
    sleep 1
    cmd_server_start
}

# Show server status and port usage [server,status]
cmd_server_status() {
    local pid_file port
    pid_file="$(_pingcrm_server_pid_file)"
    port="$(_pingcrm_server_port)"

    if _pingcrm_server_is_running; then
        echo -e "  ${GREEN}● Running${NC} (PID $(cat "$pid_file"))"
    else
        echo -e "  ${YELLOW}○ Stopped${NC} (no PID file)"
    fi

    local pids
    pids="$(_pingcrm_port_pids)"
    if [[ -n "$pids" ]]; then
        echo -e "  ${CYAN}Port $port listeners:${NC}"
        lsof -i ":$port" -P -n 2>/dev/null | grep LISTEN | sed 's/^/    /'
    else
        echo -e "  ${YELLOW}Port $port is free${NC}"
    fi
}

# Tail the server log (Ctrl-C to exit) [server,logs]
cmd_server_logs() {
    local log_file
    log_file="$(_pingcrm_server_log_file)"
    if [[ -f "$log_file" ]]; then
        tail -f "$log_file"
    else
        warning "No log file yet at $log_file"
        exit 1
    fi
}

# Force-kill anything listening on the configured port [server,kill]
cmd_server_kill() {
    local port pids
    port="$(_pingcrm_server_port)"
    pids="$(_pingcrm_port_pids)"

    if [[ -z "$pids" ]]; then
        info "Port $port already free"
    else
        warning "Killing PID(s) on port $port: $pids"
        echo "$pids" | xargs kill 2>/dev/null || true
        sleep 1
        pids="$(_pingcrm_port_pids)"
        if [[ -n "$pids" ]]; then
            warning "Some processes still alive — sending SIGKILL"
            echo "$pids" | xargs kill -9 2>/dev/null || true
        fi
        success "Port $port cleared"
    fi
    rm -f "$(_pingcrm_server_pid_file)"
}
