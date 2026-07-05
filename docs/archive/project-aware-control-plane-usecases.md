# Nagare Project-aware Control Plane UI Use Cases

**作成日:** 2026-06-17
**対象:** Nagare の新構想に基づく UI 試作
**前提:** Nagare は Agent 単体の管理画面ではなく、Project-aware Agent Runtime Control Plane として、Project 文脈に応じて Agent / Runtime / Session / Skill / MCP / Tool / Policy / Trace を解決する。

---

## 1. UI の中心を変える

従来の中心は Work Item と Agent だった。新構想では、ユーザーが最初に知りたいことは「この Project では、どの観点で、誰が、どの Runtime/Session と道具を使って動くのか」である。

そのため UI の一次オブジェクトを次の順に置く。

1. **Project**: 作業対象、目的、制約、許可された Domain / Rubric / Skill / MCP / Tool を束ねる。
2. **Assignment**: Project x Domain x Agent の担当関係。役割、Runtime、Session 方針、追加プロンプト、追加道具を持つ。
3. **Capability Surface**: Skill / MCP / Tool がどのスコープで許可、拒否、付与、実行可能になっているかを解決した結果。
4. **Runtime Session**: Codex Thread、OpenCode Session、Claude Code Session、OpenClaw Session などへの接続状態。
5. **Trace**: なぜその Domain / Agent / Runtime / Tool が選ばれ、何が実行され、何を人間が確認すべきか。

Agent は重要だが、単独で完結する管理対象ではなく、Project と Assignment の中で使われる専門家として扱う。

---

## 2. ユースケース一覧

### UC-01 Project を選び、作業文脈を確認する

ユーザーは Nagare を開き、現在の Project が何で、どの Domain / Rubric / Policy が有効かを確認する。

**知りたい問い**

- これはどの Project の制御面か。
- この Project の目的、制約、作業ルートは何か。
- どの Domain と Rubric が有効か。
- いま不足している設定は何か。

**主要画面:** Project Control Plane Overview, Project Context

### UC-02 Domain / Rubric を Project に適用する

ユーザーは Project に必要な Domain を選び、Domain Rubric と Project Rubric の合成結果を確認する。

**知りたい問い**

- この Project では何の観点で作業・評価するのか。
- 汎用 Domain Rubric に Project 固有の基準がどう追加されているか。
- Rubric の不足や衝突はあるか。

**主要画面:** Project Context, Assignment Board

### UC-03 Project x Domain x Agent の Assignment を作る

ユーザーは Domain ごとに担当 Agent を割り当てる。割り当てには役割、Runtime、Session 方針、追加 Prompt、追加 Skill / MCP / Tool が含まれる。

**知りたい問い**

- この Domain を誰が担当するのか。
- Agent はどの Runtime で動くのか。
- 新規 Session か、既存 Session 継続か。
- 担当が未設定、重複、過剰になっていないか。

**主要画面:** Assignment Board

### UC-04 Skill / MCP / Tool の有効範囲を制御する

ユーザーは Registry にある Skill / MCP / Tool を Project、Domain、Agent、Assignment、Runtime、Session のスコープで制御する。

**知りたい問い**

- その道具は登録済みか。
- Project で許可されているか、拒否されているか。
- Domain や Agent の制約に合うか。
- Runtime 上で実際に呼び出せるか。
- 他の Project や Agent を汚染していないか。

**主要画面:** Capability Scope Resolver

### UC-05 Runtime Session を作成、継続、分岐、停止する

ユーザーは Codex Thread、OpenCode Session、Claude Code Session、OpenClaw Session などを共通の Runtime Session として確認し、必要に応じて再開、分岐、停止する。

**知りたい問い**

- どの Runtime にどの Session があるか。
- その Session はどの Project / Domain / Agent / Task に結びついているか。
- 継続してよいか、新規にすべきか、分岐すべきか。
- 権限や接続状態に問題がないか。

**主要画面:** Runtime Session Bindings

### UC-06 Organizer が依頼を解決して実行する

ユーザーは自然文で依頼し、Organizer は Project、Domain、Rubric、Assignment、Runtime、Session、Capability Surface、Policy を順に解決して Runtime Adapter に送る。

**知りたい問い**

- 依頼はどの Project / Domain に解釈されたか。
- どの Assignment が選ばれたか。
- どの Session に送られたか。
- どの Skill / MCP / Tool が使える状態だったか。
- 人間の承認が必要か。

