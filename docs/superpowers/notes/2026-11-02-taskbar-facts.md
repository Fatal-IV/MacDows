# Taskbar takeover facts verified on a real PC (2026-11-02, Windows 11, one monitor)

Throwaway scripts (not in the repo) hid the taskbar and put it back after every step.

## Hiding
- Baseline: `Shell_TrayWnd` rect 0,1020 to 1920,1080 (60 px), work area 1920x1020, appbar state 0 (auto-hide off).
- Hide the window only (`ShowWindow(SW_HIDE)`): the taskbar is gone, but the **work area stays 1020**.
- Auto-hide only (`SHAppBarMessage` `ABM_SETSTATE` with `ABS_AUTOHIDE`): the work area becomes **1080** at once, but the taskbar **stayed on screen** while two buttons were flashing (Windows does not hide a taskbar that needs attention).
- **Both together** (auto-hide, then `SW_HIDE`): taskbar not visible, work area 1080, still hidden 8 s later; maximized windows grew into the freed space. Restoring (`SW_SHOW`, then the original `ABM_SETSTATE`) brought back the taskbar and the 1020 work area.
- With the taskbar hidden by `SW_HIDE`, UI Automation sees an **empty tree** (0 elements) under `Shell_TrayWnd`.
- With the taskbar visible, UI Automation reads the tray: items have `AutomationId` `SystemTrayIcon` and names such as the Wi-Fi network, volume ("Kulaklıklar: 100%"), battery, clock, input language, microphone indicator, and the "show hidden icons" chevron.

## Flyouts while the taskbar is hidden (screenshots checked)
- **Win+A** opens quick settings (Wi-Fi, Bluetooth, brightness, volume, battery) at the bottom right.
- **Win+N** opens the notification center with the calendar at the bottom right.
- **Win+B, then Enter** opens the hidden-icons popup at the bottom right.
- Pressing the same shortcut again closes Win+A / Win+N. Esc must NOT be sent blindly (it would reach whatever window has focus).

## Not verified
Explorer restart (the taskbar is recreated), a second monitor (`Shell_SecondaryTrayWnd`), Win+B behavior when the chevron is hidden by the user's settings (Enter would then press the first tray icon), whether the Windows Start menu position follows the freed work area, behavior after a reboot while auto-hide is on.
