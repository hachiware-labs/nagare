# delta-request

## Delta ID
- DR-20260526-ui-e2e-agent-run-replay

## Delta Type
- FEATURE

## 目的
- 回答後に表示される次の agent run 導線を、UI入力から実行可能な command にし、E2E がテスト用 agent command で実行して再export後の進行を確認する。

## 変更対象（In Scope）
- Static UI: `run_agent` 状態の Human Input Panel が `nagare item run <work_id> --prompt "<text>"` を生成する。
- E2E test: 回答後の再export UIから agent run command を読み取り、テスト用 `--command` に置換してCLI実行し、再export後の状態を検証する。
- spec / plan: agent run replay 段階の検証範囲を記録する。

## 非対象（Out of Scope）
- Static UI から直接 CLI を実行するサーバ/API実装。
- 実Codex等の外部 agent runtime を呼ぶE2E。
- CLI command 体系や workflow logic の変更。

## Candidate Files/Artifacts
- crates/nagare-core/src/ui.rs
- crates/nagare-core/tests/static_ui_export.rs
- tests/e2e/static-ui.spec.ts
- docs/spec.md
- docs/plan.md

## 差分仕様
- DS-01:
  - Given: Work Item が `ready` で次アクションが `run_agent` である。
  - When: Static UI Detail を export する。
  - Then: Human Input Panel は `nagare item run <work_id> --prompt "<text>"` を生成できる。
- DS-02:
  - Given: DS-01 の command が表示されている。
  - When: E2E が表示 command を argv 化し、外部 runtime に依存しないテスト用 `--command` に置換して CLI 実行する。
  - Then: agent run が成功し、Work Item は次の review / verification 導線へ進む。
- DS-03:
  - Given: DS-02 の実行後である。
  - When: E2E が再exportして Detail を開き直す。
  - Then: Timeline に新しい work run / evidence が表示され、Next Action Panel は `review` または後続導線を表示する。

## 受入条件（Acceptance Criteria）
- AC-01: `run_agent` 状態の Human Input Panel に `nagare item run <work_id> --prompt` が表示される。
- AC-02: E2E が回答 command 実行後、agent run command を読み取り、テスト用 `--command` に置換して実行する。
- AC-03: 再export後の Detail に agent run の evidence と後続 next action が表示される。
- AC-04: `npm run test:e2e` と `cargo test --workspace` が成功する。

## Verify Profile
- static check: Required
- targeted unit: Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: Static UI E2E の段階的検証範囲を `docs/spec.md` に直接反映するため。

## 制約
- E2E は外部 agent runtime に依存しない。
- テスト用 command 実行は一時 project 内に限定する。

## Review Gate
- required: No
- reason: UI command builder とE2E範囲の拡張であり、runtime実行方式は変更しないため。

## 未確定事項
- Q-01: なし
