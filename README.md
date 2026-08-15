# ripVID

https://github.com/user-attachments/assets/db3929ad-2251-4d2b-8641-d3c9e34bb72d

A modern desktop app for downloading videos from YouTube, X/Twitter, TikTok, and 1000+ sites. Built with Tauri, React, and Rust.

## Installation

**Windows / macOS / Linux** - Download from [Releases](https://github.com/honeycomb-Technologies/ripVID/releases/latest)

**Arch Linux (AUR)**
```bash
yay -S ripvid
```

**Linux (curl)**
```bash
curl -fsSL https://raw.githubusercontent.com/honeycomb-Technologies/ripVID/main/install.sh | bash
```

**Windows (PowerShell)**
```powershell
irm https://raw.githubusercontent.com/honeycomb-Technologies/ripVID/main/install.ps1 | iex
```

## Features

- Download from YouTube, X/Twitter, TikTok, Facebook, Instagram, and 1000+ sites
- Highest quality H.264/MP4 video or MP3 audio extraction
- Auto-updates for app and dependencies (yt-dlp, ffmpeg)
- Smart cookie handling for age-restricted content
- Cross-platform: Windows, macOS, Linux
- Lightweight Rust backend

## How It Works

On first launch, ripVID downloads yt-dlp and ffmpeg automatically. These are updated daily in the background. The app checks for updates every 30 minutes and installs them seamlessly.

## Development

**Prerequisites:** Node.js 18+, Rust, Bun

```bash
git clone https://github.com/honeycomb-Technologies/ripVID.git
cd ripVID
bun install
bun run tauri:dev
```

**Build:**
```bash
bun run tauri:build
```

## Tech Stack

- **Frontend:** React, TypeScript, Tailwind CSS, Vite
- **Backend:** Tauri v2, Rust, tokio
- **Tools:** yt-dlp, ffmpeg, deno (all auto-downloaded on first launch)

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Binary download fails | Check internet, allow GitHub access through firewall |
| Download fails | Verify URL, app auto-retries with browser cookies |
| Update issues | Download latest from Releases manually |

## Legal

Users are responsible for complying with all applicable laws. Do not download copyrighted content without permission or violate platform terms of service.

## License

Apache License 2.0 - See [LICENSE](LICENSE)

## Links

- [Releases](https://github.com/honeycomb-Technologies/ripVID/releases)
- [Issues](https://github.com/honeycomb-Technologies/ripVID/issues)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) | [Tauri](https://tauri.app) | [FFmpeg](https://ffmpeg.org)
