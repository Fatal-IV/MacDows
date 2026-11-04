# Changelog

All notable changes to MacDows are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [Unreleased]

### Added
- The dock shows the apps pinned to the Windows taskbar (programs, File Explorer and Store apps) in Windows's order, and follows Windows when something is pinned or unpinned. A running pinned app keeps its place and gets a dot; a click brings it forward or starts it. Settings > Dynamic Island and dock > "Show pinned apps".
- Right-click menu in the dock: new window, unpin from the taskbar, pin to the taskbar, change or reset the icon, quit the app, and (on the empty dock) add app, MacDows settings and Task Manager. Windows often refuses to pin for other programs; then MacDows keeps the pin itself.
- Drag the pinned apps to reorder them (only the order in the dock changes, not the Windows taskbar), and give any dock app a custom icon from a picture or a program.
- Window previews: hover a running app to see its windows as small live pictures; click one to bring it forward or close it. A badge shows the window count from two windows. Can be turned off in settings.
- Drives (local disks and USB drives) in the dock; a click opens them in File Explorer. Can be turned off in settings.
- Win+1 to Win+9 start or focus the pinned apps in dock order, Win+Shift+number opens a new window. Windows does not let programs register these keys, so MacDows uses a keyboard hook that only looks at Win and the digits; an empty slot is left to Windows. Can be turned off in settings.
- "Add app" in the dock's right-click menu: a searchable list of installed programs and Store apps; a click pins one.
- The Bluetooth tile of the quick settings shows the name of the connected device ("Name +1" for more).
- A dock button that jumps to the desktop (Win+D). Off by default; Settings > Dynamic Island and dock.
- The app follows the light or dark theme of Windows by default, and can be set to light or dark.
- The installer asks whether MacDows should start with Windows (a fresh install only; off by default).
- Windows's own Start and Search are used, as they are on a normal PC (the Start and Search buttons of the dock open them, and so do the Windows key and Win+S). While one of Windows's menus (Start, Search, quick settings) is open, the dock stays in view under it.
- Settings > Appearance > "Paint the Windows Start menu in the theme color". Off by default: MacDows then only makes Windows's system theme light or dark like its own. On, Start also takes the theme color; Windows only allows that through the system accent color, so every app's accent color is the theme color while MacDows runs. Your own colors come back when MacDows closes, whichever way it ends.
- The music bars in the notch follow the real audio the PC is playing (five frequency bands, analysed locally; nothing is recorded or sent anywhere).
- The notch player shows the elapsed and remaining time with a bar you can click to seek, and a system volume slider (the mouse wheel over it works too).

### Changed
- The notch is now called the Dynamic Island in the app, the installer and the settings.
- "Start with Windows" is off by default. Matching the color of Windows's Start and Search to the theme is on by default.
- The Settings window has its own title bar, a left menu with categories and its own controls.
- The notch now follows Nectar's design: one damped spring for width, height and corners, concave corners that join it to the screen edge, a slide-in at startup, a status pill and a music pill you switch with the mouse wheel or a click on the clock, a music pill that opens itself when playback starts, a full player with artwork, a scrolling title and an album-colored glow.
- **License changed from MIT to GPL-3.0**, because the notch is adapted from Nectar (GPL-3.0). See `NOTICE`.
- Turkish interface text now addresses you formally ("siz").

### Fixed
- The battery of a Bluetooth headset could show an old value: the same headset has several device entries in Windows and the stale one could win. The freshest reading is used now.
- The browser's own right-click menu ("Save image as", "Inspect") no longer shows on the dock and the Dynamic Island.
- In a fullscreen app the notch and dock no longer disappear completely; they appear when the pointer reaches the screen edge.
- Settings could open behind other windows and be unreachable while the taskbar was replaced; it now comes to the front.

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
