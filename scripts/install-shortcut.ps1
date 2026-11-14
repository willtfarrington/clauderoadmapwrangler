<#
.SYNOPSIS
    Builds the release exes, installs them and (re)creates the desktop shortcut and the
    Start-menu entry (D-29; EP-37, final at EP-28a).
.DESCRIPTION
    pwsh -File scripts\install-shortcut.ps1
    The build runs with the window up: the window runs from its installed copy in
    %LOCALAPPDATA%\clauderoadmapwrangler\bin (CRW_HOME\bin in a trial), not from target\release.
    `crw install` copies the build there, each installed exe renamed to a dated previous copy
    first (Windows lets a running exe be renamed, not replaced). It writes the app's own icon
    beside them (app.ico, drawn by the tool: the tray's tile without its badges; the earlier
    clauderoadmapwrangler.ico, egui's logo, is removed, so the shell shows the new icon at once), refreshes the copy of crw.exe on PATH when there is one (`crw path on` puts it in
    %USERPROFILE%\.local\bin, beside claude.exe; `crw path off` removes it), and asks the window
    to restart into the new build. The window does so unless something would be lost (a headless
    run of its own that works or waits, an open dialog...): it then keeps running, and its
    File > Install and restart, or the tray's, restarts it later. -NoRestart leaves the window as
    it is. A trial (CRW_HOME set) installs into CRW_HOME\bin, asks no window to restart, and must
    name scratch folders with -Desktop and -StartMenu.
    Both shortcuts are called "Roadmap Wrangler" (D-2 addendum), open the installed window in its
    own folder, carry the AppUserModelID below (the id Windows attributes the app to, D-21) and
    show the tool's icon. The start-at-login entry is the tool's own switch, never this script's:
    File > Start at login in the window, or `crw autostart on|off`, which starts the installed
    window hidden in the tray when you sign in. The script ends by printing where the entry and
    the PATH copy stand. -Desktop and -StartMenu put the shortcuts elsewhere (EP-1 checked the
    script against scratch folders); -NoBuild skips the build.
#>
param(
    [string] $Desktop = [Environment]::GetFolderPath('Desktop'),
    [string] $StartMenu = [Environment]::GetFolderPath('Programs'),
    [switch] $NoBuild,
    [switch] $NoRestart
)
$ErrorActionPreference = 'Stop'
# A trial never points the owner's shortcuts at its scratch install (rule 13).
if ($env:CRW_HOME -and -not ($PSBoundParameters.ContainsKey('Desktop') -and $PSBoundParameters.ContainsKey('StartMenu'))) {
    throw "CRW_HOME is set (a trial): name scratch folders with -Desktop and -StartMenu"
}
$root = Split-Path -Parent $PSScriptRoot
$crw = Join-Path $root 'target\release\crw.exe'
# The install folder, as crw_core::install::bin_dir names it.
$bin = if ($env:CRW_HOME) {
    Join-Path $env:CRW_HOME 'bin'
} else {
    Join-Path $env:LOCALAPPDATA 'clauderoadmapwrangler\bin'
}
$exe = Join-Path $bin 'clauderoadmapwrangler.exe'
$name = 'Roadmap Wrangler'
$appId = 'willtfarrington.clauderoadmapwrangler'
$description = 'Dashboard and session launcher for Claude Code roadmap projects'

