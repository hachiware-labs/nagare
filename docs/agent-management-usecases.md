# Agent Management Use Cases

この文書は、Nagare の Agent 管理を「ツール非依存の Agent Profile」として扱いながら、Codex / Codex CLI / OpenClaw などのツール差分を自然に吸収するための要件とユースケースを整理する。

正本仕様は `docs/spec.md`、既存の管理モデルは `docs/agent_management.md`、データ形状は `docs/agent_data_model.md` を参照する。この文書は、実装順序と使い勝手を固めるための設計確認メモである。

## Product Position

Nagare の Agent 管理は、単なる「外部ツール設定」ではない。

Nagare が管理する中心概念は Agent Profile である。Agent Profile は「何を担当する Agent か」を表し、実行ツール、モデル、スキル、Prompt、担当範囲を組み合わせて、Work Item に対して実行可能な Agent として解決される。

```text
Agent Profile
  What kind of work this agent should handle.

Tool Binding
  Which tool runs it: Codex, Codex CLI, OpenClaw, etc.

Model Selection
  Which model/provider is used inside that tool.

Skill Assignment
  Which skill sets are attached to this agent.

Prompt Config
  How this agent should behave.

Routing Scope
  Which domains or Domains this agent should be selected for.
```

ツール非依存にする理由は、ユーザーが管理したいものが「Codex というプロセス」ではなく「レビュー担当」「フロントエンド実装担当」「ローカルモデルで動く軽量レビュアー」のような働きだからである。

ただし、ツール差分は隠しすぎない。Codex と OpenClaw では設定可能なモデルや接続情報が違うため、UI は選択した tool に応じて必要な項目だけを表示する。

## Requirements

### R1. Agent Profile Is Tool-Agnostic

Agent Profile は Codex / Codex CLI / OpenClaw のどれかに固定された概念ではなく、共通の管理単位として扱う。

Agent Profile が持つ共通項目:

- `id`
- `display_name`
- `role`
- `working_dir`
- `tool_kind`
- `runtime`
- `adapter`
- `model`
- `skill_set_ids`
- `domain_ids`
- `artifact_type_ids`
- `prompt`
- `output_contracts`

### R2. Tool-Specific Fields Are Contextual

tool ごとに設定できる項目は異なるため、保存形式と UI は tool 差分を扱える必要がある。

Codex:

- OpenAI model を選べる
- Base URL は通常不要
- provider は OpenAI 扱い

Codex CLI:

- OpenAI model を選べる
- CLI runtime / adapter / working directory が重要
- Base URL は通常不要

OpenClaw:

- provider を選べる: OpenAI / Ollama / LM Studio
- OpenAI provider では OpenAI model を選べる
- Ollama / LM Studio では model と Base URL を設定できる
- Base URL は provider に応じて必須または推奨になる

### R3. Model Selection Is Separate From Tool Selection

tool を選ぶことと model を選ぶことは別の意思決定として扱う。

例:

- `tool_kind = codex_cli`, `model.provider = openai`, `model.id = gpt-5.3-codex`
- `tool_kind = openclaw`, `model.provider = ollama`, `model.id = qwen2.5-coder:32b`, `model.base_url = http://127.0.0.1:11434/v1`

### R4. Skills Can Be Attached Per Agent

Project rule による path-based skill selection に加えて、Agent Profile に個別の skill set を割り当てられる。

実行時の skill set 解決:

```text
resolved_skill_sets =
  project_rule.skill_sets
  + agent_profile.skill_set_ids
```

重複は取り除く。capability probe の結果により、skill set は applied または skipped として記録する。

### R5. Prompt Is Agent-Specific And Future-Proof

Agent には Prompt を設定できる。最初は単一の instructions でもよいが、後で改善できるように `description` にすべてを詰め込まない。

MVP では既存互換のため `description` を表示用 summary / dispatch hint として残し、新しい実行指示は `prompt.instructions` に保存する。既存 Agent Profile に `prompt.instructions` がない場合は、移行期間中だけ `description` を instructions として扱う。

最小構成:

- `prompt.instructions`
- `prompt.version`

将来拡張候補:

- `prompt.role`
- `prompt.operating_rules`
- `prompt.style_guide`
- `prompt.template_id`
- `prompt.version`

### R6. Prompt Preview Is Needed For Usability

Agent Prompt は、単体の入力値ではなく、Work Item goal、Domain context、Human feedback、Handoff context、Skill context、Nagare output contract と合成される。