**主要画面:** Project Control Plane Overview, Trace Inspector

### UC-07 Policy / Approval を確認する

ユーザーは allow / ask / deny の判定と、承認待ちの理由を確認する。

**知りたい問い**

- 何が許可され、何が拒否されているか。
- なぜ承認が必要なのか。
- 承認するとどの Runtime / Session / Tool に影響するか。
- 危険操作が誤って通っていないか。

**主要画面:** Capability Scope Resolver, Trace Inspector

### UC-08 Trace から実行理由と結果を確認する

ユーザーは実行後に、Agent 選定理由、Rubric、Tool Call、Diff、Test、Review、Score を追跡する。

**知りたい問い**

- 結局どの Agent が何をしたのか。
- なぜその Agent / Runtime / Tool が使われたのか。
- 結果は Project / Domain / Rubric を満たしているか。
- 次に人間が見るべきことは何か。

**主要画面:** Trace Inspector

### UC-09 欠落・衝突・接続失敗を解決する

ユーザーは Runtime 未接続、Tool 未許可、Session 期限切れ、Rubric 未設定などの問題を解決する。

**知りたい問い**

- 何が原因で実行できないのか。
- どのスコープを直せばよいか。
- 修正は Project 限定か、Agent 限定か、Runtime 限定か。

**主要画面:** Overview, Capability Scope Resolver, Runtime Session Bindings

---

## 3. 画面構成

| 画面 | 目的 | 対応ユースケース |
| --- | --- | --- | --- |
| `30-project-control-plane-overview.svg` | Project を中心に、解決パイプライン、未対応、最近の Trace を一覧する | UC-01, UC-06, UC-09 |
| `31-project-context.svg` | Project の目的、Domain、Rubric、Policy、Assignment 不足を確認する | UC-01, UC-02 |
| `32-assignment-board.svg` | Domain ごとの担当 Agent、Runtime、Session 方針を調整する | UC-03 |
| `33-capability-scope-resolver.svg` | Skill / MCP / Tool のスコープ別判定と Effective Capability を見る | UC-04, UC-07, UC-09 |
| `34-runtime-session-bindings.svg` | Runtime Session / Thread の紐づきと継続方針を見る | UC-05 |
| `35-trace-inspector.svg` | 実行理由、選定、Tool Call、承認、成果を説明する | UC-06, UC-07, UC-08 |

---

## 4. 起動から最初の処理までの画面再検討

旧構想では、初回導線の主な関門は `AI接続ミニチェック` だった。新構想では、Nagare が最初の処理を安全に始めるには、単に AI が接続されているだけでは足りない。

最初の処理までに解決すべきものは次の順になる。

```text
Launch
  -> Project Resolver
  -> Runtime Adapter Readiness
  -> Work Item Composer
  -> Organizer Preflight
  -> Work Item Run Trace
  -> Result Review
```

### 4.1 初回導線で必要な画面

| 順 | 画面 | 主目的 | 必須条件 | 既存SVGとの関係 |
| --- | --- | --- | --- | --- |
| 1 | Launch / Project Resolver | カレントフォルダ、既存Project、未登録Projectを見分け、作業文脈を確定する | Project を1つ選ぶ、または作る | 新規画面が必要 |
| 2 | Project Readiness Overview | Project の目的、Domain、Rubric、Policy、Assignment 不足を見せる | 致命的な不足がなければ依頼へ進める | `30`, `31` を初回向けに使える |
| 3 | Runtime Adapter Readiness | Codex / Codex CLI / OpenCode / Claude Code / OpenClaw のうち、実行可能な Runtime を確認する | 少なくとも1つの Runtime が実行可能 | `34` を初回向けに簡略化した新規画面が必要 |
| 4 | Work Item Composer | ユーザーが依頼を1つ入力する。Project / Domain / Scope は推定する | 依頼本文だけ必須 | 新規画面が必要 |
| 5 | Organizer Preflight | Organizer が解決した Project、Domain、Assignment、Runtime、Session、Capability、Policy を実行前に確認する | deny がない、ask は承認済み | `30`, `32`, `33`, `34` の要約画面として新規画面が必要 |
| 6 | Work Item Run Trace | Dispatcher / Organizer から Agent 実行、Tool Call、Review までを追う | 実行中状態と人間待ちが分かる | `35` を実行中状態に拡張 |
| 7 | Result Review | 最初の処理の結論、成果物、Rubric 結果、次の改善候補を見る | 採用、差し戻し、続行ができる | `35` の完了状態、または別画面 |

