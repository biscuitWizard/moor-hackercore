#!/bin/bash

# hacker.sh - Docker Compose management script for hackercore
# Usage: ./hacker.sh [start|stop|restart|status]

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
    
    # Initialize and fetch submodules before starting
    init_submodules
    
    docker-compose up -d
    print_success "Services started successfully!"
    print_info "You can connect to the MUD via telnet on localhost:8888"
    print_info "Or connect via websocket on localhost:8080"
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
        "")
            print_error "No command specified."
            echo
            echo "Usage: $0 [start|stop|restart|status]"
            echo
            echo "Commands:"
            echo "  start   - Start all hackercore services"
            echo "  stop    - Stop all hackercore services"
            echo "  restart - Restart all hackercore services"
            echo "  status  - Show status of all services"
            exit 1
            ;;
        *)
            print_error "Unknown command: $command"
            echo
            echo "Usage: $0 [start|stop|restart|status]"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