そのため、Agent 編集画面には「最終 Prompt Preview」を置く。実行せずに、代表的な Work Item に対してどのような Prompt になるか確認できるようにする。

### R7. UI Shows Capabilities And Warnings

スキルやモデル設定は、保存できるだけでは不十分である。

UI は次を表示する:

- tool が利用可能か
- runtime healthcheck が通るか
- capability probe が新しいか
- selected skill sets が applied になるか
- skipped skill sets と理由
- model / base_url の不足

この表示は Agent 実行後だけでなく、Agent 詳細画面の readiness check として確認できる必要がある。

### R8. Routing Is Independent From Tool

Dispatch は Agent Profile の role、Domain / Artifact Type scope、specialties、skills、prompt summary を見て選択する。Codex か OpenClaw かは候補比較の一部ではあるが、最初の分類軸にしない。

### R9. Skill Catalog Is Explicit

Agent に skill set を追加できるようにするには、skill set 自体の catalog も確認できる必要がある。

Skill catalog は Project config の `[skill_sets]` だけではなく、installed skill packages も元にする。`[skill_sets]` は「この Project で使う skill set の宣言」であり、skill package は「その skill がどこから来て、どの version / ref を使っているか」を表す。

UI では少なくとも次を表示する:

- skill set id
- package id
- source
- version / ref
- paths
- required capabilities
- optional capabilities
- 参照している Agent 数
- 参照している Project Rule 数

MVP では skill set の作成・編集を TOML 中心にしてもよいが、Agent へ追加する画面では catalog から選べるようにする。

### R10. Dispatch Preview Is A First-Class Check

Agent 管理の使い勝手を固めるには、「この Work Item ならどの Agent が選ばれるか」を実行前に確認できる必要がある。

Dispatch Preview は次を表示する:

- candidate agents
- matched Domain / Artifact Type
- matched project rule
- project rule skill sets
- agent-specific skill sets
- selected target agent
- fallback or warning reason

### R11. Skills Have Installable Packages

外部の skill catalog、例として ClawHub や Vercel-style skills のような配布元から skill を取り込めるようにする。

Nagare は少なくとも次の source kind を扱える設計にする:

- `bundled`: Nagare 同梱
- `local`: ローカル directory
- `git`: Git repository / branch / tag / commit
- `hub`: ClawHub などの外部 catalog
- `imported`: 手動で取り込んだ skill package

Skill package は次の情報を持つ:

- `id`
- `display_name`
- `source_kind`
- `source_uri`
- `version`
- `resolved_ref`
- `checksum`
- `installed_path`
- `provided_skill_sets`
- `trust_level`

Project で使う skill は、package source と resolved ref を lock する。これにより「自分の環境では動いたが他の人の環境で skill が違う」を避ける。

### R12. Users Can Discover And Install Skills

ユーザーは、Agent 編集画面または Skill catalog 画面から必要な skill を探し、インストールし、Agent に追加できる。

ユーザーに見せる主導線は、package 管理ではなく Agent 作成・編集中の skill selection である。

```text
Create Agent
  -> Skills
  -> Search or choose recommended skills
  -> Install if needed
  -> Add to this Agent
  -> Save Agent
```

探索では次を軸に検索できる:

- keyword
- domain
- tool compatibility
- required capabilities
- provider
- installed / not installed
- trusted / unverified

インストール時には次を確認する:

- source
- version / ref
- provided skill sets
- required capabilities
- files to be installed
- trust warning

通常フローでは、ユーザーは Skill Package と Skill Set の違いを意識しなくてよい。UI は「スキル」として表示し、source / ref / checksum は詳細表示に隠す。

### R13. Users Can Develop Local Skills

ユーザーは自分で skill package を作れる必要がある。

Nagare は local skill development の最小導線を持つ:

- skill package scaffold を作る
- manifest を validate する
- sample Agent / Work Item で dry run する
- required capabilities を宣言する
- package を local catalog に登録する

MVP では `nagare skill init` 相当の CLI か、手動作成した local directory の登録でよい。重要なのは、外部 catalog 由来の skill と自作 skill を同じ catalog 画面で扱えることである。

## Use Cases

### UC1. Codex CLI Worker を追加する

目的:

ユーザーは、OpenAI model を使う Codex CLI 実装担当 Agent を作りたい。

