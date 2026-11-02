; Runs before the uninstall: give the Windows taskbar back, remove the guard copy and the autostart entry.
!macro NSIS_HOOK_PREUNINSTALL
  ExecWait '"$INSTDIR\macdows.exe" --restore-taskbar'
  RMDir /r "$APPDATA\MacDows\guard"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "MacDows"
!macroend
