# Create New Item から完了までのユースケース

この文書は、ユーザーが Nagare の画面で `Create New Item` を実行し、Agent の実行、
レビュー、検証、人間の承認を経て完了するまでの画面シナリオを定義する。

目的は、ユーザーが「依頼したことが、どの Agent によって、どこまで処理されたか」を
履歴として理解できる画面にすることである。Agent Flow は独立した装飾ではなく、
Work Item の `WorkItemHistoryStep` として扱う。

## 1. ユースケース概要

### 1.1 ゴール

- ユーザーが短い依頼文から Work Item を作成できる。
- 作成直後から、現在状態、担当 Agent、次の操作が見える。
- 実行中は「処理中」と「何を処理しているか」が同じ画面で分かる。
- 完了時は、最終回答、検証結果、承認履歴、実行履歴を一つの流れで確認できる。
- 失敗や質問が発生した場合も、回復操作またはユーザー入力が主操作として提示される。

### 1.2 登場人物とシステム

- User: 依頼を作成し、必要な判断、入力、承認を行う。
- UI Server: Work Item、Event、Run、Artifact、Verification を画面に表示する。
- Workflow Controller: 次に進めるべき action を判断し、WorkflowDecision を作る。
- Dispatch Agent: 依頼内容から適切な target agent を選ぶ。
- Work Agent: 実作業を行い、成果物、証跡、回答を作る。
- Review Agent: 受入条件に対して成果物を確認する。
- Verification Runner: 設定された検証コマンドを実行する。
- Human Approver: 最終結果を確認し、承認または差し戻しを行う。

### 1.3 前提条件

- プロジェクトに Nagare が初期化されている。
- UI Server が起動している。
- default agents が設定されている。
- `dispatch_agent`、`work_agent`、`review_agent` のうち、少なくとも dispatch と work に使える Agent が存在する。
- ユーザーは Work Item Board を開いている。

## 2. 画面構成

### 2.1 Work Item Board

初期画面。ユーザーは `Create New Item` から依頼を作成する。

表示する情報:

- Quick Composer: title、request、acceptance criteria、expected artifacts、verification hint、work folder。
- Work Queue: item title、state、active actor、latest event、next action。
- Selected Summary: 選択中 Work Item の現在状態、担当 Agent、次の操作。

作成直後にユーザーへ伝えるべきこと:

- Work Item が作成されたこと。
- すでに処理が始まっているのか、ユーザー確認待ちなのか。
- 現在の active actor が誰か。
- 次にユーザーが押すべき primary action があるか。

### 2.2 Work Item Detail

作成後の中心画面。Agent Flow はこの画面の `Processing History` として表示する。

上から順に表示する:

1. Header: Work Item title、state badge、next action、active actor。
2. Current Summary: current state、reason、latest event、last result、next action。
3. Final Answer or Current Output: 完了前は暫定結果、完了後は最終回答。
4. Processing History: Request から Done までの event を時系列に表示する。
5. Inspector: 選択した event の詳細、artifact、verification log、review finding を表示する。

### 2.3 状態表示の基本ルール

- 状態は常に `State`、`Actor`、`Next action` の3点で表示する。
- primary action は1つだけ表示する。
- 実行中は「処理中」だけで止めず、実行中の event 名と actor を出す。
- `No action required` は完了または本当に待機不要なときだけ使う。
- Agent Flow は別パネルに重複表示せず、Processing History に統合する。
- IDだけを見せず、event summary と関連する artifact / review / verification を並べる。

## 3. 状態モデル

### 3.1 Work Item の画面状態

| UI State | 意味 | Primary Action |
| --- | --- | --- |
| Draft Input | ユーザーが依頼内容を入力中 | Create New Item |
| Queued | Work Item が作成され、実行待ち | Open Detail |
| Processing | Agent または検証が実行中 | View Running Step |
| Needs Input | Agent がユーザー入力を要求 | Answer Question |
| Needs Review | Review 結果の確認が必要 | Open Review |
| Needs Verification | 検証が未実行または再実行待ち | Run Verification |
| Needs Recovery | 失敗または差し戻しがあり回復案が必要 | Open Recovery Plan |
| Ready For Approval | 結果、review、verification が揃い承認待ち | Approve Result |
| Done | 人間が承認し完了 | Open Summary |

### 3.2 状態色

| 種別 | 用途 |
| --- | --- |
| Blue | 処理中、選択中、次に進める状態 |
| Amber | ユーザー確認、質問、承認待ち、注意 |
| Red | 失敗、検証失敗、回復が必要 |
| Green | 成功、検証通過、承認済み、完了 |
| Gray | 下書き、待機、補助情報 |

### 3.3 History Step の基本形

Processing History の各 step は同じ形で表示する。

