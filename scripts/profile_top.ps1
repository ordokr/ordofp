# Extract a ranked hot list (self / inclusive sample weight per function)
# from a samply --save-only profile (Firefox Profiler JSON, gzipped).
#
# Usage: pwsh scripts/profile_top.ps1 -Path path/to/steady_profile.json.gz [-Top 30]
#
# Part of the perf-measurement harness: samply (xperf/ETW-backed on
# Windows) records the e2e_workload driver; this script symbolicates the
# binary's frames via dbghelp (PDB beside the exe) and prints the ranked lists
# optimization work starts from. Frames in system DLLs are rolled up under
# the DLL name (ntdll.dll ≈ heap + OS).

param(
    [Parameter(Mandatory = $true)][string]$Path,
    [int]$Top = 30
)

$ErrorActionPreference = 'Stop'

# ── dbghelp symbolication helper ─────────────────────────────────────────────
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class DbgHelp {
    public const uint SYMOPT_UNDNAME = 0x2, SYMOPT_DEFERRED_LOADS = 0x4;

    [DllImport("dbghelp.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern bool SymInitializeW(IntPtr hProcess, string UserSearchPath, bool fInvadeProcess);

    [DllImport("dbghelp.dll")]
    public static extern uint SymSetOptions(uint SymOptions);

    [DllImport("dbghelp.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern ulong SymLoadModuleExW(IntPtr hProcess, IntPtr hFile, string ImageName,
        string ModuleName, ulong BaseOfDll, uint DllSize, IntPtr Data, uint Flags);

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Ansi)]
    public struct SYMBOL_INFO {
        public uint SizeOfStruct; public uint TypeIndex; public ulong Reserved1; public ulong Reserved2;
        public uint Index; public uint Size; public ulong ModBase; public uint Flags; public ulong Value;
        public ulong Address; public uint Register; public uint Scope; public uint Tag;
        public uint NameLen; public uint MaxNameLen;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 2048)] public string Name;
    }

    [DllImport("dbghelp.dll", SetLastError = true)]
    public static extern bool SymFromAddr(IntPtr hProcess, ulong Address, out ulong Displacement, ref SYMBOL_INFO Symbol);
}
'@

$hProc = [IntPtr]0x1234
[void][DbgHelp]::SymSetOptions([DbgHelp]::SYMOPT_UNDNAME -bor [DbgHelp]::SYMOPT_DEFERRED_LOADS)
if (-not [DbgHelp]::SymInitializeW($hProc, $null, $false)) {
    throw "SymInitialize failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
}

$moduleBase = @{}   # lib index -> load base (0 = unloadable)
function Resolve-Frame([int]$libIdx, [object]$libs, [long]$rva) {
    $lib = $libs[$libIdx]
    if (-not $moduleBase.ContainsKey($libIdx)) {
        $base = 0
        # Only symbolicate modules whose image+PDB are local (the workload exe);
        # system DLLs roll up under their name.
        $pdb = [IO.Path]::ChangeExtension($lib.path, '.pdb')
        if ((Test-Path $lib.path) -and (Test-Path $pdb)) {
            $base = [DbgHelp]::SymLoadModuleExW($hProc, [IntPtr]::Zero, $lib.path, $null, 0, 0, [IntPtr]::Zero, 0)
        }
        $moduleBase[$libIdx] = $base
    }
    $base = $moduleBase[$libIdx]
    if ($base -eq 0) { return $lib.name }
    $sym = New-Object DbgHelp+SYMBOL_INFO
    $sym.SizeOfStruct = 88
    $sym.MaxNameLen = 2000
    $disp = [uint64]0
    if ([DbgHelp]::SymFromAddr($hProc, [uint64]$base + [uint64]$rva, [ref]$disp, [ref]$sym)) {
        return $sym.Name
    }
    return "$($lib.name)+0x$($rva.ToString('x'))"
}

