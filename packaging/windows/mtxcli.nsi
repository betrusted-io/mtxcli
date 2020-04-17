; mtxcli.nsi
;
;--------------------------------

; The name of the installer
Name "Mtxcli"

; The file to write
OutFile "install_mtxcli.exe"

; Request application privileges for Windows Vista
RequestExecutionLevel admin

; Build Unicode installer
; Unicode True

; The default installation directory
InstallDir $PROGRAMFILES64\Mtxcli

; Registry key to check for directory (so if you install again, it will
; overwrite the old one automatically)
InstallDirRegKey HKLM "Software\Mtxcli" "Install_Dir"

;--------------------------------

; Pages

Page components
Page directory
Page instfiles

UninstPage uninstConfirm
UninstPage instfiles

;--------------------------------

; The stuff to install
Section "Mtxcli (required)"
  SectionIn RO

  ; Set output path to the installation directory.
  SetOutPath $INSTDIR

  ; Put file there
  File "mtxcli.exe"

  ; Write the installation path into the registry
  WriteRegStr HKLM SOFTWARE\Mtxcli "Install_Dir" "$INSTDIR"

  ; Write the uninstall keys for Windows
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mtxcli" "DisplayName" "Mtxcli"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mtxcli" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mtxcli" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mtxcli" "NoRepair" 1
  WriteUninstaller "$INSTDIR\uninstall.exe"

SectionEnd

; Optional section (can be disabled by the user)
Section "Start Menu Shortcuts"

  CreateDirectory "$SMPROGRAMS\Mtxcli"
  CreateShortcut "$SMPROGRAMS\Mtxcli\Uninstall.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0
  CreateShortcut "$SMPROGRAMS\Mtxcli\Mtxcli.lnk" 'C:\Windows\System32\cmd.exe' '/K "$INSTDIR\mtxcli.exe" --help' "$INSTDIR\mtxcli.exe" 0
  ; CreateShortcut "$SMPROGRAMS\Mtxcli\Mtxcli.lnk" "$INSTDIR\mtxcli.exe" "" "$INSTDIR\mtxcli.exe" 0

SectionEnd

;--------------------------------

; Uninstaller

Section "Uninstall"

  ; Remove registry keys
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mtxcli"
  DeleteRegKey HKLM SOFTWARE\Mtxcli

  ; Remove files and uninstaller
  Delete $INSTDIR\mtxcli.exe
  Delete $INSTDIR\uninstall.exe

  ; Remove shortcuts, if any
  Delete "$SMPROGRAMS\Mtxcli\*.*"

  ; Remove directories used
  RMDir "$SMPROGRAMS\Mtxcli"
  RMDir "$INSTDIR"

SectionEnd
