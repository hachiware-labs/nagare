# @hachiware-labs/nagare

[English](README.md)

Nagare Rust CLI のための薄い npm launcher です。

この package は MVP 0 の配布経路を準備するためのものです。CI release build が用意できた段階で、platform binary package を追加します。

リポジトリルートからのローカル開発では以下のように実行します。

```powershell
nagare doctor
```

install された `nagare` binary からコマンドを実行します。

```powershell
nagare init
nagare locale show
nagare agent add --id codex-impl --runtime codex-local --adapter process.codex-cli --working-dir . --description "実装と検証向け" --specialties implementation,verification
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
