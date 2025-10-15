#!/bin/bash

# hacker.sh - Docker Compose management script for hackercore
# Usage: ./hacker.sh [start|stop|restart|status|clean|rebuild|clean-rebuild|logs|deploy|update]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
GIT_HEADS_FILE="$SCRIPT_DIR/.git_heads"

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to detect the operating system
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get >/dev/null 2>&1; then
            echo "ubuntu"
        elif command -v yum >/dev/null 2>&1; then
            echo "centos"
        elif command -v dnf >/dev/null 2>&1; then
            echo "fedora"
        elif command -v pacman >/dev/null 2>&1; then
            echo "arch"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Function to check if Docker is installed
check_docker() {
    if command -v docker >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to check if Docker Compose is installed
check_docker_compose() {
    if command -v docker-compose >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to provide Docker installation instructions
install_docker_instructions() {
    local os=$(detect_os)
    
    print_error "Docker is not installed on your system."
    echo
    print_info "Please install Docker using one of the following methods:"
    echo
    
    case $os in
        "ubuntu")
            echo "For Ubuntu/Debian:"
            echo "  sudo apt-get update"
            echo "  sudo apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release"
            echo "  curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg"
            echo "  echo \"deb [arch=\$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu \$(lsb_release -cs) stable\" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null"
            echo "  sudo apt-get update"
            echo "  sudo apt-get install -y docker-ce docker-ce-cli containerd.io"
            echo "  sudo usermod -aG docker \$USER"
            echo "  # Log out and back in for group changes to take effect"
            ;;
        "centos")
            echo "For CentOS/RHEL:"
            echo "  sudo yum install -y yum-utils"
            echo "  sudo yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo"
            echo "  sudo yum install -y docker-ce docker-ce-cli containerd.io"
            echo "  sudo systemctl start docker"
            echo "  sudo systemctl enable docker"
            echo "  sudo usermod -aG docker \$USER"
            ;;
        "fedora")
            echo "For Fedora:"
            echo "  sudo dnf install -y dnf-plugins-core"
            echo "  sudo dnf config-manager --add-repo https://download.docker.com/linux/fedora/docker-ce.repo"
            echo "  sudo dnf install -y docker-ce docker-ce-cli containerd.io"
            echo "  sudo systemctl start docker"
            echo "  sudo systemctl enable docker"
            echo "  sudo usermod -aG docker \$USER"
            ;;
        "arch")
            echo "For Arch Linux:"
            echo "  sudo pacman -S docker"
            echo "  sudo systemctl start docker"
            echo "  sudo systemctl enable docker"
            echo "  sudo usermod -aG docker \$USER"
            ;;
        "macos")
            echo "For macOS:"
            echo "  # Install Docker Desktop for Mac from:"
            echo "  # https://docs.docker.com/desktop/mac/install/"
            echo "  # Or use Homebrew:"
            echo "  brew install --cask docker"
            ;;
        "windows")
            echo "For Windows:"
            echo "  # Install Docker Desktop for Windows from:"
            echo "  # https://docs.docker.com/desktop/windows/install/"
            ;;
        *)
            echo "Please visit https://docs.docker.com/get-docker/ for installation instructions for your system."
            ;;
    esac
    
    echo
    print_info "After installation, you may need to log out and back in for group changes to take effect."
    echo
}

# Function to provide Docker Compose installation instructions
install_docker_compose_instructions() {
    local os=$(detect_os)
    
    print_error "Docker Compose is not installed on your system."
    echo
    print_info "Please install Docker Compose using one of the following methods:"
    echo
    
    case $os in
        "ubuntu"|"centos"|"fedora"|"arch")
            echo "For Linux systems:"
            echo "  # Download the latest version"
            echo "  sudo curl -L \"https://github.com/docker/compose/releases/latest/download/docker-compose-\$(uname -s)-\$(uname -m)\" -o /usr/local/bin/docker-compose"
            echo "  sudo chmod +x /usr/local/bin/docker-compose"
            echo "  # Verify installation"
            echo "  docker-compose --version"
            echo
            echo "Alternative (if you have pip installed):"
            echo "  sudo pip install docker-compose"
            ;;
        "macos")
            echo "For macOS:"
            echo "  # If you installed Docker Desktop, Docker Compose is included"
            echo "  # Or install via Homebrew:"
            echo "  brew install docker-compose"
            echo "  # Or via pip:"
            echo "  pip install docker-compose"
            ;;
        "windows")
            echo "For Windows:"
            echo "  # Docker Compose is included with Docker Desktop for Windows"
            echo "  # Or install via pip:"
            echo "  pip install docker-compose"
            ;;
        *)
            echo "Please visit https://docs.docker.com/compose/install/ for installation instructions for your system."
            ;;
    esac
    
    echo
}

