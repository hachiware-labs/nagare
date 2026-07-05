# delta-request

## Delta ID
- DR-20260526-ui-serve-agent-visibility

## Delta Type
- UX

## 目的
- ローカルUIサーバのホーム画面で、現在使われる既定Agentと登録済みAgent Profileを確認できるようにする。

## 変更対象（In Scope）
- UI server home: Agent Defaults を表示する。
- UI server home: Agent Profiles の一覧を表示する。
- E2E: ホーム画面から既定Agent領域と登録Agentが見えることを検証する。

## 非対象（Out of Scope）
- Agent Profile の作成・編集UI。
- `nagare agent use` 相当の変更フォーム。

## 受入条件（Acceptance Criteria）
- AC-01: UI server home に Agent Defaults が表示される。
- AC-02: UI server home に Agent Profiles が表示される。
- AC-03: `npm run test:e2e` が成功する。
