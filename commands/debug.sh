#!/usr/bin/env bash

# Debug dotenv configuration and loaded variables
cmd_debug_dotenv() {
    local -A args
    parse_arguments args "$@"
    local project_root
    project_root=$(get_project_root)
    local env="${args[env]:-${APP_ENV:-${CLI_ENV:-dev}}}"

    line
    info "Dotenv Debug Information"
    line
    echo

    # Show current environment
    echo -e "${YELLOW}Environment:${NC} $env"
    echo -e "${YELLOW}Project Root:${NC} $project_root"
    echo

    # Check which .env files exist
    echo -e "${YELLOW}Available .env files:${NC}"
    local -a env_files=(
        ".env"
        ".env.dist"
        ".env.local"
        ".env.$env"
        ".env.$env.local"
    )

    local found_any=false
    for file in "${env_files[@]}"; do
        local full_path="$project_root/$file"
        if [[ -f "$full_path" ]]; then
            check "$file ($(wc -l < "$full_path") lines)"
            found_any=true
        else
            echo -e "  ${PURPLE}[-]${NC} $file (not found)"
        fi
    done

    if [[ "$found_any" != "true" ]]; then
        warning "No .env files found"
    fi
    echo

    # Show load order
    echo -e "${YELLOW}Load order:${NC}"
    echo "  1. .env (or .env.dist as fallback)"
    if [[ "$env" != "test" ]]; then
        echo "  2. .env.local (skipped in test environment)"
    fi
    if [[ "$env" != "local" ]]; then
        echo "  3. .env.$env"
        echo "  4. .env.$env.local"
    fi
    echo

    # Show loaded variables
    if [[ -n "$DOTENV_VARS" ]]; then
        echo -e "${YELLOW}Variables loaded by dotenv:${NC}"

        # Split DOTENV_VARS into array
        IFS=',' read -ra loaded_vars <<< "$DOTENV_VARS"

        # Create table data
        local -a table_headers=("Variable" "Value" "Length")
        local -a table_rows=()

        for var in "${loaded_vars[@]}"; do
            if [[ -v "$var" ]]; then
                local value="${!var}"
                local value_len="${#value}"

                # Truncate long values for display
                if [[ $value_len -gt 50 ]]; then
                    local display_value="${value:0:47}..."
                else
                    local display_value="$value"
                fi

                # Escape special characters for display
                display_value="${display_value//$'\n'/\\n}"
                display_value="${display_value//$'\t'/\\t}"

                table_rows+=("$var|$display_value|$value_len")
            else
                table_rows+=("$var|(unset)|0")
            fi
        done

        # Print table
        print_table table_headers table_rows
        echo
        success "Total: ${#loaded_vars[@]} variables loaded"
    else
        warning "No variables loaded by dotenv (DOTENV_VARS is empty)"
        info "Run a command to trigger .env loading first"
    fi

    line
}

# Show environment variable value with expansion details
cmd_debug_env() {
    local -A args
    parse_arguments args "$@"
    local var_name="${args[_positional]}"
    var_name="${var_name## }"  # Trim leading space

    if [[ -z "$var_name" ]]; then
        error "Usage: debug:env <variable_name>"
        echo "Example: debug:env PATH"
        return 1
    fi

    line
    info "Environment Variable Debug: $var_name"
    line
    echo

    if [[ -v "$var_name" ]]; then
        local value="${!var_name}"
        echo -e "${YELLOW}Name:${NC} $var_name"
        echo -e "${YELLOW}Value:${NC} $value"
        echo -e "${YELLOW}Length:${NC} ${#value} characters"
        echo

        # Check if loaded by dotenv
        if [[ -n "$DOTENV_VARS" ]] && [[ ",$DOTENV_VARS," == *",$var_name,"* ]]; then
            check "Loaded by dotenv"
        else
            info "Not loaded by dotenv (may be system variable)"
        fi

        # Show first 500 chars if long
        if [[ ${#value} -gt 100 ]]; then
            echo
            echo -e "${YELLOW}Preview (first 100 chars):${NC}"
            echo "${value:0:100}..."
        fi
    else
        error "Variable '$var_name' is not set"
        return 1
    fi

    line
}

# Reload .env files
cmd_debug_reload_env() {
    local -A args
    parse_arguments args "$@"
    local project_root
    project_root=$(get_project_root)
    local override="${args[override]:-false}"

    info "Reloading .env files from $project_root..."

    if [[ "$override" == "true" ]]; then
        warning "Override mode: Will overwrite existing variables"
        overload_env "$project_root"
    else
        info "Standard mode: Existing variables will not be overwritten"
        load_env "$project_root"
    fi

    if [[ $? -eq 0 ]]; then
        success "Environment files reloaded"

        if [[ -n "$DOTENV_VARS" ]]; then
            IFS=',' read -ra loaded_vars <<< "$DOTENV_VARS"
            info "Loaded ${#loaded_vars[@]} variables"
        fi
    else
        error "Failed to reload environment files"
        return 1
    fi
}