事前条件:

- Project が初期化済み
- Codex CLI がローカルにある

シナリオ:

1. Settings > Agents で Create Agent を押す。
2. Basic で `id = codex-worker`, `role = worker`, `working_dir = .` を入力する。
3. Tool で `Codex CLI` を選ぶ。
4. Model で `gpt-5.3-codex` を選ぶ。
5. Skills で `rust`, `test-runner`, `repo-maintenance` を選ぶ。
6. Prompt に「小さく実装し、検証結果を最後に書く」と入力する。
7. Domain / Artifact Type scope に `general` と `backend` を選ぶ。
8. 保存する。

期待結果:

- `.nagare/agents/codex-worker.toml` に Agent Profile が保存される。
- Base URL は表示されない、または disabled になる。
- skill set は Agent Profile に保存される。
- Agent 一覧で Tool / Model / Skills / Scope が確認できる。
- Probe により Codex CLI の利用可否が表示される。

要件確認:

- R1: Agent Profile として保存できる。
- R2: Codex CLI 用の項目だけが表示される。
- R3: model selection が tool selection と分離されている。
- R4: agent-specific skills を保存できる。
- R7: runtime/probe 状態を表示できる。

### UC2. OpenClaw + Ollama Reviewer を追加する

目的:

ユーザーは、ローカル Ollama model を使う軽量レビュー Agent を作りたい。

事前条件:

- OpenClaw が利用可能
- Ollama が `http://127.0.0.1:11434/v1` で動いている

シナリオ:

1. Create Agent を押す。
2. Tool で `OpenClaw` を選ぶ。
3. Provider で `Ollama` を選ぶ。
4. Base URL に `http://127.0.0.1:11434/v1` を入力する。
5. Model に `qwen2.5-coder:32b` を入力する。
6. Role に `reviewer` を設定する。
7. Skills で `code-review`, `static-analysis` を選ぶ。
8. Prompt に「重大度順にレビューし、修正案は具体的に出す」と入力する。
9. 保存する。

期待結果:

- OpenClaw provider が Ollama のため Base URL が必須として扱われる。
- provider/model/base_url が Agent Profile に保存される。
- OpenClaw adapter の capability に合わない skill は warning になる。
- Agent 一覧では Codex Agent と同じ表で比較できる。

要件確認:

- R2: tool/provider に応じて必須項目が変わる。
- R3: provider-specific model selection ができる。
- R4: skill set を個別付与できる。
- R7: capability による applied/skipped が見える。

### UC3. OpenClaw + OpenAI Agent を追加する

目的:

ユーザーは、OpenClaw 経由で OpenAI provider の model を使う Agent を作りたい。

シナリオ:

1. Tool で `OpenClaw` を選ぶ。
2. Provider で `OpenAI` を選ぶ。
3. Model に OpenAI model を選ぶ。
4. Base URL は空のまま保存する。

期待結果:

- OpenAI provider の場合、Base URL は不要として扱われる。
- OpenClaw 固有の provider 設定は保存される。
- Codex と同じ OpenAI model でも、adapter は OpenClaw として扱われる。

要件確認:

- R2: OpenClaw 内でも provider によって UI が変わる。
- R3: model selection は tool_kind と provider の組み合わせで表現できる。

### UC4. Agent に後から skill を追加する

目的:

ユーザーは既存の frontend worker に accessibility skill を追加したい。

シナリオ:

1. Agent 詳細を開く。
2. Skills タブを開く。
3. `accessibility-review` と `storybook` を選ぶ。
4. 保存する。
5. 次の frontend Work Item を実行する。

期待結果:

- Agent Profile に `skill_set_ids` が保存される。
- 実行時に project rule の skill set と Agent skill set がマージされる。
- 実際に使われた skill set は Resolved Skill Context に記録される。
- capability が足りない skill は skipped として理由が残る。

要件確認:

- R4: Agent-specific skill assignment ができる。
- R7: applied/skipped の結果が見える。
- R8: skill は dispatch と execution の両方に使える材料になる。

### UC5. Domain に応じて Agent が選ばれる

目的:

ユーザーは frontend Work Item を作り、Nagare に適切な Agent を選ばせたい。

シナリオ:

1. Work Item 作成時に Domain `web`、Domain `frontend` を選ぶ。
2. Work Item を advance する。
3. Dispatch が candidate Agent を比較する。
4. `artifact_type_ids = ["frontend"]` を持つ Agent が優先される。

