const fs = require("fs");
const path = require("path");

const outDir = path.join(__dirname, "..", "docs", "design-assets", "svg");
fs.mkdirSync(outDir, { recursive: true });
for (const file of fs.readdirSync(outDir)) {
  if (file.endsWith(".svg")) fs.unlinkSync(path.join(outDir, file));
}

const C = {
  bg: "#f8fafc",
  surface: "#ffffff",
  surface2: "#f8fafc",
  text: "#020617",
  muted: "#334155",
  faint: "#94a3b8",
  line: "#e2e8f0",
  blue: "#4338ca",
  blueSoft: "#eef2ff",
  green: "#047857",
  greenSoft: "#ecfdf5",
  amber: "#b45309",
  amberSoft: "#fffbeb",
  red: "#b91c1c",
  redSoft: "#fef2f2",
  gray: "#475569",
  graySoft: "#f1f5f9",
};

function esc(value) {
  return String(value).replace(/[&<>"]/g, (ch) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
  })[ch]);
}

function svgDoc(width, height, body) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">
  <rect width="${width}" height="${height}" fill="${C.bg}"/>
  <style>
    .title{font:700 24px Inter,Segoe UI,"Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.text}}
    .h2{font:700 17px Inter,Segoe UI,"Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.text}}
    .h3{font:700 14px Inter,Segoe UI,"Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.text}}
    .body{font:500 13px Inter,Segoe UI,"Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.text}}
    .small{font:500 11px Inter,Segoe UI,"Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.muted}}
    .tiny{font:600 10px Inter,Segoe UI,"Yu Gothic UI",Meiryo,Arial,sans-serif;fill:${C.muted}}
    .mono{font:500 12px Consolas,Menlo,monospace;fill:${C.text}}
    .nav{font:650 13px Inter,Segoe UI,Arial,sans-serif;fill:${C.muted}}
    .nav-on{font:700 13px Inter,Segoe UI,Arial,sans-serif;fill:${C.blue}}
  </style>
  ${body}
</svg>`;
}

function rect(x, y, w, h, fill = C.surface, stroke = C.line, r = 8) {
  return `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" fill="${fill}" stroke="${stroke}"/>`;
}

function line(x1, y1, x2, y2, stroke = C.line) {
  return `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${stroke}"/>`;
}

function circle(cx, cy, r, fill = C.surface, stroke = C.line) {
  return `<circle cx="${cx}" cy="${cy}" r="${r}" fill="${fill}" stroke="${stroke}" stroke-width="2"/>`;
}

function txt(x, y, text, cls = "body", fill = null) {
  return `<text x="${x}" y="${y}" class="${cls}"${fill ? ` style="fill:${fill}"` : ""}>${esc(text)}</text>`;
}

function pillWidth(text) {
  return Math.max(48, text.length * 7 + 18);
}

function pill(x, y, text, kind = "gray") {
  const map = {
    blue: [C.blueSoft, C.blue],
    green: [C.greenSoft, C.green],
    amber: [C.amberSoft, C.amber],
    red: [C.redSoft, C.red],
    gray: [C.graySoft, C.gray],
  };
  const [bg, fg] = map[kind] || map.gray;
  const w = pillWidth(text);
  return `<rect x="${x}" y="${y}" width="${w}" height="24" rx="12" fill="${bg}" stroke="${bg}"/>
  <text x="${x + 9}" y="${y + 16}" class="tiny" style="fill:${fg}">${esc(text)}</text>`;
}

function button(x, y, text, primary = false, w = null) {
  const width = w || Math.max(86, text.length * 8 + 24);
  const fill = primary ? C.blue : C.surface;
  const stroke = primary ? C.blue : C.line;
  const color = primary ? "#ffffff" : C.text;
  return `<rect x="${x}" y="${y}" width="${width}" height="34" rx="7" fill="${fill}" stroke="${stroke}"/>
  <text x="${x + 14}" y="${y + 22}" class="body" style="fill:${color}">${esc(text)}</text>`;
}

function disabledButton(x, y, text, w = null) {
  const width = w || Math.max(86, text.length * 8 + 24);
  return `<rect x="${x}" y="${y}" width="${width}" height="34" rx="7" fill="${C.graySoft}" stroke="${C.line}"/>
  <text x="${x + 14}" y="${y + 22}" class="body" style="fill:${C.faint}">${esc(text)}</text>`;
}

function appChrome(active, body, width = 1440, height = 960) {
  const navItems = ["Work Items", "Agents", "Settings"];
  const nav = navItems.map((item, i) => {
    const y = 118 + i * 38;
    const selected = item === active;
    return `${selected ? rect(18, y - 22, 164, 32, C.blueSoft, C.blueSoft, 7) : ""}
      ${txt(34, y, item, selected ? "nav-on" : "nav")}`;
  }).join("");
  return svgDoc(width, height, `
    ${rect(0, 0, 200, height, "#ffffff", C.line, 0)}
    ${txt(28, 42, "Nagare", "title")}
    ${txt(30, 64, "HACHIWARE LABS", "tiny", C.blue)}
    ${txt(30, 82, "Agent work control", "small")}
    ${nav}
    ${line(200, 0, 200, height)}
    ${body}
  `);
}

function sectionTitle(x, y, title, meta = "") {
  return `${txt(x, y, title, "h2")}${meta ? txt(x, y + 20, meta, "small") : ""}`;
}

function tableRow(x, y, w, cols, badges = []) {
  const badgeWidths = badges.map((b) => pillWidth(b[0]));
  const badgeTotal = badgeWidths.reduce((sum, width) => sum + width, 0) + Math.max(0, badges.length - 1) * 10;
  const badgeStart = badges.length ? x + w - badgeTotal - 14 : x + w;
  const colWidth = (badgeStart - x - 28) / cols.length;
  let out = rect(x, y, w, 54, C.surface, C.line, 0);
  cols.forEach((col, i) => {
    out += txt(x + 14 + i * colWidth, y + 21, col[0], i === 0 ? "body" : "small");
    if (col[1]) out += txt(x + 14 + i * colWidth, y + 39, col[1], "small");
  });
  let badgeX = badgeStart;
  badges.forEach((b, i) => {
    out += pill(badgeX, y + 15, b[0], b[1]);
    badgeX += badgeWidths[i] + 10;
  });
  return out;
}

function workItemBoard() {
  const rows = [
    [[["work_0048", "READMEの英語版を更新"], ["実行中", "2分前"], ["writing-agent", "target accepted"], ["次: 完了待ち", "artifact pending"]], [["0", "gray"]]],
    [[["work_0042", "調査ソース一覧を作成"], ["要確認", "12分前"], ["research-agent", "run succeeded"], ["次: 検証", "evidenceあり"]], [["0", "gray"]]],
    [[["work_0041", "dispatch fallbackを修正"], ["失敗", "28分前"], ["writing-agent", "run failed"], ["次: handoff検討", "stderrあり"]], [["1", "red"]]],
    [[["work_0038", "tutorial copyを下書き"], ["下書き", "2時間前"], ["未確定", "dispatch draft"], ["次: 依頼先確認", "candidate 3"]], [["0", "gray"]]],
  ];
  const body = `
    ${txt(232, 48, "Work Items", "title")}
    ${pill(1110, 30, "locale ja-JP", "blue")}
    ${pill(1226, 30, "dispatch: dispatch-app", "gray")}
    ${rect(232, 82, 894, 188, C.surface, C.line, 8)}
    ${sectionTitle(252, 118, "仕事を依頼", "対象と依頼文だけ入れて、依頼先を確認する")}
    ${rect(252, 138, 690, 58, C.surface2, C.line, 7)}
    ${txt(272, 172, "docsの変更点を確認して、READMEに反映して", "body")}
    ${pill(252, 214, "対象 docs/", "blue")}
    ${pill(348, 214, "目的: 更新", "gray")}
    ${pill(448, 214, "Agentおまかせ", "green")}
    ${button(952, 142, "依頼先を確認", true, 134)}
    ${button(952, 184, "下書き保存", false, 110)}
    ${rect(1150, 82, 246, 702, C.surface, C.line, 8)}
    ${sectionTitle(1170, 118, "選択中のタスク", "次の操作を確認")}
    ${txt(1170, 174, "work_0042", "h3")}
    ${txt(1170, 198, "調査ソース一覧を作成", "small")}
    ${pill(1170, 226, "要確認", "amber")}
    ${pill(1248, 226, "検証待ち", "amber")}
    ${txt(1170, 288, "担当", "h3")}
    ${txt(1170, 314, "research-agent", "body")}
    ${txt(1170, 366, "次の操作", "h3")}
    ${txt(1170, 392, "Evidenceを検証してから承認する", "small")}
    ${button(1170, 430, "検証を実行", true, 112)}
    ${button(1170, 478, "詳細を見る", false, 100)}
    ${button(1170, 526, "分析を開く", false, 104)}
    ${line(1170, 596, 1374, 596)}
    ${txt(1170, 636, "分析・デバッグ", "h3")}
    ${txt(1170, 662, "Agent / Dispatch / Run は", "small")}
    ${txt(1170, 682, "詳細フィルタで絞り込む", "small")}
    ${rect(232, 302, 894, 482, C.surface, C.line, 8)}
    ${sectionTitle(252, 338, "作業キュー", "依頼、担当Agent、次の操作をまとめて見る")}
    ${button(990, 326, "詳細フィルタ", false, 116)}
    ${txt(252, 374, "すぐ確認", "h3")}
    ${pill(252, 390, "要確認 6", "amber")}
    ${pill(340, 390, "失敗 3", "red")}
    ${pill(410, 390, "承認待ち 4", "green")}
    ${pill(510, 390, "実行中 2", "blue")}
    ${rect(252, 424, 854, 40, C.surface2, C.line, 7)}
    ${txt(272, 449, "依頼文・Agent・証跡で検索", "small")}
    ${txt(266, 492, "作業", "tiny")}
    ${txt(420, 492, "状態", "tiny")}
    ${txt(572, 492, "担当", "tiny")}
    ${txt(724, 492, "次の操作", "tiny")}
    ${txt(1048, 492, "警告", "tiny")}
    ${rows.map((row, i) => tableRow(252, 504 + i * 62, 854, row[0], row[1])).join("")}
  `;
  return appChrome("Work Items", body);
}

function workItemDetail() {
  const timeline = `
    ${pill(512, 120, "current target: research-agent", "blue")}
    ${pill(730, 120, "人への質問あり", "amber")}
    ${pill(900, 120, "7 events", "gray")}
    ${line(548, 168, 548, 728, C.line)}

    ${circle(548, 186, 8, C.blueSoft, C.blue)}
    ${rect(574, 166, 420, 46, C.surface, C.line, 7)}
    ${pill(594, 187, "Request", "blue")}
    ${txt(694, 202, "調査ソース一覧を作成", "h3")}
    ${txt(884, 202, "12:03", "small")}

    ${circle(548, 250, 8, C.greenSoft, C.green)}
    ${rect(574, 230, 420, 46, C.surface, C.line, 7)}
    ${pill(594, 241, "Dispatch", "green")}
    ${txt(706, 256, "target research-agent", "h3")}
    ${pill(884, 241, "0 warnings", "green")}

    ${circle(548, 314, 8, C.greenSoft, C.green)}
    ${rect(574, 294, 420, 46, C.surface, C.line, 7)}
    ${pill(594, 305, "Run", "green")}
    ${txt(668, 320, "exit 0 / 44s / research-agent", "h3")}
    ${txt(914, 320, "12:05", "small")}

    ${circle(548, 378, 8, C.blueSoft, C.blue)}
    ${rect(574, 358, 420, 46, C.surface, C.line, 7)}
    ${pill(594, 369, "Artifact", "blue")}
    ${txt(696, 384, "art_0010 / ev_0011", "h3")}
    ${pill(856, 369, "2 records", "blue")}

    ${circle(548, 442, 8, C.amberSoft, C.amber)}
    ${rect(574, 422, 420, 46, C.surface, C.line, 7)}
    ${pill(594, 433, "Review", "amber")}
    ${txt(684, 448, "追加条件の確認が必要", "h3")}
    ${pill(866, 433, "question", "amber")}

    ${circle(548, 542, 10, C.amberSoft, C.amber)}
    ${rect(574, 486, 420, 142, "#ffffff", C.blue, 8)}
    ${txt(594, 510, "⌄ 人への質問", "h3")}
    ${pill(704, 493, "selected", "blue")}
    ${pill(786, 493, "回答待ち", "amber")}
    ${txt(594, 550, "release note のURLを追加してよいですか？", "h3")}
    ${txt(594, 572, "Agentが判断できない前提を人に確認中。", "small")}
    ${button(594, 586, "回答する", true, 96)}
    ${button(702, 586, "Artifact", false, 86)}
    ${button(800, 586, "Handoff", false, 96)}

    ${circle(548, 672, 8, C.graySoft, C.gray)}
    ${rect(574, 652, 420, 46, C.surface, C.line, 7)}
    ${pill(594, 663, "Verification", "gray")}
    ${txt(714, 678, "回答後に検証", "h3")}
    ${txt(914, 678, "next", "small")}

    ${circle(548, 734, 8, C.graySoft, C.gray)}
    ${txt(574, 740, "必要なら Handoff / 再実行 / 追加指示をこの下に追加", "tiny")}
  `;
  const body = `
    ${txt(232, 48, "work_0042 / 調査ソース一覧を作成", "title")}
    ${pill(694, 27, "要確認", "amber")}
    ${button(1196, 26, "検証を実行", true, 112)}
    ${disabledButton(1320, 26, "承認は検証後", 118)}
    ${rect(232, 92, 248, 692, C.surface, C.line, 8)}
    ${sectionTitle(252, 128, "概要", "文脈を見失わない")}
    ${txt(252, 178, "対象", "h3")}
    ${txt(252, 202, "docs/research/source-list.md", "small")}
    ${txt(252, 248, "Agents", "h3")}
    ${txt(252, 272, "dispatch: dispatch-app", "small")}
    ${txt(252, 296, "target: research-agent", "small")}
    ${txt(252, 340, "最新状態", "h3")}
    ${pill(252, 356, "依頼先決定", "green")}
    ${pill(252, 388, "run成功", "green")}
    ${pill(252, 420, "回答待ち", "amber")}
    ${txt(252, 476, "次に必要", "h3")}
    ${txt(252, 502, "Agentからの質問に回答する", "small")}
    ${txt(512, 104, "実行タイムライン", "h2")}
    ${timeline}
    ${rect(1024, 92, 372, 692, C.surface, C.line, 8)}
    ${sectionTitle(1044, 128, "Inspector: 人への質問", "selected timeline item")}
    ${pill(1044, 168, "selected", "blue")} ${pill(1128, 168, "回答待ち", "amber")}
    ${txt(1044, 220, "次の操作", "h3")}
    ${txt(1044, 246, "質問に回答し、後続の検証へ進める。", "small")}
    ${txt(1044, 278, "前後", "h3")}
    ${button(1044, 296, "Review", false, 82)}
    ${button(1138, 296, "Verification", false, 112)}
    ${txt(1044, 350, "質問", "h3")}
    ${rect(1044, 368, 318, 78, C.surface2, C.line, 6)}
    ${txt(1060, 396, "release note のURLを", "small")}
    ${txt(1060, 416, "成果物に追加してよいですか？", "small")}
    ${txt(1044, 496, "回答後の処理", "h3")}
    ${pill(1044, 514, "instruction update", "amber")}
    ${pill(1182, 514, "verification next", "gray")}
    ${button(1044, 566, "回答する", true, 96)}
    ${button(1156, 566, "Artifactを見る", false, 120)}
    ${button(1044, 614, "Handoff作成", false, 116)}
  `;
  return appChrome("Work Items", body);
}

function agentProfiles() {
  const body = `
    ${txt(232, 48, "Agent Profiles", "title")}
    ${button(1232, 26, "Add Agent", true, 106)}
    ${button(1350, 26, "Defaults", false, 92)}
    ${rect(232, 92, 782, 622, C.surface, C.line, 8)}
    ${sectionTitle(252, 128, "Profiles", "Compare routing hints, runtime, adapter, and probe state")}
    ${tableRow(252, 166, 742, [["research-agent", "Research Agent"], ["researcher", "research, synthesis"], ["process.codex-cli", "working_dir ."], ["healthy", "probe fresh"]], [["default", "blue"]])}
    ${tableRow(252, 236, 742, [["writing-agent", "Writing Agent"], ["writer", "writing, editing"], ["process.codex-cli", "working_dir ."], ["healthy", "probe fresh"]], [])}
    ${tableRow(252, 306, 742, [["dispatch-app", "Dispatch App"], ["dispatcher", "planning, dispatch"], ["stdio.codex-app-server", "working_dir ."], ["unknown", "probe stale"]], [["stale", "amber"]])}
    ${rect(1042, 92, 354, 622, C.surface, C.line, 8)}
    ${sectionTitle(1062, 128, "Defaults", "Nagare agents")}
    ${txt(1062, 184, "work_agent", "tiny")}
    ${txt(1062, 210, "writing-agent", "h3")}
    ${txt(1062, 256, "review_agent", "tiny")}
    ${txt(1062, 282, "writing-agent", "h3")}
    ${txt(1062, 328, "dispatch_agent", "tiny")}
    ${txt(1062, 354, "dispatch-app", "h3")}
    ${button(1062, 420, "Change defaults", true, 148)}
  `;
  return appChrome("Agents", body);
}

function agentProfileDetail() {
  const body = `
    ${txt(232, 48, "research-agent", "title")}
    ${pill(430, 27, "researcher", "blue")}
    ${button(1250, 26, "Update", true, 86)}
    ${rect(232, 92, 340, 620, C.surface, C.line, 8)}
    ${sectionTitle(252, 128, "Profile Declaration", "User-maintained routing hints")}
    ${txt(252, 178, "display_name", "tiny")}
    ${txt(252, 204, "Research Agent", "h3")}
    ${txt(252, 250, "description", "tiny")}
    ${txt(252, 276, "Collects sources and synthesizes findings.", "small")}
    ${txt(252, 326, "specialties", "tiny")}
    ${pill(252, 342, "research", "blue")}
    ${pill(344, 342, "synthesis", "blue")}
    ${rect(602, 92, 366, 300, C.surface, C.line, 8)}
    ${sectionTitle(622, 128, "Capability Probe", "Observed, not declared")}
    ${pill(622, 166, "fresh", "green")}
    ${txt(622, 210, "repo_read / file_edit / shell_command", "small")}
    ${txt(622, 254, "instruction_sources: AGENTS.md", "small")}
    ${rect(602, 414, 366, 298, C.surface, C.line, 8)}
    ${sectionTitle(622, 450, "Dispatch Usage", "Recent target selections")}
    ${txt(622, 500, "dispatch_0007 accepted -> work_0042", "small")}
    ${txt(622, 532, "dispatch_0011 draft -> work_0048", "small")}
    ${rect(998, 92, 398, 620, C.surface, C.line, 8)}
    ${sectionTitle(1018, 128, "Recent Runs", "Debug activity")}
    ${tableRow(1018, 166, 358, [["run_0009", "work"], ["succeeded", "exit 0"], ["12m", "ago"]], [])}
    ${tableRow(1018, 236, 358, [["run_0004", "dispatch_preview"], ["succeeded", "exit 0"], ["1h", "ago"]], [])}
  `;
  return appChrome("Agents", body);
}

function settings() {
  const body = `
    ${txt(232, 48, "Settings", "title")}
    ${rect(232, 92, 360, 280, C.surface, C.line, 8)}
    ${sectionTitle(252, 128, "Locale", "Project records use this locale")}
    ${txt(252, 184, "language", "tiny")}
    ${txt(252, 210, "ja-JP", "h3")}
    ${txt(252, 256, "timezone", "tiny")}
    ${txt(252, 282, "Asia/Tokyo", "h3")}
    ${rect(622, 92, 380, 280, C.surface, C.line, 8)}
    ${sectionTitle(642, 128, "Default Agents", "Used by Nagare")}
    ${txt(642, 184, "work_agent: writing-agent", "small")}
    ${txt(642, 218, "review_agent: writing-agent", "small")}
    ${txt(642, 252, "dispatch_agent: dispatch-app", "small")}
    ${button(642, 304, "Change defaults", true, 148)}
    ${rect(1032, 92, 364, 280, C.surface, C.line, 8)}
    ${sectionTitle(1052, 128, "Dispatch Context Budget", "Fixed for MVP")}
    ${txt(1052, 184, "candidate limit", "tiny")}
    ${txt(1052, 214, "5 fixed", "h2")}
    ${txt(1052, 258, "Not configurable until UI limits are clear.", "small")}
    ${rect(232, 404, 1164, 260, C.surface, C.line, 8)}
    ${sectionTitle(252, 440, "Supported Adapters", "Initial scope")}
    ${pill(252, 478, "process.codex-cli", "blue")}
    ${pill(410, 478, "stdio.codex-app-server", "blue")}
    ${txt(252, 548, "Excluded: Codex MCP Server, Claude Code, HTTP adapter, SDK adapter", "small")}
  `;
  return appChrome("Settings", body);
}

function inspectorShell(title, subtitle, content, width = 960, height = 960) {
  return svgDoc(width, height, `
    ${rect(36, 36, width - 72, height - 72, C.surface, C.line, 10)}
    ${txt(64, 82, title, "title")}
    ${txt(64, 106, subtitle, "small")}
    ${content}
  `);
}

function dispatchReview() {
  return inspectorShell("依頼先の確認", "実行前に、選ばれたAgentと理由を確認する", `
    ${pill(64, 142, "下書き", "amber")} ${pill(132, 142, "候補 research-agent", "blue")}
    ${txt(64, 204, "依頼の読み取り", "h2")}
    ${txt(64, 234, "先に調査が必要です。research-agentへの依頼が適しています。", "body")}
    ${txt(64, 294, "候補Agent", "h2")}
    ${tableRow(64, 320, 832, [["research-agent", "research, synthesis"], ["評価", "最適"], ["adapter", "process.codex-cli"]], [["target", "green"]])}
    ${tableRow(64, 390, 832, [["writing-agent", "writing, editing"], ["評価", "二次候補"], ["adapter", "process.codex-cli"]], [])}
    ${txt(64, 496, "不足・リスク・警告", "h2")}
    ${pill(64, 520, "source quality", "amber")}
    ${pill(190, 520, "source list不足", "amber")}
    ${pill(340, 520, "警告なし", "green")}
    ${txt(64, 606, "Agent出力", "h2")}
    ${rect(64, 630, 832, 154, C.surface2, C.line, 6)}
    ${txt(84, 662, "{ target_agent_profile_id: \"research-agent\",", "mono")}
    ${txt(84, 692, "  summary: \"Research is required...\" }", "mono")}
    ${button(64, 836, "この内容で依頼", true, 136)}
    ${button(214, 836, "選び直す", false, 100)}
    ${button(328, 836, "Agentを指定", false, 118)}
  `);
}

function runLog() {
  return inspectorShell("Run Log Inspector", "AgentRun details, transcript, and recovery actions", `
    ${pill(64, 142, "succeeded", "green")} ${pill(150, 142, "purpose work", "blue")} ${pill(260, 142, "exit 0", "green")}
    ${txt(64, 204, "Run metadata", "h2")}
    ${txt(64, 236, "agent: research-agent / adapter: process.codex-cli / duration: 44s", "body")}
    ${txt(64, 304, "stdout / stderr", "h2")}
    ${rect(64, 328, 832, 230, "#111827", "#111827", 6)}
    ${txt(84, 364, "$ codex exec --cd .", "mono", "#e5e7eb")}
    ${txt(84, 398, "Collected source list and wrote evidence artifact.", "mono", "#e5e7eb")}
    ${txt(84, 432, "stderr: <empty>", "mono", "#9ca3af")}
    ${txt(64, 620, "Linked records", "h2")}
    ${txt(64, 652, "ResolvedRunPacket: runpkt_0009 / Evidence: ev_0011 / Artifact: art_0010", "small")}
    ${button(64, 810, "Retry", false, 86)}
    ${button(164, 810, "Create handoff", true, 142)}
    ${button(322, 810, "Open artifact", false, 124)}
  `);
}

function artifactViewer() {
  return inspectorShell("Artifact Viewer", "Formatted JSON and log artifacts", `
    ${pill(64, 142, "ResolvedRunPacket", "blue")} ${pill(210, 142, "JSON", "gray")}
    ${rect(64, 190, 380, 610, C.surface2, C.line, 6)}
    ${txt(84, 226, "Artifact index", "h2")}
    ${txt(84, 276, "run_0009.log", "body")}
    ${txt(84, 314, "runpkt_0009.json", "body")}
    ${txt(84, 352, "skillctx_0008.json", "body")}
    ${rect(472, 190, 424, 610, "#0f172a", "#0f172a", 6)}
    ${txt(494, 230, "{", "mono", "#dbeafe")}
    ${txt(494, 264, "  \"agent_profile_id\": \"research-agent\",", "mono", "#dbeafe")}
    ${txt(494, 298, "  \"dispatch_plan_id\": \"dispatch_0007\",", "mono", "#dbeafe")}
    ${txt(494, 332, "  \"purpose\": \"work\"", "mono", "#dbeafe")}
    ${txt(494, 366, "}", "mono", "#dbeafe")}
  `);
}

function evidenceDetail() {
  return inspectorShell("Evidence Detail Inspector", "Check basis before approval", `
    ${pill(64, 142, "evidence", "blue")} ${pill(152, 142, "produced by research-agent", "gray")}
    ${txt(64, 210, "Claim", "h2")}
    ${txt(64, 242, "Agent Profile `research-agent` execution succeeded.", "body")}
    ${txt(64, 314, "Basis", "h2")}
    ${txt(64, 346, "Command exited with code 0 and produced run artifact art_0010.", "body")}
    ${txt(64, 430, "Approval readiness", "h2")}
    ${pill(64, 454, "verification missing", "amber")}
    ${pill(210, 454, "artifact linked", "green")}
    ${button(64, 760, "Open run", false, 102)}
    ${button(180, 760, "Open artifact", false, 124)}
    ${button(320, 760, "Run verification", true, 142)}
  `);
}

function verification() {
  return inspectorShell("Verification Inspector", "Gate before human approval", `
    ${pill(64, 142, "failed", "red")} ${pill(136, 142, "command", "gray")}
    ${txt(64, 210, "Command", "h2")}
    ${rect(64, 234, 832, 48, C.surface2, C.line, 6)}
    ${txt(84, 264, "cargo test --workspace", "mono")}
    ${txt(64, 336, "Log excerpt", "h2")}
    ${rect(64, 360, 832, 238, "#111827", "#111827", 6)}
    ${txt(84, 398, "test dispatch_contract ... FAILED", "mono", "#fecaca")}
    ${txt(84, 432, "missing required target_agent_profile_id", "mono", "#fecaca")}
    ${button(64, 760, "Re-run verify", true, 132)}
    ${button(212, 760, "Create handoff", false, 142)}
    ${button(370, 760, "Open artifact", false, 124)}
  `);
}

function reviewInspector() {
  return inspectorShell("Review Inspector", "Decision point after artifacts and verification", `
    ${pill(64, 142, "selected review", "gray")} ${pill(198, 142, "question requested", "amber")} ${pill(354, 142, "review-agent", "blue")}
    ${txt(64, 204, "Verdict", "h2")}
    ${txt(64, 236, "Approve after source list includes release note links.", "body")}
    ${txt(64, 306, "Findings", "h2")}
    ${tableRow(64, 332, 832, [["finding_001", "missing source URL"], ["severity", "medium"], ["linked", "art_0010"]], [["open", "amber"]])}
    ${tableRow(64, 402, 832, [["finding_002", "README update ok"], ["severity", "low"], ["linked", "ev_0011"]], [["ok", "green"]])}
    ${txt(64, 506, "Referenced records", "h2")}
    ${pill(64, 530, "Artifact art_0010", "blue")}
    ${pill(210, 530, "Evidence ev_0011", "green")}
    ${pill(360, 530, "Verification failed", "red")}
    ${txt(64, 624, "Next action", "h2")}
    ${txt(64, 656, "Request changes or handoff to research-agent for source repair.", "body")}
    ${button(64, 800, "Request changes", true, 154)}
    ${button(236, 800, "Create handoff", false, 142)}
    ${button(394, 800, "Open artifact", false, 124)}
  `);
}

function handoff() {
  return inspectorShell("Handoff Inspector", "Move a failed item to a better agent", `
    ${txt(64, 154, "Create handoff", "h2")}
    ${txt(64, 202, "from agent", "tiny")} ${txt(340, 202, "to agent", "tiny")}
    ${rect(64, 218, 250, 38, C.surface2, C.line, 6)} ${rect(340, 218, 250, 38, C.surface2, C.line, 6)}
    ${txt(82, 244, "writing-agent", "body")} ${txt(358, 244, "research-agent", "body")}
    ${txt(64, 310, "reason", "tiny")}
    ${rect(64, 326, 832, 88, C.surface2, C.line, 6)}
    ${txt(84, 358, "Initial run failed because source context was missing.", "small")}
    ${txt(64, 482, "Existing handoffs", "h2")}
    ${tableRow(64, 510, 832, [["handoff_0012", "writing-agent -> research-agent"], ["reason", "missing sources"], ["state", "dispatch ready"]], [])}
    ${button(64, 800, "Create handoff", true, 144)}
    ${button(224, 800, "Dispatch handoff", false, 156)}
  `);
}

function agentEdit() {
  return inspectorShell("Agent Edit Modal", "Edit declaration fields used for routing", `
    ${rect(150, 150, 660, 590, C.surface, C.line, 12)}
    ${txt(190, 196, "Edit Agent Profile", "title")}
    ${txt(190, 250, "display_name", "tiny")} ${rect(190, 266, 580, 38, C.surface2, C.line, 6)} ${txt(208, 292, "Research Agent", "body")}
    ${txt(190, 336, "role", "tiny")} ${rect(190, 352, 260, 38, C.surface2, C.line, 6)} ${txt(208, 378, "researcher", "body")}
    ${txt(480, 336, "working_dir", "tiny")} ${rect(480, 352, 290, 38, C.surface2, C.line, 6)} ${txt(498, 378, ".", "body")}
    ${txt(190, 422, "description", "tiny")} ${rect(190, 438, 580, 80, C.surface2, C.line, 6)} ${txt(208, 470, "Collects sources and synthesizes findings.", "small")}
    ${txt(190, 550, "specialties", "tiny")} ${pill(190, 566, "research", "blue")} ${pill(290, 566, "synthesis", "blue")}
    ${button(548, 668, "Cancel", false, 92)}
    ${button(654, 668, "Save", true, 88)}
  `);
}

function defaultsModal() {
  return inspectorShell("Defaults Modal", "Choose work, review, and dispatch agents", `
    ${rect(150, 140, 660, 600, C.surface, C.line, 12)}
    ${txt(190, 186, "Default Agents", "title")}
    ${txt(190, 248, "work_agent", "tiny")} ${rect(190, 264, 580, 42, C.surface2, C.line, 6)} ${txt(208, 292, "writing-agent", "body")}
    ${txt(190, 348, "review_agent", "tiny")} ${rect(190, 364, 580, 42, C.surface2, C.line, 6)} ${txt(208, 392, "writing-agent", "body")}
    ${txt(190, 448, "dispatch_agent", "tiny")} ${rect(190, 464, 580, 42, C.surface2, C.line, 6)} ${txt(208, 492, "dispatch-app", "body")}
    ${pill(190, 540, "dispatch-app probe stale", "amber")}
    ${txt(190, 588, "Warning: probe before using this agent for dispatch.", "small")}
    ${button(548, 668, "Cancel", false, 92)}
    ${button(654, 668, "Save", true, 88)}
  `);
}

const mocks = [
  ["01-work-item-board", workItemBoard()],
  ["02-work-item-detail", workItemDetail()],
  ["03-agent-profiles", agentProfiles()],
  ["04-agent-profile-detail", agentProfileDetail()],
  ["05-settings", settings()],
  ["06-dispatch-review-inspector", dispatchReview()],
  ["07-run-log-inspector", runLog()],
  ["08-artifact-viewer", artifactViewer()],
  ["09-evidence-detail-inspector", evidenceDetail()],
  ["10-verification-inspector", verification()],
  ["11-review-inspector", reviewInspector()],
  ["12-handoff-inspector", handoff()],
  ["13-agent-edit-modal", agentEdit()],
  ["14-defaults-modal", defaultsModal()],
];

for (const [name, svg] of mocks) {
  fs.writeFileSync(path.join(outDir, `${name}.svg`), svg);
}

console.log(`Generated ${mocks.length} SVG mockups in ${outDir}`);