### 4.2 最小導線

初回でも、ユーザーに設定作業を長く要求しない。最小導線は次の通り。

1. **Project Resolver**
   - 既存 Project があれば選択して開始する。
   - 未登録フォルダなら Project 名、root、目的を自動推定し、確認だけ求める。
   - Domain / Rubric / Assignment は初期候補を作り、後で修正できる。
2. **Runtime Adapter Readiness**
   - 実行可能な Runtime が1つでもあれば、最初の依頼へ進める。
   - Runtime が0件なら、接続/インストール画面を出す。
   - OpenClaw など未導入 Runtime は警告にし、主導線を止めない。ただし、その Runtime を要求する Assignment は実行不可にする。
3. **Work Item Composer**
   - 必須入力は依頼本文のみ。
   - Project、Domain、Scope、完了条件、Rubric、Agent候補、Capability候補は推定する。
4. **Organizer Preflight**
   - 実行前に `解決結果` だけを見せる。
   - Project / Domain / Assignment / Runtime / Session / Capability / Policy を1列または1本のフローで表示する。
   - `ask` 権限がある場合だけ承認UIを出す。
5. **Work Item Run Trace**
   - Dispatcher / Organizer の判断、Agent実行、Tool Call、Review を1ステップずつ表示する。
   - raw log は初期表示しない。
6. **Result Review**
   - 結論、成果物、Rubric結果、人間の次行動を出す。
   - 改善候補として Skill / MCP / Assignment / Rubric の追加を提案する。

### 4.2.1 完全な空状態からの導線

ユーザーのローカル状態に、Project、Agent、Domain、Runtime が何も登録されていない場合は、通常の Project Resolver より前に `Empty State Bootstrap` が必要になる。これは独立したフルページではなく、Control Plane 全体画面の上に表示するセットアップ用モーダルとして扱う。

この状態では、Nagare は次のように扱う。

| 対象 | 空状態での扱い | ユーザーに求めること |
| --- | --- | --- |
| Project | カレントフォルダから暫定 Project を提案する | Project 名と root の確認だけ |
| Domain | 製品同梱の最小 Domain テンプレートを候補として出す | 初回は選ばせず、`general` と推定 Domain で開始 |
| Rubric | 製品同梱の汎用 Rubric を初期適用する | 初回は編集させない |
| Agent | Runtime 接続後に `default-worker` を自動作成する | 名前やPrompt編集は後回し |
| Runtime | PATH / config / known install path を検出する | 0件なら1つだけ接続またはインストール |
| Skill / MCP / Tool | Catalog は空でもよい。初回実行には必須にしない | 依頼内容に応じて後で提案 |
| Assignment | Project + Domain + default-worker + Runtime から自動作成する | Preflight で確認だけ |
| Session | 最初の実行時に新規作成する | 再利用/分岐の判断は不要 |

完全な空状態での最小フロー:

```text
Launch
  -> Empty State Bootstrap
  -> Project Select / Create
  -> Runtime Select / Connect
  -> Auto-create default Domain / Rubric / Agent / Assignment
  -> Work Item Composer
  -> Organizer Preflight
  -> Work Item Run Trace
  -> Result Review
```

ユーザー操作としては、次の順にする。

1. **起動**
   - Control Plane 全体画面を表示する。
   - 既存 Project が0件なら、画面上にセットアップ用モーダルを表示する。
   - 背面の全体画面では `setup required` と `runtime 0` だけ分かればよい。
2. **Project Select / Create**
   - モーダル内で `プロジェクト名`, `Gitリポジトリ / 作業フォルダ` を入力・確認する。
   - `プロジェクト用の評価基準` は任意設定にし、未設定でも汎用評価基準で次へ進める。
   - 評価基準は折りたたみカードとして表示し、編集前は見出しと `編集` ボタンだけを見せる。
   - 主CTAは `次へ` に絞る。
   - フッター操作は左に `戻る`、右に `次へ` を置き、上部ステップ表示の左から右への流れと揃える。
   - `このプロジェクトを作成` と `このフォルダで開始` のような同義CTAを並べない。
   - Domain、Rubric、Agent、Skill、MCP はこの時点では細かく聞かない。