期待結果:

- Agent の tool が Codex でも OpenClaw でも候補になる。
- Domain / Artifact Type scope が一致する Agent が選ばれやすい。
- 一致する Agent がない場合、domain agent policy に従って fallback または確認になる。

要件確認:

- R1: tool-independent Agent Profile として比較できる。
- R8: routing は tool より domain/role/skill を優先できる。

### UC6. Prompt を改善する

目的:

ユーザーは Agent の Prompt を編集し、実行時にどう見えるか確認したい。

シナリオ:

1. Agent 詳細を開く。
2. Prompt タブを開く。
3. `instructions` を編集する。
4. Preview 用の sample Work Item を選ぶ、または標準サンプルを使う。
5. Prompt Preview を確認する。
6. 保存する。

期待結果:

- Agent Prompt は保存できる。
- Preview では、Agent instructions、Domain context、Skill context、Nagare Agent Context、Output Contract が合成された状態を確認できる。
- 実行せずに Prompt の改善可否を判断できる。

要件確認:

- R5: Agent-specific Prompt が保存できる。
- R6: final prompt preview がある。
- R7: skill/capability の警告が Prompt 作成にも反映される。

### UC7. 設定不足の Agent を保存または実行しようとする

目的:

ユーザーは tool-specific な必須項目が不足している Agent を誤って作らないようにしたい。

シナリオ:

1. Tool に `OpenClaw` を選ぶ。
2. Provider に `Ollama` を選ぶ。
3. Base URL を空にする。
4. 保存しようとする。

期待結果:

- UI は Base URL が必要であることを表示する。
- 保存時にも core 側で validation する。
- 不完全な Agent は dispatch 候補に出さない、または warning を表示する。

要件確認:

- R2: tool/provider-specific validation が必要。
- R7: warning と validation が UI と core の両方にある。

### UC8. 既存 Agent Profile を新しい管理モデルで表示する

目的:

ユーザーは既存の `.nagare/agents/*.toml` を壊さず、新しい Agent 管理画面で編集したい。

シナリオ:

1. 既存 Project を開く。
2. Agent 一覧を表示する。
3. 既存 Agent Profile に `tool_kind` や `prompt.instructions` が存在しない。
4. Nagare は runtime / adapter から tool kind を推定する。
5. Prompt タブでは `description` を移行元として表示する。
6. 保存時に `tool_kind` と `prompt.instructions` を明示的に書き込む。

期待結果:

- 既存 Agent は一覧から消えない。
- 保存前は互換モードとして扱われる。
- 保存後は新しい構造に正規化される。
- `description` は表示用 summary / dispatch hint として残る。

要件確認:

- R1: 新旧 Agent Profile を同じ Agent として扱える。
- R5: Prompt の移行方針が明確である。
- R7: 推定や不足が warning として見える。

### UC9. Installed skill catalog から Agent に skill を追加する

目的:

ユーザーは Project に定義済みの skill set を確認しながら、Agent に追加したい。

シナリオ:

1. Agent 作成または編集画面を開く。
2. Skills section で installed skill set の一覧を見る。
3. skill id、対象 path、`required_capabilities`、`optional_capabilities` を確認する。
4. 検索欄に `rust` や `test` を入れて候補を絞り込む。
5. `rust` と `test-runner` を checkbox で選択する。
6. 選択済み skill が chip で表示されることを確認する。
7. 保存する。

期待結果:

- 未定義の skill set id を手入力しなくてよい。
- skill set の capability requirement が見える。
- 選択済み skill が保存前に見える。
- 選択した skill set は Agent Profile に保存される。
- Agent 一覧では skill が chip として確認できる。
- Project Rule 由来の skill と Agent 固有 skill の merge は実行時 context で扱われる。

要件確認:

- R4: Agent-specific skill を設定できる。
- R7: skill の適用可否が見える。
- R9: Skill catalog が明示される。
- R10: Settings UI が Agent / Skill の設定ミスを減らす。

### UC10. Agent readiness check を行う

目的:

ユーザーは Agent を実行する前に、今その Agent が使えるか確認したい。

シナリオ:

1. Agent 詳細を開く。
2. Runtime タブで readiness check を実行する。
3. Nagare は runtime healthcheck、adapter capability、model/base_url、skill applicability を確認する。

期待結果:

