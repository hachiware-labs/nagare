# design.md（UIデザイン方針）

この文書は Nagare / 流 の画面UIにおけるデザイン方針の正本である。
機能仕様は `docs/spec.md`、全体構成は `docs/architecture.md`、Agent /
Skill / Run Packet のデータ設計は `docs/agent_data_model.md` を参照する。

## 1. UIの目的

Nagare のUIは、Agentを眺めるための管理画面ではなく、Work Itemを進めるための
作業指揮画面である。

UIで人間が判断したいことは次の5つに集約する。

1. 何をやるのか。
2. どのAgentに任せるのか。
3. 何が実行されたのか。
4. どの根拠と検証が残っているのか。
5. 次に何をすべきか。

そのため、UIの中心は常に Work Item とする。WorkflowDecision、Dispatch、
Run、Review、Evidence、Verification、Recovery、Handoff、Decision は、
Work Item Detail の中で因果関係として見せる。

### 1.1 Hachiware Labsの製品としての方向性

Nagare は Hachiware Labs の製品として、同サイトの方向性を引き継ぐ。
参照元: `https://hachiware-labs.com/`

- 人とAIが自然に協働できることを最優先にする。
- 小さく試し、現場で使い、改善を回せるUIにする。
- 人の思考を止めないため、依頼開始、進捗確認、失敗時の回復を短くする。
- Agent内部の管理画面ではなく、仕事を進めるための道具として見せる。
- トーンは白地、slate系テキスト、indigo系primary action、8px程度の角丸、
  控えめな境界線と影を基本にする。

## 2. デザイン原則

### 2.1 作業ツールとして静かにする

Nagare は運用・レビュー・デバッグのための作業ツールである。画面は装飾的にせず、
情報密度と読み取りやすさを優先する。

- 大きなhero表現、マーケティング風の余白、装飾的なカード群は使わない。
- 色は状態・警告・選択を表すために使う。
- 画面タイトルよりも、Work Item、Agent、DispatchPlan、Runの状態を目立たせる。
- 一覧画面は比較しやすいテーブルまたは密なリストを基本にする。

### 2.2 Work Item中心にする

トップレベル画面をentityごとに増やしすぎない。Run Log、Evidence、
Verification、Handoff、Artifact は独立画面ではなく、Work Item Detail の
Timeline と Inspector で扱う。

### 2.3 因果関係を見せる

Nagareで重要なのは、個別ログではなく関係である。

- DispatchPlan が target agent を決める。
- accepted DispatchPlan が Run に使われる。
- Run が Artifact と Evidence を作る。
- Review が成果物と acceptance criteria を確認する。
- Verification が実行結果を確認する。
- RecoveryPlan が失敗理由と次の回復候補を示す。
- HandoffPacket が次のAgentへ渡す文脈を固定する。
- WorkflowDecision が、次に進めるべき操作と判断根拠を記録する。

UIではこれを Timeline と Inspector で表現する。

### 2.4 宣言と観測を分ける

Agent Profile の `description` / `specialties` は宣言である。
CapabilityProbe の capabilities / instruction_sources / warnings は観測である。
UIでは混ぜず、別セクションとして表示する。

### 2.5 3つの仕事導線を分ける

Nagare の主要導線は次の3つである。

1. ささっと依頼する。
   - 依頼文、対象path、任せ方だけで始められる。
   - すぐ実行ではなく、dispatch agentが作った依頼先確認へ進む。
2. あのタスクどうなった、を見る。
   - Work Itemの状態、担当Agent、最後のRun、次の操作を一覧で見る。
   - 「要確認」「失敗」「承認待ち」「実行中」は、Work Queue上部の状態ビューとして
     置く。これは分析用filterではなく、進捗確認のショートカットである。
3. タスクを分析しデバッグする。
   - DispatchPlan、AgentRun、Evidence、Artifact、Verification、Handoffを
     必要な時だけInspectorで掘る。
   - Agent、working_dir、Dispatch、Run、Verification、Probeによる絞り込みは、
     通常時の右ペインではなく、詳細フィルタまたは分析Drawerで扱う。

