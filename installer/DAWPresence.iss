; DAWPresence installer script.

; Version, ExeDir, and Renderer are passed via /D flags:
;   iscc /DAppVersion=2.2.0 /DExeDir=..\dist\tiny-skia /DRenderer=tiny-skia DAWPresence.iss

#ifndef ExeDir
  #define ExeDir "..\target\release"
#endif

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

#ifndef Renderer
  #define Renderer "tiny-skia"
#endif

#define AppName "DAWPresence"
#define AppPublisher "MihaiStreames"
#define AppURL "https://github.com/MihaiStreames/DAWPresence"
#define AppExeName "DAWPresence.exe"

[Setup]
AppId={{E7A3C2F1-4B8D-4E6A-9F2C-1D3E5A7B9C0D}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}/issues
AppUpdatesURL={#AppURL}/releases
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
OutputDir=output
OutputBaseFilename=DAWPresence-{#AppVersion}-{#Renderer}-setup
SetupIconFile=..\assets\app\main.ico
UninstallDisplayIcon={app}\{#AppExeName}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
WizardImageFile=..\assets\installer\wizard-image.bmp
WizardSmallImageFile=..\assets\installer\wizard-small.bmp
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
MinVersion=10.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "Start {#AppName} with Windows"; GroupDescription: "Startup:"

[Files]
Source: "{#AddBackslash(ExeDir)}{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#AppName}"; ValueData: """{app}\{#AppExeName}"""; Flags: uninsdeletevalue; Tasks: autostart

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\dawpresence"
