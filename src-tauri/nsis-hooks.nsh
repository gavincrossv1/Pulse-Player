; src-tauri/nsis-hooks.nsh
; 在 NSIS 安装完成阶段，将 libmpv-2.dll 复制到与 pulse-player.exe 同级的安装根目录下
; 并注册 Windows 规范应用程序能力模型（Capabilities、SupportedTypes、RegisteredApplications 与 ProgIDs）

!macro REGISTER_ASSOC_EXT EXT EXT_UPPER
  ; 登记 SupportedTypes
  WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".${EXT}" ""
  WriteRegStr HKCU "Software\Classes\Applications\pulse-player.exe\SupportedTypes" ".${EXT}" ""

  ; 登记 Capabilities\FileAssociations
  WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".${EXT}" "PulsePlayer.AssocFile.${EXT}"

  ; 登记标准 ProgID
  WriteRegStr HKCU "Software\Classes\PulsePlayer.AssocFile.${EXT}" "" "Pulse Player ${EXT_UPPER} 媒体文件"
  WriteRegStr HKCU "Software\Classes\PulsePlayer.AssocFile.${EXT}" "FriendlyTypeName" "Pulse Player ${EXT_UPPER} 媒体文件"
  WriteRegStr HKCU "Software\Classes\PulsePlayer.AssocFile.${EXT}\DefaultIcon" "" "$INSTDIR\pulse-player.exe,0"
  WriteRegStr HKCU "Software\Classes\PulsePlayer.AssocFile.${EXT}\shell\open\command" "" '"$INSTDIR\pulse-player.exe" "%1"'

  ; 登记 OpenWithProgids 与 OpenWithList
  WriteRegStr HKCU "Software\Classes\.${EXT}\OpenWithProgids" "PulsePlayer.AssocFile.${EXT}" ""
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.${EXT}\OpenWithProgids" "PulsePlayer.AssocFile.${EXT}" ""
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.${EXT}\OpenWithList" "pulse-player.exe" ""
!macroend

!macro UNREGISTER_ASSOC_EXT EXT
  DeleteRegKey HKCU "Software\Classes\PulsePlayer.AssocFile.${EXT}"
  DeleteRegValue HKCU "Software\Classes\.${EXT}\OpenWithProgids" "PulsePlayer.AssocFile.${EXT}"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.${EXT}\OpenWithProgids" "PulsePlayer.AssocFile.${EXT}"