# Function to check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    if ! check_docker; then
        install_docker_instructions
        exit 1
    fi
    
    if ! check_docker_compose; then
        install_docker_compose_instructions
        exit 1
    fi
    
    print_success "All prerequisites are installed!"
}

# Function to check if compose file exists
check_compose_file() {
    if [[ ! -f "$COMPOSE_FILE" ]]; then
        print_error "Docker Compose file not found: $COMPOSE_FILE"
        exit 1
    fi
}

# Function to get git head for a directory
get_git_head() {
    local dir="$1"
    if [[ -d "$dir/.git" ]] || [[ -f "$dir/.git" ]]; then
        cd "$dir"
        git rev-parse HEAD 2>/dev/null || echo "no-commit"
    else
        echo "no-git"
    fi
}

# Function to save current git heads
save_git_heads() {
    print_info "Saving current git heads..."
    
    local local_head=$(get_git_head "$SCRIPT_DIR")
    local moor_head=$(get_git_head "$SCRIPT_DIR/vendor/moor")
    local dome_client_head=$(get_git_head "$SCRIPT_DIR/vendor/dome-client")
    
    cat > "$GIT_HEADS_FILE" << EOF
LOCAL_HEAD=$local_head
MOOR_HEAD=$moor_head
DOME_CLIENT_HEAD=$dome_client_head
EOF
    
    print_success "Git heads saved: local=$local_head, moor=$moor_head, dome-client=$dome_client_head"
}

# Function to check if git heads have changed
check_git_heads_changed() {
    if [[ ! -f "$GIT_HEADS_FILE" ]]; then
        print_info "No previous git heads found, will rebuild"
        return 0
    fi
    
    # Source the saved heads
    source "$GIT_HEADS_FILE"
    
    local current_moor_head=$(get_git_head "$SCRIPT_DIR/vendor/moor")
        
    if [[ "$current_moor_head" != "$MOOR_HEAD" ]]; then
        print_warning "Moor git head changed: $MOOR_HEAD -> $current_moor_head"
        return 0
    fi
    
    local current_dome_client_head=$(get_git_head "$SCRIPT_DIR/vendor/dome-client")
    if [[ "$current_dome_client_head" != "$DOME_CLIENT_HEAD" ]]; then
        print_warning "Dome-client git head changed: $DOME_CLIENT_HEAD -> $current_dome_client_head"
        return 0
    fi
    
    print_info "Moor and dome-client git heads unchanged, no rebuild needed"
    return 1
}

# Function to check and clone game directory if needed
check_game_directory() {
    print_info "Checking for game directory..."
    cd "$SCRIPT_DIR"
    
    # Create game directory if it doesn't exist
    if [[ ! -d "game" ]]; then
        print_info "Creating game/ directory..."
        mkdir -p game
    fi
    
    # Clone hackercore-core into game/objects if it doesn't exist
    if [[ ! -d "game/objects" ]]; then
        print_warning "game/objects/ directory not found, cloning repository..."
        if git clone https://github.com/biscuitWizard/moor-hackercore-core.git game/objects; then
            print_success "Successfully cloned game repository to game/objects/ directory!"
        else
            print_error "Failed to clone game repository!"
            print_info "Please check your network connection."
            print_info "Make sure you have access to https://github.com/biscuitWizard/moor-hackercore-core.git"
            exit 1
        fi
    else
        print_info "game/objects/ directory already exists"
    fi
}

# Function to initialize and fetch all submodules
init_submodules() {
    print_info "Initializing and fetching submodules..."
    cd "$SCRIPT_DIR"
    
    # Initialize submodules if they haven't been initialized
    if [[ -f ".gitmodules" ]]; then
        git submodule init
        git submodule update --recursive
        print_success "Submodules initialized and updated successfully!"
    else
        print_info "No submodules found (.gitmodules not present)"
    fi
}

# Function to start services
start_services() {
    print_info "Starting hackercore services..."
    cd "$SCRIPT_DIR"
    
    # Check and clone game directory if needed
    check_game_directory
    
    # Initialize and fetch submodules before starting
    init_submodules
    
    # Check if git heads have changed and rebuild if necessary
    if check_git_heads_changed; then
        print_info "Git heads have changed, rebuilding images..."
        rebuild_images
    fi
    
    docker-compose up -d
    print_success "Services started successfully!"
    print_info "You can connect to the MUD via telnet on localhost:8888"
    print_info "Or connect via websocket on localhost:8080"
    
    # Save current git heads after successful start
    save_git_heads
    
    # Follow logs automatically to monitor for unexpected daemon deaths
    print_info "Following moor daemon logs (Ctrl+C to exit)..."
    docker-compose logs -f moor-daemon
}

