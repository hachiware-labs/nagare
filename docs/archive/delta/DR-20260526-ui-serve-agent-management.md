# delta-request

## Delta ID
- DR-20260526-ui-serve-agent-management

## Delta Type
- FEATURE

## 目的
- 先ほどのユースケース「エージェントの追加設定」をローカルUIサーバ上で実現する。

## 変更対象（In Scope）
- UI server home: Agent Profile 追加フォームを表示する。
- UI API: `POST /api/agents` で Project-local Agent Profile を追加する。
- UI server home: Agent Defaults 設定フォームを表示する。
- UI API: `POST /api/agent-defaults` で Work / Review / Dispatch / Supervisor の既定Agentを設定する。
- E2E: ブラウザから Agent Profile を追加し、Work Agent の既定値に設定できることを検証する。

## 非対象（Out of Scope）
- Agent Profile の更新・削除UI。
- Agent doctor / probe のUI。
- Output contract 編集UI。

## 受入条件（Acceptance Criteria）
- AC-01: UIからAgent Profileを追加できる。
- AC-02: UIから既定Agentを設定できる。
- AC-03: CLIの `agent list` / `agent defaults` でも同じ状態が確認できる。
- AC-04: `npm run test:e2e` と `cargo test --workspace` が成功する。

## Verify Profile
- static check: Required
- targeted integration / E2E: Required

## Canonical Sync Mode
- mode: direct canonical update
- reason: UI server のAgent管理仕様を `docs/spec.md` に反映するため。