- Codex CLI が見つからない場合は runtime warning が出る。
- OpenClaw + Ollama で Base URL が疎通できない場合は model warning が出る。
- required capability が足りない skill は skipped として表示される。
- Agent 一覧の Status に readiness summary が出る。

要件確認:

- R2: tool/provider-specific な不足を検出できる。
- R7: readiness を UI に表示できる。
- R9: skill capability requirement と照合できる。

### UC11. Dispatch preview で Agent 選択を確認する

目的:

ユーザーは Work Item を進める前に、どの Agent が選ばれるか確認したい。

シナリオ:

1. Work Item 詳細で Dispatch Preview を開く。
2. Nagare は Domain、Project Rule、Agent scope、Agent skills を使って候補を表示する。
3. ユーザーは selected target と fallback reason を確認する。

期待結果:

- Codex / OpenClaw を横断した candidate list が見える。
- Agent 固有 skill と Project Rule skill が分けて表示される。
- domain-specific Agent がない場合は fallback policy が見える。
- 実行前に Agent 管理の設定ミスを発見できる。

要件確認:

- R4: skill set merge が確認できる。
- R8: tool-independent routing を確認できる。
- R10: Dispatch Preview が使い勝手の確認点になる。

### UC12. 外部 catalog から skill を探して Agent に追加する

目的:

ユーザーは ClawHub や Vercel-style skills のような外部 catalog から、自分の Agent に必要な skill を探して追加したい。

シナリオ:

1. Agent 詳細の Skills タブを開く。
2. Add Skill を押す。
3. Catalog source で `hub` を選ぶ。
4. keyword に `react` や `code review` を入れて検索する。
5. Nagare は skill package の候補を表示する。
6. ユーザーは required capabilities、対応 tool、source、version を確認する。
7. skill package を install する。
8. installed package が提供する skill set を Agent に追加する。

期待結果:

- ユーザーは skill set id を知らなくても探せる。
- install 前に source と trust warning が見える。
- install 後、Project は package source/ref/checksum を lock する。
- Agent Profile には installed skill set id が保存される。
- 実行時には Agent-specific skill と Project Rule skill が合成される。

要件確認:

- R9: Skill catalog が package source も扱える。
- R11: Installable skill package を扱える。
- R12: discover and install の導線がある。

現在の実装:

- remote catalog の検索・ダウンロードは次フェーズとし、MVP では provenance を記録する。
- `nagare skill add --from clawhub|vercel --source <name-or-repo> --ref <ref> --checksum <sha>` で `[skill_packages.<id>]` と `[skill_sets.<id>]` をProjectに保存する。
- 登録済み skill set は Settings Agent form の Skill Picker に表示され、Agent Profile の `skill_set_ids` として選択できる。

### UC12a. Agent 作成中に ClawHub から skill を追加する

目的:

ユーザーは OpenClaw Agent を作りながら、ClawHub にある skill をその場で探して追加したい。

シナリオ:

1. Create Agent を開く。
2. Tool に `OpenClaw` を選ぶ。
3. Model provider と model を設定する。
4. Skills step で Add Skill を押す。
5. Source は `ClawHub` が候補として表示される。
6. keyword に `browser`, `code review`, `react` などを入力する。
7. Nagare は ClawHub の候補を表示する。
8. 候補には name、summary、対応 tool、required capabilities、trust 表示が出る。
9. ユーザーは skill を選び、Install and add to this agent を押す。
10. Nagare は package を install / lock し、提供される skill set を Agent に追加する。
11. Agent 作成確認画面で「この Agent で使う Skills」に追加済み skill が表示される。
12. Agent を保存する。

期待結果:

- Agent 作成フローから離れずに skill を追加できる。
- ClawHub 由来であることは表示されるが、package/ref/checksum は詳細に隠れる。
- OpenClaw で使えない skill は disabled または warning になる。
- install 済みでない skill は、その場で install して Agent に追加できる。
- 保存後、Agent Profile には skill set id が保存され、Project には package lock が残る。

ユーザーに見える言葉:

```text
Skills for this agent
  Browser automation
  React review
  Code review checklist
```

内部で起きること:

```text
ClawHub package -> install -> lock source/ref/checksum -> provided skill set -> AgentProfile.skill_set_ids
```

要件確認:

- R11: hub source を扱える。
- R12: Agent 作成中に discover/install/add が完結する。
- R7: tool compatibility と capability warning が見える。

