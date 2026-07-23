; NSIS installer for the WCH BLE Analyzer Pro extcap plugin.
;   makensis -DVERSION=<version> packaging/windows/installer.nsi
;
; Two destinations, deliberately:
;   $INSTDIR             our own directory -- LICENSE, README, docs, uninstaller
;   $WiresharkExtcapDir  the plugin .exe, which must live where Wireshark looks
; Dropping documentation into Wireshark's own install tree would leave litter
; behind that a Wireshark upgrade or removal has no idea about.

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"

!ifndef VERSION
    !define VERSION "0.0.0"
!endif

!define PKG_NAME "wch-ble-extcap"
!define DISPLAY_NAME "WCH BLE Analyzer Pro extcap plugin"
!define PUBLISHER "DC0SK"
!define UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PKG_NAME}"

Name "${DISPLAY_NAME} ${VERSION}"
OutFile "..\..\dist\${PKG_NAME}-${VERSION}-windows-x64-setup.exe"
InstallDir "$PROGRAMFILES64\WCH BLE Extcap"
RequestExecutionLevel admin          ; needed to write to Program Files
Unicode true

Var WiresharkExtcapDir

!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

; Locate Wireshark via its registry key, falling back to the default path.
Function ResolveWireshark
    ReadRegStr $0 HKLM "Software\Wireshark" "InstallDir"
    ${If} $0 == ""
        StrCpy $0 "$PROGRAMFILES64\Wireshark"
    ${EndIf}
    StrCpy $WiresharkExtcapDir "$0\extcap"
FunctionEnd

Function .onInit
    Call ResolveWireshark
    ${IfNot} ${FileExists} "$WiresharkExtcapDir\*.*"
        MessageBox MB_YESNO|MB_ICONEXCLAMATION \
            "Wireshark's extcap directory was not found at:$\n$\n\
            $WiresharkExtcapDir$\n$\n\
            Install anyway? The plugin will not appear in Wireshark until it \
            is copied into the correct extcap directory." IDYES +2
        Abort
    ${EndIf}
FunctionEnd

Section "Install"
    ; Documentation and uninstaller in our own directory.
    SetOutPath "$INSTDIR"
    File "..\..\LICENSE"
    File "..\..\README.md"
    SetOutPath "$INSTDIR\docs"
    File /r "..\..\docs\*.*"

    ; The plugin itself, where Wireshark will actually find it.
    CreateDirectory "$WiresharkExtcapDir"
    SetOutPath "$WiresharkExtcapDir"
    File "..\..\target\x86_64-pc-windows-gnu\release\${PKG_NAME}.exe"

    ; Remember the resolved path so the uninstaller removes the right file.
    WriteRegStr HKLM "${UNINST_KEY}" "ExtcapDir" "$WiresharkExtcapDir"

    CreateDirectory "$SMPROGRAMS\${DISPLAY_NAME}"
    CreateShortcut "$SMPROGRAMS\${DISPLAY_NAME}\README.lnk" "$INSTDIR\README.md"
    CreateShortcut "$SMPROGRAMS\${DISPLAY_NAME}\Uninstall.lnk" "$INSTDIR\uninstall.exe"

    SetOutPath "$INSTDIR"
    WriteUninstaller "$INSTDIR\uninstall.exe"

    WriteRegStr HKLM "${UNINST_KEY}" "DisplayName" "${DISPLAY_NAME}"
    WriteRegStr HKLM "${UNINST_KEY}" "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "${UNINST_KEY}" "Publisher" "${PUBLISHER}"
    WriteRegStr HKLM "${UNINST_KEY}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKLM "${UNINST_KEY}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
    WriteRegDWORD HKLM "${UNINST_KEY}" "NoModify" 1
    WriteRegDWORD HKLM "${UNINST_KEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
    ; Mirrors the Install section exactly.
    ReadRegStr $0 HKLM "${UNINST_KEY}" "ExtcapDir"
    ${If} $0 != ""
        Delete "$0\${PKG_NAME}.exe"
    ${EndIf}

    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\README.md"
    RMDir /r "$INSTDIR\docs"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"

    Delete "$SMPROGRAMS\${DISPLAY_NAME}\README.lnk"
    Delete "$SMPROGRAMS\${DISPLAY_NAME}\Uninstall.lnk"
    RMDir "$SMPROGRAMS\${DISPLAY_NAME}"

    DeleteRegKey HKLM "${UNINST_KEY}"
SectionEnd