# Function to stop services
stop_services() {
    print_info "Stopping hackercore services..."
    cd "$SCRIPT_DIR"
    docker-compose down
    print_success "Services stopped successfully!"
}

# Function to restart services
restart_services() {
    print_info "Restarting hackercore services..."
    cd "$SCRIPT_DIR"
    
    # Initialize and fetch submodules before restarting
    init_submodules
    
    docker-compose restart
    print_success "Services restarted successfully!"
}

# Function to show status
show_status() {
    print_info "Checking hackercore services status..."
    cd "$SCRIPT_DIR"
    docker-compose ps
}

# Function to clean Docker images and containers
clean_docker() {
    print_info "Cleaning Docker images and containers..."
    cd "$SCRIPT_DIR"
    
    # Stop services first
    print_info "Stopping services..."
    docker-compose down
    
    # Remove containers
    print_info "Removing containers..."
    docker-compose rm -f
    
    # Remove images built by this compose file
    print_info "Removing images..."
    docker-compose down --rmi all
    
    # Clean up any dangling images
    print_info "Cleaning up dangling images..."
    docker image prune -f
    
    print_success "Docker cleanup completed!"
}

# Function to rebuild all images
rebuild_images() {
    print_info "Rebuilding all Docker images..."
    cd "$SCRIPT_DIR"
    
    # Initialize and fetch submodules before rebuilding
    init_submodules
    
    # Force rebuild all images without cache
    print_info "Building images (no cache)..."
    docker-compose build --no-cache
    
    print_success "All images rebuilt successfully!"

    # Save get heads on successful rebuild
    save_git_heads
    
    # Clean build cache to prevent bloat
    print_info "Cleaning Docker build cache..."
    docker builder prune -a --force
    print_success "Docker build cache cleaned!"
}

# Function to clean and rebuild
clean_rebuild() {
    print_info "Performing clean rebuild..."
    clean_docker
    rebuild_images
    print_success "Clean rebuild completed!"
    print_info "Note: Docker build cache was automatically cleaned during rebuild."
}

# Function to follow logs
follow_logs() {
    print_info "Following moor daemon logs (Ctrl+C to exit)..."
    cd "$SCRIPT_DIR"
    docker-compose logs -f moor-daemon
}

# Function to deploy and tag all images
deploy_images() {
    print_info "Building and tagging all Docker images for deployment..."
    cd "$SCRIPT_DIR"
    
    # Initialize and fetch submodules before building
    init_submodules
    
    # Define image tags
    local BACKEND_TAG="hackercore-backend:latest"
    local DOME_CLIENT_TAG="hackercore-dome-client:latest"
    
    print_info "Building backend image (moor services)..."
    docker build \
        --target backend \
        --build-arg BUILD_PROFILE="${BUILD_PROFILE:-release}" \
        -t "$BACKEND_TAG" \
        -f Dockerfile \
        .
    
    if [[ $? -eq 0 ]]; then
        print_success "Backend image built and tagged: $BACKEND_TAG"
    else
        print_error "Failed to build backend image"
        exit 1
    fi
    
    print_info "Building dome-client image..."
    docker build \
        -t "$DOME_CLIENT_TAG" \
        -f vendor/dome-client/Dockerfile \
        vendor/dome-client
    
    if [[ $? -eq 0 ]]; then
        print_success "Dome-client image built and tagged: $DOME_CLIENT_TAG"
    else
        print_error "Failed to build dome-client image"
        exit 1
    fi
    
    print_success "All images built and tagged successfully!"
    echo
    print_info "=== Container Tags ==="
    echo "Backend Services (moor-daemon, moor-telnet-host, moor-curl-worker, moor-vcs-worker):"
    echo "  $BACKEND_TAG"
    echo
    echo "Dome Client:"
    echo "  $DOME_CLIENT_TAG"
    echo
    print_info "You can now push these images to a registry or use them locally."
    print_info "To start services with these images, run: ./hacker.sh start"
    
    # Save git heads after successful deploy
    save_git_heads
}