if (-not $NoBuild) {
    cargo build --release --locked --manifest-path (Join-Path $root 'Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit $LASTEXITCODE)" }
}
if (-not (Test-Path $crw)) { throw "$crw is missing: build it first" }
# The copy, then the window's restart into it (crw install prints the window's answer). No
# splatting: PowerShell reads a splatted `--no-restart` as a parameter name.
if ($NoRestart) { & $crw install --no-restart } else { & $crw install }
if ($LASTEXITCODE -ne 0) { throw "crw install failed (exit $LASTEXITCODE)" }
if (-not (Test-Path $exe)) { throw "$exe is missing after the install" }

# WScript.Shell cannot set an AppUserModelID, so the shell's link object is driven through COM.
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
using System.Text;

namespace RoadmapWrangler
{
    [ComImport, Guid("000214F9-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IShellLinkW
    {
        void GetPath([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder file, int max, IntPtr findData, uint flags);
        void GetIDList(out IntPtr idList);
        void SetIDList(IntPtr idList);
        void GetDescription([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder name, int max);
        void SetDescription([MarshalAs(UnmanagedType.LPWStr)] string name);
        void GetWorkingDirectory([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder dir, int max);
        void SetWorkingDirectory([MarshalAs(UnmanagedType.LPWStr)] string dir);
        void GetArguments([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder args, int max);
        void SetArguments([MarshalAs(UnmanagedType.LPWStr)] string args);
        void GetHotkey(out short hotkey);
        void SetHotkey(short hotkey);
        void GetShowCmd(out int showCmd);
        void SetShowCmd(int showCmd);
        void GetIconLocation([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder path, int max, out int index);
        void SetIconLocation([MarshalAs(UnmanagedType.LPWStr)] string path, int index);
        void SetRelativePath([MarshalAs(UnmanagedType.LPWStr)] string path, uint reserved);
        void Resolve(IntPtr window, uint flags);
        void SetPath([MarshalAs(UnmanagedType.LPWStr)] string file);
    }

    [ComImport, Guid("886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IPropertyStore
    {
        void GetCount(out uint count);
        void GetAt(uint index, out PropertyKey key);
        void GetValue(ref PropertyKey key, out PropVariant value);
        void SetValue(ref PropertyKey key, ref PropVariant value);
        void Commit();
    }

    [StructLayout(LayoutKind.Sequential, Pack = 4)]
    struct PropertyKey
    {
        public Guid FormatId;
        public uint PropertyId;
    }

    // A PROPVARIANT holding a VT_LPWSTR, given the full x64 size so the shell never reads past it.
    [StructLayout(LayoutKind.Explicit, Size = 24)]
    struct PropVariant
    {
        [FieldOffset(0)] public ushort VarType;
        [FieldOffset(8)] public IntPtr Pointer;
    }

    [ComImport, Guid("00021401-0000-0000-C000-000000000046")]
    class ShellLinkObject { }

    public static class ShellLink
    {
        // PKEY_AppUserModel_ID
        static readonly Guid AppUserModel = new Guid("9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3");
        const ushort VT_LPWSTR = 31;

        public static void Save(string lnk, string target, string workingDirectory, string description, string appId, string icon)
        {
            var link = (IShellLinkW)new ShellLinkObject();
            link.SetPath(target);
            link.SetWorkingDirectory(workingDirectory);
            link.SetDescription(description);
            if (!string.IsNullOrEmpty(icon))
            {
                link.SetIconLocation(icon, 0);
            }
            var key = new PropertyKey { FormatId = AppUserModel, PropertyId = 5 };
            var value = new PropVariant { VarType = VT_LPWSTR, Pointer = Marshal.StringToCoTaskMemUni(appId) };
            try
            {
                var store = (IPropertyStore)link;
                store.SetValue(ref key, ref value);
                store.Commit();
            }
            finally
            {
                Marshal.FreeCoTaskMem(value.Pointer);
            }
            ((IPersistFile)link).Save(lnk, true);
        }
    }
}
'@

# The app's own icon, which `crw install` drew beside the exes (EP-28a), under a new name so that
# the shell's icon cache shows it at once; the earlier file (egui's logo) is removed.
$icon = Join-Path $bin 'app.ico'
$earlier = Join-Path $bin 'clauderoadmapwrangler.ico'
if (Test-Path -LiteralPath $earlier) { Remove-Item -LiteralPath $earlier }
if (-not (Test-Path $icon)) {
    Write-Warning "The shortcuts keep Windows' default icon: $icon is missing"
    $icon = ''
}

# The installed window starts in its own folder, so it holds no repository folder open.
foreach ($folder in @($Desktop, $StartMenu)) {
    if (-not (Test-Path $folder)) { New-Item -ItemType Directory -Path $folder | Out-Null }
    $lnk = Join-Path $folder "$name.lnk"
    [RoadmapWrangler.ShellLink]::Save($lnk, $exe, $bin, $description, $appId, $icon)
    "Shortcut: $lnk -> $exe"
}

# Where the start-at-login entry and the copy of crw on PATH stand (both read, nothing changed):
# their switches are the tool's own, `crw autostart on|off` (or File > Start at login) and
# `crw path on|off`.
$installed = Join-Path $bin 'crw.exe'
& $installed autostart
if ($LASTEXITCODE -ne 0) { Write-Warning "crw autostart did not read the entry (exit $LASTEXITCODE)" }
& $installed path
if ($LASTEXITCODE -ne 0) { Write-Warning "crw path did not read the PATH copy (exit $LASTEXITCODE)" }