### UC12b. Agent 作成中に Vercel-style skills を追加する

目的:

ユーザーは frontend / React / Next.js 用 Agent を作りながら、Vercel-style skills を追加したい。

シナリオ:

1. Create Agent を開く。
2. Role に `frontend-worker` または `reviewer` を設定する。
3. Domain に `frontend` を選ぶ。
4. Skills step に Recommended skills が表示される。
5. `React best practices`, `Next.js performance`, `UI review` などが候補に出る。
6. ユーザーは必要な skill を複数選ぶ。
7. 未インストールの skill があれば Install and add を押す。
8. Agent 作成確認画面で選択済み skills を確認する。
9. 保存する。

期待結果:

- ユーザーは Vercel-style skills の package 構造を知らなくても選べる。
- Role / Domain / Tool に応じて recommended skills が出る。
- すでに installed の skill は即追加できる。
- 未インストールの skill は install 後に自動で Agent に追加される。
- `package` と `skill set` の違いは通常表示では隠れる。

要件確認:

- R12: keyword 検索だけでなく recommended flow がある。
- R11: external / imported skill package を扱える。
- R4: Agent-specific skill として保存される。

### UC13. Git / local path から skill package を追加する

目的:

ユーザーは公開 catalog にない skill を Git repository や local path から追加したい。

シナリオ:

1. Skill catalog 画面で Add Source を押す。
2. source kind に `git` または `local` を選ぶ。
3. Git URL / ref、または local path を入力する。
4. Nagare は manifest を読み、provided skill sets を表示する。
5. ユーザーは install / register する。
6. Agent 詳細でその skill set を選択する。

期待結果:

- 外部 hub に登録されていない skill も使える。
- Git source は resolved ref と checksum を記録する。
- local source は Project-local または user-local として表示される。
- manifest が不正なら登録を拒否する。

要件確認:

- R11: `git` / `local` source kind を扱える。
- R12: install 前に package 内容を確認できる。
- R13: 自作 skill の取り込み口になる。

### UC14. 自分で skill を開発する

目的:

ユーザーは、自分のチームや Project に合わせた skill を作り、Agent に追加したい。

シナリオ:

1. `nagare skill init my-review-skill` を実行する、または UI から New Local Skill を作る。
2. Nagare は skill manifest と instructions template を作る。
3. ユーザーは required capabilities、paths、instructions を編集する。
4. `nagare skill validate` を実行する。
5. sample Work Item で dry run する。
6. local catalog に登録する。
7. Agent の Skills タブで追加する。

期待結果:

- skill package の最小構造が分かる。
- manifest validation により壊れた skill を登録しにくい。
- 自作 skill も外部 catalog 由来 skill と同じ一覧に出る。
- Agent への追加方法は外部 skill と同じになる。

要件確認:

- R13: local skill development ができる。
- R9: catalog で自作 skill と外部 skill を同じように扱える。
- R7: required capabilities の不足が readiness check に出る。

現在の実装:

- `nagare skill add --from skill-creator --path <skill-folder>` は `SKILL.md` frontmatter の `name` を package id / skill set id として推定する。
- `--requires` / `--optional` / `--paths` で Agent 実行時の capability と対象 path を登録できる。
- Settings の Add Skill 画面からも同じ `/api/skills` に登録できる。

## Usability Decisions

### Agent List

Agent 一覧は「設定ファイルの一覧」ではなく、比較表にする。

表示列:

- Agent
- Role
- Tool
- Model
- Skills
- Scope
- Status
- Actions

Tool ごとに別画面に分けない。Nagare の価値は環境横断の比較と選択にあるため、Codex / Codex CLI / OpenClaw は同じ一覧に並べる。

### Agent Detail

Agent 詳細はタブ構成にする。

- Overview: name, role, working_dir, tool
- Model: provider, model, base_url
- Skills: assigned skill sets, applied/skipped preview
- Routing: Domains, domains, dispatch hints
- Prompt: instructions, version, preview
- Runtime: adapter, external binding, healthcheck, probe

### Agent Creation Flow

Agent 作成は、設定フォームではなく短い wizard として扱う。

```text
Step 1. Basics
  name, role, Domain / Artifact Type scope

Step 2. Tool and Model
  Codex / Codex CLI / OpenClaw
  provider, model, base_url when needed

Step 3. Skills
  installed skills
  recommended skills
  search external catalog
  add Git/local source from advanced options

Step 4. Prompt
  instructions
  optional preview later

Step 5. Review
  tool, model, selected skills, warnings
```

