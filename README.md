# Shipyard

A local-first Git client for macOS. Shipyard talks to the `git` binary already on your machine, so it uses the same SSH agent, credential helper, and user identity you use in Terminal. There is no in-app login.

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
| Disk image | `src-tauri/target/release/bundle/dmg/Shipyard_1.0.0_aarch64.dmg` (Apple Silicon) or the `x64` equivalent on Intel |

To run it locally, open the `.app` (or drag it to `/Applications`). macOS Gatekeeper may warn that an unsigned local build is unidentified: right-click the app, choose **Open**, then confirm.

## Release on GitHub

No custom domain is required. GitHub Releases is the download host.

1. Create a public GitHub repo (the stable URL below uses `shipyard` as the repo name):

   ```bash
   gh repo create shipyard --public --source=. --remote=origin --push
   ```

2. Bump `version` in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` so they match.
3. Tag and push:

   ```bash
   git tag v1.0.0
   git push origin main
   git push origin v1.0.0
   ```

4. The [Release](.github/workflows/release.yml) workflow builds Apple Silicon and Intel `.dmg` files, uploads them with **stable filenames**, and opens a draft GitHub Release. Publish the draft when you have reviewed it.

After the release is published, these URLs always point at the latest `.dmg`:

```
https://github.com/<you>/shipyard/releases/latest/download/Shipyard-macos-arm64.dmg
https://github.com/<you>/shipyard/releases/latest/download/Shipyard-macos-x64.dmg
```

The filenames stay the same from version to version. Only publish one latest release at a time; GitHub’s `/latest/download/` link ignores drafts and prereleases.

You can also start a build from the Actions tab with **Run workflow**.

### Signing and notarization

Unsigned builds are fine on the machine that compiled them. To hand the `.dmg` to other people, enroll in the [Apple Developer Program](https://developer.apple.com/programs/) and add these repository secrets so the workflow can sign and notarize:

| Secret | What it is |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64 of your exported Developer ID Application `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Password for that `.p12` |
| `APPLE_SIGNING_IDENTITY` | Identity string from `security find-identity -v -p codesigning` |
| `APPLE_ID` | Apple ID email |
| `APPLE_PASSWORD` | [App-specific password](https://support.apple.com/en-us/102654) |
| `APPLE_TEAM_ID` | Team ID from your [membership page](https://developer.apple.com/account) |

See Tauri’s [macOS code signing and notarization](https://v2.tauri.app/distribute/sign/macos/) guide. Until those secrets exist, CI uses ad-hoc signing so Apple Silicon downloads are not marked damaged. Recipients still have to allow the app under System Settings → Privacy & Security.

The bundle identifier is `com.shipyard.app`. That does not require owning a `.com`.

## How credentials work

Shipyard never asks for an email, password, or Git host login. Pull and checkout run the same `git` you use in Terminal, so cached HTTPS helpers, `osxkeychain`, GitHub CLI, and SSH keys in the agent all just work.

GUI apps launched from Finder sometimes miss Homebrew’s PATH. Shipyard looks for Git in `PATH`, then `/opt/homebrew/bin/git`, `/usr/local/bin/git`, `/usr/bin/git`, and finally a login `zsh`. If a pull still cannot authenticate, run the same `git pull` once in Terminal so the credential helper can store access.

## Data

Preferences, repository groups, and window position are stored at:

`~/Library/Application Support/com.shipyard.app/settings.json`

On first launch, Shipyard copies `settings.json`, `groups.json`, and command history from `~/Library/Application Support/com.krakdown.app/` if that folder still exists.

The Settings tab has form controls; open `settings.json` from the top-right icon to edit, export, or import the full file. It stores group names and local folder paths only — never credentials.