### 2.6 Debug Explorerとして絞り込める

Work Item Boardは単なる進捗一覧ではなく、Agent作業のdebug explorerである。
Agent、working_dir、Dispatch、Run、Verification、Probeで強く絞り込めることを
要件とする。

ただしDebug Explorerは主導線ではない。初期表示では「仕事を依頼」と
「作業キュー」を優先する。右ペインは選択中Work Itemの要約と次の操作に使い、
分析・デバッグは `詳細フィルタ` / `分析を開く` から呼び出す。

### 2.7 Nielsenのユーザビリティ原則に沿う

Nagare のUIは、ヤコブ・ニールセンの10 usability heuristicsを画面レビューの
基準として使う。各原則はNagareの業務画面に合わせて次のように適用する。

1. システム状態を見えるようにする。
   - Work Item status、latest run、accepted dispatch、verification state、
     handoff stateを常に見える位置に置く。
   - Agent Run中、Probe中、Verification中は進行中状態と開始時刻を表示する。
   - DispatchPlanがdraftなのかacceptedなのかを曖昧にしない。

2. 現実の作業語彙に合わせる。
   - 画面上の主語は Work Item、Agent Profile、DispatchPlan、Run、Evidence、
     Verification とし、内部実装語を不用意に出さない。
   - `target agent`、`review agent`、`dispatch agent` の違いを表示で明確にする。
   - Run PacketやSkill Contextなど内部寄りの情報は、通常表示では要約し、
     Inspectorで詳細を開けるようにする。

3. ユーザーが制御でき、戻れるようにする。
   - draft DispatchPlanはaccept前に確認できる。
   - accepted前はre-dispatch、override agent、保留ができる。
   - 破壊的操作や状態を進める操作は、直前に対象Work ItemとAgentを確認できる。
   - 失敗時はretry、handoff、artifact確認へすぐ戻れる。

4. 一貫性と標準を守る。
   - status badge、agent badge、warning badgeの色と配置を画面間で統一する。
   - Work Item Detail内のTimeline itemは、Dispatch / Run / Evidence /
     Verificationで同じ構造を使う。
   - primary actionは画面ごとに1つを基本にし、secondary actionはmenuに寄せる。

5. エラーを予防する。
   - `agent update` ではworking_dirがproject外にならないことをUIでも検知する。
   - dispatch accept前にselection_warnings、missing_information、risksを目立たせる。
   - default agent変更時はhealth/probe未確認を警告する。
   - `item run` 前に、どのDispatchPlanまたはAgent選定根拠で実行するかを表示する。

6. 記憶に頼らせず、見れば分かるようにする。
   - Work Item Boardの行にlatest run agent、accepted dispatch target、
     verification stateを表示する。
   - Work Item DetailではSummaryを常時表示し、Timelineを見ても文脈を失わない。
   - Inspectorには関連entityへのリンクを置き、IDを手で覚えさせない。

7. 熟練者にも効率よく使えるようにする。
   - filter chips、saved views、query syntaxを用意する。
   - Agent、status、dispatch warning、latest run failedなどで高速に絞り込める。
   - よく使う操作はkeyboard shortcutを将来追加できる設計にする。ただしMVPでは
     ショートカット説明を画面に大きく出さない。

8. 美的で最小限にする。
   - Work Itemを進める判断に不要な装飾を入れない。
   - Dashboard的な大きな数値カードをMVPの主画面にしない。
   - Timeline cardは要約を中心にし、raw logやJSONはInspectorに隠す。

9. エラーを認識・診断・回復できるようにする。
   - selection_warningsは「何が問題で、どのfallbackを使ったか」を表示する。
   - failed runはexit code、stderr、artifact、次の操作を同じInspectorに出す。
   - capability不足やprobe staleは、対象Agentと不足理由を明示する。
   - Probe staleやruntime unavailableはAgent Profile DetailとBoard filterで見つけられる。

