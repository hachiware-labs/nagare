# @hachiware-labs/nagare

[English](README.md)

Nagare の Tauri デスクトップアプリと CLI を Windows x64 に配布する npm package です。

Tauri デスクトップアプリと `nagare` CLI を package に同梱するため、インストール先に Rust は不要です。

## インストール

```powershell
npm install -g @hachiware-labs/nagare
nagare
nagare doctor
```

`nagare` は Tauri デスクトップアプリを起動します。`nagare doctor` や
`nagare item list` のように引数を付けた場合だけ CLI を実行します。

初回公開は Windows x64 を対象にします。対応外の OS / CPU では npm metadata によりインストールされません。

リポジトリルートからのローカル開発では以下のように実行します。

```powershell
nagare doctor
```

install された `nagare` binary からコマンドを実行します。

```powershell
nagare init
nagare locale show
nagare agent add --id codex-impl --runtime codex-local --adapter process.codex-cli --working-dir . --description "実装とレビュー向け" --specialties implementation,review
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
