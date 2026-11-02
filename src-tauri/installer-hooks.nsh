; Runs before the uninstall: give the Windows taskbar back and remove the guard copy.
!macro NSIS_HOOK_PREUNINSTALL
  ExecWait '"$INSTDIR\macdows.exe" --restore-taskbar'
  RMDir /r "$APPDATA\MacDows\guard"
!macroend
