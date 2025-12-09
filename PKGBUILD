# Maintainer: BurgessTG <your-email@example.com>
pkgname=ripvid
pkgver=2.2.4
pkgrel=1
pkgdesc="A beautiful, modern desktop application for downloading videos from YouTube, X/Twitter, and 1000+ sites"
arch=('x86_64')
url="https://github.com/BurgessTG/ripVID"
license=('Apache-2.0')
depends=(
    'webkit2gtk-4.1'
    'gtk3'
    'libsoup3'
    'cairo'
    'gdk-pixbuf2'
    'glib2'
    'pango'
    'openssl'
    'hicolor-icon-theme'
)
makedepends=(
    'rust'
    'cargo'
    'bun'
    'webkit2gtk-4.1'
    'base-devel'
    'curl'
    'wget'
    'file'
    'openssl'
    'appmenu-gtk-module'
    'gtk3'
    'libappindicator-gtk3'
    'librsvg'
    'libvips'
    'patchelf'
)
optdepends=(
    'libappindicator-gtk3: system tray support'
)
provides=('ripvid')
conflicts=('ripvid-bin' 'ripvid-git')
source=("$pkgname-$pkgver.tar.gz::https://github.com/BurgessTG/ripVID/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
    cd "ripVID-$pkgver"

    # Install dependencies with bun
    bun install

    # Build frontend
    bun run build
}

build() {
    cd "ripVID-$pkgver"

    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target

    # Build Tauri app
    cd src-tauri
    cargo build --release --locked
}

package() {
    cd "ripVID-$pkgver"

    # Install binary
    install -Dm755 "src-tauri/target/release/video-downloader" "$pkgdir/usr/bin/ripvid"

    # Install desktop file
    install -Dm644 "src-tauri/icons/ripVID.desktop" "$pkgdir/usr/share/applications/ripvid.desktop" 2>/dev/null || \
    cat > "$pkgdir/usr/share/applications/ripvid.desktop" << EOF
[Desktop Entry]
Name=ripVID
Comment=Download videos from YouTube, X/Twitter, and 1000+ sites
Exec=ripvid
Icon=ripvid
Terminal=false
Type=Application
Categories=Network;AudioVideo;Video;
Keywords=youtube;video;download;twitter;x;
EOF

    # Install icons
    for size in 16 32 64 128 256; do
        if [ -f "src-tauri/icons/${size}x${size}.png" ]; then
            install -Dm644 "src-tauri/icons/${size}x${size}.png" \
                "$pkgdir/usr/share/icons/hicolor/${size}x${size}/apps/ripvid.png"
        fi
    done

    # Install 512x512 icon if available
    if [ -f "src-tauri/icons/128x128@2x.png" ]; then
        install -Dm644 "src-tauri/icons/128x128@2x.png" \
            "$pkgdir/usr/share/icons/hicolor/256x256/apps/ripvid.png"
    fi

    # Install license
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE" 2>/dev/null || true
}