Skills step では、ユーザーの主操作を「この Agent で使うスキルを選ぶ」に限定する。

```text
Selected for this agent
  [React best practices] [Rust test runner] [Code review checklist]

Recommended
  Based on role, domain, tool, and installed catalog.

Search
  Search installed skills, ClawHub, Vercel-style skills, Git/local sources.
```

Advanced details に隠す情報:

- package id
- source uri
- resolved ref
- checksum
- installed path
- exact provided skill set ids

保存前の Review step では、次のように確認できるようにする。

```text
Agent: Frontend Worker
Tool: OpenClaw
Model: OpenAI / gpt-5.3
Skills:
  - React best practices
  - Next.js performance
  - UI review
Warnings:
  - None
```

### Tool-Aware Form

フォームは選択された tool/provider に応じて変化する。

```text
Codex
  Show: OpenAI model
  Hide: provider selector, base_url

Codex CLI
  Show: OpenAI model, working_dir
  Hide: base_url

OpenClaw + OpenAI
  Show: provider, OpenAI model
  Hide or optional: base_url

OpenClaw + Ollama / LM Studio
  Show: provider, model, base_url
  Require: base_url
```

### Skill Assignment

Skills タブでは、単なる multi-select だけにしない。

通常表示:

- Selected for this agent
- Recommended
- Installed
- Search

表示する情報:

- skill set id
- paths
- required capabilities
- optional capabilities
- this agent can apply / will skip
- skip reason

通常表示では skill set id より display name を優先する。skill set id、package id、source ref は details に隠す。

### Skill Package And Skill Set

UI とデータモデルでは、Skill Package と Skill Set を分ける。

```text
Skill Package
  Distribution unit.
  Example: clawhub/react-review, git:https://..., local:./skills/my-skill

Skill Set
  Execution unit assigned to Agent or Project Rule.
  Example: react-review, accessibility-check, rust-test-runner
```

ユーザー操作は次の順にする:

1. 探す: external catalog / git / local から package を見つける。
2. 入れる: package を install / register する。
3. 選ぶ: package が提供する skill set を Agent に追加する。
4. 確かめる: readiness check で applied / skipped を見る。

この分離により、Vercel-style skills、ClawHub、自作 local skill を同じ Agent Skills UI で扱える。

### Skill Trust And Reproducibility

Skill は Prompt や実行指示に影響するため、source と version を明示する。

Project に保存する情報:

- source kind
- source uri
- version / ref
- resolved ref
- checksum
- installed path

UI では trust level を表示する:

- bundled
- verified
- project-local
- user-local
- unverified external

MVP では trust 判定を厳密に自動化しなくてもよい。ただし、unverified external skill を追加する場合は警告を出す。

### Prompt Editing

Prompt は最初から複雑な version management にしない。

MVP:

- instructions
- version string
- preview

後続:

- template id
- role/style/guardrails split
- prompt history
- A/B comparison

## Data Model Direction

既存の `AgentProfile` に最小追加する候補:

```rust
pub struct AgentProfile {
    pub id: String,
    pub display_name: String,
    pub runtime: String,
    pub adapter: String,
    pub role: String,
    pub working_dir: String,
    pub description: String,
    pub specialties: Vec<String>,
    pub tool_kind: AgentToolKind,
    pub skill_set_ids: Vec<String>,
    pub domain_ids: Vec<String>,
    pub artifact_type_ids: Vec<String>,
    pub model: AgentModelSelection,
    pub external: ExternalAgentBinding,
    pub prompt: AgentPromptConfig,
}

pub struct AgentPromptConfig {
    pub instructions: String,
    pub version: String,
}
```

Skill package の方向性:

```rust
pub struct SkillPackageDeclaration {
    pub id: String,
    pub display_name: String,
    pub source_kind: SkillSourceKind,
    pub source_uri: String,
    pub version: String,
    pub resolved_ref: String,
    pub checksum: String,
    pub installed_path: String,
    pub provided_skill_sets: Vec<String>,
    pub trust_level: SkillTrustLevel,
}

pub enum SkillSourceKind {
    Bundled,
    Local,
    Git,
    Hub,
    Imported,
}

pub enum SkillTrustLevel {
    Bundled,
    Verified,
    ProjectLocal,
    UserLocal,
    UnverifiedExternal,
}
```