10. 必要なヘルプとドキュメントを近くに置く。
    - 複雑な概念はtooltipやinline helpで短く説明する。
    - Dispatch Reviewでは、target選定、warnings、fallbackの意味を近くで説明する。
    - 詳細な説明は `docs/spec.md`、`docs/agent_data_model.md`、`docs/architecture.md`
      へリンクする。

この原則は、画面を増やす判断にも適用する。独立画面にすると状態の見通しや
因果関係が悪くなる場合は、Work Item Detail内のInspectorとして扱う。

### 2.8 現行ワークフローへの対応

現行実装では、Work Itemを進める中核操作は `item advance` である。
UIでは個別の `dispatch` / `run` / `review` / `verify` を並列に見せず、
まず `次へ進める` を主操作として置く。

UIで扱う一級要素:

- Work Item DoD。
  - acceptance criteria。
  - expected artifacts。
  - verification hint。
  - work_folder。
  - constraints。
- WorkflowDecision。
  - action、source、reason、requires_human、target_agent、confidence。
  - supervisor_agent 由来の判断は source と AgentRun を表示する。
- Advance result。
  - 実行したstep、停止理由、次に必要な人間操作。
- RecoveryPlan。
  - failure_class、action、target_agent、reason、command_hint、prompt_hint。
  - 複数候補がある場合は primary candidate と secondary candidate を分ける。
- ReviewResult。
  - verdict、findings、requested_changes。
  - criteria_results を acceptance criteria と並べて表示する。
- HandoffPacket。
  - current_state、open_questions、artifact_ids、diff_artifact_ids、
    failed_verification_ids、review_result_ids、next_request。

`次へ進める` は安全に進められる範囲だけを進める。人の判断が必要な場合、
UIは質問、承認、Agent上書き、RecoveryPlan採用、Handoff確認のいずれかを
次の操作として表示する。

## 3. トップレベル画面

### 3.1 Work Item Board

目的:

- ささっと仕事を依頼する。
- 依頼済みWork Itemの状態を素早く確認する。
- 問題のあるWork Itemを分析・デバッグできる状態にする。
- Work Item Detailへ移動する。

主な表示:

- Quick Request Composer。
  - 依頼文。
  - 対象path。
  - work_folder。
  - acceptance criteria。
  - expected artifacts。
  - verification hint。
  - constraints。
  - 任せ方または目的preset。
  - dispatch agent / candidate limit。
  - `依頼先を確認` primary action。
- Work Queue。
  - quick status views。
    - 要確認。
    - 失敗。
    - 承認待ち。
    - 実行中。
  - Work Item ID、title、status。
  - latest run status / latest run agent。
  - latest workflow decision。
  - accepted dispatch target。
  - dispatch warning count。
  - review verdict / criteria state。
  - verification state。
  - recovery state。
  - handoff count。
  - work_folder / path。
  - updated time。
- Selected Work Item Summary。
  - 選択中Work Item。
  - 現在状態。
  - 担当Agent。
  - latest workflow decision。
  - 次の操作。
  - recovery candidate有無。
  - `詳細を見る`。
  - `分析を開く`。
- Debug Filters / Analysis Drawer。
  - agent profile。
  - working_dir / work_folder。
  - dispatch state。
  - run result。
  - verification state。
  - handoff state。
  - probe state。

Work Queueの行表示:

- Work Item ID、title、status。
- latest run status / latest run agent。
- latest workflow decision。
- accepted dispatch target。
- review verdict / criteria state。
- dispatch warning count。
- verification state。
- recovery state。
- handoff count。
- work_folder / path。
- updated time。

主なfilter:

- status。
- agent profile。
- dispatch state。
- run result。
- verification state。
- review verdict / criteria state。
- recovery state。
- handoff state。
- working_dir / work_folder。
- probe state。
- updated time。
- text search。

優先saved view:

- Needs attention。
- Dispatch warnings。
- Failed latest run。
- Ready for review。
- Agent-specific view。
- Criteria failed。
- Recovery available。

デザイン:

