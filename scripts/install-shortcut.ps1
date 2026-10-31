<#
.SYNOPSIS
    Builds the release exes and (re)creates the desktop shortcut and the Start-menu entry (D-29).
.DESCRIPTION
    pwsh -File scripts\install-shortcut.ps1
    Close the app first: Windows will not let the build replace a running exe.
    Both shortcuts are called "Roadmap Wrangler" (D-2 addendum) and open the app exe. Both carry
    the AppUserModelID below, the id Windows attributes the app's toasts to (D-21, EP-15).
    -Desktop and -StartMenu put the shortcuts elsewhere (EP-1 checked the script against scratch
    folders); -NoBuild skips the build. Skeleton: the icon, the start-at-login entry and the
    final form arrive at EP-28.
#>
param(
    [string] $Desktop = [Environment]::GetFolderPath('Desktop'),
    [string] $StartMenu = [Environment]::GetFolderPath('Programs'),
    [switch] $NoBuild
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'target\release\clauderoadmapwrangler.exe'
$name = 'Roadmap Wrangler'
$appId = 'willtfarrington.clauderoadmapwrangler'
$description = 'Dashboard and session launcher for Claude Code roadmap projects'

if (-not $NoBuild) {
    cargo build --release --locked --manifest-path (Join-Path $root 'Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit $LASTEXITCODE)" }
}
if (-not (Test-Path $exe)) { throw "$exe is missing: build it first" }

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

        public static void Save(string lnk, string target, string workingDirectory, string description, string appId)
        {
            var link = (IShellLinkW)new ShellLinkObject();
            link.SetPath(target);
            link.SetWorkingDirectory(workingDirectory);
            link.SetDescription(description);
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

foreach ($folder in @($Desktop, $StartMenu)) {
    if (-not (Test-Path $folder)) { New-Item -ItemType Directory -Path $folder | Out-Null }
    $lnk = Join-Path $folder "$name.lnk"
    [RoadmapWrangler.ShellLink]::Save($lnk, $exe, $root, $description, $appId)
    "Shortcut: $lnk -> $exe"
}