- Step number: `01`、`02` のような連番。
- Kind: `request`、`dispatch`、`work`、`review`、`verification`、`input`、`handoff`、`recovery`、`approval`。
- Step title: `依頼を作成`、`Agent 選定`、`作業実行` など。
- Actor: `User`、`Workflow Controller`、`dispatch-agent`、`work-agent` など。
- State badge: `done`、`running`、`needs input`、`failed`。
- Summary: 何が起きたかを1行で説明する。
- Facts: target agent、artifact、verification、review verdict など。
- Links: source record、artifact、review、verification を開くための参照。
- Source record ids: 監査用に元 ledger record を保持する。

## 4. メインシナリオ

### Step 1: ユーザーが Create New Item を開く

ユーザー操作:

- Work Item Board で `Create New Item` を押す。

画面状態:

- Composer が開く。
- required fields が強調される。
- primary action は `Create New Item`。

Event:

- まだ永続 Event は作成しない。
- UI 内部状態として `ComposerOpened` を持つ。

### Step 2: ユーザーが依頼内容を入力する

入力例:

- title: `README のセットアップ手順を更新`
- request: `docs/setup.md の変更を README に反映して`
- acceptance criteria:
  - `README に新しい手順が反映されている`
  - `既存の説明と重複していない`
- expected artifacts: `README diff`
- verification hint: `npm test`
- work folder: `.`

画面状態:

- 入力中は `Draft Input`。
- 不足項目があれば inline validation を表示する。
- primary action は `Create New Item`。

Event:

- まだ永続 Event は作成しない。
- UI 内部状態として `ComposerEdited` を持つ。

### Step 3: Work Item が作成される

ユーザー操作:

- `Create New Item` を押す。

システム処理:

- Work Item を作成する。
- DoD、acceptance criteria、expected artifacts、verification hint を保存する。
- Workflow Controller が最初の action を決める。

生成 Event:

| Event | Actor | State |
| --- | --- | --- |
| WorkItemCreated | User | done |
| AcceptanceCriteriaRecorded | UI Server | done |
| WorkflowDecisionCreated | Workflow Controller | done |

画面状態:

- Work Item Detail へ遷移する。
- Header state は `Processing` または `Queued`。
- Current Summary に `Dispatching request` を表示する。
- Processing History には Step 01 `Work item created` が追加される。

ユーザーに伝える文言:

- `依頼を作成しました。dispatch-agent が担当 Agent を選定しています。`

### Step 4: Dispatch Agent が担当 Agent を選ぶ

システム処理:

- Dispatch Agent が依頼、criteria、working dir、agent profile を読んで候補を出す。
- DispatchPlan が作られる。
- 自動承認できる条件なら accepted にする。
- 人間確認が必要なら `Needs Review` にする。

生成 Event:

| Event | Actor | State |
| --- | --- | --- |
| DispatchRunStarted | dispatch-agent | running |
| DispatchPlanCreated | dispatch-agent | done |
| DispatchPlanAccepted | Workflow Controller | done |

画面状態:

- 実行中は Header state `Processing`、active actor `dispatch-agent`。
- History の Step 02 が `running` で強調される。
- accepted 後は target agent が Summary に表示される。

表示例:

- `Current: Selecting target agent`
- `Actor: dispatch-agent`
- `Result: writing-agent selected, confidence 0.86`
- `Next: Run writing-agent`

### Step 5: Work Agent が実作業を行う

システム処理:

- accepted DispatchPlan をもとに Work Agent を実行する。
- Run Packet、stdout/stderr、artifact、diff、evidence を保存する。
- Agent の回答を AgentOutput として記録する。

生成 Event:

| Event | Actor | State |
| --- | --- | --- |
| WorkAgentRunStarted | work-agent | running |
| ExecutionRecordCreated | work-agent | done |
| EvidenceRecorded | work-agent | done |
| AgentOutputRecorded | work-agent | done |
| WorkAgentRunCompleted | work-agent | done |

画面状態:

- Header state は `Processing`。
- Current Summary は `writing-agent is editing README`。
- History の active step は Step 03 `Work agent run`。
- Inspector から Run Log と Artifact を開ける。

ユーザーに伝える文言:

- `writing-agent が README diff を作成しています。`

### Step 6: Review Agent が受入条件を確認する

システム処理:

- Review Agent が成果物と acceptance criteria を確認する。
- criteria ごとに pass / fail を記録する。
- 問題がなければ次の検証へ進む。

生成 Event:

| Event | Actor | State |
| --- | --- | --- |
| ReviewRunStarted | review-agent | running |
| ReviewResultRecorded | review-agent | done |

画面状態:

- Header state は `Processing`。
- Current Summary は `Reviewing acceptance criteria`。
- Review Result は criteria 単位で表示する。

成功時の表示:

- `Review: passed`
- `Criteria: 2 / 2 passed`
- `Next: Run verification`

### Step 7: Verification Runner が検証する

システム処理:

- verification hint または既定の検証コマンドを実行する。
- 結果、exit code、log excerpt を保存する。

生成 Event:

| Event | Actor | State |
| --- | --- | --- |
| VerificationStarted | Verification Runner | running |
| VerificationPassed | Verification Runner | done |

画面状態:

- Header state は `Processing`。
- Current Summary は `Running npm test`。
- 通過後は `Ready For Approval`。
- primary action は `Approve Result`。

表示例:

- `Verification: passed`
- `Command: npm test`
- `Next: Approve Result`

### Step 8: ユーザーが結果を承認する

ユーザー操作:

- Final Answer、Review、Verification、History を確認する。
- `Approve Result` を押す。
- 必要なら承認コメントを入力する。

生成 Event:

| Event | Actor | State |
| --- | --- | --- |
| ApprovalGateReady | Workflow Controller | done |
| HumanDecisionApproved | User | done |
| WorkItemDone | Workflow Controller | done |

画面状態:

- Header state は `Done`。
- Final Answer が最上部で読める。
- Processing History は Step 01 から Done まで完了済みになる。
- primary action は消え、secondary action として `Open Artifact`、`Create Similar Item` を出す。

ユーザーに伝える文言:

- `完了しました。README diff、review、verification log が保存されています。`

## 5. 例外シナリオ

### 5.1 Agent が質問した場合

発生条件:

- 依頼内容、対象 path、受入条件が不足している。

生成 Event:

- `AgentQuestionRecorded`
- `WorkflowDecisionCreated(requires_human: true)`

画面状態:

- Header state は `Needs Input`。
- primary action は `Answer Question`。
- History の該当 step を amber で表示する。

再開 Event:

- `HumanInputSubmitted`
- `WorkflowDecisionCreated`

### 5.2 Review が差し戻した場合

発生条件:

- acceptance criteria の一部が failed になる。

生成 Event:

- `ReviewResultRecorded(verdict: request_changes)`
- `RecoveryPlanCreated`

画面状態:

- Header state は `Needs Recovery`。
- primary action は `Open Recovery Plan`。
- Inspector に failed criteria、finding、suggested recovery を表示する。

### 5.3 Verification が失敗した場合

発生条件:

- 検証コマンドが non-zero exit で終了する。

生成 Event:

- `VerificationFailed`
- `RecoveryPlanCreated`

画面状態:

- Header state は `Needs Recovery`。
- primary action は `Open Recovery Plan` または `Re-run Verification`。
- Inspector に command、exit code、log excerpt を表示する。

### 5.4 Artifact が不足した場合

発生条件:

- expected artifact が作られていない。

生成 Event:

- `OutputContractFailed`
- `RecoveryPlanCreated`

画面状態:

- Header state は `Needs Recovery`。
- primary action は `Apply Recovery Plan`。
- History には失敗箇所と次の回復案を同じ step 付近に表示する。

## 6. 画面別の表示判断

### 6.1 Board の行表示

Work Queue の1行は次の順に読む。

1. 依頼名。
2. 現在状態。
3. active actor。
4. latest event。
5. 次の操作。

例:

| Item | State | Actor | Latest Event | Next |
| --- | --- | --- | --- | --- |
| README のセットアップ手順を更新 | Processing | writing-agent | WorkAgentRunStarted | View Running Step |
| 調査ソース一覧を作成 | Needs Input | research-agent | AgentQuestionRecorded | Answer Question |
| release note を整理 | Ready For Approval | review-agent | VerificationPassed | Approve Result |
| README diff を反映 | Done | User | WorkItemDone | Open Summary |

### 6.2 Detail の Summary 表示

Summary は常に次の5行にする。

- Current: 今どの状態か。
- Why: その状態になっている理由。
- Actor: 現在または直近の担当。
- Latest Result: 最後に得られた結果。
- Next: 次のユーザー操作または自動処理。

### 6.3 Processing History 表示

History は似た情報を重複させない。

- `Request` と `WorkItemCreated` は同じ step にまとめる。
- `DispatchRunStarted` と `DispatchPlanCreated` は dispatch step にまとめる。
- `ExecutionRecordCreated`、`EvidenceRecorded`、`AgentOutputRecorded` は work run step の facts にまとめる。Artifact はユーザーが依頼した成果物ファイルがある場合だけ表示する。
- `ReviewRunStarted` と `ReviewResultRecorded` は review step にまとめる。
- `VerificationStarted` と `VerificationPassed/Failed` は verification step にまとめる。
- `HumanDecisionApproved` と `WorkItemDone` は completion step にまとめる。

## 7. 完了条件

このユースケースの UI は、次を満たす必要がある。

- 作成直後、ユーザーは画面上部だけで現在状態と次の操作を理解できる。
- 実行中、どの Agent が何をしているかが見える。
- 完了時、Final Answer、Review、Verification、History が1画面内でつながっている。
- History は 5 から 7 step 程度で読め、低レベル Event を重複表示しない。
- ユーザー操作が必要な場合は primary action が1つに絞られている。
- 失敗時は失敗理由、影響、回復操作が同じ Inspector で見える。