3. **Runtime Select / Connect**
   - Codex / Codex CLI / OpenCode / Claude Code / OpenClaw を自動検出する。
   - 検出できた Runtime があれば、それを既定 Runtime として提案する。
   - 0件なら `Runtimeを1つ接続` を主CTAにする。
   - Runtime 候補はドロップダウンで1つ選ぶ。
   - ドロップダウンには検出済み Runtime だけを表示する。
   - 下の設定パネルは選択された Runtime に応じて切り替える。
   - 未検出 Runtime のインストール案内や再検出は、この選択画面ではなく診断/接続導線で扱う。
   - 認証、Provider、Base URL などは、それが必要な Runtime を選んだ時だけ入力を求める。
   - 接続テストに成功したら、`default-worker` Agent を自動作成する。

#### Runtime 別の設定表示

Runtime 設定画面は1つの共通フォームにし、ドロップダウンで選んだ Runtime に応じて下段の設定内容だけを差し替える。ユーザーに Runtime 固有の差分を覚えさせず、必要な入力だけをその場で出す。

| Runtime | モデル指定 | 追加で見せる項目 | UI方針 |
| --- | --- | --- | --- |
| Codex CLI | `codex debug models` の取得結果から選ぶ | なし | モデル選択ドロップダウンを出す |
| Codex | 既定モデル、または手入力 | なし | アプリ側の既定値を優先し、手入力は補助扱い |
| Claude Code | 既定モデル、または手入力 | なし | モデル一覧を取得しないため、ラジオボタンで `既定値` / `手入力` を選ぶ |
| OpenCode | Provider 選択後、その Provider のモデル候補から選ぶ | Provider | Provider とモデルのドロップダウンを出す。モデル欄には Provider 名を重ねて表示しない |
| OpenClaw | Provider 選択後、その Provider のモデル候補から選ぶ | Provider、必要時のみ Base URL | Provider とモデルのドロップダウンを出す。OpenAI Provider では Base URL を見せない |

対応SVG:

- `38-setup-wizard-runtime.svg`: Codex CLI
- `38a-setup-wizard-runtime-codex.svg`: Codex
- `38b-setup-wizard-runtime-claude-code.svg`: Claude Code
- `38c-setup-wizard-runtime-opencode.svg`: OpenCode
- `38d-setup-wizard-runtime-openclaw.svg`: OpenClaw

4. **Auto Bootstrap**
   - `general` Domain と汎用 Rubric を作る。
   - `default-worker` Agent を作る。
   - `general -> default-worker -> selected Runtime` の Assignment を作る。
   - Skill / MCP は空のままでも開始できる。
5. **Work Item Composer**
   - ユーザーは依頼文だけ入力する。
   - Domain は `general` から始め、依頼内容で推定できる場合だけ候補表示する。
6. **Organizer Preflight**
   - 自動作成された Project / Domain / Agent / Runtime / Assignment を見せる。
   - ユーザーが「なぜこのAgentなのか」を理解できるよう、`初回既定構成` と明示する。
7. **Work Item Run**
   - 新規 Session を作成して実行する。
   - Trace に `Auto bootstrap` を記録する。

この導線では、初回にユーザーが作る必要があるものは最大でも次の2つだけにする。

1. Project
2. Runtime 接続

Agent、Domain、Rubric、Assignment は、初回実行のための暫定構成として Nagare が作る。これは隠蔽ではなく、Preflight と Trace で `自動作成されたもの` として説明する。

完全な空状態で詰まる条件:

- Runtime が1つも検出できず、ユーザーも接続できない。
- Runtime 接続テストに失敗する。
- カレントフォルダにアクセスできない。
- Runtime が要求する認証情報が未設定。

この場合でも、Project Draft は保存できるようにし、`Runtime未接続のため実行不可` と表示する。

### 4.3 初回で前面に出さないもの

次のものは重要だが、起動直後の主導線にはしない。

- 全 Domain / Rubric の詳細編集。
- 全 Skill / MCP / Tool のカタログ管理。
- Agent prompt の細かい改善。
- Runtime 固有の低レベル設定。
- Session / Thread の全ログ。
- モデルパラメータの詳細。

初回は「Project を確定し、最低1つの Runtime で最初の依頼を流し、Trace で説明できる」ことを最優先にする。

### 4.4 完全な空状態からWork Item実行までのSVG

