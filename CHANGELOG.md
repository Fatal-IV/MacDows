# Changelog

All notable changes to MacDows are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-11-05


### Added
- Control Center, Wi-Fi: the arrow on the Wi-Fi tile opens a list of networks by signal strength. Connect to saved and open networks, join new ones with a password typed right in the island (also hidden networks), disconnect, and forget a network. A wrong password says so and brings the previous network back. Company (802.1X) networks open Windows settings. Settings > Dynamic Island > Quick settings > "Wi-Fi page".
- Control Center, VPN: a VPN card shows Windows VPN connections (switch one on or off with a click, or open the list for several) and tells when a VPN app (WireGuard, OpenVPN and the like) has a tunnel up. Only stored sign-ins are used.
- Control Center, Bluetooth: "Add a device" scans for nearby devices and pairs them inside the island (a plain yes, a code to compare, or a PIN to type), and paired devices can be removed. When the setting is off, "Add a device" opens Windows settings as before.
- While a text field in the island has the keyboard, the island stays open even if the pointer leaves it, and closes when typing ends.
- Status widgets on the island: weather, battery, CPU, memory, disk and network speed sit on either side of the clock while the pointer is over the island (not while the player is open). Settings > Dynamic Island > Status has a pill preview where widgets are dragged between the left area, the right area and the "Available" tray (up to 3 per side; Alt+Left/Right moves a focused widget between areas, Alt+Up/Down reorders). The numbers are only sampled while the island is hovered and a CPU, memory, disk or network widget is placed, every 2 seconds. Weather comes from Open-Meteo for the city chosen in settings (temperature in Celsius or Fahrenheit); only the city name and its coordinates are sent, and with no city nothing is sent at all. The battery widget turns red below a chosen percentage (default 20%) unless the charger is connected, and hides itself on computers without a battery. The old "battery percentage" setting becomes the battery widget (switched off there, it stays off).

### Changed
- Settings window: the round button that fanned the categories out is gone; the categories (General, Dynamic Island, Dock, Appearance, Updates, About) are a fixed list on the left.
- Settings > Dynamic Island: each screen's card (Status, Music, Quick settings) starts folded and opens with a click on its header (a chevron shows which ones have more settings); the on/off switch does not open or close it.
- The dock's right-click menu for a running app now shows the window's own Windows menu (Restore, Move, Size, Minimize, Maximize, Close) under the dock's items, and "Quit app" is gone in favor of Windows's Close.
- Pinning and unpinning in the dock no longer touch the Windows taskbar: "Keep in Dock" adds a pin only MacDows keeps (or shows a hidden one again), "Remove from Dock" hides a Windows pin in the dock only. The dock still reads the taskbar's pins to start with.