`tool_kind` は明示的に持つ。現状は runtime / adapter から tool kind を推定できるが、UI や validation を安定させるには明示フィールドがある方が扱いやすい。

```rust
pub enum AgentToolKind {
    Codex,
    CodexCli,
    OpenClaw,
}
```

ただし、runtime / adapter と矛盾する可能性があるため、保存時 validation が必要になる。

推奨決定:

- `tool_kind` は保存する。
- 読み込み時、既存 Agent に `tool_kind` がない場合は runtime / adapter から推定する。
- 保存時、`tool_kind` と runtime / adapter の組み合わせが矛盾したら拒否する。
- `description` は表示用 summary / dispatch hint として残す。
- 実行 Prompt は `prompt.instructions` を優先する。
- `prompt.instructions` が空の場合のみ、互換処理として `description` を使う。

## MVP Scope

最初に実装する範囲:

1. Agent Profile に `tool_kind` と `skill_set_ids` を追加する。
2. 既存 Agent の `tool_kind` を runtime / adapter から推定する互換処理を入れる。
3. Agent add/update/list/show と Settings Agent form で skill set を扱う。
4. 実行時に `project_rule.skill_sets + agent_profile.skill_set_ids` を合成する。
5. Agent 一覧に Tool / Model / Skills / Scope / Actions を出す。
6. Tool/provider に応じた Model form 表示と validation を整理する。
7. Installed skill catalog を Agent form の Skills section で検索・選択可能にする。
8. Settings tabs / Agent list / Agent form の基本アクセシビリティと mobile 表示を補強する。
9. Local / Git / ClawHub / Vercel / skill-creator 由来の skill package を登録できる最小導線を追加する。
10. Skill package source/ref/checksum を記録できる形を用意する。
11. Settings の Add Skill 画面から `/api/skills` で登録できる。
12. Agent 作成中の Skills step で installed / selected を表示する。

次フェーズ:

1. Prompt を `AgentPromptConfig` に分け、`description` から互換移行する。
2. Agent readiness check を追加する。
3. Dispatch Preview を Agent/Work Item 設定確認の導線にする。
4. Prompt Preview を追加する。
5. Skill applied/skipped preview を Agent 詳細に追加する。
6. Prompt version/history を追加する。
7. Hub / Git source からの download / verify / vendor install を追加する。
8. Skill package scaffold / validate / dry run を追加する。
9. ClawHub / Vercel-style skills の remote search を追加する。
10. 未インストール skill は install 後に自動で current Agent に追加する。

## Complexity Assessment

比較的簡単:

- `skill_set_ids` の保存
- project rule skill と agent skill の merge
- Agent 一覧の表示列追加
- OpenClaw provider による Base URL 表示制御
- runtime / adapter から既存 Agent の tool kind を推定する処理
- local skill package の登録
- installed skill package の一覧表示
- installed skill catalog の検索 / checkbox 選択 UI
- Agent 一覧の情報圧縮と mobile card 表示
- Settings tabs の ARIA / focus-visible / hover 補強

中程度:

- tool/provider-specific validation
- Agent 詳細のタブ化
- skill applied/skipped の事前表示
- Agent readiness check
- git source の install / lock
- skill manifest validation

重い:

- Prompt Preview のための prompt composition 切り出し
- Prompt version/history
- Tool ごとの model catalog 自動取得
- Dispatch Preview の候補比較 UI
- external hub の検索 / install / trust 表示

実装順としては、skill 管理と model form 整理を先に行い、Prompt Preview は prompt composition の責務を整理してから入れるのが安全である。

## Open Questions

1. `specialties` と `skill_set_ids` の UI 上の違いをどう説明するか。
2. OpenAI model list は固定候補にするか、外部取得にするか。
3. Agent-specific skill は dispatch prompt の候補 summary に含めるか、実行時 context のみに使うか。
4. Skill catalog の編集を Settings UI で扱うか、MVP では TOML 編集に寄せるか。
5. Readiness check を保存時に自動実行するか、手動操作にするか。
6. Hub source は ClawHub 固有 adapter と汎用 registry adapter のどちらから始めるか。
7. Skill package は Project-local に vendor するか、user-global store を参照するか。
8. 自作 skill の publish 形式を Nagare 独自 manifest にするか、既存 skill ecosystem の manifest を包む形にするか。
