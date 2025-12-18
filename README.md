![](/public/pee-esque-tree-logo-small.png)
# Pee-Esque-Tree — Game Update Downloader

A cross-platform desktop app for downloading PS3 game updates.

> ⚠️ Important: Release binaries are currently **not code-signed**.
>
> - **macOS** will block unsigned apps by default (Gatekeeper).
> - **Windows** may show SmartScreen warnings. If you don’t want workarounds, build from source (or sign it yourself).

---

## Downloads

Get the latest builds from the **Releases** page on this GitHub repo.

---

## Build from source

### Prerequisites

- **Node.js** (recommended: 20+)
- **Rust** (stable toolchain)
- Tauri prerequisites for your OS (WebView / build tools)

### Install dependencies

```bash
npm install
```

### Run in development

```bash
npm run tauri dev
```

### Build for distribution

```bash
npm run tauri build
```

Build outputs appear in:

- `src-tauri/target/release/bundle/`

---

## Running unsigned builds (workarounds)

### macOS (Gatekeeper)

#### Option A — Use Sentinel (GUI)

If you prefer a simple GUI workaround, you can use **Sentinel** (macOS) to allow an unsigned app you downloaded. This is effectively a convenience tool for removing Gatekeeper / quarantine restrictions.

#### Option B — Finder “Open”

1. Right-click the app → **Open**
2. Confirm **Open** (you usually only need to do this once per app)

#### Option C — Command line: remove quarantine

```bash
xattr -dr com.apple.quarantine "/Applications/Pee-Esque-Tree.app"
```

You can also do this wherever the app is located.

---

## macOS self-signing (local / personal use)

This does **not** make the app trusted like a real Apple Developer ID signature, but it can reduce friction for local use.

### 1) Ad-hoc sign the app (no certificate required)

```bash
codesign --force --deep --sign - "src-tauri/target/release/bundle/macos/Pee-Esque-Tree.app"
```

Verify:

```bash
codesign --verify --deep --strict --verbose=2 "src-tauri/target/release/bundle/macos/Pee-Esque-Tree.app"
```

### 2) Remove quarantine if needed

```bash
xattr -dr com.apple.quarantine "src-tauri/target/release/bundle/macos/Pee-Esque-Tree.app"
```

### 3) Optional: Gatekeeper assessment

```bash
spctl --assess --verbose=4 "src-tauri/target/release/bundle/macos/Pee-Esque-Tree.app"
```

> Note: Without an Apple Developer ID certificate and notarisation, Gatekeeper may still warn on other Macs. Ad-hoc signing is mainly useful for **your own machine**.

---

## Windows (SmartScreen)

Unsigned apps may show:

- “Windows protected your PC”
- “Unknown publisher”

To run anyway:

1. Click **More info**
2. Click **Run anyway**

---

## Linux

Linux generally does not require code signing. Run the AppImage or install the package from the build output.

---

## GitHub Actions builds

This repo can build installers automatically via GitHub Actions.

If you are using a tag-based workflow, trigger a release build with:

```bash
git tag v0.1.0
git push origin v0.1.0
```

---

## Disclaimer

This tool is intended for legitimate update downloading and archival use. PS3, PlayStation, and related marks are the property of their respective owners.

