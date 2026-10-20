; src-tauri/nsis-hooks.nsh
; 在 NSIS 安装完成阶段，将 libmpv-2.dll 复制到与 pulse-player.exe 同级的安装根目录下
; 注意：NSIS 的 CopyFiles 命令目标参数必须为目录路径（$INSTDIR），而非文件名
!macro NSIS_HOOK_POSTINSTALL
  IfFileExists "$INSTDIR\libmpv-2.dll" done_copy_mpv 0
  IfFileExists "$INSTDIR\_up_\libs\mpv\libmpv-2.dll" 0 +3
    CopyFiles /SILENT "$INSTDIR\_up_\libs\mpv\libmpv-2.dll" "$INSTDIR"
    Goto done_copy_mpv
  IfFileExists "$INSTDIR\resources\libmpv-2.dll" 0 +3
    CopyFiles /SILENT "$INSTDIR\resources\libmpv-2.dll" "$INSTDIR"
    Goto done_copy_mpv
  IfFileExists "$INSTDIR\resources\_up_\libs\mpv\libmpv-2.dll" 0 +3
    CopyFiles /SILENT "$INSTDIR\resources\_up_\libs\mpv\libmpv-2.dll" "$INSTDIR"
    Goto done_copy_mpv
  IfFileExists "$INSTDIR\resources\libs\mpv\libmpv-2.dll" 0 done_copy_mpv
    CopyFiles /SILENT "$INSTDIR\resources\libs\mpv\libmpv-2.dll" "$INSTDIR"
  done_copy_mpv:
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\libmpv-2.dll"
!macroend

