# @hachiware-labs/nagare

[日本語](README_ja.md)

Thin npm launcher for the Nagare Rust CLI.

This package is prepared for the MVP 0 distribution path. Published platform
binary packages will be added once CI release builds are available.

For local development from the repository root:

```powershell
nagare doctor
```

Run commands through the installed `nagare` binary:

```powershell
nagare init
nagare locale show
nagare agent add --id codex-impl --runtime codex-local --adapter process.codex-cli --working-dir . --description "Implementation and verification" --specialties implementation,verification
nagare agent list
nagare agent use --work-agent codex-impl --dispatch-agent codex-impl
nagare agent defaults
nagare agent doctor codex-impl
nagare agent probe codex-impl
nagare rule check README.md
nagare item create --title "Repair failing agent work item"
nagare item preview work_0001 --command "echo dispatch preview && exit /B 0"
nagare item dispatch accept work_0001
```
