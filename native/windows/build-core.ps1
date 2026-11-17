# Builds build/win-<arch>/weft_core.dll: the native kernel (native/core)
# with Hermes linked statically, loaded by Weft.WinUI through P/Invoke.
# Run with PowerShell 7 on Windows; Python 3 on PATH and Visual Studio Build
# Tools with the C++ toolchain and CMake are required. The default architecture
# is the host's.
# Example: ./build-core.ps1 -Arch arm64 -HermesSource C:/weft/hermes-src
[CmdletBinding()]
param(
    [ValidateSet('arm64', 'x64')]
    [string] $Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant(),
    [string] $HermesSource
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
if (-not (Test-Path $vswhere)) {
    throw 'weft: Visual Studio Installer (vswhere.exe) was not found.'
}
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) {
    throw 'weft: Visual Studio C++ Build Tools were not found.'
}
$hostArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
Import-Module (Join-Path $vs 'Common7/Tools/Microsoft.VisualStudio.DevShell.dll')
Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments "-arch=$Arch -host_arch=$hostArch"
# Use the tools bundled with VS even when standalone CMake is not on PATH.
$cmakeTools = Join-Path $vs 'Common7/IDE/CommonExtensions/Microsoft/CMake'
$env:PATH = "$(Join-Path $cmakeTools 'CMake/bin');$(Join-Path $cmakeTools 'Ninja');$env:PATH"

$core = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../core'))
$build = Join-Path $PSScriptRoot "build/win-$Arch"
$configure = @(
    '-S', $core, '-B', $build, '-G', 'Ninja',
    '-DCMAKE_BUILD_TYPE=Release',
    '-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded',
    '-DCMAKE_POLICY_DEFAULT_CMP0091=NEW',
    '-DWEFT_CORE_SHARED=ON',
    '-DWEFT_CORE_TESTS=OFF',
    '-DWEFT_CONFORMANCE=OFF'
)
if ($HermesSource) {
    $configure += "-DFETCHCONTENT_SOURCE_DIR_HERMES=$((Resolve-Path $HermesSource).Path)"
}
Write-Host "weft: building the kernel for Windows $Arch"
& cmake @configure
& cmake --build $build --target weft_core
$dll = Join-Path $build 'weft_core.dll'
if (-not (Test-Path $dll)) {
    throw "weft: expected kernel DLL was not produced: $dll"
}
Write-Host "weft: wrote $dll"
