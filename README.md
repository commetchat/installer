<p align="center" style="padding-top:20px">
<h1 align="center">Installer</h1>
<p align="center">installer + updater for Commet on Windows</p>

<p align="center">
    <a href="https://matrix.to/#/#commet:matrix.org">
        <img alt="Matrix" src="https://img.shields.io/matrix/commet%3Amatrix.org?logo=matrix">
    </a>
    <a href="https://fosstodon.org/@commetchat">
        <img alt="Mastodon" src="https://img.shields.io/mastodon/follow/109894490854601533?domain=https%3A%2F%2Ffosstodon.org">
    </a>
    <a href="https://bsky.app/profile/commet.chat">
        <img alt="Bluesky" src="https://img.shields.io/badge/follow-@commet.chat-whitesmoke?style=social&logo=bluesky">
    </a>

</p>


# Development

## Building

### 1. Build UI
The installer uses an embedded webview for the UI. This page is written in vanilla HTML + JS, but needs to be bundled with webpack to a single file, so it can be easily embedded in the executable.

```
cd ui
npm i
npm run build
```

### 2. Build Installer
```
cargo build
```