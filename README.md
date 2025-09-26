# HackerCORE

<div align="center">

![MUD](https://img.shields.io/badge/MUD-Multi--User%20Dungeon-blue?style=for-the-badge&logo=terminal)
![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?style=for-the-badge&logo=docker)
![Rust](https://img.shields.io/badge/Powered%20by-Rust-orange?style=for-the-badge&logo=rust)

*A modern MUD (Multi-User Dungeon) Core built with mooR - the next generation MOO server*

</div>

## What is Hackercore?

HackerCORE is a **mooR** database built to be the foundation or starting point for future MUDs. Built on the **mooR** platform, it provides a robust, scalable foundation for interactive fiction and multiplayer text adventures.

### Features

- **Feature** - When I actually have a feature I should put it here :P

## Quick Start

### Prerequisites

The `hacker.sh` script will automatically check for and guide you through installing:

- **Docker** - Container runtime
- **Docker Compose** - Multi-container orchestration

### Running Hackercore

1. **Clone and Navigate**
   ```bash
   git clone git@github.com:biscuitWizard/moor-hackercore.git
   cd hackercore
   ```

2. **Start the MUD**
   ```bash
   ./hacker.sh start
   ```

3. **Connect and Play**
   ```bash
   # Via Telnet (traditional MUD client)
   telnet localhost 8888
   
   # Or use any MUD client like:
   # - Mudlet
   # - TinTin++
   # - MUSHclient
   ```

## The `hacker.sh` Script

Our custom management script makes running HackerCore incredibly simple:

### 📋 Available Commands

| Command | Description |
|---------|-------------|
| `./hacker.sh start` | Start all HackerCore services |
| `./hacker.sh stop` | Stop all services |
| `./hacker.sh restart` | Restart all services |
| `./hacker.sh status` | Show service status |

### 🔌 Ports

| Port | Service | Description |
|------|---------|-------------|
| 8888 | Telnet | Traditional MUD connection |
| 8080 | WebSocket | Modern web-based connection |
| 7896-7899 | Internal | Inter-service communication |

## 📁 Project Structure

```
hackercore/
├── 🎮 hacker.sh              # Management script
├── 🐳 docker-compose.yml     # Service orchestration
├── 📂 core/                   # MUD database objects
│   ├── #0.moo                # System objects
│   ├── #1.moo                # Root object
│   └── ...                   # Game objects
├── 📂 vendor/moor/           # mooR server implementation
├── 📂 vcs-worker/            # Custom worker processes
└── 📂 db/                    # Database storage
```

## Troubleshooting

### Common Issues

** "Docker not found"**
- The script will provide installation instructions for your OS
- Follow the provided commands to install Docker

** "Docker Compose not found"**
- The script will guide you through Docker Compose installation
- Usually just a few commands away

** "Permission denied"**
- Make sure the script is executable: `chmod +x hacker.sh`
- Ensure Docker daemon is running: `sudo systemctl start docker`

** "Port already in use"**
- Check what's using the port: `netstat -tulpn | grep 8888`
- Stop conflicting services or change ports in `docker-compose.yml`

## Resources

### MUD Development
- [MOO Programming Guide](https://www.hayseed.net/MOO/)
- [LambdaCore Documentation](https://www.lambda.org/)
- [MUD Client Development](https://www.mudconnect.com/)

### mooR Platform
- [mooR Documentation](https://github.com/ryan-daum/moor)
- [Rust MUD Development](https://crates.io/crates/moor)

### Community
- [MUD Development Forums](https://www.mudconnect.com/)
- [Interactive Fiction Community](https://intfiction.org/)

## Acknowledgments

- **LambdaCore** - The legendary MUD core database that inspired this project
- **mooR** - The modern Rust-based MOO server platform
- **MUD Community** - Decades of innovation in text-based gaming

---

<div align="center">

**Ready to start your adventure?**

```bash
./hacker.sh start
telnet localhost 8888
```

*Welcome to HackerCore!* 

</div>