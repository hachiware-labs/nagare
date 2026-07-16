# Nagare エージェント能力スコープ設計

| 項目 | 内容 |
| --- | --- |
| 状態 | Draft |
| 作成日 | 2026-07-16 |
| 関連文書 | `docs/nagare_prd_v1_0.md`、`docs/nagare_trace_schema_v1_0.md` |

## 1. 目的

Nagare が実行を委譲するとき、Skill、MCP、ドメイン固有の指示を「全員に見える既定値」として扱わない。各エージェントに明示的に割り当てた能力だけを有効にし、実行時に実際に有効だった構成を利用者と Trace の双方に示す。

この設計は、能力を増やすことよりも、エージェントの役割・入力・権限を小さく保ち、誤った能力の混入を見つけやすくすることを優先する。

## 2. 決定

### 2.1 能力は「登録」と「有効化」を分ける

- Skill と MCP は、ライブラリに登録されているだけでは実行中のエージェントに有効にならない。
- Work の割り当て時に選んだエージェント定義が、その実行で使える Skill、MCP、ドメイン指示を決める。
- グローバルまたは個人設定から継承された能力が実際に有効だった場合は、Nagare が警告として表示・記録する。警告は失敗ではなく、能力をエージェント固有へ移す候補を示す。
- Skill の説明文やプロンプト上の制約だけに依存しない。可能なランタイムでは、Skill の許可リスト、MCP ツールの許可・拒否、資格情報、サンドボックスを実行構成で制限する。

このため、Nagare 内の能力の状態は次の三層で扱う。

| 層 | 意味 | 実行への影響 |
| --- | --- | --- |
| グローバル登録 | 利用可能な Skill/MCP のカタログ | 直接は有効にしない |
| エージェント割り当て | 役割ごとの許可セット | そのエージェントを起動したときだけ候補になる |
| 実効構成 | ランタイムが実際に読み込んだ構成 | 実行結果と Trace に記録する |

### 2.2 強制ルールの責務を分ける

強制ルールには、ルールごとに唯一の配置先と所有者を持たせる。

| 種別 | 主な所有者 | 例 |
| --- | --- | --- |
| 成果物の品質・方針 | Reviewer | 根拠の不足、禁止された設計、レビュー観点の未達 |
| ワークフロー不変条件 | Nagare の状態遷移・Verifier | Review 前に完了へ進めない、必須証跡がない結果を採用しない |
| ツール利用権限 | エージェント定義・ランタイムアダプター | 特定ロール以外に本番 MCP を使わせない |

Reviewer は強制ルールを重要なレビュー観点として、根拠付きの verdict を残す。ただし、Reviewer だけを強制機構にしない。レビュー前完了の防止や証跡の必須化など、破られてはならない工程ルールは Nagare の状態遷移または機械的 Verifier でも検証する。

### 2.3 実効構成を可視化する

実行開始時と結果画面で、少なくとも次を表示する。

- 要求したエージェントと、実際に起動したエージェント
- 起動方式（主エージェント、名前付きサブエージェント、ルーティング先エージェント）
- 有効な Skill とその出所・版
- 有効な MCP サーバー／ツールとその出所
- エージェント指示、プロジェクト指示、Work 指示、Knowledge の各コンテキスト参照
- グローバル／個人設定からの継承を示す警告

ここで記録するのは構成の識別子、出所、版、参照である。生のプロンプト、MCP 通信内容、モデルの内部ログを Trace に複製しない。詳細ログは既存の diagnostics 参照先に委ねる。

## 3. ランタイム別の実現方式

| ツール | エージェント切替方式 | Skill のエージェント指定方法 | MCP のエージェント指定方法 | エージェントへのコンテキスト指定 |
| --- | --- | --- | --- | --- |
| Codex CLI | プロジェクトの `.codex/agents/*.toml` に名前付きサブエージェントを定義し、親エージェントが起動を依頼する。現行 CLI にはルート実行を名前で直接選ぶ `--agent` はない。 | エージェント TOML の `skills.config` で構成する。未指定の項目は親から継承し得るため、実効構成を取得する。 | エージェント TOML の `mcp_servers` で指定する。 | `developer_instructions`、プロジェクトの `AGENTS.md`／設定、親タスク、Knowledge 参照を分けて記録する。 |
| OpenCode | `.opencode/agents/` のエージェント定義を使う。`opencode run --agent <name>` で主エージェントを直接選べ、`@agent` でサブエージェントを指定できる。 | エージェントごとの `permission.skill` で許可対象を制限する。拒否された Skill は公開しない。 | エージェントごとの permission で MCP ツール名パターンを許可・拒否する。 | エージェント Markdown の指示、プロジェクト設定、起動プロンプト、Knowledge 参照を分ける。 |
| OpenClaw | `agents.list[].id` を単位に、チャネルの binding、既定エージェント、またはサブエージェント起動時の agent ID で切り替える。`requireAgentId` によりサブエージェントの明示指定を必須にできる。 | エージェントの `skills` 許可リストを使う。非空リストは既定値を置き換える最終セットとして扱える。 | エージェントの tools 許可・拒否、ツールプロファイルで MCP を含むツールを制限する。 | agent ID、workspace、agentDir、モデル、エージェント指示、ルーティング元、Work 指示を構成として記録する。 |
| Claude Code | `.claude/agents/*.md` のプロジェクトエージェントを使う。`claude --agent <name>` で主エージェントを選べ、`@mention` で名前付きサブエージェントを指定できる。 | エージェント frontmatter の `skills` は事前読込に使う。厳密に限定する場合は `tools` の許可リストで Skill ツールも制限する。 | エージェント定義内の `mcpServers` を使うと、そのエージェント実行だけに接続できる。 | エージェント本文の system prompt、`CLAUDE.md`、起動タスク、Knowledge 参照を分ける。 |

