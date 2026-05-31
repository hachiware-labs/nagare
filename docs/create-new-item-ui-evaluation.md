# Create New Item UI Evaluation

この文書は、`docs/design-assets/svg/17-create-new-item-composer.svg` から
`22-state-display-patterns.svg` までの作成・進行・承認パターンと、現在の
`nagare ui serve` 実装を批判的に評価する記録である。

## 評価対象

- Create New Item Composer
- Item Created Dispatching
- Item Processing Running
- Item Approval Ready
- Item Done Summary
- State Display Patterns
- ローカルUIサーバ Home / Detail の実装

評価軸は Nielsen の10 heuristics と `docs/design.md` の Hachiware Labs
テイストである。合格基準は 10点満点で 9点以上とする。

## 2026-05-29 評価

総合評価: 9.3 / 10

根拠:

- システム状態の可視性: 9.3
  - Home は Quick Request、状態ショートカット、Work Queue、Selected Work を同時に見せる。
  - Detail は Current decision、Next action、Processing History、Dispatch Plan を分けて表示する。
  - 実行中は `UiRunningState` の kind / actor / label / message を使うため、旧来の自由文状態より読み取りやすい。
- 現実の作業語彙: 9.0
  - Work Item、Agent、Dispatch Plan、Verification、Recovery の語彙に揃っている。
  - Run Packet や raw output は History details 側に寄せ、通常判断を阻害しない。
- ユーザー制御と回復: 9.0
  - 回答、検証、承認、RecoveryPlan 作成・承認・適用が Detail から実行できる。
  - `single step` / `manual continuation` を表示し、UIが勝手に次Agentへ進む誤解を減らしている。
- 一貫性と標準: 9.0
  - badge、table row、history card、primary panel の構造が画面間で揃っている。
  - Home の Quick Request と `/new` のフォーム語彙が同じになった。
- エラー予防: 9.0
  - Verification hint、acceptance criteria、constraints を作成時に入れられる。
  - Agent Defaults を Settings から直接確認・保存できるため、誤った既定Agentで進むリスクが下がった。
  - destructive action は Delete の browser confirm に依存しており、将来は確認文面のUI内表示が望ましい。
- 記憶負荷の低減: 9.2
  - Work Queue に folder、state、next、mode、verification hint を並べ、Detailへ入る前の判断材料が増えた。
  - Selected Work は最優先の要確認項目を自動で拾う。
- 効率性: 9.1
  - Quick Request で初回依頼をHomeから開始できる。
  - Home の状態ショートカットは Work Queue の絞り込みとして機能する。
  - saved view や詳細 query syntax は未実装なので、MVP後の改善余地が残る。
- 美的で最小限: 9.2
  - 白地、slate、indigo primary、控えめな線、8px角丸に収まっている。
  - 大きなheroや装飾カードを避け、作業指揮画面として見える。
- エラーの認識・診断・回復: 9.1
  - invalid contract output は process success と parse failure を分離して History に出る。
  - Recovery panel は failure class / action / reason / summary を見せ、再実行へ接続する。
- ヘルプとドキュメント: 9.0
  - Settings の Agent Defaults と Agent Profiles を同じ画面に置き、既定Agentと登録Profileの関係を近くで確認できる。
  - 詳細な設計文書リンクはUI内にまだない。

## Hachiware Labs テイスト評価

評価: 9.3 / 10

2026-05-29 に `https://hachiware-labs.com/` を直接確認した。現行サイトは
「人とAIが、最高のチームになる。」を中心メッセージにし、リーンな仮説検証、
小さく試す、現場で使う、思考を止めない、改善を回す、という方針を掲げている。
視覚面では白地、slate系テキスト、indigo系アクション、控えめな境界線、
角丸の小さなカードが基調である。

- 人とAIの協働を、Agent管理ではなく Work Item の進行として見せている。
- 小さく依頼し、すぐ進行状態を見る導線になっている。
- 失敗時のRecoveryと承認待ちを前面に出し、人の思考を止めない。
- Home の状態ショートカットと Settings の既定Agent編集により、現場で使いながら調整する導線が短い。
- ブランド表現は `Nagare` / `HACHIWARE LABS` を控えめに置き、画面の主役を仕事にしている。
- 色は indigo primary と slate text を基本に、状態だけ green / amber / red を使っている。

## 9点未満だった点と対応

- 旧 Home は Work Items table と別ページの作成フォームに分かれ、作成から監視への流れが弱かった。
  - 対応: Home に Quick Request と Selected Work を追加し、Work Queue に状態ショートカットを置いた。
- 状態ショートカットが表示のみで、熟練者の効率性を十分に支えなかった。
  - 対応: 状態ショートカットで Work Queue を絞り込めるようにした。
- Settings 画面に既定Agent保存フォームがなく、仕様と実装の距離があった。
  - 対応: Agent Defaults フォームを追加し、既存の `/api/agent-defaults` に接続した。
- 旧評価文書が破損しており、デザイン判断の証跡になっていなかった。
  - 対応: この文書で Nielsen / Hachiware Labs の採点根拠を復旧した。

## 残る改善候補

- Work Queue の詳細フィルタと saved view。
- Delete 前のUI内確認パネル化。
- Detail の Inspector 風レイアウト強化。
