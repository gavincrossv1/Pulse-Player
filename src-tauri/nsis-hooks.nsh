; src-tauri/nsis-hooks.nsh
; 在 NSIS 安装完成阶段，将 libmpv-2.dll 复制到与 pulse-player.exe 同级的安装根目录下
; 并注册 Windows 规范应用程序能力模型（Capabilities、SupportedTypes、RegisteredApplications）
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

    WriteRegStr HKCR "Applications\pulse-player.exe" "FriendlyAppName" "Pulse Player"
    WriteRegStr HKCR "Applications\pulse-player.exe\DefaultIcon" "" "$INSTDIR\pulse-player.exe,0"
    WriteRegStr HKCR "Applications\pulse-player.exe\shell\open\command" "" '"$INSTDIR\pulse-player.exe" "%1"'

    ; 2. 登记 SupportedTypes 媒体格式扩展名
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".mp4" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".mkv" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".avi" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".flv" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".mov" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".wmv" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".webm" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".ts" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".m4v" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".rmvb" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".mp3" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".flac" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".wav" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".m4a" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".aac" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".ogg" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".ape" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".opus" ""
    WriteRegStr HKCU "Software\Applications\pulse-player.exe\SupportedTypes" ".wma" ""

    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".mp4" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".mkv" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".avi" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".flv" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".mov" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".wmv" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".webm" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".ts" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".m4v" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".rmvb" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".mp3" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".flac" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".wav" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".m4a" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".aac" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".ogg" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".ape" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".opus" ""
    WriteRegStr HKCR "Applications\pulse-player.exe\SupportedTypes" ".wma" ""

    ; 3. 注册 Windows 规范 Capabilities 与 RegisteredApplications (使 Win11 设置“默认应用”可检索 Pulse Player)
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities" "ApplicationName" "Pulse Player"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities" "ApplicationDescription" "Pulse Player 本地极简离线媒体播放器"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities" "ApplicationIcon" "$INSTDIR\pulse-player.exe,0"
    WriteRegStr HKCU "Software\RegisteredApplications" "PulsePlayer" "Software\PulsePlayer\Capabilities"

    ; 4. 默认登记 Capabilities\FileAssociations
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".mp4" "PulsePlayer.AssocFile.mp4"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".mkv" "PulsePlayer.AssocFile.mkv"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".avi" "PulsePlayer.AssocFile.avi"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".flv" "PulsePlayer.AssocFile.flv"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".mov" "PulsePlayer.AssocFile.mov"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".wmv" "PulsePlayer.AssocFile.wmv"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".webm" "PulsePlayer.AssocFile.webm"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".ts" "PulsePlayer.AssocFile.ts"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".m4v" "PulsePlayer.AssocFile.m4v"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".rmvb" "PulsePlayer.AssocFile.rmvb"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".mp3" "PulsePlayer.AssocFile.mp3"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".flac" "PulsePlayer.AssocFile.flac"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".wav" "PulsePlayer.AssocFile.wav"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".m4a" "PulsePlayer.AssocFile.m4a"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".aac" "PulsePlayer.AssocFile.aac"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".ogg" "PulsePlayer.AssocFile.ogg"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".ape" "PulsePlayer.AssocFile.ape"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".opus" "PulsePlayer.AssocFile.opus"
    WriteRegStr HKCU "Software\PulsePlayer\Capabilities\FileAssociations" ".wma" "PulsePlayer.AssocFile.wma"

    ; 广播外壳刷新
    System::Call 'Shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\libmpv-2.dll"
  RMDir /r "$INSTDIR\_up_"
  RMDir /r "$INSTDIR\resources"
  DeleteRegKey HKCU "Software\Applications\pulse-player.exe"
  DeleteRegKey HKCR "Applications\pulse-player.exe"
  DeleteRegKey HKCU "Software\PulsePlayer"
  DeleteRegValue HKCU "Software\RegisteredApplications" "PulsePlayer"
  System::Call 'Shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend
