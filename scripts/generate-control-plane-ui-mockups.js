const fs = require("fs");
const path = require("path");

const outDir = path.join(__dirname, "..", "docs", "design-assets", "svg");
fs.mkdirSync(outDir, { recursive: true });

const C = {
  bg: "#f7f8fb",
  ink: "#111827",
  muted: "#526174",
  faint: "#6b7280",
  surface: "#ffffff",
  panel: "#f1f5f9",
  line: "#d8e0ea",
  strongLine: "#b8c4d3",
  primary: "#4f46e5",
  primarySoft: "#eef2ff",
  blue: "#2563eb",
  blueSoft: "#dbeafe",
  teal: "#0f766e",
  tealSoft: "#ccfbf1",
  green: "#15803d",
  greenSoft: "#dcfce7",
  amber: "#b45309",
  amberSoft: "#fef3c7",
  red: "#b91c1c",
  redSoft: "#fee2e2",
  purple: "#7c3aed",
  purpleSoft: "#f3e8ff",
  slateSoft: "#e5e7eb",
};

const W = 1440;
const H = 1024;
const SIDEBAR = 204;

function esc(value) {
  return String(value).replace(/[&<>"]/g, (ch) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
  })[ch]);
}

function slug(value) {
  return String(value).toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

function svgDoc(title, desc, body, height = H) {
  const id = slug(title);
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${height}" viewBox="0 0 ${W} ${height}" role="img" aria-labelledby="${id}-title ${id}-desc">
  <title id="${id}-title">${esc(title)}</title>
  <desc id="${id}-desc">${esc(desc)}</desc>
  <rect width="${W}" height="${height}" fill="${C.bg}"/>
  <style>
    .title{font:700 25px Inter,"Segoe UI","Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.ink}}
    .kicker{font:700 10px Inter,"Segoe UI",Arial,sans-serif;fill:${C.primary}}
    .h1{font:750 22px Inter,"Segoe UI","Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.ink}}
    .h2{font:700 16px Inter,"Segoe UI","Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.ink}}
    .h3{font:700 13px Inter,"Segoe UI","Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.ink}}
    .body{font:500 12px Inter,"Segoe UI","Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.ink}}
    .small{font:500 11px Inter,"Segoe UI","Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.muted}}
    .tiny{font:700 10px Inter,"Segoe UI","Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.muted}}
    .mono{font:500 11px Consolas,Menlo,monospace;fill:${C.ink}}
    .nav{font:650 13px Inter,"Segoe UI",Arial,sans-serif;fill:${C.muted}}
    .nav-on{font:750 13px Inter,"Segoe UI",Arial,sans-serif;fill:${C.primary}}
  </style>
  ${body}
</svg>`;
}

function rect(x, y, w, h, fill = C.surface, stroke = C.line, r = 8, sw = 1) {
  return `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"/>`;
}

function line(x1, y1, x2, y2, color = C.line, sw = 1) {
  return `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${color}" stroke-width="${sw}"/>`;
}

function pathD(d, color = C.line, sw = 2, fill = "none") {
  return `<path d="${d}" fill="${fill}" stroke="${color}" stroke-width="${sw}" stroke-linecap="round" stroke-linejoin="round"/>`;
}

function arrow(x1, y1, x2, y2, color = C.strongLine) {
  const head = x2 >= x1
    ? `<path d="M ${x2 - 8} ${y2 - 5} L ${x2} ${y2} L ${x2 - 8} ${y2 + 5}" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round"/>`
    : `<path d="M ${x2 + 8} ${y2 - 5} L ${x2} ${y2} L ${x2 + 8} ${y2 + 5}" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round"/>`;
  return `${line(x1, y1, x2, y2, color, 2)}${head}`;
}

function txt(x, y, value, cls = "body", color = null) {
  return `<text x="${x}" y="${y}" class="${cls}"${color ? ` style="fill:${color}"` : ""}>${esc(value)}</text>`;
}

function pillWidth(text, min = 54) {
  const raw = String(text);
  const wide = [...raw].reduce((sum, ch) => sum + (ch.charCodeAt(0) > 255 ? 11 : 7), 0);
  return Math.max(min, wide + 20);
}

function tone(kind) {
  return {
    primary: [C.primarySoft, C.primary],
    blue: [C.blueSoft, C.blue],
    teal: [C.tealSoft, C.teal],
    green: [C.greenSoft, C.green],
    amber: [C.amberSoft, C.amber],
    red: [C.redSoft, C.red],
    purple: [C.purpleSoft, C.purple],
    gray: [C.panel, C.muted],
  }[kind] || [C.panel, C.muted];
}

function pill(x, y, text, kind = "gray", w = null) {
  const [bg, fg] = tone(kind);
  const width = w || pillWidth(text);
  return `${rect(x, y, width, 24, bg, bg, 12)}
  ${txt(x + 10, y + 16, text, "tiny", fg)}`;
}

function button(x, y, text, primary = false, w = null) {
  const width = w || Math.max(86, pillWidth(text, 72) + 12);
  const fill = primary ? C.primary : C.surface;
  const stroke = primary ? C.primary : C.strongLine;
  const fg = primary ? "#ffffff" : C.ink;
  return `${rect(x, y, width, 34, fill, stroke, 7)}
  ${txt(x + 14, y + 22, text, "body", fg)}`;
}

function chrome(active, pageTitle, pageSub, actions, content, desc, height = H) {
  const navItems = [
    ["Work Items", "ワーク"],
    ["Projects", "プロジェクト"],
    ["artifact-types", "ドメイン"],
    ["Agents", "エージェント"],
    ["Skills", "スキル"],
    ["MCP", "MCP"],
    ["Settings", "設定"],
  ];
  const nav = navItems
    .map((item, i) => {
      const y = 120 + i * 39;
      const on = item[0] === active;
      return `<g>
        ${on ? rect(18, y - 24, 168, 33, C.primarySoft, C.primarySoft, 7) : ""}
        ${txt(34, y - 2, item[1], on ? "nav-on" : "nav")}
      </g>`;
    }).join("");

  return svgDoc(pageTitle, desc || pageSub, `
    <g id="sidebar">
      ${rect(0, 0, SIDEBAR, height, C.surface, C.line, 0)}
      ${txt(28, 43, "Nagare", "title")}
      ${txt(30, 64, "制御プレーン", "kicker")}
      ${txt(30, 82, "プロジェクト対応ランタイム", "small")}
      ${nav}
    </g>
    <g id="page-header">
      ${txt(232, 48, pageTitle, "title")}
      ${txt(232, 72, pageSub, "small")}
      ${actions}
    </g>
    <g id="content">
      ${content}
    </g>
  `, height);
}

function sectionLabel(x, y, title, meta = "") {
  return `${txt(x, y, title, "h2")}${meta ? txt(x, y + 20, meta, "small") : ""}`;
}

function statusDot(x, y, kind = "green") {
  const [bg, fg] = tone(kind);
  return `<circle cx="${x}" cy="${y}" r="7" fill="${bg}" stroke="${fg}" stroke-width="2"/>`;
}

function projectControlPlaneOverview() {
  function gate(x, y, n, title, meta, kind = "primary") {
    const [bg, fg] = tone(kind);
    return `<g>
      ${rect(x, y, 134, 94, C.surface, C.line, 8)}
      <circle cx="${x + 24}" cy="${y + 27}" r="14" fill="${bg}" stroke="${fg}" stroke-width="1.5"/>
      ${txt(x + 19, y + 32, n, "tiny", fg)}
      ${txt(x + 48, y + 30, title, "h3")}
      ${txt(x + 16, y + 61, meta, "small")}
    </g>`;
  }

  const domains = [
    ["設計", "green"], ["エージェント実行", "red"], ["ツール方針", "amber"],
    ["UX", "blue"], ["実装", "green"], ["テスト", "amber"], ["ドキュメント", "teal"],
  ].map(([label, kind], i) => {
    const x = 300 + i * 92;
    return `${statusDot(x, 704, kind)}${txt(x - 40, 730, label, "tiny")}`;
  }).join("");

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 118, 296, 188, "#fbfcfe", C.line, 8)}
    ${txt(274, 154, "プロジェクト構成", "h2")}
    ${txt(274, 184, "Nagare", "h1")}
    ${txt(274, 210, "ランタイム制御プレーン", "small")}
    ${pill(274, 236, "ルート: ~/workspace/nagare", "gray", 210)}
    ${pill(274, 268, "確認 2 / 禁止 1", "amber", 154)}

    ${pathD("M 548 212 C 608 212, 608 212, 668 212", C.strongLine, 2)}
    <path d="M 660 207 L 668 212 L 660 217" fill="none" stroke="${C.strongLine}" stroke-width="2"/>
    ${rect(668, 118, 452, 188, "#fbfcfe", C.line, 8)}
    ${txt(690, 154, "実行構成の解決", "h2")}
    ${txt(690, 180, "依頼をランタイムへ送る前に、プロジェクト文脈で束を解決する", "small")}
    ${pill(690, 210, "プロジェクト", "primary")}
    ${pill(802, 210, "ドメイン", "blue")}
    ${pill(892, 210, "ルーブリック", "purple", 108)}
    ${pill(1016, 210, "割り当て", "green")}
    ${pill(690, 246, "セッション", "amber")}
    ${pill(790, 246, "機能", "teal")}
    ${pill(852, 246, "ポリシー", "red")}
    ${pill(946, 246, "トレース", "green")}

    ${rect(1144, 118, 232, 188, C.redSoft, C.red, 8)}
    ${txt(1166, 154, "現在", "h2", C.red)}
    ${txt(1166, 184, "OpenClaw が未接続", "h3", C.red)}
    ${txt(1166, 210, "エージェント実行ドメインの", "small")}
    ${txt(1166, 232, "割り当てが実行不可", "small")}
    ${button(1166, 258, "ランタイム切替", true, 126)}

    ${rect(252, 354, 1124, 196, "#fbfcfe", C.line, 8)}
    ${sectionLabel(274, 390, "解決順序", "構図の主役は、エージェントではなくプロジェクトからランタイムへ流れる解決順序")}
    ${gate(274, 426, "1", "プロジェクト", "Nagare")}
    ${arrow(408, 473, 430, 473)}
    ${gate(430, 426, "2", "ドメイン", "UX / 実行", "blue")}
    ${arrow(564, 473, 586, 473)}
    ${gate(586, 426, "3", "ルーブリック", "プロジェクト補正", "purple")}
    ${arrow(720, 473, 742, 473)}
    ${gate(742, 426, "4", "割り当て", "エージェント+役割", "green")}
    ${arrow(876, 473, 898, 473)}
    ${gate(898, 426, "5", "セッション", "再利用 / 分岐", "amber")}
    ${arrow(1032, 473, 1054, 473)}
    ${gate(1054, 426, "6", "機能", "実行可能な道具", "teal")}
    ${arrow(1188, 473, 1210, 473)}
    ${gate(1210, 426, "7", "トレース", "理由 + 結果", "green")}

    ${rect(252, 594, 690, 184, "#fbfcfe", C.line, 8)}
    ${sectionLabel(274, 630, "ドメインカバー", "ドメインごとの担当・ランタイム・承認状態を一列で把握する")}
    ${line(300, 704, 852, 704, C.strongLine, 2)}
    ${domains}

    ${rect(970, 594, 406, 184, "#fbfcfe", C.line, 8)}
    ${sectionLabel(992, 630, "要確認", "人間が見るべきものだけを上げる")}
    ${pill(992, 666, "OpenClaw未検出", "red", 152)}
    ${pill(992, 700, "ファイル書き込み確認", "amber", 190)}
    ${pill(992, 734, "テスト割り当て不足", "amber", 178)}

    ${rect(252, 812, 1124, 70, C.panel, C.line, 8)}
    ${txt(274, 846, "最近のトレース", "h3")}
    ${txt(410, 846, "trace_1048: UX -> designer-agent -> Codexスレッド -> hachi-search -> ルーブリック 6/7", "body")}
    ${pill(1226, 828, "レビュー待ち", "amber", 110)}
  `;

  return chrome(
    "Projects",
    "プロジェクト制御",
    "プロジェクト文脈でドメイン、割り当て、ランタイム、機能、トレースを束ねる",
    `${button(1246, 28, "依頼を開始", true, 112)}${button(1110, 28, "プロジェクト切替", false, 132)}`,
    content,
    "プロジェクト文脈からランタイム実行とトレースまでの解決順序を示す制御プレーン概要。"
  );
}

function projectContext() {
  const domainRail = [
    ["設計", "green"], ["エージェント実行", "red"], ["ツール方針", "amber"],
    ["UX", "blue"], ["実装", "green"], ["テスト", "amber"],
  ].map(([name, kind], i) => {
    const y = 650 + i * 38;
    return `${pill(274, y, name, kind, 150)}${line(424, y + 12, 472, y + 12, C.strongLine, 2)}`;
  }).join("");

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 344, 430, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "プロジェクト定義", "h2")}
    ${txt(276, 198, "Nagare", "h1")}
    ${txt(276, 226, "複数ランタイムをプロジェクト文脈で調停する", "small")}
    ${line(276, 258, 566, 258)}
    ${txt(276, 294, "ルート", "h3")}
    ${txt(276, 318, "~/workspace/nagare", "mono")}
    ${txt(276, 362, "制約", "h3")}
    ${pill(276, 386, "ランタイム固有実装へ閉じない", "primary", 250)}
    ${pill(276, 420, "道具の範囲を混ぜない", "red", 204)}
    ${pill(276, 454, "トレースで説明できる", "green", 190)}

    ${rect(636, 124, 380, 430, "#fbfcfe", C.line, 8)}
    ${txt(660, 162, "ルーブリック構成", "h2")}
    ${txt(660, 188, "置き換えではなく、継承と補正で評価する", "small")}
    ${rect(690, 238, 260, 70, C.panel, C.line, 8)}
    ${txt(716, 278, "共通ルーブリック", "h3")}
    ${rect(670, 324, 300, 78, C.blueSoft, C.blue, 8)}
    ${txt(696, 368, "+ ドメインルーブリック", "h3", C.blue)}
    ${rect(650, 418, 340, 84, C.purpleSoft, C.purple, 8)}
    ${txt(676, 466, "+ プロジェクトルーブリック", "h3", C.purple)}
    ${arrow(596, 338, 636, 338)}
    ${arrow(1016, 338, 1060, 338)}

    ${rect(1060, 124, 316, 430, "#fbfcfe", C.line, 8)}
    ${txt(1084, 162, "実行準備", "h2")}
    ${txt(1084, 188, "開始前に詰まる設定を先に出す", "small")}
    ${pill(1084, 230, "未割り当て: テスト", "amber", 184)}
    ${pill(1084, 268, "未接続: OpenClaw", "red", 168)}
    ${pill(1084, 306, "機能範囲: hachi-search", "green", 214)}
    ${pill(1084, 344, "確認: ファイル書き込み", "amber", 202)}
    ${button(1084, 424, "未設定を解消", true, 124)}
    ${button(1222, 424, "詳細を見る", false, 104)}

    ${rect(252, 590, 1124, 286, "#fbfcfe", C.line, 8)}
    ${txt(276, 626, "ドメイン別評価基準", "h2")}
    ${txt(276, 650, "プロジェクトが使う観点を左に、合成後の評価基準を右に置く", "small")}
    ${domainRail}
    ${rect(498, 650, 380, 184, C.surface, C.line, 8)}
    ${txt(522, 686, "有効な評価基準", "h2")}
    ${txt(522, 722, "ランタイム抽象化", "body")}
    ${txt(522, 752, "セッション対応の追跡性", "body")}
    ${txt(522, 782, "ツール範囲分離", "body")}
    ${rect(928, 650, 374, 184, C.surface, C.line, 8)}
    ${txt(952, 686, "プロジェクトポリシー", "h2")}
    ${pill(952, 720, "許可: github-mcp", "green", 152)}
    ${pill(952, 754, "確認: ファイル書き込み", "amber", 194)}
    ${pill(952, 788, "禁止: figma-mcp", "red", 150)}
    ${arrow(878, 742, 928, 742)}
  `;

  return chrome(
    "Projects",
    "プロジェクト文脈",
    "プロジェクトはフォルダではなく、目的・制約・ルーブリック・ポリシーを束ねる作業文脈",
    `${button(1246, 28, "プロジェクト編集", true, 132)}${button(1112, 28, "ルーブリック確認", false, 138)}`,
    content,
    "プロジェクト定義、ルーブリック構成、実行準備、ドメイン別ポリシーを示すプロジェクト文脈画面。"
  );
}

function domainQualitySettings() {
  function selectBox(x, y, label, value, width = 250) {
    return `${txt(x, y, label, "h3")}
    ${rect(x, y + 14, width, 38, C.surface, C.strongLine, 7)}
    ${txt(x + 14, y + 39, value, "body")}
    ${pathD(`M ${x + width - 28} ${y + 29} L ${x + width - 20} ${y + 37} L ${x + width - 12} ${y + 29}`, C.muted, 1.8)}`;
  }

  function artifactRow(y, name, desc, kind, active = false) {
    return `<g>
      ${rect(276, y, 318, 72, active ? C.primarySoft : C.surface, active ? C.primarySoft : C.line, 7)}
      <rect x="276" y="${y}" width="4" height="72" rx="2" fill="${active ? C.primary : C.blue}"/>
      ${txt(300, y + 28, name, "h3", active ? C.primary : null)}
      ${txt(300, y + 52, desc, "small")}
      ${pill(492, y + 20, kind, active ? "primary" : "gray", 78)}
    </g>`;
  }

  function criterion(y, title, body, score, kind = "green") {
    return `<g>
      ${rect(636, y, 430, 74, C.surface, C.line, 7)}
      ${txt(658, y + 28, title, "h3")}
      ${txt(658, y + 52, body, "small")}
      ${pill(962, y + 22, score, kind, 84)}
    </g>`;
  }

  function agentChip(x, y, name, role, kind = "green") {
    return `${rect(x, y, 254, 58, C.surface, C.line, 7)}
    ${txt(x + 16, y + 24, name, "h3")}
    ${pill(x + 150, y + 16, role, kind, 82)}
    ${txt(x + 16, y + 44, "この成果物種別の依頼で候補に入る", "small")}`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 96, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "ドメイン品質設定", "h1")}
    ${txt(276, 190, "ドメインごとの共通基準と、成果物種別ごとのレビュー観点を管理します。", "body")}
    ${button(1154, 154, "ドメインを追加", true, 124)}
    ${button(1286, 154, "複製", false, 68)}

    ${rect(252, 248, 366, 596, "#fbfcfe", C.line, 8)}
    ${txt(276, 288, "ドメイン選択", "h1")}
    ${selectBox(276, 320, "ドメイン", "開発", 294)}
    ${txt(276, 402, "このドメインの成果物種別", "h2")}
    ${artifactRow(426, "コード変更", "実装、テスト追加、修正作業", "使用中", true)}
    ${artifactRow(510, "レビュー結果", "差分確認、品質評価、採用判断", "使用中")}
    ${artifactRow(594, "テスト結果", "再現、検証、失敗原因の切り分け", "準備中")}
    ${artifactRow(678, "ドキュメント", "README、仕様、手順の更新", "使用中")}
    ${txt(276, 790, "別ドメインの成果物種別はここには表示しません。", "small", C.muted)}

    ${rect(636, 248, 456, 596, "#fbfcfe", C.line, 8)}
    ${txt(660, 288, "コード変更", "h1")}
    ${txt(660, 316, "AIに渡す品質基準とレビュー観点", "body")}
    ${button(946, 278, "編集", false, 72)}
    ${pill(660, 340, "適用: プロジェクト nagare", "blue", 196)}
    ${pill(872, 340, "最終更新: 6分前", "gray", 146)}
    ${txt(660, 404, "必ず守る前提", "h2")}
    ${txt(660, 430, "既存構成とテスト方針を尊重し、不要な大規模リファクタを避ける。", "body")}
    ${txt(660, 466, "禁止事項", "h2")}
    ${txt(660, 492, "未確認の破壊的操作、認証情報の露出、実装範囲外の設計変更。", "body")}
    ${txt(660, 548, "レビュー観点", "h2")}
    ${criterion(572, "目的整合性", "依頼の目的と変更内容が一致している", "必須")}
    ${criterion(656, "既存パターン尊重", "周辺コードの設計とUI規則に沿っている", "必須")}
    ${criterion(740, "確認可能性", "テストまたは目視で確認できる", "推奨", "amber")}

    ${rect(1112, 248, 244, 596, "#fbfcfe", C.line, 8)}
    ${txt(1132, 288, "利用状況", "h2")}
    ${txt(1132, 316, "この成果物種別を使うエージェント", "small")}
    ${agentChip(1132, 344, "create-agent", "作業")}
    ${agentChip(1132, 416, "review-agent", "レビュー", "blue")}
    ${txt(1132, 522, "最近の品質記録", "h2")}
    ${pill(1132, 548, "採用率 82%", "green", 104)}
    ${pill(1132, 584, "差し戻し 3件", "amber", 120)}
    ${txt(1132, 650, "見直し候補", "h2")}
    ${txt(1132, 678, "レビュー観点「確認可能性」の", "small")}
    ${txt(1132, 700, "記述を具体化する候補あり。", "small")}
    ${button(1132, 738, "改善候補を見る", false, 142)}
  `;

  return chrome(
    "artifact-types",
    "ドメイン",
    "ドメインと成果物種別ごとに品質基準を管理する",
    "",
    content,
    "ドメインに応じた成果物種別選択、選択成果物種別の品質基準、利用中エージェントと品質記録を確認する画面。"
  );
}

function assignmentBoard() {
  function lane(y, domain, agent, runtime, state, kind, capabilities) {
    return `${rect(252, y, 1124, 74, C.surface, C.line, 7)}
      ${txt(276, y + 29, domain, "h2")}
      ${txt(276, y + 52, "ドメイン", "tiny")}
      ${arrow(428, y + 37, 476, y + 37)}
      ${txt(502, y + 29, agent, "h3")}
      ${txt(502, y + 52, "エージェント / 役割: レビュアー", "small")}
      ${arrow(678, y + 37, 720, y + 37)}
      ${pill(742, y + 24, runtime, "primary", 118)}
      ${arrow(872, y + 37, 914, y + 37)}
      ${pill(934, y + 24, state, kind, 104)}
      ${capabilities.map((cap, i) => pill(1060 + i * 112, y + 24, cap[0], cap[1], 100)).join("")}`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 246, 192, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "カバー状況", "h2")}
    <circle cx="334" cy="232" r="54" fill="${C.greenSoft}" stroke="${C.green}" stroke-width="14"/>
    <path d="M334 178 A54 54 0 0 1 382 256" fill="none" stroke="${C.amber}" stroke-width="14" stroke-linecap="round"/>
    ${txt(312, 238, "6/7", "h1", C.ink)}
    ${txt(276, 298, "テストドメインが未割り当て", "small")}

    ${rect(526, 124, 850, 192, "#fbfcfe", C.line, 8)}
    ${txt(550, 162, "割り当て定義", "h2")}
    ${txt(550, 190, "プロジェクト x ドメイン x エージェントの1対1対応に、ランタイム・セッション方針・機能を載せる", "small")}
    ${pill(550, 230, "プロジェクト: Nagare", "primary", 168)}
    ${pill(736, 230, "スレッド方針: 再利用優先", "green", 218)}
    ${pill(970, 230, "自動代替禁止", "red", 126)}
    ${pill(1112, 230, "範囲限定", "teal", 98)}

    ${rect(252, 354, 1124, 54, C.panel, C.line, 7)}
    ${txt(276, 388, "ドメイン", "tiny")}
    ${txt(502, 388, "エージェント", "tiny")}
    ${txt(742, 388, "ランタイム", "tiny")}
    ${txt(934, 388, "セッション状態", "tiny")}
    ${txt(1060, 388, "機能", "tiny")}
    ${lane(408, "設計", "architect-agent", "Codex", "準備完了", "green", [["レビュー", "teal"], ["GitHub", "blue"]])}
    ${lane(494, "ツール方針", "policy-agent", "Claude", "承認待ち", "amber", [["ポリシー", "purple"], ["書込確認", "amber"]])}
    ${lane(580, "エージェント実行", "runtime-agent", "OpenClaw", "未接続", "red", [["アダプタ", "teal"], ["トレース", "blue"]])}
    ${lane(666, "実装", "worker-agent", "Codex CLI", "実行中", "green", [["実装", "teal"], ["テスト", "green"]])}

    ${rect(252, 782, 1124, 84, C.amberSoft, C.amber, 8)}
    ${txt(276, 818, "不足", "h2", C.amber)}
    ${txt(330, 818, "テストドメインはプロジェクトで有効だが割り当てがない。不透明な自動代替を避けるため先に担当を決める。", "body")}
    ${button(1030, 802, "テスト担当を作成", true, 152)}
    ${button(1200, 802, "一時的に無効化", false, 142)}
  `;

  return chrome(
    "Projects",
    "割り当てボード",
    "ドメインごとの担当、ランタイム、セッション方針、機能を一本の流れで確認する",
    `${button(1242, 28, "割り当て追加", true, 130)}`,
    content,
    "プロジェクト、ドメイン、エージェントの一対一割り当てと不足を示すボード。"
  );
}

function capabilityResolver() {
  function gate(y, label, owner, decision, kind, width) {
    const x = 414 + (760 - width) / 2;
    return `${rect(x, y, width, 52, C.surface, C.line, 8)}
    ${txt(x + 18, y + 23, label, "h3")}
    ${txt(x + 18, y + 42, owner, "small")}
    ${pill(x + width - 122, y + 14, decision, kind, 100)}`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 284, 720, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "選択中", "h2")}
    ${txt(276, 194, "hachiware-labs/hachi-search", "h3")}
    ${txt(276, 220, "スキル / Web・ローカル検索", "small")}
    ${line(276, 250, 512, 250)}
    ${pill(276, 282, "プロジェクト範囲", "primary", 152)}
    ${pill(276, 318, "対象: Codex", "blue", 118)}
    ${pill(276, 354, "全体付与しない", "red", 142)}
    ${txt(276, 420, "範囲混在の防止", "h3")}
    ${txt(276, 448, "全体インストールだけでは", "small")}
    ${txt(276, 470, "エージェントへ自動付与しない", "small")}

    ${rect(568, 124, 492, 720, "#fbfcfe", C.line, 8)}
    ${txt(592, 162, "範囲解決", "h2")}
    ${txt(592, 188, "登録から実行可能性まで、各スコープで絞り込む", "small")}
    ${gate(236, "レジストリ", "全体カタログ", "登録済み", "green", 430)}
    ${arrow(814, 288, 814, 318)}
    ${gate(318, "プロジェクト", "Nagare許可ポリシー", "許可", "green", 390)}
    ${arrow(814, 370, 814, 400)}
    ${gate(400, "ドメイン", "UX / ドキュメント", "許可", "green", 350)}
    ${arrow(814, 452, 814, 482)}
    ${gate(482, "エージェント", "researcher-agent", "付与済み", "primary", 310)}
    ${arrow(814, 534, 814, 564)}
    ${gate(564, "割り当て", "UX -> designer-agent", "未付与", "gray", 270)}
    ${arrow(814, 616, 814, 646)}
    ${gate(646, "ランタイム", "Codexスレッド_8cc2", "実行可", "green", 230)}

    ${rect(1092, 124, 284, 720, "#fbfcfe", C.line, 8)}
    ${txt(1116, 162, "実行時機能", "h2")}
    ${txt(1116, 188, "実行時に見える道具", "small")}
    ${pill(1116, 232, "許可: 2", "green", 96)}
    ${txt(1116, 266, "hachi-search", "body")}
    ${txt(1116, 292, "github-mcp", "body")}
    ${pill(1116, 344, "確認: 1", "amber", 96)}
    ${txt(1116, 378, "ファイル書き込み", "body")}
    ${pill(1116, 430, "禁止: 1", "red", 96)}
    ${txt(1116, 464, "figma-mcp", "body")}
    ${line(1116, 520, 1352, 520)}
    ${txt(1116, 558, "次の操作", "h3")}
    ${button(1116, 582, "割り当てへ付与", true, 146)}
    ${button(1116, 630, "プロジェクトから外す", false, 168)}
  `;

  return chrome(
    "Projects",
    "機能範囲の解決",
    "スキル / MCP / ツールを登録・許可・付与・実行可能性に分けて解決する",
    `${button(1236, 28, "機能を追加", true, 120)}${button(1104, 28, "ポリシー編集", false, 116)}`,
    content,
    "スキル、MCP、ツールの登録範囲から実行時に使える機能までを示す解決画面。"
  );
}

function librarySkills() {
  function tab(x, label, active = false, width = 96) {
    return `${rect(x, 144, width, 34, active ? C.primarySoft : C.surface, active ? C.primarySoft : C.line, 7)}
    ${txt(x + 18, 166, label, active ? "nav-on" : "nav")}`;
  }

  function selectBox(x, y, label, value, width = 220) {
    return `${txt(x, y, label, "h3")}
    ${rect(x, y + 14, width, 38, C.surface, C.strongLine, 7)}
    ${txt(x + 14, y + 39, value, "body")}
    ${pathD(`M ${x + width - 28} ${y + 29} L ${x + width - 20} ${y + 37} L ${x + width - 12} ${y + 29}`, C.muted, 1.8)}`;
  }

  function skillRow(y, name, desc, source, status, statusKind, provided, usedBy) {
    return `<g>
      ${rect(276, y, 1056, 82, C.surface, C.line, 0)}
      <rect x="276" y="${y}" width="4" height="82" rx="2" fill="${C.teal}"/>
      ${txt(300, y + 28, name, "h3")}
      ${txt(300, y + 50, desc, "small")}
      ${txt(704, y + 36, source, "body")}
      ${pill(840, y + 23, status, statusKind, 92)}
      ${txt(982, y + 28, provided, "small")}
      ${txt(982, y + 52, usedBy, "small")}
      ${button(1230, y + 24, "詳細", false, 72)}
    </g>`;
  }

  function scopeNode(x, y, label, value, kind) {
    return `${rect(x, y, 176, 74, C.surface, C.line, 8)}
    ${txt(x + 16, y + 26, label, "h3")}
    ${txt(x + 16, y + 50, value, "small")}
    ${pill(x + 104, y + 18, kind, kind === "許可" ? "green" : kind === "制限" ? "amber" : "gray", 58)}`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 86, "#fbfcfe", C.line, 8)}
    ${txt(276, 190, "スキルを登録し、プロジェクトやエージェントへ必要な範囲だけ提供します。", "body")}
    ${button(1162, 150, "スキルを追加", true, 112)}
    ${button(1274, 150, "同期", false, 74)}

    ${tab(276, "スキル", true, 92)}
    ${tab(378, "更新あり", false, 104)}
    ${tab(492, "未提供", false, 96)}
    ${tab(598, "問題あり", false, 104)}

    ${rect(252, 238, 1124, 76, "#fbfcfe", C.line, 8)}
    ${txt(276, 270, "追加後の設定", "h2")}
    ${txt(276, 296, "追加したスキルを使うには、エージェント画面で利用するスキルとして選択します。", "body")}
    ${button(1160, 260, "エージェント設定へ", false, 150)}

    ${rect(252, 340, 1124, 504, "#fbfcfe", C.line, 8)}
    ${txt(276, 380, "登録済みスキル", "h1")}
    ${selectBox(276, 416, "提供元", "すべて", 180)}
    ${selectBox(478, 416, "提供先", "すべて", 180)}
    ${selectBox(680, 416, "状態", "すべて", 160)}
    ${rect(832, 430, 250, 38, C.surface, C.strongLine, 7)}
    ${txt(850, 455, "名前で検索", "small", C.muted)}
    ${button(1102, 430, "絞り込み", false, 92)}
    ${button(1206, 430, "クリア", false, 78)}

    ${line(276, 496, 1332, 496)}
    ${txt(300, 526, "名前", "tiny")}
    ${txt(704, 526, "提供元", "tiny")}
    ${txt(840, 526, "状態", "tiny")}
    ${txt(982, 526, "提供状況", "tiny")}
    ${line(276, 544, 1332, 544)}
    ${skillRow(544, "hachiware-labs/hachi-search", "ローカル資料、Wiki、Web検索を横断する", "Vercel Skills", "登録済み", "green", "Project: nagare で許可", "Agent: researcher に提供")}
    ${skillRow(626, "hachiware-labs/hachi-ui", "SVG UI試作と自己評価を支援する", "Clawhub", "更新あり", "amber", "全体登録のみ", "まだ提供なし")}
    ${skillRow(708, "local/docs-review-skill", "ドキュメント変更のレビュー観点を提供する", "ローカル", "登録済み", "green", "Project: nagare で許可", "Agent: review-agent に提供")}
  `;

  return chrome(
    "Skills",
    "スキル管理",
    "スキルを登録し、必要な範囲だけ提供する",
    "",
    content,
    "スキルをMCPとは別に管理し、プロジェクトやエージェントへの提供範囲を確認する画面。"
  );
}

function libraryMcp() {
  function selectBox(x, y, label, value, width = 220) {
    return `${txt(x, y, label, "h3")}
    ${rect(x, y + 14, width, 38, C.surface, C.strongLine, 7)}
    ${txt(x + 14, y + 39, value, "body")}
    ${pathD(`M ${x + width - 28} ${y + 29} L ${x + width - 20} ${y + 37} L ${x + width - 12} ${y + 29}`, C.muted, 1.8)}`;
  }

  function mcpRow(y, name, desc, transport, auth, runtimeScope, status, statusKind, action) {
    return `<g>
      ${rect(276, y, 1056, 86, C.surface, C.line, 0)}
      <rect x="276" y="${y}" width="4" height="86" rx="2" fill="${C.purple}"/>
      ${txt(300, y + 28, name, "h3")}
      ${txt(300, y + 52, desc, "small")}
      ${txt(562, y + 36, transport, "body")}
      ${txt(654, y + 36, auth, "body")}
      ${txt(770, y + 30, runtimeScope, "small")}
      ${pill(1016, y + 23, status, statusKind, 92)}
      ${txt(1130, y + 36, action, "small")}
      ${button(1230, y + 26, "詳細", false, 72)}
    </g>`;
  }

  function gate(x, y, title, body, kind) {
    return `${rect(x, y, 230, 76, C.surface, C.line, 8)}
    ${txt(x + 16, y + 27, title, "h3")}
    ${txt(x + 16, y + 52, body, "small")}
    ${pill(x + 150, y + 18, kind, kind === "接続" ? "green" : kind === "確認" ? "amber" : "gray", 64)}`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 86, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "MCP管理", "h1")}
    ${txt(276, 190, "MCPサーバーを登録し、接続・認証・対応ランタイムを管理します。割り当てはエージェントで行います。", "body")}
    ${button(1162, 150, "MCPを追加", true, 112)}
    ${button(1274, 150, "接続確認", false, 94)}

    ${rect(252, 238, 1124, 76, "#fbfcfe", C.line, 8)}
    ${txt(276, 270, "エージェントで選べる条件", "h2")}
    ${txt(276, 296, "当面はランタイム種別ごとの固定テーブルで、エージェントへ付与できるMCPだけ候補に表示します。", "body")}
    ${button(1160, 260, "エージェント設定へ", false, 150)}

    ${rect(252, 340, 1124, 504, "#fbfcfe", C.line, 8)}
    ${txt(276, 380, "登録済み MCP", "h1")}
    ${selectBox(276, 416, "接続方式", "すべて", 180)}
    ${selectBox(478, 416, "認証", "すべて", 180)}
    ${selectBox(680, 416, "対応ランタイム", "すべて", 180)}
    ${rect(882, 430, 200, 38, C.surface, C.strongLine, 7)}
    ${txt(900, 455, "名前で検索", "small", C.muted)}
    ${button(1102, 430, "絞り込み", false, 92)}
    ${button(1206, 430, "クリア", false, 78)}

    ${line(276, 496, 1332, 496)}
    ${txt(300, 526, "名前", "tiny")}
    ${txt(562, 526, "方式", "tiny")}
    ${txt(654, 526, "認証", "tiny")}
    ${txt(770, 526, "対応ランタイム / スコープ", "tiny")}
    ${txt(1016, 526, "状態", "tiny")}
    ${txt(1130, 526, "割り当て", "tiny")}
    ${line(276, 544, 1332, 544)}
    ${mcpRow(544, "github-mcp", "Issue、PR、Actionsを参照・操作する", "stdio", "Token", "Codex/Claude: プロジェクト限定、Nagare管理", "接続済み", "green", "エージェントで設定")}
    ${mcpRow(626, "filesystem-mcp", "許可された作業フォルダを読み書きする", "stdio", "なし", "Codex/Claude/OpenCode: プロジェクト限定", "確認待ち", "amber", "エージェントで設定")}
    ${mcpRow(708, "figma-mcp", "デザインファイルを参照する", "http", "OAuth", "Claude: ユーザー全体のみ", "未接続", "red", "候補外")}
  `;

  return chrome(
    "MCP",
    "MCP",
    "MCPをスキルとは別に登録し、接続と提供範囲を管理する",
    "",
    content,
    "MCPサーバーをスキルとは別に管理し、接続、認証、プロジェクトやエージェントへの提供範囲を確認する画面。"
  );
}

function agentMcpAssignment() {
  function mcpCandidate(y, name, desc, scope, permissions, statusKind = "green") {
    return `<g>
      ${rect(276, y, 720, 96, C.surface, C.line, 7)}
      <rect x="276" y="${y}" width="4" height="96" rx="2" fill="${C.purple}"/>
      ${txt(300, y + 28, name, "h3")}
      ${txt(300, y + 52, desc, "small")}
      ${txt(300, y + 76, scope, "small", C.muted)}
      ${pill(748, y + 18, permissions, statusKind, 120)}
      ${button(878, y + 56, "追加", true, 82)}
    </g>`;
  }

  function assignedMcp(y, name, desc, permissions) {
    return `<g>
      ${rect(1030, y, 302, 84, C.surface, C.line, 7)}
      ${txt(1050, y + 28, name, "h3")}
      ${txt(1050, y + 50, desc, "small")}
      ${pill(1050, y + 58, permissions, "green", 122)}
      ${button(1238, y + 25, "外す", false, 70)}
    </g>`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 118, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "review-agent", "h1")}
    ${txt(276, 190, "レビュー担当のエージェントに、ランタイムが安全に扱えるMCPだけを付与します。", "body")}
    ${pill(276, 208, "ランタイム: Codex CLI", "primary", 166)}
    ${pill(456, 208, "MCP範囲: プロジェクト", "green", 174)}
    ${pill(644, 208, "プロジェクト: nagare", "gray", 154)}
    ${button(1166, 166, "保存", true, 80)}
    ${button(1258, 166, "キャンセル", false, 96)}

    ${rect(252, 270, 1124, 74, "#fbfcfe", C.line, 8)}
    ${txt(276, 300, "選択候補の条件", "h2")}
    ${txt(276, 326, "Codex CLIは固定テーブル上でプロジェクト単位の付与が可能なため、該当するMCPだけ表示します。", "body")}
    ${txt(1030, 326, "候補外: グローバルのみ / 未接続", "small", C.muted)}

    ${rect(252, 372, 768, 472, "#fbfcfe", C.line, 8)}
    ${txt(276, 412, "このエージェントで選べるMCP", "h1")}
    ${rect(276, 436, 326, 38, C.surface, C.strongLine, 7)}
    ${txt(294, 461, "名前で検索", "small", C.muted)}
    ${button(620, 436, "権限で絞り込み", false, 132)}
    ${button(762, 436, "クリア", false, 78)}
    ${mcpCandidate(502, "github-mcp", "Issue、PR、Actionsを参照・操作する", "Codex CLI: .codex/config.tomlへproject限定で反映", "参照 + 操作")}
    ${mcpCandidate(614, "filesystem-mcp", "許可された作業フォルダを読み書きする", "Codex CLI: 実行フォルダ内に限定して反映", "読取 + 書込", "amber")}

    ${rect(1030, 372, 326, 472, "#fbfcfe", C.line, 8)}
    ${txt(1050, 412, "割り当て済みMCP", "h2")}
    ${txt(1050, 438, "実行時にこのエージェントだけへ渡します。", "small")}
    ${assignedMcp(474, "github-mcp", "レビュー時にPRとIssueを確認", "参照 + 操作")}
    ${rect(1050, 586, 262, 112, C.panel, C.line, 8)}
    ${txt(1070, 618, "候補に出さないもの", "h3")}
    ${txt(1070, 646, "figma-mcp: 未接続", "small")}
    ${txt(1070, 672, "グローバルのみMCP: エージェント単位で制御不可", "small")}
    ${txt(1070, 740, "詳細な除外理由はMCP管理で確認", "small", C.muted)}
  `;

  return chrome(
    "Agents",
    "エージェントMCP設定",
    "エージェントのランタイムで安全に扱えるMCPだけを付与する",
    "",
    content,
    "エージェントのランタイム能力に応じて、選択可能なMCPだけを表示し、割り当て済みMCPを確認する画面。"
  );
}

function runtimeSessions() {
  function runtimeColumn(x, title, color, rows) {
    return `${rect(x, 228, 250, 470, "#fbfcfe", C.line, 8)}
    ${txt(x + 18, 264, title, "h2")}
    ${rows.map((row, i) => {
      const y = 296 + i * 82;
      return `${rect(x + 18, y, 214, 62, C.surface, C.line, 7)}
      ${txt(x + 34, y + 24, row[0], "h3")}
      ${txt(x + 34, y + 44, row[1], "small")}
      ${pill(x + 138, y + 20, row[2], row[3], 80)}`;
    }).join("")}
    ${line(x + 125, 196, x + 125, 228, color, 3)}`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 70, "#fbfcfe", C.line, 8)}
    ${txt(276, 166, "ランタイムセッション対応表", "h2")}
    ${txt(520, 166, "各ランタイムの固有名を、プロジェクト / ドメイン / エージェント / タスクの共通対応で読む", "body")}
    ${pill(1116, 146, "再利用優先", "green", 116)}
    ${pill(1248, 146, "分岐可", "blue", 86)}

    ${runtimeColumn(252, "Codex", C.primary, [["スレッド_8cc2", "Nagare / UX", "実行中", "green"], ["run_51e0", "実装 / worker", "待機", "gray"]])}
    ${runtimeColumn(532, "Claude Code", C.purple, [["session_12", "ツール方針", "承認待ち", "amber"], ["session_09", "ドキュメント / writer", "待機", "gray"]])}
    ${runtimeColumn(812, "OpenCode", C.teal, [["session_204", "ドキュメント / writer", "実行中", "green"], ["session_188", "設計 / architect", "分岐", "blue"]])}
    ${runtimeColumn(1092, "OpenClaw", C.red, [["チャネル未検出", "エージェント実行", "未検出", "red"]])}

    ${rect(252, 738, 784, 110, C.redSoft, C.red, 8)}
    ${txt(276, 776, "OpenClaw 接続エラー", "h2", C.red)}
    ${txt(276, 806, "OpenClawエージェントの割り当てが実行不可。", "body")}
    ${txt(276, 830, "インストール、またはランタイム切替が必要。", "body")}
    ${button(806, 796, "ランタイム切替", true, 130)}
    ${button(950, 796, "手順を見る", false, 100)}

    ${rect(1064, 738, 312, 110, "#fbfcfe", C.line, 8)}
    ${txt(1088, 776, "選択中の対応", "h2")}
    ${txt(1088, 806, "Codexスレッド_8cc2 / Nagare / UX", "body")}
    ${pill(1088, 824, "再開", "green", 76)}
    ${pill(1178, 824, "分岐可", "blue", 86)}
  `;

  return chrome(
    "Projects",
    "ランタイムセッション対応",
    "Codexスレッド / OpenCodeセッション / Claude Codeセッション / OpenClawチャネルを共通対応で扱う",
    `${button(1264, 28, "セッション作成", true, 126)}${button(1128, 28, "接続確認", false, 104)}`,
    content,
    "各ランタイム固有のセッションを共通のプロジェクト、ドメイン、エージェント、タスク対応で示す画面。"
  );
}

function traceInspector() {
  function step(x, y, n, title, detail, kind) {
    const [bg, fg] = tone(kind);
    return `<g>
      <circle cx="${x}" cy="${y}" r="17" fill="${bg}" stroke="${fg}" stroke-width="2"/>
      ${txt(x - 5, y + 5, n, "tiny", fg)}
      ${rect(x + 28, y - 27, 244, 58, C.surface, C.line, 8)}
      ${txt(x + 44, y - 4, title, "h3")}
      ${txt(x + 44, y + 18, detail, "small")}
    </g>`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 376, 720, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "ステップ履歴", "h2")}
    ${txt(276, 188, "依頼から結果までを一対一で追う", "small")}
    ${line(300, 246, 300, 702, C.strongLine, 2)}
    ${step(300, 252, "1", "プロジェクト解決", "Nagare / ルート確認", "green")}
    ${step(300, 338, "2", "ドメイン推定", "UX + ツール方針", "green")}
    ${step(300, 424, "3", "割り当て選択", "designer-agent / Codex", "green")}
    ${step(300, 510, "4", "機能解決", "hachi-search実行可", "green")}
    ${step(300, 596, "5", "承認要求", "ファイル書き込み確認", "amber")}
    ${step(300, 682, "6", "結果レビュー", "ルーブリック 6/7", "amber")}

    ${rect(660, 124, 354, 720, "#fbfcfe", C.line, 8)}
    ${txt(684, 162, "選定理由", "h2")}
    ${txt(684, 188, "選定理由をログではなく判断単位で表示", "small")}
    ${pill(684, 236, "ドメイン一致 0.91", "green", 158)}
    ${txt(684, 270, "UXルーブリックを担当可能", "body")}
    ${pill(684, 318, "プロジェクト適合", "green", 150)}
    ${txt(684, 352, "Nagare UI 指針を保持", "body")}
    ${pill(684, 400, "ランタイム再利用", "blue", 146)}
    ${txt(684, 434, "Codexスレッド継続可", "body")}
    ${pill(684, 482, "利用可能な道具", "teal", 142)}
    ${txt(684, 516, "hachi-search / github-mcp", "body")}
    ${line(684, 580, 990, 580)}
    ${txt(684, 622, "評価結果", "h2")}
    ${pill(684, 652, "6件通過", "green", 96)}
    ${pill(792, 652, "1件レビュー", "amber", 124)}
    ${txt(684, 704, "未確認: モバイル幅でセッション表が読めるか", "small")}

    ${rect(1048, 124, 328, 720, "#fbfcfe", C.line, 8)}
    ${txt(1072, 162, "判断", "h2")}
    ${txt(1072, 188, "依頼者が今判断すること", "small")}
    ${rect(1072, 226, 280, 128, C.amberSoft, C.amber, 8)}
    ${txt(1094, 264, "結論", "h2", C.amber)}
    ${txt(1094, 294, "解決は説明可能。", "body")}
    ${txt(1094, 320, "ファイル書き込みだけ承認待ち。", "body")}
    ${txt(1072, 406, "根拠", "h2")}
    ${pill(1072, 436, "UIモック変更", "blue", 136)}
    ${pill(1072, 470, "道具呼び出しOK", "green", 140)}
    ${pill(1072, 504, "レビュー 6/7", "amber", 116)}
    ${button(1072, 612, "承認して続行", true, 134)}
    ${button(1222, 612, "詳細ログ", false, 100)}
  `;

  return chrome(
    "Work Items",
    "トレース詳細",
    "どのエージェントが、なぜ選ばれ、どのセッション / ツールで何をしたかを説明する",
    `${button(1258, 28, "結果を採用", true, 116)}${button(1132, 28, "差し戻し", false, 102)}`,
    content,
    "ステップ履歴、エージェント選定理由、根拠、現在の判断を示すトレース詳細画面。"
  );
}

function emptyStateBootstrap() {
  function wizardStep(x, y, n, label, detail, kind) {
    const [bg, fg] = tone(kind);
    return `${rect(x, y, 176, 82, C.surface, C.line, 8)}
    <circle cx="${x + 28}" cy="${y + 30}" r="15" fill="${bg}" stroke="${fg}" stroke-width="2"/>
    ${txt(x + 23, y + 35, n, "tiny", fg)}
    ${txt(x + 54, y + 30, label, "h3")}
    ${txt(x + 18, y + 62, detail, "small")}`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 92, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "ワーク", "h1")}
    ${txt(276, 190, "依頼、実行前確認、実行中、レビュー、完了までの流れをここで追います。", "body")}
    ${pill(1120, 150, "セットアップ必要", "amber", 138)}
    ${pill(1268, 150, "ランタイム 0", "red", 108)}

    ${rect(252, 256, 1124, 252, "#fbfcfe", C.line, 8)}
    ${txt(276, 292, "ワークの流れ", "h2")}
    ${txt(276, 318, "プロジェクトとランタイムが未設定のため、まだ作成・実行はできません。", "small")}
    ${rect(276, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(300, 390, "依頼", "h3")}
    ${txt(300, 418, "停止中", "small", C.amber)}
    ${arrow(468, 403, 500, 403)}
    ${rect(500, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(524, 390, "実行前確認", "h3")}
    ${txt(524, 418, "プロジェクトが必要", "small", C.amber)}
    ${arrow(692, 403, 724, 403)}
    ${rect(724, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(748, 390, "実行中", "h3")}
    ${txt(748, 418, "ランタイムが必要", "small", C.red)}
    ${arrow(916, 403, 948, 403)}
    ${rect(948, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(972, 390, "レビュー", "h3")}
    ${txt(972, 418, "待機中", "small")}
    ${arrow(1140, 403, 1172, 403)}
    ${rect(1172, 354, 180, 98, C.panel, C.line, 8)}
    ${txt(1196, 390, "完了", "h3")}
    ${txt(1196, 418, "待機中", "small")}

    ${rect(252, 548, 544, 236, "#fbfcfe", C.line, 8)}
    ${txt(276, 584, "セットアップ状態", "h2")}
    ${pill(276, 622, "プロジェクト: 未設定", "amber", 178)}
    ${pill(276, 660, "ランタイム: 未接続", "red", 174)}
    ${pill(276, 698, "ワーク: 作成不可", "amber", 154)}

    ${rect(832, 548, 544, 236, "#fbfcfe", C.line, 8)}
    ${txt(856, 584, "セットアップ後", "h2")}
    ${pill(856, 622, "ドメイン: general", "blue", 156)}
    ${pill(1028, 622, "エージェント: default-worker", "green", 228)}
    ${pill(856, 660, "割り当て: general -> worker", "teal", 226)}
    ${pill(1106, 660, "セッション: 実行時に作成", "amber", 216)}
    ${txt(856, 728, "自動作成内容は実行前確認とトレースで表示され、後から変更できます。", "small")}

    <rect x="204" y="0" width="1236" height="1024" fill="#111827" opacity="0.18"/>
    ${rect(410, 176, 620, 526, C.surface, C.strongLine, 8)}
    ${txt(448, 224, "セットアップが必要です", "h1")}
    ${txt(448, 254, "ワークを実行するために、プロジェクトとランタイム接続だけ確認します。", "body")}
    ${txt(448, 278, "その他の構成は暫定作成し、あとで変更できます。", "small")}

    ${txt(448, 334, "このウィザードで行うこと", "h2")}
    ${wizardStep(448, 366, "1", "プロジェクト", "選択または作成", "primary")}
    ${arrow(624, 408, 646, 408)}
    ${wizardStep(646, 366, "2", "ランタイム", "1つだけ接続", "blue")}
    ${arrow(822, 408, 844, 408)}
    ${wizardStep(844, 366, "3", "準備完了", "ワークへ進む", "green")}

    ${rect(448, 500, 544, 76, C.greenSoft, C.green, 8)}
    ${txt(472, 532, "自動作成されるもの", "h3", C.green)}
    ${txt(472, 558, "generalドメイン / 汎用ルーブリック / default-workerエージェント / 割り当て", "small")}

    ${button(448, 626, "セットアップを開始", true, 160)}
  `;

  return chrome(
    "Work Items",
    "ワーク",
    "セットアップが必要な場合はウィザードから開始します",
    "",
    content,
    "空状態のワーク画面に、プロジェクト作成とランタイム接続を行うセットアップウィザードを重ねた画面。"
  );
}

function setupWizardProjectRuntime() {
  function wizardStep(x, y, n, label, state, kind) {
    const [bg, fg] = tone(kind);
    return `${rect(x, y, 166, 58, C.surface, C.line, 8)}
    <circle cx="${x + 24}" cy="${y + 29}" r="14" fill="${bg}" stroke="${fg}" stroke-width="2"/>
    ${txt(x + 19, y + 34, n, "tiny", fg)}
    ${txt(x + 48, y + 25, label, "h3")}
    ${txt(x + 48, y + 45, state, "small")}`;
  }

  function inputRow(y, label, value, width = 416, action = "") {
    return `${txt(412, y, label, "h3")}
    ${rect(412, y + 14, width, 40, C.surface, C.strongLine, 7)}
    ${txt(428, y + 40, value, "body")}
    ${action ? button(412 + width + 14, y + 14, action, false, 76) : ""}`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 92, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "ワーク", "h1")}
    ${txt(276, 190, "セットアップ中です。完了するとワークを作成できます。", "body")}
    ${pill(1120, 150, "セットアップ中", "blue", 126)}
    ${pill(1260, 150, "ランタイム 0", "red", 108)}

    ${rect(252, 256, 1124, 252, "#fbfcfe", C.line, 8)}
    ${txt(276, 292, "ワークの流れ", "h2")}
    ${txt(276, 318, "プロジェクトとランタイムの確認が終わるまで、作成・実行は保留されます。", "small")}
    ${rect(276, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(300, 390, "依頼", "h3")}
    ${txt(300, 418, "セットアップ待ち", "small", C.amber)}
    ${arrow(468, 403, 500, 403)}
    ${rect(500, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(524, 390, "実行前確認", "h3")}
    ${txt(524, 418, "待機中", "small")}
    ${arrow(692, 403, 724, 403)}
    ${rect(724, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(748, 390, "実行中", "h3")}
    ${txt(748, 418, "待機中", "small")}
    ${arrow(916, 403, 948, 403)}
    ${rect(948, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(972, 390, "レビュー", "h3")}
    ${txt(972, 418, "待機中", "small")}
    ${arrow(1140, 403, 1172, 403)}
    ${rect(1172, 354, 180, 98, C.panel, C.line, 8)}
    ${txt(1196, 390, "完了", "h3")}
    ${txt(1196, 418, "待機中", "small")}

    <rect x="204" y="0" width="1236" height="1024" fill="#111827" opacity="0.18"/>
    ${rect(344, 116, 752, 792, C.surface, C.strongLine, 8)}
    ${txt(386, 164, "プロジェクトを選ぶ", "h1")}
    ${txt(386, 194, "まず作業対象をプロジェクトとして登録します。ランタイム接続は次のステップで行います。", "body")}
    ${wizardStep(386, 232, "1", "プロジェクト", "入力中", "primary")}
    ${arrow(552, 261, 580, 261)}
    ${wizardStep(580, 232, "2", "ランタイム", "次", "gray")}
    ${arrow(746, 261, 774, 261)}
    ${wizardStep(774, 232, "3", "準備完了", "未完了", "gray")}

    ${rect(386, 326, 668, 246, "#fbfcfe", C.line, 8)}
    ${txt(412, 362, "プロジェクト情報", "h2")}
    ${txt(412, 390, "必須入力はプロジェクト名と場所だけです。", "small")}
    ${inputRow(424, "プロジェクト名", "nagare")}
    ${inputRow(504, "Gitリポジトリ / 作業フォルダ", "C:/Users/naruhide/workspace/nagare", 436, "選択")}

    ${rect(386, 596, 668, 62, "#fbfcfe", C.line, 8)}
    ${txt(412, 632, "評価基準（任意）", "h2")}
    ${button(944, 612, "編集", false, 78)}

    ${rect(386, 700, 668, 46, C.greenSoft, C.green, 8)}
    ${txt(412, 728, "確定後にランタイム接続へ進みます。評価基準は未設定でも続行できます。", "small", C.green)}

    ${button(386, 858, "戻る", false, 82)}
    ${button(962, 858, "次へ", true, 92)}
  `;

  return chrome(
    "Work Items",
    "ワーク",
    "セットアップ中です。まずプロジェクトを選択または作成します",
    "",
    content,
    "セットアップ開始後に、Gitリポジトリまたは作業フォルダを選びプロジェクトを確定する画面。"
  );
}

function setupWizardRuntimeConnection(runtimeKey = "codex-cli") {
  function wizardStep(x, y, n, label, state, kind) {
    const [bg, fg] = tone(kind);
    return `${rect(x, y, 166, 58, C.surface, C.line, 8)}
    <circle cx="${x + 24}" cy="${y + 29}" r="14" fill="${bg}" stroke="${fg}" stroke-width="2"/>
    ${txt(x + 19, y + 34, n, "tiny", fg)}
    ${txt(x + 48, y + 25, label, "h3")}
    ${txt(x + 48, y + 45, state, "small")}`;
  }

  function selectBox(x, y, label, value, width = 312) {
    return `${txt(x, y, label, "h3")}
    ${rect(x, y + 14, width, 40, C.surface, C.strongLine, 7)}
    ${txt(x + 16, y + 40, value, "body")}
    ${pathD(`M ${x + width - 28} ${y + 31} L ${x + width - 20} ${y + 39} L ${x + width - 12} ${y + 31}`, C.muted, 1.8)}`;
  }

  function fieldBox(x, y, label, value, width = 312, buttonText = "") {
    const buttonPart = buttonText
      ? `${rect(x + width - 84, y + 20, 70, 28, C.surface, C.strongLine, 6)}${txt(x + width - 70, y + 39, buttonText, "tiny")}`
      : "";
    return `${txt(x, y, label, "h3")}
    ${rect(x, y + 14, width, 40, C.surface, C.strongLine, 7)}
    ${txt(x + 16, y + 40, value, "body")}
    ${buttonPart}`;
  }

  function radio(x, y, label, selected, meta = "") {
    return `<g>
      <circle cx="${x + 9}" cy="${y + 10}" r="8" fill="${C.surface}" stroke="${selected ? C.primary : C.strongLine}" stroke-width="2"/>
      ${selected ? `<circle cx="${x + 9}" cy="${y + 10}" r="4" fill="${C.primary}"/>` : ""}
      ${txt(x + 26, y + 14, label, "body", selected ? C.ink : C.muted)}
      ${meta ? txt(x + 26, y + 34, meta, "small") : ""}
    </g>`;
  }

  const configs = {
    "codex-cli": {
      candidate: "Codex CLI（検出済み）",
      title: "Codex CLI",
      settings: `
        ${selectBox(412, 592, "モデル", "GPT-5.5", 316)}
        ${button(412, 664, "接続テスト", false, 104)}
        ${pill(532, 670, "未実行", "gray", 84)}
      `,
    },
    codex: {
      candidate: "Codex（アプリ接続）",
      title: "Codex",
      settings: `
        ${radio(412, 592, "既定モデルを使う", true)}
        ${radio(412, 636, "モデル名を手入力する", false)}
        ${rect(704, 630, 282, 40, C.surface, C.strongLine, 7)}
        ${txt(724, 656, "未入力", "small", C.muted)}
        ${button(412, 704, "接続テスト", false, 104)}
        ${pill(532, 710, "未実行", "gray", 84)}
      `,
    },
    claude: {
      candidate: "Claude Code（検出済み）",
      title: "Claude Code",
      settings: `
        ${radio(412, 592, "既定モデルを使う", true)}
        ${radio(412, 636, "モデル名を手入力する", false)}
        ${rect(704, 630, 282, 40, C.surface, C.strongLine, 7)}
        ${txt(724, 656, "例: sonnet", "small", C.muted)}
        ${button(412, 704, "接続テスト", false, 104)}
        ${pill(532, 710, "未実行", "gray", 84)}
      `,
    },
    opencode: {
      candidate: "OpenCode（検出済み）",
      title: "OpenCode",
      settings: `
        ${selectBox(412, 592, "Provider", "OpenAI", 220)}
        ${selectBox(656, 592, "モデル", "gpt-5.5", 330)}
        ${button(412, 664, "接続テスト", false, 104)}
        ${pill(532, 670, "未実行", "gray", 84)}
      `,
    },
    openclaw: {
      candidate: "OpenClaw（検出済み）",
      title: "OpenClaw",
      settings: `
        ${selectBox(412, 592, "Provider", "OpenAI", 220)}
        ${selectBox(656, 592, "モデル", "gpt-5.5", 330)}
        ${button(412, 664, "接続テスト", false, 104)}
        ${pill(532, 670, "未実行", "gray", 84)}
      `,
    },
  };
  const cfg = configs[runtimeKey] || configs["codex-cli"];

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 92, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "ワーク", "h1")}
    ${txt(276, 190, "セットアップ中です。ランタイムを1つ選ぶとワークを作成できます。", "body")}
    ${pill(1120, 150, "セットアップ中", "blue", 126)}
    ${pill(1260, 150, "ランタイム 1", "green", 108)}

    ${rect(252, 256, 1124, 252, "#fbfcfe", C.line, 8)}
    ${txt(276, 292, "ワークの流れ", "h2")}
    ${txt(276, 318, "ランタイム接続が終わるまで、作成・実行は保留されます。", "small")}
    ${rect(276, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(300, 390, "依頼", "h3")}
    ${txt(300, 418, "セットアップ待ち", "small", C.amber)}
    ${arrow(468, 403, 500, 403)}
    ${rect(500, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(524, 390, "実行前確認", "h3")}
    ${txt(524, 418, "待機中", "small")}
    ${arrow(692, 403, 724, 403)}
    ${rect(724, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(748, 390, "実行中", "h3")}
    ${txt(748, 418, "待機中", "small")}
    ${arrow(916, 403, 948, 403)}
    ${rect(948, 354, 192, 98, C.panel, C.line, 8)}
    ${txt(972, 390, "レビュー", "h3")}
    ${txt(972, 418, "待機中", "small")}
    ${arrow(1140, 403, 1172, 403)}
    ${rect(1172, 354, 180, 98, C.panel, C.line, 8)}
    ${txt(1196, 390, "完了", "h3")}
    ${txt(1196, 418, "待機中", "small")}

    <rect x="204" y="0" width="1236" height="1024" fill="#111827" opacity="0.18"/>
    ${rect(344, 116, 752, 792, C.surface, C.strongLine, 8)}
    ${txt(386, 164, "ランタイムを設定", "h1")}
    ${txt(386, 194, "このプロジェクトで最初に使う実行先を1つ選びます。詳細設定は選択したものだけ表示します。", "body")}
    ${wizardStep(386, 232, "1", "プロジェクト", "完了", "green")}
    ${arrow(552, 261, 580, 261)}
    ${wizardStep(580, 232, "2", "ランタイム", "選択中", "primary")}
    ${arrow(746, 261, 774, 261)}
    ${wizardStep(774, 232, "3", "準備完了", "次", "gray")}

    ${rect(386, 326, 668, 136, "#fbfcfe", C.line, 8)}
    ${txt(412, 362, "ランタイム候補", "h2")}
    ${selectBox(412, 392, "ランタイム", cfg.candidate, 356)}

    ${rect(386, 494, 668, 236, "#fbfcfe", C.line, 8)}
    ${txt(412, 530, "選択中の設定", "h2")}
    ${txt(412, 558, cfg.title, "h1")}
    ${cfg.settings}

    ${button(386, 858, "戻る", false, 82)}
    ${button(962, 858, "次へ", true, 92)}
  `;

  return chrome(
    "Work Items",
    "ランタイム設定",
    "セットアップ中です。最初に使うランタイムを接続します",
    "",
    content,
    `${cfg.title}を最初に使うランタイムとして選び、必要なモデル指定と接続確認を行う画面。`
  );
}

function workItemComposer() {
  function selectBox(x, y, label, value, width = 312) {
    return `${txt(x, y, label, "h3")}
    ${rect(x, y + 14, width, 40, C.surface, C.strongLine, 7)}
    ${txt(x + 16, y + 40, value, "body")}
    ${pathD(`M ${x + width - 28} ${y + 31} L ${x + width - 20} ${y + 39} L ${x + width - 12} ${y + 31}`, C.muted, 1.8)}`;
  }

  function workRow(y, title, project, status, statusKind, updated, summary) {
    return `<g>
      ${rect(276, y, 1056, 64, C.surface, C.line, 0)}
      ${txt(300, y + 26, title, "h3")}
      ${txt(300, y + 48, project, "small")}
      ${pill(716, y + 20, status, statusKind, 92)}
      ${txt(854, y + 38, summary, "body")}
      ${txt(1118, y + 38, updated, "small")}
      ${button(1234, y + 16, "詳細", false, 70)}
    </g>`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 316, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "新規ワーク", "h1")}
    ${txt(276, 190, "依頼内容を入力して実行します。プロジェクトは任意で選べます。", "body")}
    ${txt(276, 238, "依頼内容", "h2")}
    ${rect(276, 260, 1056, 82, C.surface, C.strongLine, 8)}
    ${txt(300, 298, "READMEを新構想に合わせて更新してください", "body")}
    ${selectBox(276, 374, "プロジェクト（任意）", "未選択", 320)}
    ${txt(620, 414, "未選択でも実行できます", "small")}
    ${button(1164, 388, "実行", true, 92)}

    ${rect(252, 480, 1124, 362, "#fbfcfe", C.line, 8)}
    ${txt(276, 520, "ワーク一覧", "h1")}
    ${txt(276, 548, "これまでの依頼と現在の状態を確認します。", "body")}
    ${selectBox(276, 578, "プロジェクト", "すべて", 220)}
    ${selectBox(520, 578, "処理状態", "すべて", 220)}
    ${rect(764, 592, 220, 40, C.surface, C.strongLine, 7)}
    ${txt(782, 618, "キーワードで検索", "small", C.muted)}
    ${button(1008, 592, "絞り込み", false, 92)}
    ${button(1112, 592, "クリア", false, 78)}
    ${line(276, 654, 1332, 654)}
    ${txt(300, 686, "ワーク", "tiny")}
    ${txt(716, 686, "状態", "tiny")}
    ${txt(854, 686, "回答サマリ", "tiny")}
    ${txt(1118, 686, "更新", "tiny")}
    ${line(276, 704, 1332, 704)}
    ${workRow(704, "READMEを新構想に合わせて更新", "nagare", "実行中", "blue", "2分前", "READMEを確認中")}
    ${workRow(768, "ランタイム設定画面を整理", "nagare", "完了", "green", "18分前", "不要なラベルを削除")}
  `;

  return chrome(
    "Work Items",
    "ワーク",
    "新しい依頼を実行し、これまでのワークを一覧で確認する",
    "",
    content,
    "新規ワークの依頼フォームと、既存ワークの一覧を同じ画面で扱うワーク画面。"
  );
}

function organizerPreflight() {
  function resolveStep(x, y, n, title, value, kind) {
    const [bg, fg] = tone(kind);
    return `${rect(x, y, 150, 88, C.surface, C.line, 8)}
    <circle cx="${x + 25}" cy="${y + 26}" r="14" fill="${bg}" stroke="${fg}" stroke-width="2"/>
    ${txt(x + 20, y + 31, n, "tiny", fg)}
    ${txt(x + 50, y + 29, title, "h3")}
    ${txt(x + 16, y + 60, value, "small")}`;
  }

  const content = `
    ${rect(232, 96, 1164, 828, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 92, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "実行前確認", "h1")}
    ${txt(276, 190, "実行前に、自動作成された構成と解決結果だけを確認する。拒否がなければワークを実行できる。", "body")}
    ${pill(1172, 150, "実行可能", "green", 100)}

    ${rect(252, 258, 1124, 178, "#fbfcfe", C.line, 8)}
    ${txt(276, 294, "解決された実行構成", "h2")}
    ${resolveStep(276, 326, "1", "プロジェクト", "nagare", "primary")}
    ${arrow(426, 370, 448, 370)}
    ${resolveStep(448, 326, "2", "ドメイン", "general", "blue")}
    ${arrow(598, 370, 620, 370)}
    ${resolveStep(620, 326, "3", "エージェント", "default-worker", "green")}
    ${arrow(770, 370, 792, 370)}
    ${resolveStep(792, 326, "4", "ランタイム", "Codex CLI", "teal")}
    ${arrow(942, 370, 964, 370)}
    ${resolveStep(964, 326, "5", "セッション", "新規", "amber")}
    ${arrow(1114, 370, 1136, 370)}
    ${resolveStep(1136, 326, "6", "トレース", "自動構成を記録", "purple")}

    ${rect(252, 486, 352, 286, "#fbfcfe", C.line, 8)}
    ${txt(276, 522, "ポリシー", "h2")}
    ${pill(276, 560, "拒否: 0", "green", 96)}
    ${pill(276, 598, "確認: 書き込み時", "amber", 156)}
    ${pill(276, 636, "外部送信: ランタイム既定", "gray", 210)}
    ${txt(276, 700, "初回は危険操作を自動承認しない。", "small")}

    ${rect(636, 486, 352, 286, "#fbfcfe", C.line, 8)}
    ${txt(660, 522, "自動セットアップ", "h2")}
    ${pill(660, 560, "ドメイン: general", "blue", 154)}
    ${pill(660, 598, "ルーブリック: 一般品質", "purple", 194)}
    ${pill(660, 636, "エージェント: default-worker", "green", 224)}
    ${pill(660, 674, "割り当てを作成", "teal", 150)}
    ${txt(660, 728, "自動作成としてトレースに残す。", "small")}

    ${rect(1020, 486, 356, 286, "#fbfcfe", C.line, 8)}
    ${txt(1044, 522, "実行可能", "h2")}
    ${txt(1044, 556, "ワークを実行できます。", "body")}
    ${txt(1044, 586, "実行後は実行トレースへ移動します。", "small")}
    ${button(1044, 640, "ワークを実行", true, 130)}
    ${button(1214, 640, "戻る", false, 86)}
  `;

  return chrome(
    "Work Items",
    "実行前確認",
    "ワーク実行前に、プロジェクト / ドメイン / エージェント / ランタイム / セッション / ポリシーの解決結果を確認する",
    `${button(1218, 28, "ワークを実行", true, 130)}`,
    content,
    "ワーク実行前に、解決済みの実行構成とポリシー状態を確認する画面。"
  );
}

function workItemRunTrace() {
  function labelValue(x, y, label, lines, width = 420) {
    const values = Array.isArray(lines) ? lines : [lines];
    return `<g>
      ${txt(x, y, label, "tiny", C.muted)}
      ${rect(x, y + 10, width, 108, C.panel, C.line, 6)}
      ${values.slice(0, 5).map((value, i) => txt(x + 12, y + 31 + i * 18, value, i === 0 ? "body" : "small")).join("")}
    </g>`;
  }

  function stepRow(y, n, title, role, owner, state, kind, fields) {
    const labels = {
      organizer: ["判断材料", "割り当て結果", "設定した流れ", "理由"],
      worker: ["入力", "作業結果", "成果物", "補足"],
      review: ["レビュー対象", "評価", "スコア", "指摘"],
    }[role] || ["入力", "結果", "成果物", "補足"];
    const roleColor = role === "organizer" ? C.blue : role === "review" ? C.purple : C.green;
    const roleText = role === "organizer" ? "Role: オーガナイザー" : role === "review" ? "Role: レビュー" : "Role: ワーカー";
    const running = state === "実行中";
    const cardFill = running ? "#eff6ff" : C.surface;
    const cardStroke = running ? C.blueSoft : C.line;
    const accentWidth = running ? 7 : 5;

    return `<g>
      ${rect(276, y, 1056, 318, cardFill, cardStroke, 8)}
      <rect x="276" y="${y}" width="${accentWidth}" height="318" rx="3" fill="${roleColor}"/>
      <circle cx="308" cy="${y + 35}" r="15" fill="${tone(kind)[0]}" stroke="${tone(kind)[1]}" stroke-width="2"/>
      ${txt(303, y + 40, n, "tiny", tone(kind)[1])}
      ${txt(338, y + 32, title, "h2")}
      ${txt(458, y + 32, roleText, "small", roleColor)}
      ${txt(990, y + 36, owner, "body")}
      ${pill(1176, y + 20, state, kind, 92)}
      ${running ? txt(1074, y + 64, "現在処理中", "small", C.blue) : ""}
      ${labelValue(338, y + 58, labels[0], fields[0], 450)}
      ${labelValue(812, y + 58, labels[1], fields[1], 410)}
      ${labelValue(338, y + 190, labels[2], fields[2], 450)}
      ${labelValue(812, y + 190, labels[3], fields[3], 410)}
    </g>`;
  }

  const content = `
    ${rect(232, 96, 1164, 2074, C.surface, C.line, 8)}
    ${rect(252, 124, 1124, 168, "#fbfcfe", C.line, 8)}
    ${txt(276, 162, "ワーク詳細", "h1")}
    ${txt(276, 190, "READMEを新構想に合わせて更新", "h2")}
    ${pill(276, 222, "実行中", "blue", 92)}
    ${txt(392, 238, "プロジェクト: nagare", "body")}
    ${txt(580, 238, "更新: 2分前", "body")}
    ${button(1258, 146, "停止", false, 74)}

    ${rect(252, 324, 1124, 156, "#fbfcfe", C.line, 8)}
    ${txt(276, 360, "回答サマリ", "h2")}
    ${txt(276, 394, "現在の結論", "h3")}
    ${txt(382, 394, "README差分案は作成済み。レビューで構想整合性を確認中です。", "body")}
    ${txt(276, 424, "次に必要な確認", "h3")}
    ${txt(382, 424, "レビュー完了後、ユーザーは差分案とレビュー指摘を確認します。", "body")}
    ${txt(276, 452, "現時点ではユーザーの承認待ちはありません。", "small")}
    ${button(1052, 390, "差分を見る", false, 104)}
    ${button(1174, 390, "レビュー結果", false, 118)}

    ${rect(252, 520, 1124, 1588, "#fbfcfe", C.line, 8)}
    ${txt(276, 560, "ステップ", "h1")}
    ${txt(276, 588, "各ステップで受け取ったもの、担当者、結果を確認します。", "body")}
    ${line(276, 624, 1332, 624)}
    ${txt(338, 654, "ステップ", "tiny")}
    ${txt(990, 654, "担当者", "tiny")}
    ${txt(1176, 654, "状態", "tiny")}
    ${line(276, 672, 1332, 672)}
    ${stepRow(692, "1", "依頼受付", "organizer", "Nagare", "完了", "green", [["READMEを新構想に合わせて更新", "プロジェクト指定は未選択のまま実行", "依頼文は短く、対象ファイルはREADMEと解釈", "追加条件や締切は指定なし", "危険操作の指示は含まれていない"], ["README更新ワークを作成", "対象プロジェクトは現在の文脈から nagare と推定", "初回セットアップ済みのRuntimeを利用候補に設定", "実行前確認を経由せず通常実行へ進行", "履歴に表示する回答サマリを初期化"], ["work-1048", "依頼本文、作成時刻、推定プロジェクトを記録", "状態は実行中として登録", "詳細画面でステップ追跡できるよう初期行を作成", "ユーザー操作は実行ボタンのみ"], ["ユーザー確認は不要", "次のステップで担当エージェントを決定", "プロジェクト未選択でも開始できる方針に従う", "後続で推定が誤っていれば差し戻し可能", "この時点では成果物はまだない"]])}
    ${txt(616, 1020, "work-1048 を判断材料として次へ", "tiny", C.muted)}
    ${arrow(806, 1014, 806, 1030, C.strongLine)}
    ${stepRow(1030, "2", "割り当て", "organizer", "オーガナイザー", "完了", "green", [["ワーク内容と現在のプロジェクト", "README更新と構想反映が主目的", "対象はドキュメント作成寄りの作業", "レビューが必要な変更と判断", "利用可能RuntimeはCodex CLI"], ["作成エージェントを選定", "レビューエージェントを後続ステップに設定", "default-workerではなく文書作成向けに割り当て", "レビュー完了後に回答サマリへ統合予定", "承認が必要な操作はまだない"], ["assignment: create-agent", "review-agent をレビュー担当として予約", "作成ステップとレビューステップを分離", "担当、Runtime、状態をトレースへ保存", "ユーザーが後で割り当て理由を追えるよう記録"], ["汎用ワーカーではなく文書作成寄りに割り当て", "実行前に危険な操作は検出されていない", "スキル追加は不要と判断", "モデルやProviderの詳細は表示しない", "内部ログは詳細ログ側へ下げる"]])}
    ${txt(592, 1358, "assignment: create-agent を入力として次へ", "tiny", C.muted)}
    ${arrow(806, 1352, 806, 1368, C.strongLine)}
    ${stepRow(1368, "3", "作成", "worker", "create-agent", "完了", "green", [["READMEとNagareの新構想メモ", "既存説明と新しい制御プレーン構想を比較", "Project-aware control plane の説明が不足", "初回導線とRuntime連携の説明も不足", "ワーク一覧/詳細の考え方は未反映"], ["説明文と差分案を作成", "Project-aware control plane の説明をREADMEへ追加", "初回セットアップからワーク実行までの流れを追記", "RuntimeとAgentの関係をツール非依存で記述", "ユーザーに見える概念を中心に文章化"], ["README差分案", "概要、初回導線、Runtime連携の説明を含む", "Agent/Runtime/Skillの関係を整理", "ワーク一覧と詳細で状態を追える前提を追加", "未コミットの差分として保持"], ["未コミットの差分として保持", "レビュー完了後にユーザー確認へ進める", "表現はまだ最終ではない", "用語の揺れをレビュー対象にする", "成果物はこのステップで初めて発生"]])}
    ${txt(604, 1696, "README差分案 をレビュー対象として次へ", "tiny", C.muted)}
    ${arrow(806, 1690, 806, 1706, C.strongLine)}
    ${stepRow(1706, "4", "レビュー", "review", "review-agent", "実行中", "blue", [["README差分案", "作成エージェントが出した説明文と変更案", "新構想の資料と照合", "ユーザーが優先したUI導線とも照合", "未確定の用語がないか確認"], ["構想整合性はOK", "RuntimeよりProject/Workを中心に説明できている", "初回導線の記述は概ね自然", "スキル管理の説明は追加余地あり", "表現修正1件を指摘し、確認を継続中"], ["README品質 / 構想整合性: 82 / 100", "構想整合性 28/30", "分かりやすさ 24/30", "具体性 18/25", "残件 12/15"], ["用語の揺れと導線説明の不足を指摘", "追加修正が必要なら作成ステップへ戻す", "レビュー結果は詳細画面に残す", "長いログは初期表示しない", "完了後に回答サマリへ反映予定"]])}
  `;

  return chrome(
    "Work Items",
    "ワーク詳細",
    "ワークの現在状態、回答サマリ、ステップごとの担当者を確認する",
    "",
    content,
    "一覧から開いたワークの状態、回答サマリ、担当者付きステップを表示する詳細画面。",
    2210
  );
}

const screens = [
  ["30-project-control-plane-overview.svg", projectControlPlaneOverview()],
  ["31-project-context.svg", projectContext()],
  ["31a-domain-quality-settings.svg", domainQualitySettings()],
  ["32-assignment-board.svg", assignmentBoard()],
  ["33-capability-scope-resolver.svg", capabilityResolver()],
  ["34-runtime-session-bindings.svg", runtimeSessions()],
  ["35-trace-inspector.svg", traceInspector()],
  ["36-empty-state-bootstrap.svg", emptyStateBootstrap()],
  ["37-setup-wizard-project-runtime.svg", setupWizardProjectRuntime()],
  ["38-setup-wizard-runtime.svg", setupWizardRuntimeConnection()],
  ["38a-setup-wizard-runtime-codex.svg", setupWizardRuntimeConnection("codex")],
  ["38b-setup-wizard-runtime-claude-code.svg", setupWizardRuntimeConnection("claude")],
  ["38c-setup-wizard-runtime-opencode.svg", setupWizardRuntimeConnection("opencode")],
  ["38d-setup-wizard-runtime-openclaw.svg", setupWizardRuntimeConnection("openclaw")],
  ["39-work-item-composer.svg", workItemComposer()],
  ["40-organizer-preflight.svg", organizerPreflight()],
  ["41-work-item-run-trace.svg", workItemRunTrace()],
  ["42-library-skills.svg", librarySkills()],
  ["43-library-mcp.svg", libraryMcp()],
  ["44-agent-mcp-assignment.svg", agentMcpAssignment()],
];

for (const [name, svg] of screens) {
  fs.writeFileSync(path.join(outDir, name), svg, "utf8");
}

console.log(`Generated ${screens.length} hachi-ui SVG mockups in ${outDir}`);