- 上部にQuick Request Composerを置く。
- 中央はWork Queueの密なリストまたはテーブル。
- Work Queue上部に「すぐ確認」の状態ビューを置く。
- 右側は選択中Work Itemの要約、DoD、latest workflow decision、次の操作を置く。
- 行は1 Work Item = 1 compact rowを基本にし、statusと異常だけ色で強調する。
- Debug Explorerは初期表示の主役にしない。`詳細フィルタ` または
  `分析を開く` から開く。

### 3.2 Work Item Detail

目的:

- 1つのWork Itemの正本画面として扱う。
- WorkflowDecision、Dispatch、Run、Review、Evidence、Verification、
  Recovery、Handoff、Decisionを一続きの流れとして確認する。
- 追加指示、質問、Handoff、回復、再実行も同じTimeline上のイベントとして確認する。
- 次に必要な操作を実行する。

レイアウト:

```text
┌────────────────────────────────────────────────────────────┐
│ Header: title / status / workflow decision / primary action │
├───────────────┬──────────────────────────┬─────────────────┤
│ Summary       │ Execution Timeline        │ Inspector       │
│               │                          │                 │
│ description   │ Dispatch accepted         │ selected detail │
│ agents        │ ↓                         │ log/json/actions│
│ DoD/path      │ Run succeeded             │                 │
│ recovery      │ ↓                         │                 │
│               │ Review / criteria         │                 │
│               │ ↓                         │                 │
│               │ WorkflowDecision          │                 │
└───────────────┴──────────────────────────┴─────────────────┘
```

主な操作:

- next / advance。
- advance until blocked。
- decision preview。
- dispatch accept。
- run。
- review。
- verify。
- recovery create / accept / apply。
- handoff create。
- handoff dispatch。
- approve。

Action hierarchy:

- Headerのprimary actionは常に1つにする。
- primary actionはWork Itemの現在状態から決める。
- 通常時のprimary actionは `次へ進める` とする。
- 実行前に判断内容を確認すべき場合は `判断を確認` とする。
- dispatch draftの場合は `依頼先を確認`。
- dispatch acceptedかつ未実行の場合は `実行する`。
- run runningの場合は primary actionを出さず、進行中状態を表示する。
- run failedの場合は `回復案を見る` をprimaryにし、retry / handoffはsecondaryにする。
- review requested changesの場合は `回復案を見る` または `修正依頼を適用` をprimaryにする。
- criteria failedの場合は `未充足条件を見る` をprimaryにする。
- run succeededかつreview missingの場合は `レビューを実行`。
- review passedかつverification missingの場合は `検証を実行`。
- verification failedの場合は `回復案を見る` をprimaryにし、再検証 / handoff / artifact確認はsecondaryにする。
- review passedかつverification passedの場合だけ `承認する` をprimaryにする。
- approveできない状態では `承認` をprimaryにしない。表示する場合はdisabledにし、
  「検証後に承認可」のように理由を添える。

デザイン:

- Headerには状態と次アクションを置く。
- 左Summaryは固定幅で、Work Itemの文脈を常に見せる。
- 中央Timelineは縦方向の作業履歴。イベントを時系列に並べ、必要な行だけ展開する。
- 右InspectorはTimelineで選択した項目の詳細。
- タブ中心にはしない。Timeline + Inspectorを基本構造にする。

Timeline model:

- Timeline は `request -> workflow_decision -> dispatch -> run -> artifact/evidence -> review -> verification -> recovery/handoff/approval` を基本の流れにする。
- Handoff、人への質問、人の回答、追加指示、回復、再実行は同じTimeline上のイベントとして追加する。
- Timeline は左に細い縦線とnode、右に各stepの短い要約を置き、流れが上から下へ読めるようにする。
- Timeline item を選択すると、その詳細、関連Artifact、前後stepへの移動、次アクションが右Inspectorに出る。
- Timeline item は accordion row とし、閉じた状態では step、status、agent、時刻、warning、artifact count だけを出す。
- 選択中 row だけを展開し、短い説明、主要リンク、次アクションを表示する。
- 詳細な raw log、JSON、transcript、findings は中央Timelineには出さず、右Inspectorで扱う。
- Timeline は長くなるため、古い完了済みイベントは compact 表示にできる。
- Work Item Summary には DoD、current target、latest workflow decision、latest review verdict、
  criteria state、open recovery、open handoff、next action を表示する。
