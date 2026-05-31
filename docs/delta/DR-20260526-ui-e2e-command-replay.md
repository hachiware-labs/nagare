# delta-request

## Delta ID
- DR-20260526-ui-e2e-command-replay

## Delta Type
- FEATURE

## 目的
- Static UI の Human Input Panel が作る command を Playwright E2E が実際に CLI として実行し、再export後の状態変化まで検証する。

## 変更対象（In Scope）
- E2E test: UI command の読み取り、CLI実行、再export、状態確認を追加する。
- package / Playwright 設定: E2E 実行入口を維持する。
- spec / plan: E2E の検証範囲を更新する。

## 非対象（Out of Scope）
- Static UI から直接 CLI を実行するサーバ/API実装。
- 実Codex等の外部 agent runtime を呼ぶE2E。
- workflow logic / CLI command 体系の変更。

## Candidate Files/Artifacts
- tests/e2e/static-ui.spec.ts
- docs/spec.md
- docs/plan.md
- package.json
- package-lock.json
- playwright.config.ts

## 差分仕様
- DS-01:
  - Given: `needs_input` の Work Item があり、Static UI が export 済みである。
  - When: Playwright が Detail の Human Input Panel に回答を入力する。
  - Then: UI上の command は `nagare item answer <work_id> --answer "<text>"` に更新される。
- DS-02:
  - Given: DS-01 の command が表示されている。
  - When: E2E がその command を安全に argv へ変換し、CLI として実行する。
  - Then: `feedback <id> recorded item_status=ready` が返る。
- DS-03:
  - Given: DS-02 の実行後である。
  - When: E2E が `nagare ui open --open false` を再実行し、生成HTMLを開き直す。
  - Then: Work Item detail に `human_feedback` が表示され、Next Action Panel は次の `run_agent` 導線を表示する。

## 実装スライス
- S-01: Human Input Panel の command 生成をブラウザ操作で検証する。
- S-02: 表示 command を shell eval せず argv 化し、テスト用 project 内で CLI 実行する。
- S-03: 再exportした Static UI で `human_feedback` と次アクションを検証する。

## 受入条件（Acceptance Criteria）
- AC-01: `npm run test:e2e` が UI command 更新だけでなく、表示 command の CLI 実行と再export後の状態確認まで検証する。
- AC-02: E2E は表示 command を shell eval せず、quoted argv として解析して実行する。
- AC-03: `needs_input` への回答後、再exportした detail で `human_feedback` と `run_agent` が確認できる。
- AC-04: `cargo test --workspace` と `npm run test:e2e` が成功する。

## Verify Profile
- static check: Required
- targeted unit: Not Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: Static UI E2E の仕様範囲を `docs/spec.md` に直接反映するため。

## 制約
- E2Eは外部 agent runtime に依存しない。
- テスト対象 command はテスト用一時 project 内でのみ実行する。

## Review Gate
- required: No
- reason: E2E の検証範囲拡張であり、product runtime logic は変更しないため。

## 未確定事項
- Q-01: なし
