# Krakdown

A local-first Git client for macOS. Krakdown talks to the `git` binary already on your machine, so it uses the same SSH agent, credential helper, and user identity you use in Terminal. There is no in-app login.

## Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) via `rustup`
- [Xcode Command Line Tools](https://developer.apple.com/xcode/resources/): `xcode-select --install`
- Git on your PATH (Homebrew and `/usr/bin/git` are also discovered automatically)

## Develop

```bash
npm install
npm run tauri dev
```

This starts Vite on `http://localhost:1420` and opens the native Krakdown window.

## Export a native Mac app

Build the release bundle:

```bash
npm run tauri build
```

Tauri compiles the Rust backend and packages the Vue frontend into a macOS app. When the build finishes, look here:

| Artifact | Path |
| --- | --- |
| App bundle | `src-tauri/target/release/bundle/macos/Krakdown.app` |
| Disk image | `src-tauri/target/release/bundle/dmg/Krakdown_0.1.0_aarch64.dmg` (Apple Silicon) or the `x64` equivalent on Intel |

To run it locally, open the `.app` (or drag it to `/Applications`). macOS Gatekeeper may warn that the app is unsigned. That is expected for a machine-local build: right-click the app, choose **Open**, then confirm.

### Sharing the app with other Macs

Unsigned builds are fine on the machine that compiled them. To distribute Krakdown to other people you need an [Apple Developer](https://developer.apple.com/programs/) account, then:

1. Set a unique bundle identifier in [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) (`identifier` is already `com.krakdown.app`).
2. Sign with a Developer ID Application certificate.
3. Notarize the `.app` or `.dmg` with Apple.

See Tauri’s guide: [macOS code signing and notarization](https://v2.tauri.app/distribute/sign/macos/).

Apple Silicon and Intel produce different binaries. Build on the architecture you intend to ship, or use a universal target if you need both.

## How credentials work

Krakdown never asks for an email, password, or Git host login. Pull and checkout run the same `git` you use in Terminal, so cached HTTPS helpers, `osxkeychain`, GitHub CLI, and SSH keys in the agent all just work.

GUI apps launched from Finder sometimes miss Homebrew’s PATH. Krakdown looks for Git in `PATH`, then `/opt/homebrew/bin/git`, `/usr/local/bin/git`, `/usr/bin/git`, and finally a login `zsh`. If a pull still cannot authenticate, run the same `git pull` once in Terminal so the credential helper can store access.

## Data

Repository groups are stored at:

`~/Library/Application Support/com.krakdown.app/groups.json`

That file is safe to inspect or edit. It stores group names and local folder paths only — never credentials.
