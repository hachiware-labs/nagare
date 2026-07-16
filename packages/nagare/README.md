# @hachiware-labs/nagare

[日本語](README_ja.md)

Windows x64 npm distribution for the Nagare Rust CLI.

The `nagare` executable is bundled in the package, so Rust is not required on
the machine that installs it.

## Install

```powershell
npm install -g @hachiware-labs/nagare
nagare doctor
```

This initial release supports Windows x64. The npm metadata prevents installs
on unsupported operating systems and CPU architectures.

For local development from the repository root:

```powershell
nagare doctor
```

Run commands through the installed `nagare` binary:

```powershell
nagare init
nagare locale show
nagare agent add --id codex-impl --runtime codex-local --adapter process.codex-cli --working-dir . --description "Implementation and review" --specialties implementation,review
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