- Inspector は選択中のTimeline itemだけを詳しく表示するが、前後の関連 item へ1クリックで移動できる。
- 同じ種類の item が複数ある場合、Inspector 上部に previous / next を表示する。
- Debug時は「どのイベントで判断が変わったか」を追えることを優先する。

### 3.3 Agent Profiles

目的:

- Agent Profileを一覧・比較する。
- dispatchに必要なrouting hintの不足を発見する。
- Agent Profile Detailへ移動する。

主な表示:

- id、display_name、role。
- runtime、adapter、working_dir。
- description有無。
- specialties。
- latest probe / health state。
- default agent assignment。

主な操作:

- agent add。
- agent update。
- agent defaults。
- agent use。
- agent doctor。
- agent probe。

### 3.4 Agent Profile Detail

目的:

- 1つのAgent Profileの宣言情報と観測情報を分けて確認する。
- dispatch品質を改善する。

表示セクション:

- Profile Declaration。
- Runtime Health。
- Capability Probe。
- Dispatch Usage。
- Recent Runs。

主な操作:

- agent update。
- agent doctor。
- agent probe。
- default agentに設定。

### 3.5 Settings

目的:

- Project単位の基本設定を扱う。

表示・操作:

- locale。
- timezone。
- work_agent。
- review_agent。
- dispatch_agent。
- supervisor_agent。
- supported adapters。
- dispatch context budget。

Context Budget:

- 初期MVPでは最大5候補で固定する。
- 設定化はしない。
- UIでは「Dispatch candidate limit: 5 fixed」として表示する。

## 4. Inspector / Drawer

Inspector は独立した詳細ページではなく、Work Item Detail の右側に出る
作業用 drawer として扱う。目的は「読む」ことだけではなく、次の操作へ進むこと。
どの Inspector も上部に関連Agent、状態、前後のTimeline itemへの移動を置く。

### 4.1 Workflow Decision Inspector

開く場所:

- Work Item Detail の WorkflowDecision event。
- Header の `判断を確認`。
- Advance後の停止理由。

やること:

- action、source、reason、requires_human、confidenceを確認する。
- supervisor_agentが判断した場合は、対応する `workflow_supervision` AgentRunを確認する。
- command_hintとwarningsを確認する。
- 次に実行される操作が、人間の意図と合っているかを確認する。

主な操作:

- advance。
- advance until blocked。
- override action。
- open supervisor run。
- previous / next eventへ移動。

### 4.2 Dispatch Review Inspector

開く場所:

- Work Item Detail の Timeline。
- Handoff Inspector から作られた再dispatch。

やること:

- DispatchPlanのstatusを確認する。
- target agent、summary、risks、missing_informationを確認する。
- selection_warningsを確認する。
- candidate agent一覧を確認する。
- raw dispatch outputを確認する。
- ResolvedRunPacketを確認する。
- なぜそのAgentへ任せるのかを確認する。

主な操作:

- accept。
- re-dispatch。
- override agent。
- run with target。
- previous / next eventへ移動。

### 4.3 Run Log Inspector

開く場所:

- Work Item Detail の Runs。
- Agent Profile Detail の Recent Runs。

やること:

- AgentRunのpurpose、agent、adapter、exit code、durationを確認する。
- stdout / stderrを確認する。
- app-server transcriptを確認する。
- ResolvedRunPacketを見る。
- produced Artifact / Evidenceを見る。
- 失敗時に、adapter失敗、agent判断失敗、検証不足のどれかを切り分ける。

主な操作:

- retry。
- create handoff。
- open artifact。
- open review。

### 4.4 Artifact Viewer

開く場所:

- Run Log Inspector。
- Evidence Detail Inspector。
- Verification Inspector。
- Dispatch Review Inspector。
- Resolved Context。

やること:

- run log、raw output、JSON artifactを読む。
- ResolvedSkillContext / ResolvedRunPacketを整形表示する。
- 関連entityへ戻る。
- どの Run / Review / Verification がその成果物を参照したかを表示する。

主な操作:

- open source run。
- open evidence。
- open review。

### 4.5 Evidence Detail Inspector

やること:

- claim、basis、produced_by、artifactを確認する。
- approve可能な根拠が揃っているかを見る。
- 関連Run / Verification / Artifactへ移動する。

### 4.6 Verification Inspector

やること:

- verification command、result、logを確認する。
- failed時は再verifyまたはhandoffへ進む。
- Review前に承認可能な状態かを確認する。

主な操作:

- re-run verify。
- open artifact。
- request review。

### 4.7 Review Inspector

開く場所:

- Review Agent の AgentRun。
- Verification 後の次アクション。
- Handoff 前後の判断点。

やること:

- review_agent の verdict、findings、risk、requested changesを確認する。
- acceptance criteriaごとの criteria_results を確認する。
- criteria failed / missing の場合は、どの条件が承認を止めているかを見る。
- reviewが参照した Artifact / Evidence / Verification を確認する。
- approve、request changes、rerun、handoff のどれが妥当かを判断する。
- 同じ Work Item の前回Reviewとの差分を見る。

主な操作:

- approve。
- request changes。
- rerun。
- create handoff。
- open referenced artifact。

### 4.8 Recovery Inspector

開く場所:

- RecoveryPlan event。
- run failed、review requested changes、verification failed、artifact不足、diff不足のTimeline item。
- Header の `回復案を見る`。

やること:

- failure_class、action、target_agent、reason、summaryを確認する。
- command_hint、prompt_hint、warningsを確認する。
- 複数のdraft RecoveryPlanから採用候補を選ぶ。
- accepted planがあれば、適用済みか未適用かを確認する。
- 回復不能な場合はHandoffへ接続する。

主な操作:

- create recovery plan。
- accept recovery plan。
- apply accepted plan。
- open failed run。
- create handoff。

### 4.9 Handoff Inspector

やること:

- from agent、to agent、reason、summaryを入力・確認する。
- handoff packetに含める current state、open questions、artifact、diff artifact、
  failed verification、review result、next requestを確認する。
- handoff dispatchを実行する。
- 作成されたDispatchPlanをDispatch Reviewへ接続する。

### 4.10 Agent Edit Modal

やること:

- display_name、role、working_dirを編集する。
- description、specialtiesを編集する。
- agent add / update を担う。

### 4.11 Defaults Modal

やること:

- work_agent、review_agent、dispatch_agent、supervisor_agentを選ぶ。
- health / probe未確認のAgentを警告する。

## 5. 視覚スタイル

### 5.1 色

色は意味に使う。

- slate / neutral: 通常情報、背景、境界線。
- indigo: 選択中、リンク、実行可能なprimary action。
- green: succeeded、passed、done。
- amber: draft、warning、missing information。
- red: failed、blocked、selection warning。
- gray: superseded、disabled、archived。

単一色相に寄せすぎない。特に紫、濃紺、ベージュ、茶系だけで画面を作らない。
Hachiware Labs本体サイトの延長として、白地、slate系テキスト、indigo系primary
actionを基準にする。ただしアプリ画面ではhero画像や装飾キャラクターを主役にせず、
仕事の進行状態と次の操作を優先する。

### 5.2 タイポグラフィ

- 見出しは控えめにする。
- Work Item titleとstatusを最も読みやすくする。
- Timeline card内の見出しは小さく、密度を保つ。
- 文字サイズをviewport幅で拡縮しない。
- letter spacingは0を基本にする。

### 5.3 コンポーネント

