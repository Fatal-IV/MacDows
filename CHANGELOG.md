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
- **Languages**: English and Turkish. Missing translations fall back to English.
- **Settings file** at `%APPDATA%\MacDows\settings.json`. A broken or empty file never stops the app: it starts with defaults and keeps the broken file as `settings.json.bad`.
- **Automatic updates** from GitHub Releases, with signed update packages.

### Known limitations
- Primary monitor only.
- The Windows taskbar stays visible. Hiding or replacing it is planned for a later version.
- The notch shows only the time and media; calendar, system monitor, clipboard history, launcher and notifications are planned.
- The tray menu language changes after a restart.
