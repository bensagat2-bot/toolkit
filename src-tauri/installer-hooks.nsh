; NSIS installer hooks (referenced by bundle.windows.nsis.installerHooks)
; Kill processes that lock the app's own files (the exe, bundled platform-tools
; adb/fastboot, scrcpy and Unisoc spd_dump) so the installer can overwrite them
; instead of showing "error while opening file for writing ... abort/retry".

!macro NSIS_HOOK_PREINSTALL
  nsExec::ExecToLog 'taskkill /F /IM "V1Per Servicing Toolkit.exe"'
  nsExec::ExecToLog 'taskkill /F /IM "adb.exe"'
  nsExec::ExecToLog 'taskkill /F /IM "fastboot.exe"'
  nsExec::ExecToLog 'taskkill /F /IM "scrcpy.exe"'
  nsExec::ExecToLog 'taskkill /F /IM "spd_dump.exe"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'taskkill /F /IM "V1Per Servicing Toolkit.exe"'
  nsExec::ExecToLog 'taskkill /F /IM "adb.exe"'
  nsExec::ExecToLog 'taskkill /F /IM "fastboot.exe"'
  nsExec::ExecToLog 'taskkill /F /IM "scrcpy.exe"'
  nsExec::ExecToLog 'taskkill /F /IM "spd_dump.exe"'
!macroend