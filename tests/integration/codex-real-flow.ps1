param(
  [string]$Root = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot "..\.."))
$tmpRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot ".tmp"))

if ([string]::IsNullOrWhiteSpace($Root)) {
  $Root = Join-Path $tmpRoot "nagare-codex-real-flow"
}

$resolvedRoot = [System.IO.Path]::GetFullPath($Root)
if (-not $resolvedRoot.StartsWith($tmpRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Refusing to recreate root outside repo .tmp: $resolvedRoot"
}

function Invoke-Nagare {
  param([string[]]$NagareArgs)

  $output = & cargo run -q -p nagare-cli -- @NagareArgs 2>&1
  if ($LASTEXITCODE -ne 0) {
    $text = $output -join "`n"
    throw "nagare command failed ($LASTEXITCODE): cargo run -q -p nagare-cli -- $($NagareArgs -join ' ')`n$text"
  }
  return $output -join "`n"
}

Write-Host "Checking Codex CLI..."
$codexVersion = & codex --version 2>&1
if ($LASTEXITCODE -ne 0) {
  throw "Codex CLI is required for this integration smoke test.`n$($codexVersion -join "`n")"
}
Write-Host ($codexVersion -join "`n")

if (Test-Path -LiteralPath $resolvedRoot) {
  Remove-Item -LiteralPath $resolvedRoot -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $resolvedRoot | Out-Null
Set-Content -LiteralPath (Join-Path $resolvedRoot "README.md") -Encoding UTF8 -Value @"
# Temporary Nagare Codex Flow

This folder is recreated by tests/integration/codex-real-flow.ps1.
"@

Write-Host "Initializing Nagare project at $resolvedRoot..."
Invoke-Nagare @("init", "--root", $resolvedRoot) | Write-Host

$createOutput = Invoke-Nagare @(
  "item", "create",
  "--root", $resolvedRoot,
  "--title", "Create onboarding guide",
  "--description", "Create ONBOARDING.md as a real Codex generation smoke test.",
  "--artifact", "ONBOARDING.md",
  "--acceptance", "ONBOARDING.md is created",
  "--acceptance", "The guide includes no more than three steps"
)
Write-Host $createOutput

if ($createOutput -notmatch "created\s+(work_\d+)") {
  throw "Could not parse work item id from output: $createOutput"
}
$itemId = $Matches[1]

Write-Host "Running real Codex generation through Nagare worker..."
Invoke-Nagare @(
  "item", "run", $itemId,
  "--root", $resolvedRoot,
  "--agent", "worker",
  "--prompt", "Create ONBOARDING.md in this folder. Include a title, prerequisites, no more than three steps, and verification."
) | Write-Host

$artifactPath = Join-Path $resolvedRoot "ONBOARDING.md"
if (-not (Test-Path -LiteralPath $artifactPath)) {
  throw "Expected artifact was not created: $artifactPath"
}

$artifactText = Get-Content -LiteralPath $artifactPath -Raw
foreach ($required in @("# Onboarding", "## Prerequisites", "## Steps", "## Verification")) {
  if (-not $artifactText.Contains($required)) {
    throw "Generated artifact is missing required text: $required"
  }
}

Write-Host "Running real Codex review through Nagare reviewer..."
Invoke-Nagare @(
  "item", "review", $itemId,
  "--root", $resolvedRoot,
  "--agent", "reviewer",
  "--prompt", "Review ONBOARDING.md against the acceptance criteria, artifact presence, and readability."
) | Write-Host

$showOutput = Invoke-Nagare @("item", "show", $itemId, "--root", $resolvedRoot)
Write-Host $showOutput

foreach ($required in @("actor=worker", "actor=reviewer", "ONBOARDING.md", "approval_gate: state=ready", "out_0007`twork`tparsed")) {
  if (-not $showOutput.Contains($required)) {
    throw "Final work item output is missing required text: $required"
  }
}

Write-Host "Codex real generation integration smoke passed: $itemId"