### Fixed
- Wi-Fi: networks that only accept WPA3 can be joined with a typed password (the list now carries each network's security type). A slow network is reported as "No answer from the network" instead of "Wrong password": a wrong password is now recognized when Windows tries and gives up. A failed attempt puts the network's previously saved profile back instead of leaving the failed one, and the wait no longer rescans the whole network list every half second.
- VPN: certificate (IKEv2) and EAP profiles, which have no stored password, can connect from the island. The adapter list is read from properly aligned memory, and Windows's IP-HTTPS adapter no longer makes the tile read as "VPN on".
- Bluetooth pairing: "rejected" is shown only when the device or you declined (canceled or rejected by handler); timeouts and other failures say "failed". A paired device that gets its name during scanning no longer shows up in the list of devices to pair.
- Dock menu on a running app: Move, Size, Minimize and the other window commands bring the window forward first, so they work on background windows.
- Typing in an island text field no longer dismisses a notification banner that is still showing when the field closes.
- Shutting down Windows while MacDows runs no longer shows two error windows (taskkill failing with 0xc0000142) whose dismissal cancelled the shutdown: MacDows skips restarting Start and Search while Windows is shutting down or logging off.
- Notifications cleared from the island's history stay cleared: they used to come back at the next refresh or restart because Windows still listed them.

### Known limitations
- The new Wi-Fi, VPN and Bluetooth pages were tried on one PC only; WPA3-only networks, certificate and EAP VPN profiles and several Bluetooth pairing kinds have unit tests but were not tried against real hardware.
- Primary monitor only.
- The hidden-icons button of the dock may not open its panel on some systems (it needs a Windows Explorer restart).
- The installer is not signed with a code-signing certificate, so Windows SmartScreen may show a warning.

## [0.1.0] - 2026-11-04

First public release. A Dynamic Island at the top of the screen and a dock at the bottom that take the place of the Windows taskbar (it comes back whenever you quit MacDows or turn the switch off). Windows's own Start, Search and quick settings stay.

### Added
- The dock shows the apps pinned to the Windows taskbar (programs, File Explorer and Store apps) in Windows's order, and follows Windows when something is pinned or unpinned. A running pinned app keeps its place and gets a dot; a click brings it forward or starts it. Settings > Dock > "Show pinned apps".
- Right-click menu in the dock: new window, unpin from the taskbar, pin to the taskbar, change or reset the icon, quit the app, and (on the empty dock) add app, MacDows settings and Task Manager. Windows often refuses to pin for other programs; then MacDows keeps the pin itself.
- Drag the pinned apps to reorder them (only the order in the dock changes, not the Windows taskbar), and give any dock app a custom icon from a picture or a program.
- Window previews: hover a running app to see its windows as small live pictures; click one to bring it forward or close it. A badge shows the window count from two windows. Can be turned off in settings.
- Drives (local disks and USB drives) in the dock; a click opens them in File Explorer. Can be turned off in settings.
- Win+1 to Win+9 start or focus the pinned apps in dock order, Win+Shift+number opens a new window. Windows does not let programs register these keys, so MacDows uses a keyboard hook that only looks at Win and the digits; an empty slot is left to Windows. Can be turned off in settings.
- "Add app" in the dock's right-click menu: a searchable list of installed programs and Store apps; a click pins one.
- The Bluetooth tile of the quick settings shows the name of the connected device ("Name +1" for more).
- A dock button that jumps to the desktop (Win+D). Off by default; Settings > Dock.
- The app follows the light or dark theme of Windows by default, and can be set to light or dark.
- The installer asks whether MacDows should start with Windows (a fresh install only; off by default).
- Windows's own Start and Search are used, as they are on a normal PC (the Start and Search buttons of the dock open them, and so do the Windows key and Win+S). While one of Windows's menus (Start, Search, quick settings) is open, the dock stays in view under it.
- Settings > Appearance > "Paint the Windows Start menu in the theme color". Off by default: MacDows then only makes Windows's system theme light or dark like its own. On, Start also takes the theme color; Windows only allows that through the system accent color, so every app's accent color is the theme color while MacDows runs. Your own colors come back when MacDows closes, whichever way it ends.
- The music bars in the notch follow the real audio the PC is playing (five frequency bands, analysed locally; nothing is recorded or sent anywhere).
- The notch player shows the elapsed and remaining time with a bar you can click to seek, and a system volume slider (the mouse wheel over it works too).
- **Dynamic Island settings**: a page of its own in Settings. Every screen (Status, Music, Calendar and timer, Quick settings, Notifications, Shelf) can be switched off, but at least one stays on. Choose what the wheel, a click and the middle button do, the order of the screens, whether the island returns to the last screen, how it behaves in fullscreen apps (edge or never) and its size. Old settings are carried over.
- **Hover opens the player**: while something is playing (or paused) the island opens the player with its controls as soon as the pointer rests on it. Can be turned off.
- **Media source buttons**: when two or more programs have media (Spotify, a browser tab, a video player) the player shows a button with the icon and name of each; the paused ones fade. Click one to see and control it.
- **Settings are split into pages**: General, Dynamic Island, Dock (including "Replace the Windows taskbar"), Appearance, Updates and About.
- **Dock** at the bottom center, above the taskbar: one icon per open app window, click to bring it to the front, icons grow when you hover them, a dot marks each running app. The dock has a black, edge-attached shape with curved sides.
- **Stable dock order**: icons stay where they first appeared, new apps are added at the end, closed apps disappear.
- **Store app icons**: apps installed from the Microsoft Store, such as WhatsApp, now show their real icon instead of a blank one.
- **Fullscreen detection**: the notch and dock hide automatically while a fullscreen app is in front (a fullscreen video or a game) and come back when you leave fullscreen. Works event-based, without polling.
- **Visibility modes** for the notch and the dock, set separately: *Fixed* (default, always visible except in fullscreen), *Smart* (hides when a window covers it, shows when you point at it) and *Peek* (appears when the cursor reaches the screen edge). Both bars are shown for a few seconds after launch so you can see the app started.
- **Single instance**: starting MacDows again does not open a second copy; it opens the Settings window of the running one.
- **Languages**: English and Turkish. Missing translations fall back to English.
- **Settings file** at `%APPDATA%\MacDows\settings.json`. A broken or empty file never stops the app: it starts with defaults and keeps the broken file as `settings.json.bad`.
- **Signed releases**: every release is built by GitHub Actions and the update packages are signed with the MacDows key, which is what the in-app update verifies.
- **MacDows logo**: a violet tile with a notch (equalizer bars) on top, a gradient "M" in the middle and a dock with three dots at the bottom. It replaces the placeholder icons in the tray, the window and the installer. Small versions (24 and 16 px) are simplified so they stay readable.
- **Branded installer** `MacDows-Setup.exe`: one window with a single Install button, a progress bar and an Open button, in English or Turkish. It installs MacDows for the current user (no administrator prompt) and installs WebView2 first if the PC does not have it (needs internet).
- Every release also carries `MacDows-Setup.exe` and a `SHA256SUMS.txt` to verify the download.
- **Updates inside the app**: MacDows looks for a new release 30 seconds after it starts and then every 12 hours, quietly. When one exists a small blue dot appears on the notch and Settings gets an **Updates** card with the new version, what is new, and an **Update and restart** button. Nothing is installed until you press it; the update is verified with the MacDows signing key and an update that fails the check is refused. You can switch the automatic check off or press **Check now** at any time.

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

### Known limitations
- Primary monitor only.
- The Windows notification banner still appears next to the island's own.
- The hidden-icons button of the dock may not open its panel on some systems (it needs a Windows Explorer restart).
- Wi-Fi network switching, Bluetooth pairing and VPN inside the island are planned; for now the quick settings only toggle them, and "Add device" opens Windows settings.
- Not every part of the island has its own switch yet (clock format, music extras, banner time and so on are planned).
- The installer is not signed with a code-signing certificate, so Windows SmartScreen may show a warning.