| SVG候補 | 画面 | 表現すること |
| --- | --- | --- |
| `36-empty-state-bootstrap.svg` | Empty State Bootstrap | Project / Agent / Domain / Runtime が0件の時に、Control Plane上のモーダルで何から作るかを示す |
| `37-setup-wizard-project-runtime.svg` | セットアップ: プロジェクト選択 / 作成 | セットアップ開始後、プロジェクト名とGitリポジトリ / 作業フォルダを確認する。評価基準は任意設定にし、ランタイム接続は次のステップに分ける |
| `38-setup-wizard-runtime.svg` | セットアップ: ランタイム設定 | プロジェクト確定後、最初に使うランタイムを1つ選び、必要な接続情報だけ確認する |
| `39-work-item-composer.svg` | Work Item Composer | 依頼本文のみ必須、Project/Domain/Scope/完了条件は推定 |
| `40-organizer-preflight.svg` | Organizer Preflight | 実行前の解決結果、Policy ask/deny、Session方針 |
| `41-work-item-run-trace.svg` | Work Item Run Trace | OrganizerからAgent実行、Tool Call、Reviewまでの実行中表示 |

`42-result-review.svg` は、Work Itemが完了した後の画面として別途作る。今回の範囲は「実行できるまで」と「実行開始を確認できるまで」に絞る。

### 4.4.1 左ナビゲーションの考え方

左ナビはシステム内部の構成要素を並べない。ユーザーが日常的に選ぶ作業領域だけにする。
UIに表示する名称は日本語を基本にし、英語の内部概念名は必要な説明や開発資料に下げる。製品名、コマンド名、パッケージIDは固有名としてそのまま扱う。

| 左ナビ表示 | 内部概念 | 役割 | 含めるもの |
| --- | --- | --- |
| ワーク | Work Items | 依頼、実行前確認、実行中、レビュー、完了までの流れを見る | ワーク一覧、ワーク作成、実行前確認、実行トレース、結果レビュー |
| プロジェクト | Projects | プロジェクト文脈を管理する | プロジェクト概要、ドメイン、ルーブリック、割り当て、プロジェクト範囲の機能、プロジェクトポリシー |
| エージェント | Agents | エージェントを管理する | エージェント一覧、エージェント詳細、プロンプト、ランタイム設定、エージェント範囲のスキル/MCP |
| ライブラリ | Library | 共有カタログを管理する | スキル / MCP / ツール / ランタイムアダプタの登録元とバージョン |
| 設定 | Settings | アプリ全体の設定を管理する | 認証、保存先、既定ランタイム、環境設定 |

次の項目は左ナビに直接出さない。

- `Assignments`: Project の中の担当関係として表示する。
- `Resources`: `Library` では共有カタログ、Project内では Capability Surface として表示する。
- `Sessions`: Work Item Trace または Project の Runtime Binding 内で表示する。
- `Traces`: Work Item の詳細内で表示する。
- `Policies`: Project / Agent / Runtime / Session の各スコープ内で表示する。

### 4.5 初回導線の成功条件

- ユーザーが「まず何をすればいいか」を起動後すぐ理解できる。
- Project が未登録でも、依頼開始までに必要な入力が増えすぎない。
- Runtime が複数あっても、最初は `使える Runtime` と `使えない Runtime` の差だけ分かればよい。
- Skill / MCP / Tool は隠さないが、初回の主作業にしない。
- 実行直前には、どの Agent / Runtime / Session / Capability が使われるか説明できる。
- 最初の処理後には、なぜその Agent が動いたか Trace で説明できる。

---

## 5. ユースケースシナリオ

画面は単独の管理画面としてではなく、次のシナリオをユーザーが迷わず進めることを基準に作る。

### Scenario A: Project の状態を見て、今日の詰まりを解消する

**状況:** ユーザーは Nagare Project を開き、作業を始める前に、どの設定が実行を妨げるかを確認する。

**流れ**

1. `30-project-control-plane-overview.svg` で Active Project と Resolution Path を確認する。
2. Human Attention で `OpenClaw program not found`、`filesystem write approval`、`testing assignment gap` を把握する。
3. Runtime 問題は `34-runtime-session-bindings.svg` へ進む。
4. Assignment 不足は `32-assignment-board.svg` へ進む。
5. Tool / permission 問題は `33-capability-scope-resolver.svg` へ進む。

**成功条件**

- ユーザーが「いま何が詰まっているか」を 1 画面で説明できる。
- それぞれの詰まりが、どの画面で直すものか分かる。
- Agent や Runtime の内部ログを読まなくても次の操作が分かる。

### Scenario B: 新しい Domain を Project に追加し、担当 Agent を割り当てる

**状況:** Project に `testing` Domain は必要だが、まだ Assignment がない。

**流れ**