Hermes Agent は本設計の対応対象外とする。

## 4. Codex の採用方針

当面の Codex アダプターは CLI を標準実行面とする。

- `codex exec --json` を、実行状態と結果を Nagare の Work／Trace に取り込むための基本インターフェースとする。
- プロジェクト内の `.codex/agents/` に定義したサブエージェントを、役割ごとの能力境界として利用する。
- CLI の制約上、Nagare は「要求した名前付きエージェント」と「実際に起動されたサブエージェント」を区別して扱う。親への依頼だけで実際の起動を証明できない場合は、実効エージェントを未検証として表示する。

Codex App Server は、将来、デスクトップでの逐次表示、会話の継続、承認要求、詳細なイベントストリームを Nagare に統合する必要が生じたときの追加アダプター候補とする。現時点では CLI アダプターを置き換えない。

## 5. アダプター共通の実効構成契約

各ランタイムアダプターは、実行前後に次の論理情報を Nagare へ返す。

| 項目 | 内容 |
| --- | --- |
| `requested_agent` | Work が要求したエージェント ID または名前 |
| `actual_agent` | ランタイムが実際に起動したエージェント。検証不能なら `unknown` と理由を残す |
| `launch_mode` | `named_main_agent`、`named_subagent`、`routed_agent` など |
| `agent_definition_ref` | エージェント定義のパスまたは識別子、版またはハッシュ |
| `effective_skills` | Skill ID、版、出所、割り当て／継承の別 |
| `effective_mcps` | MCP サーバーまたはツール ID、出所、許可状態 |
| `context_refs` | エージェント指示、プロジェクト指示、Work 指示、Knowledge の参照 |
| `scope_diagnostics` | グローバル継承、未検証の制限、期待外の能力などの警告 |

`effective_*` はアダプターが観測できた事実を優先する。希望した設定だけを記録して、実際の継承や暗黙の既定値を見落とさない。

## 6. Trace への反映方針

既存の Trace schema の `agent`、`runtime`、`knowledge`、`diagnostics` を壊さず、将来の schema 拡張で実効構成参照を加える。

```json
{
  "agent_selection": {
    "requested_agent": "researcher",
    "actual_agent": "researcher",
    "launch_mode": "named_subagent",
    "definition_ref": ".codex/agents/researcher.toml"
  },
  "effective_capabilities": {
    "skills": [{"id": "hachi-search", "source": "project", "assignment": "agent"}],
    "mcps": [{"id": "search", "source": "agent", "permission": "allow"}]
  },
  "context_refs": [
    {"kind": "agent_instructions", "ref": ".codex/agents/researcher.toml"},
    {"kind": "project_instructions", "ref": "AGENTS.md"}
  ],
  "scope_diagnostics": []
}
```

この JSON は方向性を示す例であり、現行 `trace.jsonl` の互換 schema をこの文書だけで変更するものではない。具体的なフィールド名、必須性、移行は Trace schema の改訂で決める。

## 7. MVP の完了条件

1. Work ごとに、選択したエージェントと有効な Skill／MCP／コンテキスト参照を確認してから実行できる。
2. 対応ランタイムのアダプターが、可能な範囲で実効構成を収集し、収集不能な部分を未検証として報告する。
3. グローバルまたは個人由来の継承が検出されたとき、実行画面と Trace に警告が残る。
4. Reviewer の verdict と、状態遷移・Verifier による工程上の強制を別々に追跡できる。
5. MCP のエージェント限定はランタイム設定だけで安全境界だと見なさず、必要な場合は資格情報とサンドボックスの分離も確認する。

## 8. 参照した公式ドキュメント

- [Codex Subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents.md)
- [Codex App Server](https://learn.chatgpt.com/docs/app-server.md)
- [OpenCode Agents](https://opencode.ai/docs/agents)
- [OpenCode Skills](https://opencode.ai/docs/skills/)
- [OpenClaw agent configuration](https://docs.openclaw.ai/gateway/config-agents)
- [OpenClaw Skills](https://docs.openclaw.ai/tools/skills)
- [Claude Code Subagents](https://code.claude.com/docs/en/sub-agents)
