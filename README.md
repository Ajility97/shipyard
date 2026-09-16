# Shipyard

A local-first Git client for macOS. Shipyard talks to the `git` binary already on your machine, so it uses the same SSH agent, credential helper, and user identity you use in Terminal. There is no in-app login.

<p align="center">
  <a href="https://github.com/fylzero/shipyard/releases/latest/download/Shipyard-macos-arm64.dmg"><img src="docs/download-macos.svg" alt="Download Shipyard for macOS" height="56"></a>
  <a href="https://buymeacoffee.com/fylzero1"><img src="docs/buy-me-a-coffee.svg" alt="Buy me a coffee" height="56"></a>
</p>

## Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) via `rustup` (`cargo` must be on your PATH; if a new terminal cannot find it, run `source "$HOME/.cargo/env"`)
- [Xcode Command Line Tools](https://developer.apple.com/xcode/resources/): `xcode-select --install`
- Git on your PATH (Homebrew and `/usr/bin/git` are also discovered automatically)

## Develop

```bash
npm install
npm run tauri dev
```

This starts Vite on `http://localhost:1420` and opens the native Shipyard window.

## Export a native Mac app

```bash
npm run tauri build
```

When the build finishes:

| Artifact | Path |
| --- | --- |
| App bundle | `src-tauri/target/release/bundle/macos/Shipyard.app` |
| Disk image | `src-tauri/target/release/bundle/dmg/Shipyard_1.0.0_aarch64.dmg` |

To run it locally, open the `.app` (or drag it to `/Applications`). macOS Gatekeeper may warn that an unsigned local build is unidentified: right-click the app, choose **Open**, then confirm.