1. `31-project-context.svg` で Project が使う Domain と Rubric Stack を確認する。
2. `32-assignment-board.svg` で `testing domain が未割当` の Gap を見る。
3. `testing担当を作成` から Agent、Runtime、Session policy、追加 Capability を選ぶ。
4. 追加後、Coverage が `7/7` になり、依頼時の不透明な fallback が不要になる。

**成功条件**

- Domain / Rubric / Agent の関係が混ざらずに理解できる。
- Assignment が「Agent を選ぶ」だけではなく、Runtime と Session 方針を含むことが分かる。
- 未割当 Domain が依頼時に勝手に fallback されないことが分かる。

### Scenario C: Skill / MCP / Tool を Project 限定で安全に追加する

**状況:** ユーザーは `hachi-search` を Nagare Project で使いたいが、他 Project や他 Agent へ勝手に広げたくない。

**流れ**

1. `33-capability-scope-resolver.svg` で selected capability を確認する。
2. Scope Funnel で Registry、Project、Domain、Agent、Assignment、Runtime、Session の判定を順に確認する。
3. Effective Surface で実行時に見えるもの、ask、deny を確認する。
4. 必要な場合だけ `Assignmentへ付与` する。

**成功条件**

- 「インストール済み」と「この Assignment で使える」が別物だと分かる。
- Project 限定、Runtime target 限定、Session callable の違いが分かる。
- Global install によるスコープ汚染を避けられる。

### Scenario D: Runtime 固有の Thread / Session を共通の Binding として管理する

**状況:** Project には Codex、Claude Code、OpenCode、OpenClaw が混在している。ユーザーは Runtime 固有名ではなく、Project / Domain / Agent / Task の紐づきで見たい。

**流れ**

1. `34-runtime-session-bindings.svg` で Runtime ごとの Session / Thread を見る。
2. 各 Runtime の項目を `Project / Domain / Agent / Task` の共通 Binding として確認する。
3. OpenClaw の `not found` を見て、Runtime 切替かインストール手順へ進む。
4. Codex Thread は reuse、fork の可否を確認する。

**成功条件**

- Runtime 固有差分が一覧で自然に吸収されている。
- OpenClaw 未導入のエラーが「何をすればよいか」と一緒に見える。
- 継続、分岐、新規作成の判断ができる。

### Scenario E: 依頼後に、なぜその Agent が動いたかを確認する

**状況:** ユーザーは依頼の結果を見たが、どの Agent がなぜ選ばれ、どの Tool が使われたかを確認したい。

**流れ**

1. `35-trace-inspector.svg` の Step Timeline で Project resolved から Result reviewed までを追う。
2. Why This Agent で Domain match、Project fit、Runtime reuse、Tool surface を確認する。
3. Decision で結論と人間の承認待ちを確認する。
4. 必要なら詳細ログへ進む。

**成功条件**

- 「結局どの Agent が使われたか」が分かる。
- Agent 選定理由がログではなく判断単位で分かる。
- Tool、Session、Rubric result が一つの流れで説明できる。

---

## 6. 画面作成時の判定基準

各画面は、次の問いに答えられない場合は作り直す。

- この画面はどのシナリオの、どのステップを支えるのか。
- ユーザーが最初に見るべき状態は何か。
- 次に見るべき根拠は何か。
- 次の操作はどこにあるか。
- Project / Domain / Agent / Runtime / Session / Capability / Policy / Trace の境界が混ざっていないか。
- Runtime 固有の名前を見せる場合、それが共通 Binding とどう関係するか説明できるか。
- 詳細ログや内部イベントを初期表示に出しすぎていないか。

---

## 7. UI 原則

- **Project 起点:** Work Item や Agent の前に Project 文脈を見せる。
- **解決順序を可視化:** Project -> Domain -> Assignment -> Runtime Session -> Capability Surface -> Policy -> Trace の順に読む。
- **Scope を混ぜない:** Registry、Project、Domain、Agent、Assignment、Runtime、Session の違いを画面上で保持する。
- **主判断と監査を分ける:** 初期表示は「状態、担当、次の操作」。ログ、JSON、細かいイベントは Trace で掘る。
- **Runtime 差分を自然に吸収:** Codex Thread / OpenCode Session / Claude Code Session / OpenClaw Session は Runtime Session として同じ表で扱い、固有設定は詳細に下げる。
- **汚染を防ぐ:** Skill / MCP / Tool は Project や Assignment の限定範囲で有効化し、グローバルな副作用を初期操作にしない。
