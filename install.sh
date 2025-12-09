#!/bin/bash
#
# ripVID Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/BurgessTG/ripVID/main/install.sh | bash
#

set -e

VERSION="2.2.4"
REPO="BurgessTG/ripVID"
INSTALL_DIR="/usr/local/bin"
APP_NAME="ripvid"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════╗"
    echo "║           ripVID Installer v${VERSION}          ║"
    echo "║     Video Downloader for YouTube & X      ║"
    echo "╚═══════════════════════════════════════════╝"
    echo -e "${NC}"
}

detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f /etc/arch-release ]; then
            echo "arch"
        elif [ -f /etc/debian_version ]; then
            echo "debian"
        elif [ -f /etc/fedora-release ]; then
            echo "fedora"
        elif [ -f /etc/redhat-release ]; then
            echo "rhel"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

install_arch() {
    echo -e "${GREEN}Installing on Arch Linux...${NC}"

    # Check if already installed
    if pacman -Qi ripvid &>/dev/null || pacman -Qi ripvid-bin &>/dev/null; then
        echo -e "${YELLOW}ripVID is already installed. Updating...${NC}"
    fi

    # Install from AUR (if yay or paru available)
    if command -v yay &>/dev/null; then
        yay -S ripvid-bin --noconfirm
    elif command -v paru &>/dev/null; then
        paru -S ripvid-bin --noconfirm
    else
        # Manual installation
        echo -e "${YELLOW}No AUR helper found. Installing manually...${NC}"
        TEMP_DIR=$(mktemp -d)
        cd "$TEMP_DIR"

        # Download and extract AppImage
        curl -fsSL "https://github.com/${REPO}/releases/download/v${VERSION}/ripVID_${VERSION}_amd64.AppImage" -o ripvid.AppImage
        chmod +x ripvid.AppImage

        # Install to /opt
        sudo mkdir -p /opt/ripvid
        sudo mv ripvid.AppImage /opt/ripvid/
        sudo ln -sf /opt/ripvid/ripvid.AppImage /usr/local/bin/ripvid

        # Create desktop entry
        sudo tee /usr/share/applications/ripvid.desktop > /dev/null << EOF
[Desktop Entry]
Name=ripVID
Comment=Download videos from YouTube, X/Twitter, and 1000+ sites
Exec=/opt/ripvid/ripvid.AppImage
Icon=ripvid
Terminal=false
Type=Application
Categories=Network;AudioVideo;Video;
EOF

        cd -
        rm -rf "$TEMP_DIR"
    fi

    echo -e "${GREEN}✓ ripVID installed successfully!${NC}"
}

install_debian() {
    echo -e "${GREEN}Installing on Debian/Ubuntu...${NC}"

    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Download deb package
    curl -fsSL "https://github.com/${REPO}/releases/download/v${VERSION}/ripVID_${VERSION}_amd64.deb" -o ripvid.deb

    # Install
    sudo dpkg -i ripvid.deb || sudo apt-get install -f -y

    cd -
    rm -rf "$TEMP_DIR"

    echo -e "${GREEN}✓ ripVID installed successfully!${NC}"
}

install_fedora() {
    echo -e "${GREEN}Installing on Fedora/RHEL...${NC}"

    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Download rpm package
    curl -fsSL "https://github.com/${REPO}/releases/download/v${VERSION}/ripVID-${VERSION}-1.x86_64.rpm" -o ripvid.rpm

    # Install
    sudo dnf install -y ./ripvid.rpm || sudo rpm -i ripvid.rpm

    cd -
    rm -rf "$TEMP_DIR"

    echo -e "${GREEN}✓ ripVID installed successfully!${NC}"
}

install_linux_generic() {
    echo -e "${GREEN}Installing using AppImage...${NC}"

    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"

    # Download AppImage
    curl -fsSL "https://github.com/${REPO}/releases/download/v${VERSION}/ripVID_${VERSION}_amd64.AppImage" -o ripvid.AppImage
    chmod +x ripvid.AppImage

    # Install to /opt
    sudo mkdir -p /opt/ripvid
    sudo mv ripvid.AppImage /opt/ripvid/
    sudo ln -sf /opt/ripvid/ripvid.AppImage /usr/local/bin/ripvid

    # Create desktop entry
    sudo mkdir -p /usr/share/applications
    sudo tee /usr/share/applications/ripvid.desktop > /dev/null << EOF
[Desktop Entry]
Name=ripVID
Comment=Download videos from YouTube, X/Twitter, and 1000+ sites
Exec=/opt/ripvid/ripvid.AppImage
Icon=ripvid
Terminal=false
Type=Application
Categories=Network;AudioVideo;Video;
EOF

    cd -
    rm -rf "$TEMP_DIR"

    echo -e "${GREEN}✓ ripVID installed successfully!${NC}"
}

install_macos() {
    echo -e "${YELLOW}macOS support coming soon!${NC}"
    echo "For now, please download from: https://github.com/${REPO}/releases"
    exit 1
}

install_windows() {
    echo -e "${GREEN}Installing on Windows...${NC}"
    echo ""
    echo "Please download the installer from:"
    echo -e "${BLUE}https://github.com/${REPO}/releases/download/v${VERSION}/ripVID_${VERSION}_x64-setup.exe${NC}"
    echo ""
    echo "Or use PowerShell:"
    echo -e "${YELLOW}Invoke-WebRequest -Uri 'https://github.com/${REPO}/releases/download/v${VERSION}/ripVID_${VERSION}_x64-setup.exe' -OutFile 'ripVID-setup.exe'; Start-Process ripVID-setup.exe${NC}"
    exit 0
}

main() {
    print_banner

    OS=$(detect_os)
    echo -e "Detected OS: ${BLUE}${OS}${NC}"
    echo ""

    case "$OS" in
        arch)
            install_arch
            ;;
        debian)
            install_debian
            ;;
        fedora|rhel)
            install_fedora
            ;;
        linux)
            install_linux_generic
            ;;
        macos)
            install_macos
            ;;
        windows)
            install_windows
            ;;
        *)
            echo -e "${RED}Unsupported operating system: $OSTYPE${NC}"
            echo "Please download manually from: https://github.com/${REPO}/releases"
            exit 1
            ;;
    esac

    echo ""
    echo -e "${GREEN}Installation complete!${NC}"
    echo -e "Run ${BLUE}ripvid${NC} to start the application."
    echo ""
    echo -e "Report issues: ${BLUE}https://github.com/${REPO}/issues${NC}"
}

main "$@"
