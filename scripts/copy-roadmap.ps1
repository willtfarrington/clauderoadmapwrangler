<#
.SYNOPSIS
    Copies the files the dashboard reads of one project into a scratch portfolio, for a screenshot
    run over a copy (EP-36, CLAUDE.md rule 5).
.DESCRIPTION
    pwsh -File scripts\copy-roadmap.ps1 -Project <project folder> -Into <copy root>
    Writes <copy root>\<the project's folder name> with what a row reads, and nothing else:
    roadmap\ (its own files: the README, the briefs, the owner tasks, the inbox, the older forms),
    roadmap\adr and docs\adr, docs\handoffs, CLAUDE.md and .claude\CLAUDE.md, the license,
    DECISIONS.md at the root or one folder down, and the stack's manifests at the root and one
    folder down, as the stack chips look for them. A lock file is written empty, since only its
    presence counts, and .git is never copied, so the copy is no repository. Run it once per
    project, one call at a time (rule 9), and never on a data repository (the house style's
    rule 5). Then point a screenshot run at the copy root with CRW_SCREENSHOT_ROOT or the setup
    flag root=. The copy and its pictures stay in the session's scratchpad and never enter git
    (rule 3).
#>
param(
    [Parameter(Mandatory)] [string] $Project,
    [Parameter(Mandatory)] [string] $Into
)
$ErrorActionPreference = 'Stop'
$source = (Resolve-Path -LiteralPath $Project).Path
$name = Split-Path -Leaf $source
$target = Join-Path $Into $name
if (Test-Path -LiteralPath $target) { throw "$target exists: copy into a fresh folder" }

# The folders the decision log (crw_core::decisions) and the stack chips (badges::stack) skip one
# level down, and the files the chips read in each place.
$logSkips = 'target', 'node_modules', 'venv', 'private'
$stackSkips = 'target', 'node_modules', 'dist', 'build', 'venv', 'data', 'private', 'roadmap',
    'docs', 'fixtures', 'vendor', 'out', 'bin', 'obj'
$manifests = 'Cargo.toml', 'pyproject.toml', '.python-version', 'package.json', 'tsconfig.json',
    'compose.yml', 'compose.yaml', 'docker-compose.yml', 'docker-compose.yaml', 'Dockerfile',
    'index.html'
$locks = 'uv.lock', 'poetry.lock', 'pnpm-lock.yaml', 'yarn.lock', 'bun.lock', 'bun.lockb',
    'package-lock.json'

function Join-Relative([string] $Place, [string] $File) {
    if ($Place) { Join-Path $Place $File } else { $File }
}

function Copy-One([string] $Relative, [switch] $Empty) {
    $from = Join-Path $source $Relative
    if (-not (Test-Path -LiteralPath $from -PathType Leaf)) { return }
    $to = Join-Path $target $Relative
    New-Item -ItemType Directory -Force (Split-Path -Parent $to) | Out-Null
    if ($Empty) {
        New-Item -ItemType File $to | Out-Null
    } else {
        Copy-Item -LiteralPath $from -Destination $to
    }
}

# A folder's own files, not its subfolders.
function Copy-Files([string] $Relative, [string] $Filter = '*') {
    $dir = Join-Path $source $Relative
    if (-not (Test-Path -LiteralPath $dir -PathType Container)) { return }
    Get-ChildItem -LiteralPath $dir -File -Filter $Filter |
        ForEach-Object { Copy-One (Join-Path $Relative $_.Name) }
}

Copy-Files 'roadmap'
Copy-Files 'roadmap\adr' '*.md'
Copy-Files 'docs\adr' '*.md'
Copy-Files 'docs\handoffs' '*.md'
Copy-One 'CLAUDE.md'
Copy-One '.claude\CLAUDE.md'
Copy-One 'DECISIONS.md'
Get-ChildItem -LiteralPath $source -File |
    Where-Object { $_.Name -match '^(LICEN[CS]E|COPYING|UNLICENSE)' } |
    ForEach-Object { Copy-One $_.Name }

$folders = Get-ChildItem -LiteralPath $source -Directory | ForEach-Object Name
foreach ($folder in $folders) {
    if ($folder -notmatch '^\.' -and $logSkips -notcontains $folder.ToLower()) {
        Copy-One (Join-Relative $folder 'DECISIONS.md')
    }
}
$places = @('') + @($folders | Where-Object {
        $_ -notmatch '^[._]' -and $stackSkips -notcontains $_.ToLower()
    })
foreach ($place in $places) {
    foreach ($file in $manifests) { Copy-One (Join-Relative $place $file) }
    foreach ($file in $locks) { Copy-One (Join-Relative $place $file) -Empty }
    $dir = if ($place) { Join-Path $source $place } else { $source }
    Get-ChildItem -LiteralPath $dir -File -Filter '*.ps1' | Sort-Object Name |
        Select-Object -First 1 | ForEach-Object { Copy-One (Join-Relative $place $_.Name) }
}

$count = @(Get-ChildItem -LiteralPath $target -Recurse -File -ErrorAction SilentlyContinue).Count
"copied $count files of $name into $target"