- 操作はicon + labelを基本にする。
- 既知操作にはアイコンボタンを使い、tooltipで説明する。
- binary stateはtoggle / checkbox。
- option setはmenu。
- mode切替はsegmented control。
- 数値はinput / stepper / slider。
- repeated itemだけをcardにする。
- cardの中にcardを入れない。

### 5.4 状態表示

状態はbadgeで表す。

- WorkItemStatus badge。
- AgentRunStatus badge。
- DispatchPlanStatus badge。
- VerificationStatus badge。
- Probe health badge。

warning countやmissing countは小さなcount badgeで表示する。

## 6. MVP優先順

1. Quick Request Composer。
2. Work Queue。
3. Work Item Detail。
4. Workflow Decision Inspector / Advance操作。
5. Recovery Inspector。
6. Dispatch Review Inspector。
7. Run Log Inspector / Artifact Viewer。
8. Review Inspector / Criteria results。
9. Evidence / Verification / Handoff Inspector強化。
10. Agent Profiles。
11. Agent Profile Detail。
12. Settings / Defaults Modal。

## 7. 非目標

MVPでは次を行わない。

- Marketing landing page。
- 独立したDispatchPlan一覧画面。
- 独立したRun Log一覧画面。
- 独立したEvidence一覧画面。
- 独立したArtifact一覧画面。
- Context Budgetの設定化。
- 装飾的なdashboard。

これらは、Work Item BoardとWork Item Detailの運用体験が固まってから検討する。

## 8. 画面デザイン画像

デザイン画像は、方針の確認用にSVGとPNGを併置する。

- Work Item Board: [SVG](design-assets/svg/01-work-item-board.svg) / [PNG](design-assets/png/01-work-item-board.png)
- Work Item Detail: [SVG](design-assets/svg/02-work-item-detail.svg) / [PNG](design-assets/png/02-work-item-detail.png)
- Agent Profiles: [SVG](design-assets/svg/03-agent-profiles.svg) / [PNG](design-assets/png/03-agent-profiles.png)
- Agent Profile Detail: [SVG](design-assets/svg/04-agent-profile-detail.svg) / [PNG](design-assets/png/04-agent-profile-detail.png)
- Settings: [SVG](design-assets/svg/05-settings.svg) / [PNG](design-assets/png/05-settings.png)
- Dispatch Review Inspector: [SVG](design-assets/svg/06-dispatch-review-inspector.svg) / [PNG](design-assets/png/06-dispatch-review-inspector.png)
- Run Log Inspector: [SVG](design-assets/svg/07-run-log-inspector.svg) / [PNG](design-assets/png/07-run-log-inspector.png)
- Artifact Viewer: [SVG](design-assets/svg/08-artifact-viewer.svg) / [PNG](design-assets/png/08-artifact-viewer.png)
- Evidence Detail Inspector: [SVG](design-assets/svg/09-evidence-detail-inspector.svg) / [PNG](design-assets/png/09-evidence-detail-inspector.png)
- Verification Inspector: [SVG](design-assets/svg/10-verification-inspector.svg) / [PNG](design-assets/png/10-verification-inspector.png)
- Review Inspector: [SVG](design-assets/svg/11-review-inspector.svg) / [PNG](design-assets/png/11-review-inspector.png)
- Handoff Inspector: [SVG](design-assets/svg/12-handoff-inspector.svg) / [PNG](design-assets/png/12-handoff-inspector.png)
- Agent Edit Modal: [SVG](design-assets/svg/13-agent-edit-modal.svg) / [PNG](design-assets/png/13-agent-edit-modal.png)
- Defaults Modal: [SVG](design-assets/svg/14-defaults-modal.svg) / [PNG](design-assets/png/14-defaults-modal.png)
- Workflow Decision Inspector: [SVG](design-assets/svg/15-workflow-decision-inspector.svg) / [PNG](design-assets/png/15-workflow-decision-inspector.png)
- Recovery Inspector: [SVG](design-assets/svg/16-recovery-inspector.svg) / [PNG](design-assets/png/16-recovery-inspector.png)

画像生成元は `scripts/generate-ui-mockups.js` とする。
