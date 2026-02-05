!include "MUI2.nsh"
!include "LogicLib.nsh"

; Helper macro for StrStr
!include "StrFunc.nsh"
${StrStr}

Name "MGIT"
OutFile "mgit-setup.exe"
InstallDir "$PROGRAMFILES64\Mgit"
InstallDirRegKey HKCU "Software\Mgit" ""
RequestExecutionLevel admin

!define MUI_ABORTWARNING
!define MUI_ICON "..\..\mgit-gui\resource\logo64x64.ico"
!define MUI_UNICON "..\..\mgit-gui\resource\logo64x64.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"

Section "MGIT (required)" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"

  ; Copy files
  File "..\..\target\release\mgit.exe"
  File "..\..\target\release\mgit-gui.exe"

  ; Write Uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Registry for Add/Remove programs
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mgit" "DisplayName" "MGIT"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mgit" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mgit" "Publisher" "SoFunny"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mgit" "DisplayIcon" "$INSTDIR\mgit-gui.exe"

  ; Add to PATH
  ReadRegStr $0 HKCU "Environment" "PATH"
  ${If} $0 == ""
    StrCpy $0 "$INSTDIR"
  ${Else}
    ; Check if already in PATH to avoid duplication
    ${StrStr} $1 $0 "$INSTDIR"
    ${If} $1 == ""
      StrCpy $0 "$0;$INSTDIR"
    ${EndIf}
  ${EndIf}
  WriteRegStr HKCU "Environment" "PATH" $0
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd

Section "Start Menu Shortcuts"
  CreateDirectory "$SMPROGRAMS\Mgit"
  CreateShortcut "$SMPROGRAMS\Mgit\MGIT GUI.lnk" "$INSTDIR\mgit-gui.exe" "" "$INSTDIR\mgit-gui.exe" 0
  CreateShortcut "$SMPROGRAMS\Mgit\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Desktop Shortcut"
  CreateShortcut "$DESKTOP\MGIT GUI.lnk" "$INSTDIR\mgit-gui.exe" "" "$INSTDIR\mgit-gui.exe" 0
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\mgit.exe"
  Delete "$INSTDIR\mgit-gui.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\Mgit\MGIT GUI.lnk"
  Delete "$SMPROGRAMS\Mgit\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Mgit"
  Delete "$DESKTOP\MGIT GUI.lnk"

  DeleteRegKey HKCU "Software\Mgit"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Mgit"

  ; Note: Removing from PATH is omitted for safety
SectionEnd
