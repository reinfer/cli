#!/usr/bin/env bash
set -euo pipefail

# Orchestrator script to update spec, preprocess it, and generate Rust client
# This script will:
# 1. Prompt for branch (default: develop)
# 2. Pull the latest OpenAPI spec from that branch
# 3. Preprocess the spec to fix type issues
# 4. Generate the Rust client

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
DEFAULT_BRANCH="develop"
SPEC_FILE="out/reinfer-v1.openapi-v3.1.json"
PREPROCESSED_SPEC="out/reinfer-v1.openapi-v3.1.preprocessed.json"
RUST_CLIENT_DIR="api"

# Colors for output - using brighter/more visible colors
RED='\033[1;31m'     # Bright red
GREEN='\033[1;32m'   # Bright green
YELLOW='\033[1;33m'  # Bright yellow
CYAN='\033[1;36m'    # Bright cyan (more visible than blue)
NC='\033[0m'         # No Color

log_info() {
    echo -e "${CYAN}▶${NC} $1"
}

log_success() {
    echo -e "${GREEN}✔${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

# Function to prompt for branch
prompt_for_branch() {
    echo >&2
    log_info "Which branch do you want to use for the OpenAPI spec?" >&2
    echo -n "Enter branch name (default: ${DEFAULT_BRANCH}): " >&2
    read -r branch_input
    
    if [[ -z "${branch_input}" ]]; then
        echo "${DEFAULT_BRANCH}"
    else
        echo "${branch_input}"
    fi
}

# Function to run update-spec.sh
update_spec() {
    local branch="$1"
    log_info "Updating OpenAPI spec from branch: $branch"
    
    if ! "$SCRIPT_DIR/openapi-client/fetch-spec-from-github.sh" --ref "$branch" -o "$SPEC_FILE"; then
        log_error "Failed to update spec from branch: $branch"
        exit 1
    fi
    
    log_success "Spec updated from $branch"
}

# Function to run preprocess-spec.py
preprocess_spec() {
    log_info "Preprocessing OpenAPI spec to fix type issues"
    
    if ! python3 "$SCRIPT_DIR/openapi-client/preprocess-spec.py" "$SPEC_FILE" "$PREPROCESSED_SPEC"; then
        log_error "Failed to preprocess spec"
        exit 1
    fi
    
    log_success "Spec preprocessed"
}

# Function to run generate-rust-client.sh
generate_client() {
    log_info "Generating Rust client from preprocessed spec"
    
    if ! "$SCRIPT_DIR/openapi-client/generate-rust-client.sh" -i "$PREPROCESSED_SPEC" -o "$RUST_CLIENT_DIR"; then
        log_error "Failed to generate Rust client"
        exit 1
    fi
    
    log_success "Rust client generated"
}

# Function to check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if required scripts exist
    local required_scripts=("openapi-client/fetch-spec-from-github.sh" "openapi-client/preprocess-spec.py" "openapi-client/generate-rust-client.sh")
    for script in "${required_scripts[@]}"; do
        if [[ ! -f "$SCRIPT_DIR/$script" ]]; then
            log_error "Required script not found: $SCRIPT_DIR/$script"
            exit 1
        fi
    done
    
    # Check if Python 3 is available
    if ! command -v python3 &> /dev/null; then
        log_error "Python 3 is required but not found"
        exit 1
    fi
    
    # Check if Docker is available (required for generate-rust-client.sh)
    if ! command -v docker &> /dev/null; then
        log_error "Docker is required but not found"
        exit 1
    fi
    
    # Check if Docker daemon is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi
    
    log_success "All prerequisites satisfied"
}

# Function to clean up spec files
cleanup() {
    log_info "Cleaning up spec files"
    rm -f "$SPEC_FILE" "$PREPROCESSED_SPEC"
    log_success "Spec files cleaned up"
}

# Main execution
main() {
    cd "$PROJECT_ROOT"
    
    echo "=================================================="
    echo "  OpenAPI Spec Update & Rust Client Generator"
    echo "=================================================="
    echo
    
    # Check prerequisites
    check_prerequisites
    
    # Prompt for branch
    BRANCH=$(prompt_for_branch)
    
    echo
    log_info "Starting workflow with branch: $BRANCH"
    echo
    
    # Create output directory if it doesn't exist
    mkdir -p "$(dirname "$SPEC_FILE")"
    
    # Step 1: Update spec
    update_spec "$BRANCH"
    echo
    
    # Step 2: Preprocess spec
    preprocess_spec
    echo
    
    # Step 3: Generate client
    generate_client
    echo
    
    # Clean up spec files
    cleanup
    
    echo
    log_success "Workflow completed successfully!"
    echo
    log_info "Generated Rust API client: $RUST_CLIENT_DIR"
    echo
}

# Handle script interruption
trap 'log_warning "Script interrupted"; cleanup; exit 130' INT TERM

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