# Function to update a submodule to latest version
update_submodule() {
    local submodule_path=${1:-"vendor/moor"}  # Default to vendor/moor for backward compatibility
    
    # Extract submodule name from path for display purposes
    local submodule_name=$(basename "$submodule_path")
    
    print_info "Updating $submodule_path submodule to latest version..."
    cd "$SCRIPT_DIR"
    
    # Check if submodule directory exists
    if [[ ! -d "$submodule_path" ]]; then
        print_error "$submodule_path directory not found!"
        print_info "Available submodules:"
        if [[ -f ".gitmodules" ]]; then
            git config --file .gitmodules --get-regexp path | cut -d' ' -f2 | sed 's/^/  /'
        else
            print_info "  No submodules found (.gitmodules not present)"
        fi
        exit 1
    fi
    
    # Get current commit hash before update
    local current_commit=$(get_git_head "$SCRIPT_DIR/$submodule_path")
    print_info "Current $submodule_name commit: $current_commit"
    
    # Navigate to submodule and pull latest changes
    print_info "Pulling latest changes from $submodule_path..."
    cd "$SCRIPT_DIR/$submodule_path"
    
    # Check if we're on a valid branch (not detached HEAD)
    local current_branch=$(git branch --show-current 2>/dev/null || echo "detached")
    if [[ "$current_branch" == "detached" ]]; then
        # Try to determine the default branch (main or master)
        local default_branch="main"
        if ! git show-ref --verify --quiet refs/remotes/origin/main; then
            if git show-ref --verify --quiet refs/remotes/origin/master; then
                default_branch="master"
            fi
        fi
        print_warning "Currently on detached HEAD, switching to $default_branch branch..."
        git checkout "$default_branch"
    fi
    
    # Determine the default branch for pulling
    local remote_branch="main"
    if ! git show-ref --verify --quiet refs/remotes/origin/main; then
        if git show-ref --verify --quiet refs/remotes/origin/master; then
            remote_branch="master"
        fi
    fi
    
    # Pull the latest changes
    if git pull origin "$remote_branch"; then
        print_success "Successfully pulled latest changes from $submodule_path!"
        
        # Get new commit hash
        local new_commit=$(git rev-parse HEAD)
        print_info "Updated $submodule_name commit: $new_commit"
        
        # Go back to main directory and commit the submodule update
        cd "$SCRIPT_DIR"
        print_info "Committing submodule update to main repository..."
        
        if git add "$submodule_path" && git commit -m "Update $submodule_path submodule to latest version

Previous commit: $current_commit
New commit: $new_commit

Automated update via hacker.sh"; then
            print_success "Submodule update committed successfully!"
            print_info "You may want to run './hacker.sh clean-rebuild' to rebuild with the latest changes."
        else
            print_warning "Submodule was updated but commit failed (may already be up to date)"
        fi
        
    else
        print_error "Failed to pull latest changes from $submodule_path!"
        print_info "Please check your network connection and try again."
        exit 1
    fi
}

# Main function
main() {
    local command=${1:-""}
    
    # Check prerequisites first
    check_prerequisites
    check_compose_file
    
    case $command in
        "start")
            start_services
            ;;
        "stop")
            stop_services
            ;;
        "restart")
            restart_services
            ;;
        "status")
            show_status
            ;;
        "clean")
            clean_docker
            ;;
        "rebuild")
            rebuild_images
            ;;
        "clean-rebuild")
            clean_rebuild
            ;;
        "logs")
            follow_logs
            ;;
        "deploy")
            deploy_images
            ;;
        "update")
            local submodule_path=${2:-""}
            update_submodule "$submodule_path"
            ;;
        "")
            print_error "No command specified."
            echo
            echo "Usage: $0 [start|stop|restart|status|clean|rebuild|clean-rebuild|logs|deploy|update [submodule_path]]"
            echo
            echo "Commands:"
            echo "  start        - Start all hackercore services (automatically follows logs)"
            echo "  stop         - Stop all hackercore services"
            echo "  restart      - Restart all hackercore services"
            echo "  status       - Show status of all services"
            echo "  clean        - Stop services and remove all Docker images/containers"
            echo "  rebuild      - Force rebuild all Docker images (no cache)"
            echo "  clean-rebuild - Clean everything and rebuild from scratch"
            echo "  logs         - Follow moor daemon logs"
            echo "  deploy       - Build and tag all Docker images for deployment"
            echo "  update [path] - Update submodule to latest version (default: vendor/moor)"
            exit 1
            ;;
        *)
            print_error "Unknown command: $command"
            echo
            echo "Usage: $0 [start|stop|restart|status|clean|rebuild|clean-rebuild|logs|deploy|update [submodule_path]]"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
