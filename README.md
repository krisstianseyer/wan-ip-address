# WAN IP Address

A tiny Windows desktop app that shows your public WAN IP address, copies it in one click, and keeps a local history of the last 50 addresses.

## Stack

- Tauri v2 (Rust backend)
- Vue 3 + TypeScript + Vite
- Tailwind CSS v4, Verdana
- Public lookup via `api.ipify.org` with `api64.ipify.org` fallback

## Features

- Public IPv4 or IPv6 with source label
- One-click copy, `Ctrl+Shift+C`
- Refresh via button or `Ctrl+R`
- Local history (last 50, IPv4/IPv6 filter), stored in the app data folder
- About block with the GitHub repository link, opened in the default browser
- Custom frameless title bar with version chip, remembered window size and position
- No telemetry; only the lookup request leaves the machine

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

Outputs:

- Installer: `src-tauri/target/release/bundle/nsis/WAN IP Address_1.2.0_x64-setup.exe`
- Portable: `src-tauri/target/release/wan-ip-address.exe`
