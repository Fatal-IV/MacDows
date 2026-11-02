# Changelog

All notable changes to MacDows are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-11-02

First public release. A notch at the top of the screen and a dock at the bottom, as an extra layer on top of the Windows taskbar. The taskbar itself is not touched in this version.

### Added
- **Notch**: a small pill at the top center of the primary monitor showing the time. Hover to expand it with a spring animation.
- **Media panel** in the notch: album art, title, artist and previous / play-pause / next buttons for whatever is playing (Spotify, browser video, and other apps that use the Windows media controls). Shows "Nothing playing" when idle.
- **Dock** at the bottom center, above the taskbar: one icon per open app window, click to bring it to the front, icons grow when you hover them, a dot marks each running app. The dock has a black, edge-attached shape with curved sides.
- **Stable dock order**: icons stay where they first appeared, new apps are added at the end, closed apps disappear.
- **Store app icons**: apps installed from the Microsoft Store, such as WhatsApp, now show their real icon instead of a blank one.
- **Fullscreen detection**: the notch and dock hide automatically while a fullscreen app is in front (a fullscreen video or a game) and come back when you leave fullscreen. Works event-based, without polling.
- **Visibility modes** for the notch and the dock, set separately: *Fixed* (default, always visible except in fullscreen), *Smart* (hides when a window covers it, shows when you point at it) and *Peek* (appears when the cursor reaches the screen edge). Both bars are shown for a few seconds after launch so you can see the app started.
- **Settings window** opened from the tray icon: visibility modes and language.
- **Tray icon** with *Settings* and *Quit*.
- **Single instance**: starting MacDows again does not open a second copy; it opens the Settings window of the running one.
- **Languages**: English and Turkish. Missing translations fall back to English.
- **Settings file** at `%APPDATA%\MacDows\settings.json`. A broken or empty file never stops the app: it starts with defaults and keeps the broken file as `settings.json.bad`.
- **Signed releases**: every release is built by GitHub Actions and the update packages are signed with the MacDows key, which is what the in-app update verifies.
- **MacDows logo**: a violet tile with a notch (equalizer bars) on top, a gradient "M" in the middle and a dock with three dots at the bottom. It replaces the placeholder icons in the tray, the window and the installer. Small versions (24 and 16 px) are simplified so they stay readable.
- **Branded installer** `MacDows-Setup.exe`: one window with a single Install button, a progress bar and an Open button, in English or Turkish. It installs MacDows for the current user (no administrator prompt) and installs WebView2 first if the PC does not have it (needs internet).
- Every release also carries `MacDows-Setup.exe` and a `SHA256SUMS.txt` to verify the download.
- **Updates inside the app**: MacDows looks for a new release 30 seconds after it starts and then every 12 hours, quietly. When one exists a small blue dot appears on the notch and Settings gets an **Updates** card with the new version, what is new, and an **Update and restart** button. Nothing is installed until you press it; the update is verified with the MacDows signing key and an update that fails the check is refused. You can switch the automatic check off or press **Check now** at any time.

### Known limitations
- Primary monitor only.
- The Windows taskbar stays visible. Hiding or replacing it is planned for a later version.
- The notch shows only the time and media; calendar, system monitor, clipboard history, launcher and notifications are planned.
- The tray menu language changes after a restart.