# ── decompress + parse profile ───────────────────────────────────────────────
$fs = [System.IO.File]::OpenRead((Resolve-Path $Path))
try {
    $gz = [System.IO.Compression.GZipStream]::new($fs, [System.IO.Compression.CompressionMode]::Decompress)
    $text = [System.IO.StreamReader]::new($gz).ReadToEnd()
} finally {
    $fs.Dispose()
}
$profile = $text | ConvertFrom-Json -AsHashtable -Depth 100

$threads = $profile.threads | Sort-Object { -$_.samples.length }
Write-Host "threads recorded:"
foreach ($t in $threads) {
    Write-Host ("  {0,-34} samples={1}" -f "$($t.processName)/$($t.name)", $t.samples.length)
}
$main = $threads[0]

$stackFrame = $main.stackTable.frame
$stackPrefix = $main.stackTable.prefix
$frameFunc = $main.frameTable.func
$frameAddr = $main.frameTable.address
$funcResource = $main.funcTable.resource
$resourceLib = $main.resourceTable.lib
$libs = $profile.libs

# func index -> display name (via a representative frame's address + lib).
# All hashtable keys are cast to [long]: JSON numbers parse as Int64 and a
# mixed Int32 lookup would silently never match.
$funcAddr = @{}
for ($i = 0; $i -lt $main.frameTable.length; $i++) {
    $f = [long]$frameFunc[$i]
    if (-not $funcAddr.ContainsKey($f)) { $funcAddr[$f] = $frameAddr[$i] }
}
$funcDisplay = @{}
function Get-FuncName([long]$func) {
    if ($funcDisplay.ContainsKey($func)) { return $funcDisplay[$func] }
    $name = '(root)'
    $addr = $funcAddr[$func]
    if ($null -ne $addr -and $addr -ge 0) {
        $res = $funcResource[$func]
        if ($res -ge 0) {
            $name = Resolve-Frame ([int]$resourceLib[$res]) $libs ([long]$addr)
        } else {
            $name = "0x$($addr.ToString('x'))"
        }
    }
    $funcDisplay[$func] = $name
    return $name
}

# ── self / inclusive aggregation ─────────────────────────────────────────────
$sampleStacks = $main.samples.stack
$weights = $main.samples.weight
$n = $main.samples.length

$self = @{}
$incl = @{}
$total = 0.0

for ($i = 0; $i -lt $n; $i++) {
    $stack = $sampleStacks[$i]
    if ($null -eq $stack) { continue }
    $w = if ($null -ne $weights) { [double]$weights[$i] } else { 1.0 }
    $total += $w

    $leaf = Get-FuncName ([int]$frameFunc[$stackFrame[$stack]])
    $self[$leaf] = [double]$self[$leaf] + $w

    $seen = @{}
    $cur = $stack
    while ($null -ne $cur) {
        $name = Get-FuncName ([int]$frameFunc[$stackFrame[$cur]])
        if (-not $seen.ContainsKey($name)) {
            $incl[$name] = [double]$incl[$name] + $w
            $seen[$name] = $true
        }
        $cur = $stackPrefix[$cur]
    }
}

Write-Host ""
Write-Host ("total sample weight: {0} (thread: {1})" -f $total, $main.name)

function Show-Table($title, $rows) {
    Write-Host ""
    Write-Host $title
    Write-Host ("{0,8} {1,8} {2,9} {3,9}  {4}" -f 'self', 'incl', 'self%', 'incl%', 'function')
    foreach ($r in $rows) {
        $name = $r.Key
        $sv = [double]$self[$name]
        $iv = [double]$incl[$name]
        Write-Host ("{0,8:N0} {1,8:N0} {2,8:N1}% {3,8:N1}%  {4}" -f $sv, $iv, (100 * $sv / $total), (100 * $iv / $total), $name)
    }
}

Show-Table "── top by SELF ──" ($self.GetEnumerator() | Sort-Object { -$_.Value } | Select-Object -First $Top)
Show-Table "── top by INCLUSIVE ──" ($incl.GetEnumerator() | Sort-Object { -$_.Value } | Select-Object -First $Top)
