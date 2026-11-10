<#
.SYNOPSIS
    Builds the release exes, installs them and (re)creates the desktop shortcut and the
    Start-menu entry (D-29, EP-37).
.DESCRIPTION
    pwsh -File scripts\install-shortcut.ps1
    The build runs with the window up: the window runs from its installed copy in
    %LOCALAPPDATA%\clauderoadmapwrangler\bin (CRW_HOME\bin in a trial), not from target\release.
    `crw install` copies the build there, each installed exe renamed to a dated previous copy
    first (Windows lets a running exe be renamed, not replaced), and asks the window to restart
    into it, which it does unless something would be lost (a headless run of its own that works
    or waits, an open dialog...): it then keeps running, and its File > Install and restart, or
    the tray's, restarts it later. -NoRestart leaves the window as it is. A window started from
    target\release before the first install (EP-37) holds its exe, so that one build needs the
    window quit from the tray first. A trial (CRW_HOME set) installs into CRW_HOME\bin, asks no
    window to restart, and must name scratch folders with -Desktop and -StartMenu.
    Both shortcuts are called "Roadmap Wrangler" (D-2 addendum) and open the installed window.
    Both carry the AppUserModelID below, the id Windows attributes the app's toasts to (D-21,
    EP-15), and the icon the window's title bar shows: eframe's default (egui's logo, eframe's
    data\icon.png, MIT OR Apache-2.0), written beside the installed exe as an .ico at the sizes
    Windows asks for (the owner's ask, 2026-11-05; the app's own icon replaces both at EP-28).
    -Desktop and -StartMenu put the shortcuts elsewhere (EP-1 checked the script against scratch
    folders); -NoBuild skips the build. Skeleton: the start-at-login entry and the final form
    arrive at EP-28.
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

# The PNG the window's title bar shows: eframe's default icon, found through cargo's metadata.
function Find-WindowIcon {
    $metadata = cargo metadata --format-version 1 --locked --manifest-path (Join-Path $root 'Cargo.toml') |
        ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw "cargo metadata failed (exit $LASTEXITCODE)" }
    $eframe = $metadata.packages | Where-Object name -EQ 'eframe' | Select-Object -First 1
    if (-not $eframe) { throw 'eframe is not among the packages' }
    $png = Join-Path (Split-Path -Parent $eframe.manifest_path) 'data\icon.png'
    if (-not (Test-Path $png)) { throw "$png is missing" }
    $png
}

# Writes $png into $ico as one PNG-compressed image per size, which Windows reads since Vista.
function Write-Icon([string] $png, [string] $ico) {
    Add-Type -AssemblyName System.Drawing
    $sizes = @(16, 20, 24, 32, 40, 48, 64, 72, 96, 128, 256)
    $source = [System.Drawing.Image]::FromFile($png)
    try {
        $images = foreach ($size in $sizes) {
            $bitmap = [System.Drawing.Bitmap]::new($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
            $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
            $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
            $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
            $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
            $graphics.DrawImage($source, 0, 0, $size, $size)
            $graphics.Dispose()
            $stream = [System.IO.MemoryStream]::new()
            $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
            $bitmap.Dispose()
            , $stream.ToArray()
        }
    }
    finally {
        $source.Dispose()
    }
    $out = [System.IO.MemoryStream]::new()
    $writer = [System.IO.BinaryWriter]::new($out)
    # ICONDIR: reserved, type 1 (an icon), the count; then a 16-byte entry per image.
    $writer.Write([uint16] 0)
    $writer.Write([uint16] 1)
    $writer.Write([uint16] $sizes.Count)
    $offset = 6 + 16 * $sizes.Count
    for ($i = 0; $i -lt $sizes.Count; $i++) {
        $side = [byte] ($sizes[$i] % 256)  # 0 stands for 256
        $writer.Write($side)
        $writer.Write($side)
        $writer.Write([byte] 0)    # no palette
        $writer.Write([byte] 0)    # reserved
        $writer.Write([uint16] 1)  # colour planes
        $writer.Write([uint16] 32) # bits per pixel
        $writer.Write([uint32] $images[$i].Length)
        $writer.Write([uint32] $offset)
        $offset += $images[$i].Length
    }
    foreach ($image in $images) { $writer.Write([byte[]] $image) }
    $writer.Flush()
    [System.IO.File]::WriteAllBytes($ico, $out.ToArray())
    $writer.Dispose()
}

$icon = Join-Path $bin 'clauderoadmapwrangler.ico'
try {
    Write-Icon (Find-WindowIcon) $icon
    "Icon: $icon"
}
catch {
    Write-Warning "The shortcuts keep Windows' default icon: $_"
    $icon = ''
}

# The installed window starts in its own folder, so it holds no repository folder open.
foreach ($folder in @($Desktop, $StartMenu)) {
    if (-not (Test-Path $folder)) { New-Item -ItemType Directory -Path $folder | Out-Null }
    $lnk = Join-Path $folder "$name.lnk"
    [RoadmapWrangler.ShellLink]::Save($lnk, $exe, $bin, $description, $appId, $icon)
    "Shortcut: $lnk -> $exe"
}
