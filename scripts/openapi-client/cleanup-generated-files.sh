#!/usr/bin/env bash
set -euo pipefail

# Cleanup script for generated Rust API client
# This script removes unnecessary generated files and fixes common issues

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
API_DIR="$PROJECT_ROOT/api"

# Colors for output
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
CYAN='\033[1;36m'
NC='\033[0m'

log_info() {
    echo -e "${CYAN}▶${NC} $1"
}

log_success() {
    echo -e "${GREEN}✔${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Check if api directory exists
if [[ ! -d "$API_DIR" ]]; then
    log_warning "API directory not found: $API_DIR"
    exit 1
fi

cd "$API_DIR"

log_info "Cleaning up generated API client files..."

# 1. Remove git_push.sh
if [[ -f "git_push.sh" ]]; then
    rm -f git_push.sh
    log_success "Removed git_push.sh"
else
    log_warning "git_push.sh not found (already removed?)"
fi

# 2. Remove docs folder
if [[ -d "docs" ]]; then
    rm -rf docs
    log_success "Removed docs folder"
else
    log_warning "docs folder not found (already removed?)"
fi

# 3. Remove empty doc comments using sed
log_info "Removing empty doc comments..."
if sed -i '/^ *\/\/\/ *$/d' src/models/* 2>/dev/null; then
    log_success "Empty doc comments removed"
else
    log_warning "Failed to remove empty doc comments (files may not exist)"
fi

# 4. Run cargo clippy --fix --all-targets (from project root)
cd "$PROJECT_ROOT"
log_info "Running cargo clippy --fix --all-targets on entire project..."
if cargo clippy --fix --all-targets --allow-dirty --allow-staged; then
    log_success "Cargo clippy fixes applied"
else
    log_warning "Cargo clippy encountered some issues (this may be normal)"
fi

log_success "Client cleanup completed!"