!macroend

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
    ; 彻底清理打包解压的临时多级资源副本目录，释放冗余占用的磁盘空间
    RMDir /r "$INSTDIR\_up_"
    RMDir /r "$INSTDIR\resources"

    ; 1. 登记 Applications\pulse-player.exe (使 Windows 打开方式推荐列表识别)
    WriteRegStr HKCU "Software\Applications\pulse-player.exe" "FriendlyAppName" "Pulse Player"
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\DefaultIcon" "" "$INSTDIR\pulse-player.exe,0"
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\shell\open\command" "" '"$INSTDIR\pulse-player.exe" "%1"'

    WriteRegStr HKCU "Software\Classes\Applications\pulse-player.exe" "FriendlyAppName" "Pulse Player"
    WriteRegStr HKCU "Software\Classes\Applications\pulse-player.exe\DefaultIcon" "" "$INSTDIR\pulse-player.exe,0"
    WriteRegStr HKCU "Software\Classes\Applications\pulse-player.exe\shell\open\command" "" '"$INSTDIR\pulse-player.exe" "%1"'

    ; 2. 注册 Windows 规范 Capabilities 与 RegisteredApplications (使 Win11 设置“默认应用”可检索 Pulse Player)
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities" "ApplicationName" "Pulse Player"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities" "ApplicationDescription" "Pulse Player 本地极简离线媒体播放器"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities" "ApplicationIcon" "$INSTDIR\pulse-player.exe,0"
    WriteRegStr HKCU "Software\RegisteredApplications" "PulsePlayer" "Software\PulsePlayer\Capabilities"

    ; 3. 登记所有 19 种主流媒体格式能力、ProgID 及 OpenWithProgids
    !insertmacro REGISTER_ASSOC_EXT "mp4" "MP4"
    !insertmacro REGISTER_ASSOC_EXT "mkv" "MKV"
    !insertmacro REGISTER_ASSOC_EXT "avi" "AVI"
    !insertmacro REGISTER_ASSOC_EXT "flv" "FLV"
    !insertmacro REGISTER_ASSOC_EXT "mov" "MOV"
    !insertmacro REGISTER_ASSOC_EXT "wmv" "WMV"
    !insertmacro REGISTER_ASSOC_EXT "webm" "WEBM"
    !insertmacro REGISTER_ASSOC_EXT "ts" "TS"
    !insertmacro REGISTER_ASSOC_EXT "m4v" "M4V"
    !insertmacro REGISTER_ASSOC_EXT "rmvb" "RMVB"
    !insertmacro REGISTER_ASSOC_EXT "mp3" "MP3"
    !insertmacro REGISTER_ASSOC_EXT "flac" "FLAC"
    !insertmacro REGISTER_ASSOC_EXT "wav" "WAV"
    !insertmacro REGISTER_ASSOC_EXT "m4a" "M4A"
    !insertmacro REGISTER_ASSOC_EXT "aac" "AAC"
    !insertmacro REGISTER_ASSOC_EXT "ogg" "OGG"
    !insertmacro REGISTER_ASSOC_EXT "ape" "APE"
    !insertmacro REGISTER_ASSOC_EXT "opus" "OPUS"
    !insertmacro REGISTER_ASSOC_EXT "wma" "WMA"

    ; 广播外壳刷新
    System::Call 'Shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\libmpv-2.dll"
  RMDir /r "$INSTDIR\_up_"
  RMDir /r "$INSTDIR\resources"
  DeleteRegKey HKCU "Software\Applications\pulse-player.exe"
  DeleteRegKey HKCU "Software\Classes\Applications\pulse-player.exe"
  DeleteRegKey HKCR "Applications\pulse-player.exe"
  DeleteRegKey HKCU "Software\PulsePlayer"
  DeleteRegValue HKCU "Software\RegisteredApplications" "PulsePlayer"

  ; 彻底清理全部 19 种 ProgID 及 OpenWith 登记，杜绝注册表残留
  !insertmacro UNREGISTER_ASSOC_EXT "mp4"
  !insertmacro UNREGISTER_ASSOC_EXT "mkv"
  !insertmacro UNREGISTER_ASSOC_EXT "avi"
  !insertmacro UNREGISTER_ASSOC_EXT "flv"
  !insertmacro UNREGISTER_ASSOC_EXT "mov"
  !insertmacro UNREGISTER_ASSOC_EXT "wmv"
  !insertmacro UNREGISTER_ASSOC_EXT "webm"
  !insertmacro UNREGISTER_ASSOC_EXT "ts"
  !insertmacro UNREGISTER_ASSOC_EXT "m4v"
  !insertmacro UNREGISTER_ASSOC_EXT "rmvb"
  !insertmacro UNREGISTER_ASSOC_EXT "mp3"
  !insertmacro UNREGISTER_ASSOC_EXT "flac"
  !insertmacro UNREGISTER_ASSOC_EXT "wav"
  !insertmacro UNREGISTER_ASSOC_EXT "m4a"
  !insertmacro UNREGISTER_ASSOC_EXT "aac"
  !insertmacro UNREGISTER_ASSOC_EXT "ogg"
  !insertmacro UNREGISTER_ASSOC_EXT "ape"
  !insertmacro UNREGISTER_ASSOC_EXT "opus"
  !insertmacro UNREGISTER_ASSOC_EXT "wma"

  System::Call 'Shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend
