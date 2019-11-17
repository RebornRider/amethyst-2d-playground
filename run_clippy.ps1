#!/usr/bin/env powershell

#Requires -Version 5

function Convert-WrrayToArgs ($arg, $list) {
    if($list) {
        $list | ForEach-Object { "-$arg $_ ``" } | Out-String
    }
}

$clippyArgs += Convert-WrrayToArgs -arg A -list (Get-Content ".\clippy_lints_to_allow.txt")
$clippyArgs += Convert-WrrayToArgs -arg W -list (Get-Content ".\clippy_lints_to_warn.txt")
$clippyArgs += Convert-WrrayToArgs -arg D -list (Get-Content ".\clippy_lints_to_deny.txt")

$clippyCommand = "cargo clippy --all-targets -- $clippyArgs"
Write-Host "--- Running clippy!"
Write-Host "Clippy rules: $clippyCommand"
Invoke-Expression "cargo clean"
Invoke-Expression $clippyCommand

if ($LASTEXITCODE -ne 0) {exit $LASTEXITCODE}
