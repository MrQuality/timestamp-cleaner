Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
Name "Timestamp Cleaner ${VERSION}"
OutFile "${OUTPUT}"
RequestExecutionLevel user
SetCompressor /SOLID zlib
SetOverwrite off
Icon "${ICON}"
UninstallIcon "${ICON}"
VIProductVersion "${PE_VERSION}"
VIAddVersionKey /LANG=1033 "ProductName" "Timestamp Cleaner"
VIAddVersionKey /LANG=1033 "FileDescription" "Timestamp Cleaner per-user setup"
VIAddVersionKey /LANG=1033 "FileVersion" "${VERSION}"
VIAddVersionKey /LANG=1033 "LegalCopyright" "Timestamp Cleaner contributors"
!define KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\TimestampCleaner-${VERSION}"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${PAYLOAD}\LICENSE"
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

!macro Initialize
  SetShellVarContext current
  SetRegView 64
  ${IfNot} ${RunningX64}
    MessageBox MB_OK|MB_ICONSTOP "This package requires 64-bit Windows." /SD IDOK
    SetErrorLevel 2
    Abort
  ${EndIf}
  ; The destination is fixed: neither registry data nor /D controls deletion paths.
  StrCpy $INSTDIR "$LOCALAPPDATA\Programs\TimestampCleaner\${VERSION}"
  System::Call 'kernel32::CreateMutexW(p 0, i 0, w "Local\TimestampCleanerSetup-${VERSION}") p .r0 ?e'
  Pop $1
  ${If} $0 == 0
  ${OrIf} $1 == 183
    MessageBox MB_OK|MB_ICONSTOP "Another setup or uninstall is active. Close it and retry." /SD IDOK
    SetErrorLevel 2
    Abort
  ${EndIf}
!macroend

Function .onInit
  !insertmacro Initialize
  IfFileExists "$INSTDIR\*.*" 0 ready
    MessageBox MB_OK|MB_ICONSTOP "This version's folder already exists. Uninstall this version and move any remaining personal files before reinstalling." /SD IDOK
    SetErrorLevel 2
    Abort
  ready:
FunctionEnd

Function un.onInit
  ; NSIS initializes INSTDIR to the original folder before its temporary copy runs.
  StrCpy $2 $INSTDIR
  !insertmacro Initialize
  StrCmp $2 $INSTDIR location_ok
    MessageBox MB_OK|MB_ICONSTOP "Run the uninstaller from its original installation folder." /SD IDOK
    SetErrorLevel 2
    Abort
  location_ok:
  IfFileExists "$INSTDIR\timestamp-cleaner.exe" 0 ready
  ClearErrors
  FileOpen $0 "$INSTDIR\timestamp-cleaner.exe" a
  ${If} ${Errors}
    MessageBox MB_OK|MB_ICONSTOP "Close Timestamp Cleaner before uninstalling." /SD IDOK
    SetErrorLevel 2
    Abort
  ${EndIf}
  FileClose $0
  ready:
FunctionEnd

Section "Install"
  ClearErrors
  !include "${INSTALL_FILES}"
  SetOutPath "$INSTDIR"
  ClearErrors
  WriteUninstaller "$INSTDIR\uninstall.exe"
  CreateShortcut "$SMPROGRAMS\Timestamp Cleaner ${VERSION}.lnk" "$INSTDIR\timestamp-cleaner.exe"
  WriteRegStr HKCU "${KEY}" "DisplayName" "Timestamp Cleaner ${VERSION}"
  WriteRegStr HKCU "${KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${KEY}" "Publisher" "Timestamp Cleaner contributors"
  WriteRegStr HKCU "${KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${KEY}" "DisplayIcon" "$INSTDIR\timestamp-cleaner.exe"
  WriteRegStr HKCU "${KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${KEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegStr HKCU "${KEY}" "URLInfoAbout" "https://github.com/MrQuality/timestamp-cleaner"
  WriteRegDWORD HKCU "${KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${KEY}" "NoRepair" 1
  ${If} ${Errors}
    MessageBox MB_OK|MB_ICONSTOP "Setup could not finish. Check folder permissions and use uninstall.exe to remove the partial installation before retrying." /SD IDOK
    SetErrorLevel 2
    Abort
  ${EndIf}
SectionEnd

Section "Uninstall"
  ClearErrors
  !include "${UNINSTALL_FILES}"
  ${If} ${Errors}
    MessageBox MB_OK|MB_ICONSTOP "Some application files could not be removed. Close programs using them and retry uninstalling." /SD IDOK
    SetErrorLevel 2
    Abort
  ${EndIf}
  DeleteRegKey HKCU "${KEY}"
  Delete "$SMPROGRAMS\Timestamp Cleaner ${VERSION}.lnk"
  Delete "$INSTDIR\uninstall.exe"
  SetOutPath "$TEMP"
  !include "${REMOVE_DIRS}"
  ; Empty directories only: never recursively remove files added by the user.
  RMDir "$INSTDIR"
SectionEnd
