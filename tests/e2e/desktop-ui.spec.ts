import { expect, test } from "@playwright/test";
import path from "node:path";
import { pathToFileURL } from "node:url";

const repoRoot = path.resolve(__dirname, "../..");
const desktopIndexUrl = pathToFileURL(
  path.join(repoRoot, "apps", "nagare-desktop", "src", "index.html"),
).toString();

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}

async function openWorkDetail(page, id = "work_1") {
  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await page.locator(`[data-work-id="${id}"] [data-work-open]`).click();
}

function baseState(initialized: boolean) {
  return {
    app: { name: "Nagare", version: "0.1.0", ui_source: "mock" },
    root: "C:/nagare-desktop-e2e",
    initialized,
    project: initialized ? projectView() : null,
    work_items: [],
    agents: initialized ? [
      { id: "worker", name: "Writer", role: "worker", description: "文書作成を担当", prompt: "文書作成を担当する", tool_kind: "codex_cli", runtime: "codex-local", model: "gpt-5-codex", specialties: ["docs"], domain_ids: [], artifact_type_ids: [], skill_set_ids: [], mcp_connection_ids: [] },
      { id: "reviewer", name: "Reviewer", role: "reviewer", description: "レビューを担当", prompt: "レビューを担当する", tool_kind: "codex_cli", runtime: "codex-local", model: "gpt-5-codex", specialties: ["review"], domain_ids: [], artifact_type_ids: [], skill_set_ids: [], mcp_connection_ids: [] },
    ] : [],
    domains: initialized ? [
      { id: "product-docs", name: "プロダクト文書", description: "READMEやリリースノートを扱う", shared_knowledge: ["用語集"], common_rubric: ["読みやすい"], dispatch_hints: ["docs"], artifact_type_count: 1 },
    ] : [],
    artifact_types: initialized ? [
      { id: "readme", domain_id: "product-docs", name: "README", description: "セットアップ文書", knowledge: ["READMEテンプレート"], rubric: ["## 手順の再現性 (50)"], dispatch_hints: ["readme"], rubric_count: 1, rubric_score_total: 50 },
    ] : [],
    skill_sets: [],
    skill_packages: initialized ? [
      { id: "markdown-tools", source_kind: "clawhub", source: "markdown-tools", provided_skill_sets: ["markdown-tools"] },
    ] : [],
    mcp_connections: initialized ? [
      { id: "github", name: "GitHub MCP", tool_kind: "codex_cli", runtime_label: "Codex CLI", agent_assignable: true, command: "npx", args: ["-y", "@modelcontextprotocol/server-github"], test_status: "success" },
      { id: "openclaw-search", name: "OpenClaw Search MCP", tool_kind: "openclaw", runtime_label: "OpenClaw", agent_assignable: false, command: "npx", args: ["-y", "openclaw-search"], test_status: "passed" },
    ] : [],
    mcp_capabilities: [],
    runtimes: [
      { id: "codex", label: "Codex CLI", command: "codex", detail: "codex mock", available: true, model_note: "gpt-5-codex", model_mode: "OpenAIモデル", model_choices: ["実行環境既定", "gpt-5-codex", "手入力"] },
      { id: "openclaw", label: "OpenClaw", command: "openclaw", detail: "not found", available: false, model_note: "OpenAI / Ollama / LMStudio", model_mode: "Provider / Base URL / Model", model_choices: ["OpenAI", "Ollama", "LMStudio", "手入力"] },
    ],
    insights: initialized ? {
      review_count: 6,
      average_score_label: "82 / 100",
      concern_count: 3,
      proposal_count: 3,
      agent_scores: [
        { agent_id: "worker", agent_name: "Writer", role: "worker", review_count: 4, average_score: 88, average_score_label: "88 / 100", status_label: "良好", top_issue: "読みやすさ" },
        { agent_id: "reviewer", agent_name: "Reviewer", role: "reviewer", review_count: 2, average_score: 70, average_score_label: "70 / 100", status_label: "要改善", top_issue: "形式の準拠" },
      ],
      issue_matrix: [
        { agent_name: "Reviewer", item: "形式の準拠", rate: 60, rate_label: "60%", occurrences: 2, suggestion_kind: "プロンプト" },
        { agent_name: "Writer", item: "読みやすさ", rate: 78, rate_label: "78%", occurrences: 1, suggestion_kind: "ルーブリック" },
      ],
      proposals: [
        {
          id: "proposal-prompt-reviewer",
          kind: "プロンプト",
          title: "Reviewer のプロンプト改善",
          target_label: "Reviewer / 形式の準拠",
          summary: "形式基準を先に確認する手順を追加します",
          evidence: "形式の準拠が直近2件で60%",
          current_text: "レビューする",
          suggested_text: "形式基準を先に確認してからレビューする",
          diff_lines: ["@@ Reviewer prompt", "- レビューする", "+ 形式基準を先に確認してからレビューする"],
          next_step: "エージェント設定で編集",
          action_label: "エージェントを開く",
        },
        {
          id: "proposal-rubric-readability",
          kind: "ルーブリック",
          title: "README ルーブリック改善",
          target_label: "Writer / 手順の再現性",
          summary: "読みやすさの判定基準を明確にします",
          evidence: "読みやすさの懸念が1件",
          current_text: "読みやすい",
          suggested_text: "手順が5分以内に追える",
          diff_lines: ["@@ README rubric", "- 読みやすい", "+ 手順が5分以内に追える"],
          next_step: "ナレッジで編集",
          action_label: "ナレッジを開く",
        },
        {
          id: "operation-approval-policy",
          kind: "運用",
          title: "確認ポリシー緩和の提案",
          target_label: "プロジェクト設定 / 確認ポリシー",
          summary: "直近レビューが安定しているため、懸念がある時だけ確認する運用へ切り替える候補です",
          evidence: "レビュー10件、平均92点、懸念0件",
          current_text: "確認ポリシー: 最後に確認する",
          suggested_text: "確認ポリシー: 懸念がある時だけ確認する",
          diff_lines: ["@@ Project policy", "- 確認ポリシー: 最後に確認する", "+ 確認ポリシー: 懸念がある時だけ確認する"],
          next_step: "プロジェクト設定で確認ポリシーを確認し、人が必要と判断した場合だけ保存します。",
          action_label: "設定で確認",
        },
      ],
      applied_improvements: [
        {
          id: "applied-writer-prompt",
          kind: "プロンプト",
          title: "Writer のプロンプト改善",
          target_label: "Writer / 質問ルール",
          summary: "不明点を推測せず質問として返すルールを追加",
          applied_at: "1ヶ月前",
          effect_label: "差し戻し率 22% → 12%",
        },
      ],
      recent_reviews: [
        { work_id: "work_1", title: "README更新", agent_name: "Reviewer", verdict: "pass", score_label: "92 / 100", concerns: ["補足が長い"] },
      ],
    } : {},
  };
}

function projectView() {
  return {
    name: "nagare-desktop-e2e",
    icon: "流",
    root: "C:/nagare-desktop-e2e",
    workflow_mode: "confirm_first",
    approval_policy: "manual_final_approval",
    default_domain_id: "",
    default_artifact_type_id: "",
    organizer_agent_id: "",
    organizer_label: "標準",
    work_agent: "Writer",
    review_agent: "Reviewer",
    agent_count: 2,
    domain_count: 1,
    artifact_type_count: 1,
    work_count: 0,
    status_counts: [],
  };
}

function makeDetail(description: string) {
  return {
    root: "C:/nagare-desktop-e2e",
    item: {
      id: "work_1",
      title: description,
      description,
      project: "nagare-desktop-e2e",
      status_label: "確認待ち",
      status_kind: "review",
      next_action: "結果を確認",
      result_summary: "README のセットアップ手順を整理し、確認方法を追記しました。",
      updated_at: "now",
    },
    domain_id: "product-docs",
    artifact_type_id: "readme",
    next_action_kind: "approve",
    approval_ready: true,
    question: null,
    question_source: "",
    recovery: null,
    request: description,
    answer: "README のセットアップ手順を整理し、確認方法を追記しました。",
    artifacts: [{ title: "README.md", uri: "C:/nagare-desktop-e2e/README.md" }],
    review: {
      verdict: "pass",
      summary: "手順は再現可能です。",
      score_label: "92 / 100",
      concerns: ["補足が長い箇所があります。"],
      items: [
        { item: "手順の再現性", verdict: "pass", evidence: "順番に読めます。", score_label: "46 / 50", concern_note: "" },
        { item: "読みやすさ", verdict: "minor_concern", evidence: "補足が長いです。", score_label: "18 / 25", concern_note: "" },
      ],
    },
    steps: [
      { kind: "organizer", title: "受付・整理", state: "完了", actor: "Organizer", summary: "依頼を整理しました。", rationale: "README更新です。", input: description, output: "README", knowledge_refs: ["プロダクト文書"], diagnostics: "run packet: domain=product-docs artifact=readme agent=Writer", review_items: [] },
      { kind: "review", title: "レビュー", state: "完了", actor: "Reviewer", summary: "92点です。", rationale: "ルーブリックで確認しました。", input: "README.md", output: "92 / 100", knowledge_refs: ["READMEルーブリック"], diagnostics: "", review_items: [] },
      { kind: "organizer", title: "オーガナイザーまとめ", state: "完了", actor: "Organizer", summary: "レビュー結果を踏まえ、依頼者向けの最終回答を整えました。", rationale: "作業結果とレビュー結果を最終回答に統合しました。", input: "README.md / 92 / 100", output: "README のセットアップ手順を整理し、確認方法を追記しました。", knowledge_refs: ["プロダクト文書"], diagnostics: "", review_items: [] },
    ],
  };
}

async function installTauriMock(page, initialState, options = {}) {
  await page.addInitScript((payload) => {
    const initial = payload.initial;
    const options = payload.options || {};
    const cloneValue = (value) => JSON.parse(JSON.stringify(value));
    let appState = cloneValue(initial);
    let detail = null;
    const calls = [];
    let appStateFailuresRemaining = Number(options.failAppStateTimes || 0);
    let commandFailures = cloneValue(options.failCommands || {});
    let commandDelays = cloneValue(options.commandDelays || {});
    const improvementEffectLabel = (request) => {
      const insights = appState.insights || {};
      const targetParts = String(request?.improvement_target_label || "").split(/[\/／]/).map((part) => part.trim());
      const targetAgent = targetParts[0] || "";
      const targetItem = targetParts[1] || "";
      if (String(request?.improvement_kind || "").includes("運用") || String(request?.improvement_target_label || "").includes("確認ポリシー")) {
        return `測定中: 平均 ${String(insights.average_score_label || "-").replace(/\s*\/\s*100$/, "")}点 / 懸念${insights.concern_count || 0}件`;
      }
      const issue = (insights.issue_matrix || []).find((item) => item.agent_name === targetAgent && item.item === targetItem);
      if (issue) return `測定中: ${issue.item} ${issue.rate_label}（${issue.occurrences}件）`;
      return Number(insights.review_count || 0) > 0 ? "効果: 最近の同種懸念なし" : "効果測定中";
    };
    const recordImprovement = (request) => {
      if (!request?.improvement_proposal_id) return;
      const item = {
        id: request.improvement_proposal_id,
        proposal_id: request.improvement_proposal_id,
        kind: request.improvement_kind || "",
        title: request.improvement_title || "改善を適用",
        target_label: request.improvement_target_label || "",
        summary: request.improvement_summary || "",
        applied_at: "今",
        effect_label: improvementEffectLabel(request),
      };
      appState.insights = appState.insights || {};
      appState.insights.applied_improvements = [
        item,
        ...(appState.insights.applied_improvements || []).filter((existing) => existing.title !== item.title),
      ];
      appState.insights.proposals = (appState.insights.proposals || []).filter((proposal) => proposal.id !== request.improvement_proposal_id);
      appState.insights.proposal_count = appState.insights.proposals.length;
    };
    const dismissImprovement = (request) => {
      if (!request?.proposal_id) return;
      appState.insights = appState.insights || {};
      appState.insights.proposals = (appState.insights.proposals || []).filter((proposal) => proposal.id !== request.proposal_id);
      appState.insights.proposal_count = appState.insights.proposals.length;
    };
    const parseMcpEnvLines = (value) => {
      return String(value || "").split(/\r?\n/).map((line) => line.trim()).filter(Boolean).map((line) => {
        if (!line.includes("=")) {
          throw new Error(`環境変数 \`${line}\` は KEY=VALUE 形式で入力してください。`);
        }
        const [key, ...rest] = line.split("=");
        if (!key.trim()) {
          throw new Error("環境変数のキーが空です。");
        }
        return `${key.trim()}=${rest.join("=").trim()}`;
      });
    };
    window.__nagareCopiedText = "";
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: async (text) => {
          window.__nagareCopiedText = String(text || "");
        },
      },
    });
    const completeMockAdvance = () => {
      if (detail?.next_action_kind !== "dispatch") return;
      const rejectedFlow = detail.steps.some((step) => step.kind === "human") || /差し戻し/.test(detail.item.result_summary || "");
      detail.answer = rejectedFlow
        ? "差し戻し内容を反映して README の補足を短くしました。"
        : "README の内容を整理し、確認しやすい構成にしました。";
      detail.item.status_label = "確認待ち";
      detail.item.status_kind = "review";
      detail.item.next_action = "結果を確認";
      detail.item.result_summary = rejectedFlow
        ? "README の補足を短くし、確認しやすい構成にしました。"
        : "README の内容を整理し、確認しやすい構成にしました。";
      detail.approval_ready = true;
      detail.next_action_kind = "approve";
      detail.review = {
        verdict: "pass",
        summary: rejectedFlow ? "差し戻し内容は反映されています。" : "内容は確認できます。",
        score_label: "94 / 100",
        concerns: [],
        items: [
          { item: "手順の再現性", verdict: "pass", evidence: "順番に読めます。", score_label: "48 / 50", concern_note: "" },
          { item: "読みやすさ", verdict: "pass", evidence: "補足が短くなりました。", score_label: "24 / 25", concern_note: "" },
        ],
      };
      detail.steps = [
        ...detail.steps,
        { kind: "worker", title: "再作成", state: "完了", actor: "Writer", summary: "差し戻しコメントに沿って補足を短くしました。", rationale: "読みやすさの懸念に対応", input: "再実行指示", output: "README.md", knowledge_refs: ["READMEテンプレート"], diagnostics: "", review_items: [] },
        { kind: "review", title: "再レビュー", state: "完了", actor: "Reviewer", summary: "94点です。", rationale: "差し戻し懸念が解消されています。", input: "README.md", output: "94 / 100", knowledge_refs: ["READMEルーブリック"], diagnostics: "", review_items: detail.review.items },
      ];
      appState.work_items = [detail.item];
    };
    window.__nagareDesktopCalls = calls;
    window.__nagareDesktopSetMockState = (nextState) => {
      appState = cloneValue(nextState);
    };
    window.__nagareDesktopSetCommandFailures = (nextFailures) => {
      commandFailures = cloneValue(nextFailures || {});
    };
    window.__nagareDesktopSetCommandDelays = (nextDelays) => {
      commandDelays = cloneValue(nextDelays || {});
    };
    window.__TAURI__ = {
      core: {
        invoke: async (command, payload = {}) => {
          calls.push({ command, payload: cloneValue(payload) });
          const commandDelay = Number(commandDelays?.[command] || 0);
          if (commandDelay > 0) {
            await new Promise((resolve) => setTimeout(resolve, commandDelay));
          }
          if (command === "choose_project_folder") {
            if (Object.prototype.hasOwnProperty.call(options, "chooseProjectFolderResult")) {
              return options.chooseProjectFolderResult;
            }
            return "C:/nagare-desktop-e2e";
          }
          if (command === "choose_agent_avatar_file") {
            if (Object.prototype.hasOwnProperty.call(options, "chooseAgentAvatarResult")) {
              return options.chooseAgentAvatarResult;
            }
            return "C:/nagare-desktop-e2e/avatars/ui-worker.svg";
          }
          if (command === "app_state") {
            if (appStateFailuresRemaining > 0) {
              appStateFailuresRemaining -= 1;
              throw new Error(options.failAppStateMessage || "failed to load app state");
            }
            return cloneValue(appState);
          }
          const commandFailure = commandFailures?.[command];
          if (commandFailure) {
            throw new Error(typeof commandFailure === "string" ? commandFailure : `${command} failed`);
          }
          if (command === "initialize_project_with_runtime") {
            if (options.failInitialize) throw new Error(options.failInitialize);
            appState.initialized = true;
            appState.root = payload.request.root;
            appState.project = {
              name: payload.request.display_name || "nagare-desktop-e2e",
              icon: payload.request.icon || "流",
              root: payload.request.root,
              workflow_mode: "confirm_first",
              approval_policy: "manual_final_approval",
              default_domain_id: "",
              default_artifact_type_id: "",
              organizer_agent_id: "",
              organizer_label: "標準",
              work_agent: "Writer",
              review_agent: "Reviewer",
              agent_count: 2,
              domain_count: 1,
              artifact_type_count: 1,
              work_count: 0,
              status_counts: [],
            };
            appState.agents = [
              { id: "worker", name: "Writer", role: "worker", description: "文書作成を担当", prompt: "文書作成を担当する", tool_kind: "codex_cli", runtime: "codex", model: "gpt-5-codex", specialties: ["docs"], domain_ids: [], artifact_type_ids: [], skill_set_ids: [], mcp_connection_ids: [] },
              { id: "reviewer", name: "Reviewer", role: "reviewer", description: "レビューを担当", prompt: "レビューを担当する", tool_kind: "codex_cli", runtime: "codex", model: "gpt-5-codex", specialties: ["review"], domain_ids: [], artifact_type_ids: [], skill_set_ids: [], mcp_connection_ids: [] },
            ];
            appState.domains = [
              { id: "product-docs", name: "プロダクト文書", description: "READMEやリリースノートを扱う", shared_knowledge: ["用語集"], common_rubric: ["読みやすい"], dispatch_hints: ["docs"], artifact_type_count: 1 },
            ];
            appState.artifact_types = [
              { id: "readme", domain_id: "product-docs", name: "README", description: "セットアップ文書", knowledge: ["READMEテンプレート"], rubric: ["## 手順の再現性 (50)"], dispatch_hints: ["readme"], rubric_count: 1, rubric_score_total: 50 },
            ];
            return cloneValue(appState);
          }
          if (command === "create_work") {
            const domain = appState.domains.find((item) => item.id === payload.request.domain_id)
              || appState.domains.find((item) => item.id === appState.project.default_domain_id)
              || appState.domains[0]
              || { id: "product-docs", name: "プロダクト文書" };
            const artifact = appState.artifact_types.find((item) => item.id === payload.request.artifact_type_id)
              || appState.artifact_types.find((item) => item.id === appState.project.default_artifact_type_id)
              || appState.artifact_types.find((item) => item.domain_id === domain.id)
              || appState.artifact_types[0]
              || { id: "readme", name: "README", domain_id: domain.id };
            const workerName = appState.project.work_agent || "Writer";
            const reviewerName = appState.project.review_agent || "Reviewer";
            const artifactName = artifact.name || artifact.id || "README";
            const domainName = domain.name || domain.id || "プロダクト文書";
            const conversational = /おはよう|こんにちは|こんばんは/.test(payload.request.description);
            const artifactFileName = artifactName === "README" ? "README.md" : `${artifactName}.md`;
            const resultSummary = conversational
              ? "おはようございます。今日進めたい作業があれば、そのまま依頼を書いてください。"
              : artifactName === "README"
                ? "README のセットアップ手順を整理し、確認方法を追記しました。"
                : `${artifactName} の内容を整理し、確認方法を追記しました。`;
            const answer = conversational
              ? "おはようございます。今日進めたい作業があれば、そのまま依頼を書いてください。"
              : artifactName === "README"
                ? "README のセットアップ手順を整理し、確認方法を追記しました。"
                : `${artifactName} の構成と確認方法を整理しました。`;
            detail = {
              root: appState.root,
              item: {
                id: "work_1",
                title: payload.request.description,
                description: payload.request.description,
                project: appState.project.name,
                status_label: "確認待ち",
                status_kind: "review",
                next_action: "結果を確認",
                result_summary: resultSummary,
                updated_at: "now",
                workflow_mode: payload.request.workflow_mode,
                approval_policy: payload.request.approval_policy,
              },
              domain_id: payload.request.domain_id || domain.id,
              artifact_type_id: payload.request.artifact_type_id || artifact.id,
              next_action_kind: "approve",
              approval_ready: true,
              question: null,
              question_source: "",
              recovery: null,
              request: payload.request.description,
              answer,
              artifacts: conversational ? [] : [{ title: artifactFileName, uri: `${appState.root}/${artifactFileName}` }],
              review: {
                verdict: "pass",
                summary: conversational ? "応答として自然です。" : "手順は再現可能です。",
                score_label: "92 / 100",
                concerns: conversational ? [] : ["補足が長い箇所があります。"],
                items: conversational
                  ? [{ item: "応答の自然さ", verdict: "pass", evidence: "入力に対して自然に応答しています。", score_label: "92 / 100", concern_note: "" }]
                  : [
                    { item: "手順の再現性", verdict: "pass", evidence: "順番に読めます。", score_label: "46 / 50", concern_note: "" },
                    { item: "読みやすさ", verdict: "minor_concern", evidence: "補足が長いです。", score_label: "18 / 25", concern_note: "" },
                  ],
              },
              steps: [
                { kind: "organizer", title: "受付・整理", state: "完了", actor: "Organizer", summary: conversational ? "入力内容に応答しました。" : "依頼を整理しました。", rationale: conversational ? "成果物ファイルではなく応答を返すワークです。" : `${artifactName} の作業です。`, input: payload.request.description, output: conversational ? "応答" : artifactName, knowledge_refs: [domainName], diagnostics: `run packet: domain=${payload.request.domain_id || domain.id} artifact=${payload.request.artifact_type_id || artifact.id} agent=${workerName}`, review_items: [] },
                { kind: "review", title: "レビュー", state: "完了", actor: reviewerName, summary: "92点です。", rationale: "ルーブリックで確認しました。", input: conversational ? "応答文" : artifactFileName, output: "92 / 100", knowledge_refs: conversational ? ["応答品質"] : [`${artifactName}ルーブリック`], diagnostics: "", review_items: [] },
                { kind: "organizer", title: "オーガナイザーまとめ", state: "完了", actor: "Organizer", summary: "レビュー結果を踏まえ、依頼者向けの最終回答を整えました。", rationale: "作業結果とレビュー結果を最終回答に統合しました。", input: conversational ? "応答文 / 92 / 100" : `${artifactFileName} / 92 / 100`, output: answer, knowledge_refs: conversational ? ["応答品質"] : [domainName], diagnostics: "", review_items: [] },
              ],
            };
            if (payload.request.description.includes("質問")) {
              detail.item.status_label = "要対応・質問";
              detail.item.status_kind = "question";
              detail.item.next_action = "質問に回答";
              detail.next_action_kind = "answer_question";
              detail.approval_ready = false;
              detail.question = "対象読者は誰ですか?";
              detail.question_source = "Writer / 作成";
              detail.question_options = ["初めてNagareを使う開発者", "既存ユーザー", "運用担当者"];
              detail.answer = "";
              detail.steps = [
                { kind: "worker", title: "作成", state: "質問", actor: workerName, summary: "対象読者の確認が必要です。", rationale: "読者により表現が変わるため", input: payload.request.description, output: "質問を作成", knowledge_refs: [domainName], diagnostics: "", review_items: [] },
              ];
            }
            if (payload.request.description.includes("回復")) {
              detail.item.status_label = "要対応・回復";
              detail.item.status_kind = "recover";
              detail.item.next_action = "回復方法を選択";
              detail.next_action_kind = "recover";
              detail.approval_ready = false;
              detail.recovery = null;
              detail.answer = "";
              detail.steps = [
                { kind: "worker", title: "作成", state: "失敗", actor: workerName, summary: "出力契約を満たせませんでした。", rationale: "必須項目が不足", input: payload.request.description, output: "", knowledge_refs: [domainName], diagnostics: "missing output", review_items: [] },
              ];
            }
            if (payload.request.description.includes("自動進行")) {
              detail.item.status_label = "処理中";
              detail.item.status_kind = "running";
              detail.item.next_action = "担当を整理";
              detail.item.result_summary = "担当エージェントを整理しています。";
              detail.answer = "";
              detail.next_action_kind = "dispatch";
              detail.approval_ready = false;
              detail.review = null;
              detail.steps = [
                { kind: "organizer", title: "受付・整理", state: "処理中", actor: "Organizer", summary: "依頼内容から担当を整理しています。", rationale: "作成直後に次の判断点まで進めます。", input: payload.request.description, output: "担当整理中", knowledge_refs: [domainName], diagnostics: "", review_items: [] },
              ];
            }
            if (payload.request.approval_policy === "auto_complete_on_review_pass" && payload.request.description.includes("自動採用")) {
              detail.item.status_label = "完了";
              detail.item.status_kind = "done";
              detail.item.next_action = "完了";
              detail.item.result_summary = "レビュー合格により自動完了しました。";
              detail.next_action_kind = "done";
              detail.approval_ready = false;
              detail.review.concerns = [];
              detail.review.items = [
                { item: "手順の再現性", verdict: "pass", evidence: "順番に読めます。", score_label: "50 / 50", concern_note: "" },
                { item: "読みやすさ", verdict: "pass", evidence: "簡潔です。", score_label: "25 / 25", concern_note: "" },
              ];
              detail.steps = [
                ...detail.steps,
                { kind: "system", title: "自動採用", state: "完了", actor: "Nagare", summary: "レビュー合格で自動完了しました。", rationale: "approval_policy=auto_complete_on_review_pass", input: "レビュー結果", output: "完了", knowledge_refs: [], diagnostics: "", review_items: [] },
              ];
            }
            appState.work_items = [detail.item];
            appState.project.work_count = 1;
            return cloneValue(detail);
          }
          if (command === "get_work_detail") return cloneValue(detail);
          if (command === "read_artifact_content") {
            if (options.failReadArtifactContent) throw new Error(options.failReadArtifactContent);
            return cloneValue({
              display_path: "C:/nagare-desktop-e2e/resolved/README.md",
              content: "# README\n\nセットアップ手順を更新しました。",
              size_bytes: 42,
              truncated: false,
            });
          }
          if (command === "advance_work") {
            if (detail?.next_action_kind === "dispatch") {
              completeMockAdvance();
              return cloneValue(detail);
            }
            return cloneValue(detail);
          }
          if (command === "start_work_background") {
            const delay = Number(options.backgroundAdvanceDelay ?? 100);
            setTimeout(() => completeMockAdvance(), delay);
            return cloneValue(appState);
          }
          if (command === "answer_work") {
            detail.question = null;
            detail.question_source = "";
            detail.answer = `回答を受け取りました: ${payload.request.answer}`;
            detail.item.status_label = "確認待ち";
            detail.item.status_kind = "review";
            detail.item.next_action = "結果を確認";
            detail.next_action_kind = "approve";
            detail.approval_ready = true;
            appState.work_items = [detail.item];
            return cloneValue(detail);
          }
          if (command === "create_work_recovery") {
            detail.recovery = {
              id: "recovery_1",
              status: "draft",
              action: "retry",
              failure_class: "missing_output",
              reason: "出力契約の必須項目が不足しています",
              summary: "不足項目を明示して再実行します",
              impact: "成果物はまだ確定していません",
              handoff_completed: ["依頼整理", "初回実行"],
              handoff_pending: ["不足項目を補う"],
              target_agent: "Writer",
              command_hint: "",
              warnings: ["同じ失敗が続く場合はプロンプトを見直してください"],
              prompt_hint: "不足項目を補ってください",
            };
            return cloneValue(detail);
          }
          if (command === "accept_work_recovery") {
            detail.recovery.status = "accepted";
            return cloneValue(detail);
          }
          if (command === "apply_work_recovery") {
            detail.recovery = null;
            detail.answer = "回復後にREADMEを更新しました。";
            detail.item.status_label = "確認待ち";
            detail.item.status_kind = "review";
            detail.item.next_action = "結果を確認";
            detail.next_action_kind = "approve";
            detail.approval_ready = true;
            appState.work_items = [detail.item];
            return cloneValue(detail);
          }
          if (command === "approve_work") {
            detail.item.status_label = "完了";
            detail.item.status_kind = "done";
            detail.item.next_action = "なし";
            detail.approval_ready = false;
            detail.next_action_kind = "done";
            appState.work_items = [detail.item];
            return cloneValue(detail);
          }
          if (command === "reject_work") {
            detail.item.status_label = "処理中";
            detail.item.status_kind = "running";
            detail.item.next_action = "担当を整理";
            detail.item.result_summary = "差し戻しコメントを受け取り、担当エージェントへ再実行の指示を渡します。";
            detail.approval_ready = false;
            detail.next_action_kind = "dispatch";
            detail.steps = [
              ...detail.steps,
              { kind: "human", title: "あなたの差し戻し", state: "完了", actor: "あなた", summary: "レビュー懸念を引用して差し戻しました。", rationale: payload.request.rationale, input: "レビュー結果", output: "再実行指示", knowledge_refs: [], diagnostics: "", review_items: [] },
              { kind: "organizer", title: "再割り当て", state: "処理中", actor: "Organizer", summary: "差し戻しコメントを担当エージェントへ渡す準備をしています。", rationale: "採用されなかった結果を続きから再実行します。", input: payload.request.rationale, output: "担当整理中", knowledge_refs: ["プロダクト文書"], diagnostics: "", review_items: [] },
            ];
            appState.work_items = [detail.item];
            return cloneValue(detail);
          }
          if (command === "delete_work") {
            appState.work_items = (appState.work_items || []).filter((item) => item.id !== payload.request.id);
            appState.project.work_count = appState.work_items.length;
            if (detail?.item?.id === payload.request.id) detail = null;
            return cloneValue(appState);
          }
          if (command === "save_project_settings") {
            const request = payload.request;
            appState.project = {
              ...appState.project,
              name: request.display_name || appState.project.name,
              icon: request.icon || appState.project.icon,
              organizer_agent_id: request.organizer_agent_id === "__builtin__" ? "" : request.organizer_agent_id,
              organizer_label: request.organizer_agent_id === "__builtin__" ? "標準" : request.organizer_agent_id,
              work_agent: request.work_agent_id ? (appState.agents.find((agent) => agent.id === request.work_agent_id)?.name || request.work_agent_id) : appState.project.work_agent,
              review_agent: request.review_agent_id ? (appState.agents.find((agent) => agent.id === request.review_agent_id)?.name || request.review_agent_id) : appState.project.review_agent,
              default_domain_id: request.default_domain_id ?? appState.project.default_domain_id,
              default_artifact_type_id: request.default_artifact_type_id ?? appState.project.default_artifact_type_id,
              workflow_mode: request.workflow_mode,
              approval_policy: request.approval_policy,
            };
            recordImprovement(request);
            return cloneValue(appState);
          }
          if (command === "dismiss_improvement") {
            dismissImprovement(payload.request);
            return cloneValue(appState);
          }
          if (command === "delete_project") {
            appState = {
              ...appState,
              initialized: false,
              project: null,
              work_items: [],
              agents: [],
              domains: [],
              artifact_types: [],
              skill_packages: [],
              mcp_connections: [],
            };
            return cloneValue(appState);
          }
          if (command === "refresh_runtime_status") {
            const runtime = {
              ...appState.runtimes.find((item) => item.id === payload.request.runtime_id),
              available: options.runtimeRefreshAvailable === false ? false : true,
              detail: options.runtimeRefreshDetail || "mock refreshed",
            };
            appState.runtimes = appState.runtimes.map((item) => item.id === runtime.id ? runtime : item);
            return cloneValue({ runtime, state: appState });
          }
          if (command === "save_runtime_model_defaults") {
            const request = payload.request;
            const toolKindByRuntime = {
              claude: "claude_code",
              codex: "codex_cli",
              opencode: "opencode",
              openclaw: "openclaw",
            };
            const toolKind = toolKindByRuntime[request.runtime_id] || request.runtime_id;
            let count = 0;
            appState.agents = appState.agents.map((agent) => {
              if (agent.tool_kind !== toolKind && agent.runtime !== request.runtime_id) return agent;
              count += 1;
              return {
                ...agent,
                model: request.model || "実行環境既定",
                model_provider: request.model_provider,
                model_base_url: request.model_base_url,
              };
            });
            appState.runtimes = appState.runtimes.map((runtime) => runtime.id === request.runtime_id ? {
              ...runtime,
              configured_model: request.model,
              configured_provider: request.model_provider,
              configured_base_url: request.model_base_url,
              configured_agent_count: count,
            } : runtime);
            return cloneValue(appState);
          }
          if (command === "add_skill") {
            const request = payload.request;
            const pkg = {
              id: request.package_id,
              source_kind: request.source_kind,
              source: request.source,
              install_scope: request.install_scope,
              installed_targets: request.install_targets || [],
              provided_skill_sets: [request.skill_set_id || request.package_id],
            };
            appState.skill_packages = [
              ...appState.skill_packages.filter((item) => item.id !== pkg.id),
              pkg,
            ];
            return cloneValue(appState);
          }
          if (command === "delete_skill_package_command") {
            const packageId = payload.request.package_id;
            const pkg = appState.skill_packages.find((item) => item.id === packageId);
            const removedSkillSets = pkg?.provided_skill_sets?.length ? pkg.provided_skill_sets : [packageId];
            const detachedAgents = [];
            appState.agents = appState.agents.map((agent) => {
              if (!(agent.skill_set_ids || []).some((id) => removedSkillSets.includes(id))) return agent;
              detachedAgents.push(agent.id);
              return {
                ...agent,
                skill_set_ids: (agent.skill_set_ids || []).filter((id) => !removedSkillSets.includes(id)),
              };
            });
            appState.skill_packages = appState.skill_packages.filter((item) => item.id !== packageId);
            return cloneValue({
              state: appState,
              package_id: packageId,
              removed_skill_sets: removedSkillSets,
              detached_agents: detachedAgents,
              installed_body_removed: payload.request.remove_installed_body,
              warnings: options.skillDeleteWarnings || [],
            });
          }
          if (command === "save_mcp_connection") {
            const request = payload.request;
            if (!String(request.id || "").trim()) throw new Error("MCP接続IDを入力してください。");
            if (!String(request.display_name || "").trim()) throw new Error("MCP接続名を入力してください。");
            if (!String(request.command || "").trim()) throw new Error("MCPサーバーのコマンドを入力してください。");
            const env = parseMcpEnvLines(request.env);
            const connection = {
              id: request.id,
              name: request.display_name,
              tool_kind: request.tool_kind,
              runtime_label: request.tool_kind === "codex_cli" ? "Codex CLI" : request.tool_kind,
              agent_assignable: ["codex", "codex_cli"].includes(request.tool_kind),
              command: request.command,
              args: String(request.args || "").split(/\r?\n/).filter(Boolean),
              env,
              test_args: String(request.test_args || "").split(/\r?\n/).filter(Boolean),
              test_status: "untested",
              test_detail: "",
            };
            appState.mcp_connections = [
              ...appState.mcp_connections.filter((item) => item.id !== connection.id),
              connection,
            ];
            return cloneValue(appState);
          }
          if (command === "test_mcp_connection_command") {
            const connection = appState.mcp_connections.find((item) => item.id === payload.request.id);
            connection.test_status = options.mcpTestSuccess === false ? "failed" : "success";
            connection.test_detail = options.mcpTestDetail || "mock ok";
            return cloneValue({ state: appState, success: options.mcpTestSuccess === false ? false : true, detail: connection.test_detail });
          }
          if (command === "delete_mcp_connection_command") {
            const mcpId = payload.request.id;
            appState.mcp_connections = appState.mcp_connections.filter((item) => item.id !== mcpId);
            appState.agents = appState.agents.map((agent) => ({
              ...agent,
              mcp_connection_ids: (agent.mcp_connection_ids || []).filter((id) => id !== mcpId),
            }));
            return cloneValue(appState);
          }
          if (command === "generate_agent_prompt_draft") {
            const request = payload.request;
            const domainLabels = (request.domain_ids || [])
              .map((id) => appState.domains.find((domain) => domain.id === id)?.name)
              .filter(Boolean);
            const artifactLabels = (request.artifact_type_ids || [])
              .map((id) => appState.artifact_types.find((artifact) => artifact.id === id)?.name)
              .filter(Boolean);
            return {
              text: [
                `あなたは Nagare の「${request.display_name || "このエージェント"}」です。`,
                `主な役割: ${request.role === "reviewer" ? "成果物をルーブリックで評価し、点数・根拠・懸念を分けて返す" : "依頼に対して成果物を作成し、根拠と変更内容を明確にする"}。`,
                request.description ? `説明: ${request.description}` : "",
                request.specialties?.length ? `得意分野: ${request.specialties.join(" / ")}` : "",
                domainLabels.length ? `担当ドメイン: ${domainLabels.join(" / ")}` : "",
                artifactLabels.length ? `担当成果物: ${artifactLabels.join(" / ")}` : "",
                "",
                "守ること:",
                "- 不明点を推測で埋めず、必要なら質問として返す",
              ].filter((line) => line !== "").join("\n"),
            };
          }
          if (command === "generate_rubric_draft") {
            const request = payload.request;
            const domain = appState.domains.find((item) => item.id === request.domain_id);
            return {
              text: [
                "## 目的への適合 (30)",
                `${request.display_name || "成果物"}が依頼の目的に合っている。${request.description ? `対象: ${request.description}` : ""}`,
                "",
                "## 正確性と根拠 (30)",
                "事実、手順、判断の根拠が確認でき、誤りや未検証の断定がない。",
                "",
                "## 読みやすさ (20)",
                "対象読者が迷わず読める構成と表現になっている。",
                "",
                "## ドメイン知識の反映 (20)",
                [
                  domain?.shared_knowledge?.length ? `共通知識: ${domain.shared_knowledge.join(" / ")}` : "",
                  request.knowledge?.length ? `成果物知識: ${request.knowledge.join(" / ")}` : "",
                  "必要な知識を反映し、不要な内部用語を避けている。",
                ].filter(Boolean).join(" "),
              ].join("\n"),
            };
          }
          if (command === "save_agent") {
            const request = payload.request;
            const agent = {
              id: request.id,
              name: request.display_name,
              avatar: request.avatar,
              role: request.role,
              description: request.description,
              tool_kind: request.tool_kind,
              runtime: `${request.tool_kind}-runtime`,
              model: request.model || "実行環境既定",
              model_provider: request.model_provider,
              model_base_url: request.model_base_url,
              prompt: request.prompt,
              specialties: request.specialties || [],
              domain_ids: request.domain_ids || [],
              artifact_type_ids: request.artifact_type_ids || [],
              skill_set_ids: request.skill_set_ids || [],
              mcp_connection_ids: request.mcp_connection_ids || [],
            };
            appState.agents = [
              ...appState.agents.filter((item) => item.id !== agent.id),
              agent,
            ];
            recordImprovement(request);
            return cloneValue(appState);
          }
          if (command === "delete_agent_command") {
            appState.agents = appState.agents.filter((item) => item.id !== payload.request.id);
            return cloneValue(appState);
          }
          if (command === "save_domain") {
            const request = payload.request;
            const domain = {
              id: request.id,
              name: request.display_name,
              description: request.description,
              shared_knowledge: String(request.shared_knowledge || "").split(/\r?\n/).filter(Boolean),
              common_rubric: String(request.common_rubric || "").split(/\r?\n/).filter(Boolean),
              dispatch_hints: String(request.dispatch_hints || "").split(/\r?\n/).filter(Boolean),
              artifact_type_count: appState.artifact_types.filter((item) => item.domain_id === request.id).length,
            };
            appState.domains = [
              ...appState.domains.filter((item) => item.id !== domain.id),
              domain,
            ];
            return cloneValue(appState);
          }
          if (command === "delete_domain_command") {
            appState.domains = appState.domains.filter((item) => item.id !== payload.request.id);
            return cloneValue(appState);
          }
          if (command === "save_artifact_type") {
            const request = payload.request;
            const rubricLines = String(request.rubric || "").split(/\r?\n/).filter(Boolean);
            const artifact = {
              id: request.id,
              domain_id: request.domain_id,
              name: request.display_name,
              description: request.description,
              knowledge: String(request.knowledge || "").split(/\r?\n/).filter(Boolean),
              rubric: rubricLines,
              dispatch_hints: String(request.dispatch_hints || "").split(/\r?\n/).filter(Boolean),
              rubric_count: rubricLines.filter((line) => line.startsWith("## ")).length,
              rubric_score_total: 100,
            };
            appState.artifact_types = [
              ...appState.artifact_types.filter((item) => item.id !== artifact.id),
              artifact,
            ];
            appState.domains = appState.domains.map((domain) => ({
              ...domain,
              artifact_type_count: appState.artifact_types.filter((item) => item.domain_id === domain.id).length,
            }));
            recordImprovement(request);
            return cloneValue(appState);
          }
          if (command === "delete_artifact_type_command") {
            appState.artifact_types = appState.artifact_types.filter((item) => item.id !== payload.request.id);
            appState.domains = appState.domains.map((domain) => ({
              ...domain,
              artifact_type_count: appState.artifact_types.filter((item) => item.domain_id === domain.id).length,
            }));
            return cloneValue(appState);
          }
          throw new Error(`unexpected command: ${command}`);
        },
      },
    };
    localStorage.clear();
    if (options.storedRoot) localStorage.setItem("nagare.root", options.storedRoot);
  }, { initial: clone(initialState), options: clone(options) });
}

test("desktop prototype UI clears stale stored root when project is no longer initialized", async ({ page }) => {
  const initial = baseState(false);
  initial.root = null;
  await installTauriMock(page, initial, { storedRoot: "C:/deleted-nagare-project" });
  await page.goto(desktopIndexUrl);

  await expect(page.locator("#scr-home-empty.screen.active")).toBeVisible();
  await expect(page.getByRole("button", { name: "セットアップを開始" })).toBeVisible();
  await expect.poll(() => page.evaluate(() => localStorage.getItem("nagare.root"))).toBeNull();

  const appStateCall = await page.evaluate(() => window.__nagareDesktopCalls.find((call) => call.command === "app_state"));
  expect(appStateCall.payload.root).toBe("C:/deleted-nagare-project");
});

test("desktop prototype UI clears stale operation panels when loadState returns uninitialized", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await page.getByRole("button", { name: "設定" }).click();
  await page.locator('#scr-project-settings input[name="display_name"]').fill("一時プロジェクト");
  await page.locator("#scr-project-settings").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("保存結果: 一時プロジェクト");

  const stale = baseState(false);
  stale.root = null;
  await page.evaluate((nextState) => window.__nagareDesktopSetMockState(nextState), stale);
  await page.evaluate(() => NagareApp.loadState());

  await expect(page.locator("#scr-home-empty.screen.active")).toBeVisible();
  await expect.poll(() => page.evaluate(() => localStorage.getItem("nagare.root"))).toBeNull();
  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await expect(page.locator("#scr-project-list.screen.active")).not.toContainText("保存結果: 一時プロジェクト");
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("プロジェクトはまだありません");
});

test("desktop prototype UI shows persistent recovery when app state loading fails", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    failAppStateTimes: 1,
    failAppStateMessage: "project config is unreadable",
    storedRoot: "C:/nagare-desktop-e2e",
  });
  await page.goto(desktopIndexUrl);

  await expect(page.locator("#scr-home-empty.screen.active")).toContainText("状態を読み込めませんでした");
  await expect(page.locator("#scr-home-empty.screen.active")).toContainText("project config is unreadable");
  await expect(page.locator("#scr-home-empty.screen.active")).toContainText("保存済みの場所");

  await page.getByRole("button", { name: "再読み込み" }).click();

  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await expect(page.locator("#work-project-select")).toHaveValue("nagare-desktop-e2e");

  const appStateCalls = await page.evaluate(() => window.__nagareDesktopCalls.filter((call) => call.command === "app_state"));
  expect(appStateCalls).toHaveLength(2);
});

test("desktop prototype UI clears saved root when the stored path is missing", async ({ page }) => {
  await installTauriMock(page, baseState(false), {
    failAppStateTimes: 1,
    failAppStateMessage: "指定されたパスが見つかりません。 (os error 3)",
    storedRoot: "C:/deleted-nagare-project",
  });
  await page.goto(desktopIndexUrl);

  await expect(page.locator("#scr-home-empty.screen.active")).toBeVisible();
  await expect(page.getByRole("button", { name: "セットアップを開始" })).toBeVisible();
  await expect(page.locator("#scr-home-empty.screen.active")).not.toContainText("状態を読み込めませんでした");
  await expect.poll(() => page.evaluate(() => localStorage.getItem("nagare.root"))).toBeNull();

  const appStateCalls = await page.evaluate(() => window.__nagareDesktopCalls.filter((call) => call.command === "app_state"));
  expect(appStateCalls).toHaveLength(2);
  expect(appStateCalls[0].payload.root).toBe("C:/deleted-nagare-project");
  expect(appStateCalls[1].payload.root).toBeNull();
});

test("desktop prototype UI initializes, creates work, and approves the result", async ({ page }) => {
  await installTauriMock(page, baseState(false));
  await page.goto(desktopIndexUrl);

  await expect(page.locator("#scr-home-empty.screen.active")).toBeVisible();
  await page.getByRole("button", { name: "セットアップを開始" }).click();
  await page.getByRole("button", { name: "選択…" }).click();
  await expect(page.locator("#setup-root")).toHaveValue("C:/nagare-desktop-e2e");
  await expect(page.locator("#setup-name")).toHaveValue("nagare-desktop-e2e");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "次へ" }).click();
  await page.locator('#app-dynamic-modal [data-runtime-option="codex"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存して接続確認" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "はじめる" }).click();

  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await expect(page.locator("#work-project-select")).toHaveValue("nagare-desktop-e2e");
  await page.locator('#create-work-form textarea[name="description"]').fill("README のセットアップ手順を更新して");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  await expect(page.locator("#scr-detail-review.screen.active")).toBeVisible();
  await expect(page.locator("#attn-dot")).toBeVisible();
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("3 / 3工程");
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("README のセットアップ手順を整理");
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("レビュー済み。必要なら採用または差し戻しできます。");
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("現在: オーガナイザーまとめ / Organizer");
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("更新: now");
  await expect(page.getByRole("heading", { name: "結果" })).toBeVisible();
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("依頼への回答");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("依頼");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("README のセットアップ手順を更新して");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("オーガナイザーまとめ");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("できたもの");
  await expect(page.locator("#scr-detail-review.screen.active .answer-box")).toContainText("生成結果");
  await expect(page.locator("#scr-detail-review.screen.active .answer-box")).toContainText("README のセットアップ手順を整理");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("読みやすさ");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("採用を推奨");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("README.md");
  await expect(page.locator("#scr-detail-review.screen.active .result-concerns")).toContainText("補足が長い箇所があります。");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("差し戻しコメントは次の実行");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("プロダクト文書 / README のルーブリックで評価しています");
  await expect(page.locator("#scr-detail-review.screen.active .trace")).toContainText("読みやすさ");
  await expect(page.locator("#scr-detail-review.screen.active .trace")).toContainText("オーガナイザーまとめ");
  await page.getByRole("button", { name: "レビューをコピー" }).click();
  await expect(page.locator("#app-notice")).toContainText("レビュー結果をコピーしました。");
  const copiedReview = await page.evaluate(() => window.__nagareCopiedText);
  expect(copiedReview).toContain("レビュー結果");
  expect(copiedReview).toContain("評価: 92 / 100");
  expect(copiedReview).toContain("判断: 採用を推奨");
  expect(copiedReview).toContain("懸念:");
  expect(copiedReview).toContain("補足が長い箇所があります。");
  expect(copiedReview).toContain("評価項目:");
  expect(copiedReview).toContain("根拠: 順番に読めます。");
  await page.getByRole("button", { name: "ルーブリックを編集" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("README");
  await expect(page.locator("#app-dynamic-modal")).toContainText("成果物ごとの知識とルーブリックを設定します");
  await expect(page.locator('#app-dynamic-modal textarea[name="rubric"]')).toHaveValue(/手順の再現性/);
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await page.getByRole("button", { name: "コメントを付けて差し戻す" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("引用するレビュー懸念");
  await expect(page.locator("#app-dynamic-modal")).toContainText("補足が長い箇所があります。");
  await expect(page.locator("#app-dynamic-modal")).toContainText("読みやすさ: 補足が長いです。");
  await expect(page.locator('#app-dynamic-modal textarea[name="rationale"]')).toHaveValue(/次の懸念に対応してください/);
  await expect(page.locator('#app-dynamic-modal textarea[name="rationale"]')).toHaveValue(/読みやすさ: 補足が長いです。/);
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "戻る" }).click();
  await page.locator('[data-artifact-detail="0"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("成果物の詳細");
  await expect(page.locator("#app-dynamic-modal")).toContainText("README.md");
  await expect(page.locator("#app-dynamic-modal")).toContainText("C:/nagare-desktop-e2e/README.md");
  await expect(page.locator("#app-dynamic-modal")).toContainText("読み込んだファイル");
  await expect(page.locator("#app-dynamic-modal")).toContainText("C:/nagare-desktop-e2e/resolved/README.md");
  await expect(page.locator("#app-dynamic-modal")).toContainText("内容プレビュー");
  await expect(page.locator("#app-dynamic-modal")).toContainText("セットアップ手順を更新しました");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.getByRole("button", { name: "この結果を採用する" })).toBeVisible();
  await page.getByRole("button", { name: "この結果を採用する" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "採用して完了" }).click();

  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("この作業は完了し、結果は採用済みです");
  await expect(page.locator("#attn-dot")).toBeHidden();
  await expect(page.locator("#scr-detail-done.screen.active .status-strip")).toContainText("3 / 3工程");
  await expect(page.locator("#scr-detail-done.screen.active .status-strip")).toContainText("現在: オーガナイザーまとめ / Organizer");
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("完了サマリー");
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("README のセットアップ手順を整理");
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("採用成果物");
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("README.md");
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("評価点");
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("92 / 100");
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("3工程");
  await page.getByRole("button", { name: "ワーク一覧へ戻る" }).click();
  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await page.locator('[data-work-id="work_1"] [data-work-open]').click();
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("この作業は完了し、結果は採用済みです");

  const calls = await page.evaluate(() => window.__nagareDesktopCalls.map((call) => call.command));
  expect(calls).toEqual(expect.arrayContaining([
    "app_state",
    "choose_project_folder",
    "refresh_runtime_status",
    "initialize_project_with_runtime",
    "create_work",
    "read_artifact_content",
    "approve_work",
  ]));
  expect(calls.indexOf("refresh_runtime_status")).toBeLessThan(calls.indexOf("initialize_project_with_runtime"));
  const readCall = await page.evaluate(() => window.__nagareDesktopCalls.find((call) => call.command === "read_artifact_content"));
  expect(readCall.payload.request.uri).toBe("C:/nagare-desktop-e2e/README.md");
  const createCall = await page.evaluate(() => window.__nagareDesktopCalls.find((call) => call.command === "create_work"));
  expect(createCall.payload.request.project).toBe("nagare-desktop-e2e");
  expect(createCall.payload.request.domain_id).toBe("product-docs");
  expect(createCall.payload.request.artifact_type_id).toBe("readme");
});

test("desktop prototype UI runs the main workflow with project, domain, and agents only", async ({ page }) => {
  const initial = baseState(true);
  initial.skill_sets = [];
  initial.skill_packages = [];
  initial.mcp_connections = [];
  initial.mcp_capabilities = [];
  initial.agents = initial.agents.map((agent) => ({
    ...agent,
    skill_set_ids: [],
    mcp_connection_ids: [],
  }));
  initial.project.default_domain_id = "product-docs";
  initial.project.default_artifact_type_id = "readme";
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await page.locator('#create-work-form textarea[name="description"]').fill("README の導入手順を整理して");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("README のセットアップ手順を整理");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("プロダクト文書 / README のルーブリックで評価しています");
  await expect(page.locator("#scr-detail-review.screen.active .trace")).toContainText("Organizer");
  await expect(page.locator("#scr-detail-review.screen.active .trace")).toContainText("Reviewer");

  await page.getByRole("button", { name: "この結果を採用する" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "採用して完了" }).click();
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("この作業は完了し、結果は採用済みです");

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const createCall = calls.find((call) => call.command === "create_work");
  expect(createCall.payload.request.project).toBe("nagare-desktop-e2e");
  expect(createCall.payload.request.domain_id).toBe("product-docs");
  expect(createCall.payload.request.artifact_type_id).toBe("readme");
  expect(calls.map((call) => call.command)).toEqual(expect.arrayContaining(["create_work", "approve_work"]));
  expect(calls.map((call) => call.command)).not.toContain("install_skill_package");
  expect(calls.map((call) => call.command)).not.toContain("save_mcp_connection");
});

test("desktop prototype UI sets up project domain and agents before completing a work item", async ({ page }) => {
  await installTauriMock(page, baseState(false));
  await page.goto(desktopIndexUrl);

  await page.getByRole("button", { name: "セットアップを開始" }).click();
  await page.getByRole("button", { name: "選択…" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "次へ" }).click();
  await page.locator('#app-dynamic-modal [data-runtime-option="codex"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存して接続確認" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "はじめる" }).click();
  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();

  await page.getByRole("link", { name: /ナレッジ/ }).click();
  await page.getByRole("button", { name: "ドメインを追加" }).click();
  await page.locator('#scr-knowledge-domain input[name="id"]').fill("guide-docs");
  await page.locator('#scr-knowledge-domain input[name="display_name"]').fill("ガイド文書");
  await page.locator('#scr-knowledge-domain textarea[name="description"]').fill("導入ガイドや運用手順を扱う");
  await page.locator('#scr-knowledge-domain textarea[name="shared_knowledge"]').fill("対象読者: 初めて使う開発者\n文体: 短く具体的に");
  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("ガイド文書");

  await page.locator('[data-followup-add-artifact="guide-docs"]').click();
  await page.locator('#scr-knowledge-rubric input[name="id"]').fill("onboarding-guide");
  await page.locator('#scr-knowledge-rubric input[name="display_name"]').fill("オンボーディングガイド");
  await expect(page.locator('#scr-knowledge-rubric select[name="domain_id"]')).toHaveValue("guide-docs");
  await page.locator('#scr-knowledge-rubric textarea[name="description"]').fill("初回利用者が環境構築から最初の実行まで進めるガイド");
  await page.locator('#scr-knowledge-rubric textarea[name="knowledge"]').fill("セットアップ順序\n詰まりやすい認証手順");
  await page.locator('#scr-knowledge-rubric textarea[name="rubric"]').fill("## 完了条件 (60)\n初回利用者が最後まで進められる\n\n## 説明の具体性 (40)\n操作と確認結果が具体的である");
  await page.locator("#scr-knowledge-rubric").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("オンボーディングガイド");

  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await page.getByRole("button", { name: "新規エージェント" }).click();
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("Guide Writer");
  await page.locator('#app-dynamic-modal select[name="role"]').selectOption("worker");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "作成して設定へ" }).click();
  await expect(page.locator("#scr-agent-settings.screen.active")).toBeVisible();
  await page.locator('#scr-agent-settings textarea[name="description"]').fill("導入ガイドの作成を担当する");
  await page.locator('#scr-agent-settings textarea[name="specialties"]').fill("guide\nonboarding");
  await page.locator('#scr-agent-settings [data-agent-tab="scope"]').click();
  await page.locator('#scr-agent-settings input[name="domain_ids"][value="guide-docs"]').check();
  await page.locator('#scr-agent-settings input[name="artifact_type_ids"][value="onboarding-guide"]').check();
  await page.locator("#scr-agent-settings").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("Guide Writer");

  await page.getByRole("button", { name: "新規エージェント" }).click();
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("Guide Reviewer");
  await page.locator('#app-dynamic-modal select[name="role"]').selectOption("reviewer");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "作成して設定へ" }).click();
  await expect(page.locator("#scr-agent-settings.screen.active")).toBeVisible();
  await page.locator('#scr-agent-settings textarea[name="description"]').fill("導入ガイドをルーブリックでレビューする");
  await page.locator('#scr-agent-settings textarea[name="specialties"]').fill("review\nguide");
  await page.locator('#scr-agent-settings [data-agent-tab="scope"]').click();
  await page.locator('#scr-agent-settings input[name="domain_ids"][value="guide-docs"]').check();
  await page.locator('#scr-agent-settings input[name="artifact_type_ids"][value="onboarding-guide"]').check();
  await page.locator("#scr-agent-settings").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("Guide Reviewer");

  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await page.getByRole("button", { name: "設定" }).click();
  await page.locator('#scr-project-settings [data-project-tab="agents"]').click();
  await page.locator('#scr-project-settings select[name="work_agent_id"]').selectOption("guide-writer");
  await page.locator('#scr-project-settings select[name="review_agent_id"]').selectOption("guide-reviewer");
  await page.locator('#scr-project-settings [data-project-tab="knowledge"]').click();
  await page.locator('#scr-project-settings select[name="default_domain_id"]').selectOption("guide-docs");
  await page.locator('#scr-project-settings select[name="default_artifact_type_id"]').selectOption("onboarding-guide");
  await page.locator("#scr-project-settings").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("保存結果: nagare-desktop-e2e");

  await page.getByRole("link", { name: /ワーク/ }).click();
  await page.locator('#create-work-form textarea[name="description"]').fill("オンボーディングガイドを整備して");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("オンボーディングガイド の構成と確認方法を整理しました。");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("オンボーディングガイド");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("ガイド文書 / オンボーディングガイド のルーブリックで評価しています");
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("現在: オーガナイザーまとめ / Organizer");
  await expect(page.locator("#scr-detail-review.screen.active .trace")).toContainText("担当: Guide Reviewer");
  await page.getByRole("button", { name: "この結果を採用する" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "採用して完了" }).click();
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("この作業は完了し、結果は採用済みです");

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const commands = calls.map((call) => call.command);
  const projectCall = calls.find((call) => call.command === "save_project_settings");
  const createWorkCall = calls.find((call) => call.command === "create_work");
  expect(commands).toEqual(expect.arrayContaining([
    "initialize_project_with_runtime",
    "save_domain",
    "save_artifact_type",
    "save_agent",
    "save_project_settings",
    "create_work",
    "approve_work",
  ]));
  expect(projectCall.payload.request.work_agent_id).toBe("guide-writer");
  expect(projectCall.payload.request.review_agent_id).toBe("guide-reviewer");
  expect(projectCall.payload.request.default_domain_id).toBe("guide-docs");
  expect(projectCall.payload.request.default_artifact_type_id).toBe("onboarding-guide");
  expect(createWorkCall.payload.request.domain_id).toBe("guide-docs");
  expect(createWorkCall.payload.request.artifact_type_id).toBe("onboarding-guide");
  expect(commands).not.toContain("install_skill_package");
  expect(commands).not.toContain("save_mcp_connection");
});

test("desktop prototype UI keeps artifact preview readable when file loading fails", async ({ page }) => {
  await installTauriMock(page, baseState(true), { failReadArtifactContent: "artifact file is outside the project" });
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("README のセットアップ手順を更新して");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("README.md");

  await page.locator('[data-artifact-detail="0"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("成果物の詳細");
  await expect(page.locator("#app-dynamic-modal")).toContainText("README.md");
  await expect(page.locator("#app-dynamic-modal")).toContainText("ファイル内容を読み込めませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("artifact file is outside the project");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.getByRole("button", { name: "この結果を採用する" })).toBeVisible();
});

test("desktop prototype UI keeps setup failure visible and routes to runtime settings", async ({ page }) => {
  await installTauriMock(page, baseState(false), { failInitialize: "Codex CLI program not found" });
  await page.goto(desktopIndexUrl);

  await page.getByRole("button", { name: "セットアップを開始" }).click();
  await page.locator("#setup-name").fill("nagare-desktop-e2e");
  await page.getByRole("button", { name: "選択…" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "次へ" }).click();
  await page.locator('#app-dynamic-modal [data-runtime-option="codex"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存して接続確認" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "はじめる" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("接続確認に失敗しました");
  await expect(page.locator("#app-dynamic-modal")).toContainText("Codex CLI program not found");
  await expect(page.locator("#app-dynamic-modal")).toContainText("インストール、PATH、認証を確認してください");
  await expect(page.locator("#app-dynamic-modal").getByRole("button", { name: "はじめる" })).toBeDisabled();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "実行環境を確認" }).click();

  await expect(page.locator("#scr-settings-runtime.screen.active")).toBeVisible();
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("Codex CLI");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("OpenClaw");

  const initCall = await page.evaluate(() => window.__nagareDesktopCalls.find((call) => call.command === "initialize_project_with_runtime"));
  expect(initCall.payload.request.runtime_id).toBe("codex");
});

test("desktop prototype UI rechecks the selected runtime before setup initialization", async ({ page }) => {
  await installTauriMock(page, baseState(false), { runtimeRefreshAvailable: false, runtimeRefreshDetail: "program not found" });
  await page.goto(desktopIndexUrl);

  await page.getByRole("button", { name: "セットアップを開始" }).click();
  await page.locator("#setup-name").fill("nagare-desktop-e2e");
  await page.locator("#setup-root").fill("C:/nagare-desktop-e2e");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "次へ" }).click();
  await page.locator('#app-dynamic-modal [data-runtime-option="codex"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存して接続確認" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("接続確認に失敗しました");
  await expect(page.locator("#app-dynamic-modal")).toContainText("Codex CLI が見つかりません");
  await expect(page.locator("#app-dynamic-modal")).toContainText("program not found");
  await expect(page.locator("#app-dynamic-modal").getByRole("button", { name: "はじめる" })).toBeDisabled();

  const calls = await page.evaluate(() => window.__nagareDesktopCalls.map((call) => call.command));
  expect(calls).toContain("refresh_runtime_status");
  expect(calls).not.toContain("initialize_project_with_runtime");
});

test("desktop prototype UI blocks setup when no runtime is available", async ({ page }) => {
  const initial = baseState(false);
  initial.runtimes = initial.runtimes.map((runtime) => ({ ...runtime, available: false, detail: "not found" }));
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await page.getByRole("button", { name: "セットアップを開始" }).click();
  await page.locator("#setup-name").fill("nagare-desktop-e2e");
  await page.locator("#setup-root").fill("C:/nagare-desktop-e2e");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "次へ" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("利用できる実行環境が見つかりません");
  await expect(page.locator("#app-dynamic-modal").getByRole("button", { name: "保存して接続確認" })).toBeDisabled();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "実行環境を確認" }).click();
  await expect(page.locator("#scr-settings-runtime.screen.active")).toBeVisible();
  await expect(page.locator("#runtime-status-filter")).toHaveValue("all");
});

test("desktop prototype UI keeps setup inputs when folder selection is canceled", async ({ page }) => {
  await installTauriMock(page, baseState(false), { chooseProjectFolderResult: null });
  await page.goto(desktopIndexUrl);

  await page.getByRole("button", { name: "セットアップを開始" }).click();
  await page.locator("#setup-name").fill("入力中プロジェクト");
  await page.locator("#setup-root").fill("C:/manual-project");
  await page.getByRole("button", { name: "選択…" }).click();

  await expect(page.locator("#setup-name")).toHaveValue("入力中プロジェクト");
  await expect(page.locator("#setup-root")).toHaveValue("C:/manual-project");

  const commands = await page.evaluate(() => window.__nagareDesktopCalls.map((call) => call.command));
  expect(commands).toContain("choose_project_folder");
  expect(commands).not.toContain("initialize_project_with_runtime");
});

test("desktop prototype UI rejects a reviewed result and shows redispatch state", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("README の表現をもう一度整えて");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("採用を推奨");
  await page.getByRole("button", { name: "コメントを付けて差し戻す" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "差し戻して再実行" }).click();

  await expect(page.locator("#scr-detail-running.screen.active")).toContainText("差し戻しを受け取り、担当を整理しています");
  await expect(page.locator("#scr-detail-running.screen.active .status-strip")).toContainText("5 / 5工程");
  await expect(page.locator("#scr-detail-running.screen.active .status-strip")).toContainText("現在: 再割り当て / Organizer");
  await expect(page.locator("#scr-detail-running.screen.active .result-overview")).toContainText("再実行待ち");
  await expect(page.locator("#scr-detail-running.screen.active .result-overview")).toContainText("担当エージェントへ再実行の指示を渡します");
  await expect(page.locator("#scr-detail-running.screen.active")).toContainText("あなたの差し戻し");
  await expect(page.locator("#scr-detail-running.screen.active")).toContainText("再割り当て");
  await expect(page.getByRole("button", { name: "この結果を採用する" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "次の判断点まで進める" })).toBeVisible();

  const rejectCall = await page.evaluate(() => window.__nagareDesktopCalls.find((call) => call.command === "reject_work"));
  expect(rejectCall.payload.request.rationale).toContain("補足が長い箇所があります。");
  expect(rejectCall.payload.request.rationale).toContain("読みやすさ: 補足が長いです。");

  await page.getByRole("button", { name: "次の判断点まで進める" }).click();
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("差し戻し内容を反映");
  await expect(page.locator("#scr-detail-review.screen.active .answer-box")).toContainText("差し戻し内容を反映");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("採用を推奨");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("再作成");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("再レビュー");
  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("94 / 100");
  await expect(page.getByRole("button", { name: "この結果を採用する" })).toBeVisible();

  const advanceCall = await page.evaluate(() => window.__nagareDesktopCalls.find((call) => call.command === "advance_work"));
  expect(advanceCall.payload.request.max_steps).toBe(8);
});

test("desktop prototype UI shows progress while advancing to the next decision point", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    commandDelays: { advance_work: 600 },
  });
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("README の表現をもう一度整えて");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);
  await page.getByRole("button", { name: "コメントを付けて差し戻す" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "差し戻して再実行" }).click();

  await page.getByRole("button", { name: "次の判断点まで進める" }).click();
  await expect(page.getByRole("button", { name: "進行中..." })).toBeDisabled();
  await expect(page.locator("[data-advance-progress]")).toContainText("次の判断点まで処理しています");
  await expect(page.locator("[data-advance-progress]")).toContainText("このまま待ってください");
  await expect(page.locator("#app-notice")).toContainText("ワークを進めています。");
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("差し戻し内容を反映");
});

test("desktop prototype UI answers agent questions from work detail", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("質問が必要なREADMEを作って");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  await expect(page.locator("#scr-detail-question.screen.active")).toContainText("対象読者は誰ですか?");
  await expect(page.locator("#scr-detail-question.screen.active .status-strip")).toContainText("1 / 1工程");
  await expect(page.locator("#scr-detail-question.screen.active .status-strip")).toContainText("現在: 作成 / Writer");
  await expect(page.locator("#scr-detail-question.screen.active")).toContainText("質問元: Writer / 作成");
  await expect(page.locator('#scr-detail-question input[name="answer_choice"][value="初めてNagareを使う開発者"]')).toBeChecked();
  await page.locator('#scr-detail-question textarea[name="answer"]').fill("セットアップから読める説明にしてください");
  await page.getByRole("button", { name: "回答して再開" }).click();

  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("回答を受け取りました");
  await expect(page.getByRole("button", { name: "この結果を採用する" })).toBeVisible();

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const answerCall = calls.find((call) => call.command === "answer_work");
  expect(answerCall.payload.request.answer).toBe("初めてNagareを使う開発者\nセットアップから読める説明にしてください");
});

test("desktop prototype UI opens step diagnostics without exposing logs by default", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("README の診断ログを確認したい");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("受付・整理");
  await expect(page.locator("#scr-detail-review.screen.active")).not.toContainText("run packet: domain=product-docs");

  await page.getByRole("button", { name: "診断ログ" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("診断ログ");
  await expect(page.locator("#app-dynamic-modal")).toContainText("run packet: domain=product-docs artifact=readme agent=Writer");
  await page.evaluate(() => {
    const target = document.querySelector("#app-dynamic-modal .mono");
    const range = document.createRange();
    range.selectNodeContents(target);
    const selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
  });
  await page.locator("#app-dynamic-modal").click({ position: { x: 4, y: 4 } });
  await expect(page.locator("#app-dynamic-modal")).toContainText("run packet: domain=product-docs artifact=readme agent=Writer");
  await page.evaluate(() => window.getSelection().removeAllRanges());
});

test("desktop prototype UI keeps step details open while selecting detail text", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("README の流れを確認したい");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  const firstStep = page.locator("#scr-detail-review .step").first();
  await page.locator("#scr-detail-review .step-top").first().click();
  await expect(firstStep).toHaveClass(/open/);
  await page.locator("#scr-detail-review .step").first().locator(".io-v").first().click();
  await expect(firstStep).toHaveClass(/open/);
});

test("desktop prototype UI opens knowledge from step chips", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("README の知識参照を確認したい");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  await page.locator("#scr-detail-review .step-top").first().click();
  await page.locator('#scr-detail-review [data-knowledge-ref="プロダクト文書"]').first().click();
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toBeVisible();
  await expect(page.locator("#scr-knowledge-domain")).toContainText("プロダクト文書");
  await expect(page.locator("#scr-knowledge-domain")).toContainText("共通知識");

  await page.getByRole("link", { name: /^ワーク/ }).click();
  await page.locator('[data-work-id="work_1"]').click();
  await page.locator("#scr-detail-review .step-top").nth(1).click();
  await page.locator('#scr-detail-review [data-knowledge-ref="READMEルーブリック"]').click();
  await expect(page.locator("#scr-knowledge-rubric.screen.active")).toBeVisible();
  await expect(page.locator("#scr-knowledge-rubric")).toContainText("README");
  await expect(page.locator("#scr-knowledge-rubric")).toContainText("ルーブリック");
});

test("desktop prototype UI creates, accepts, and applies recovery plans", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("回復が必要なREADMEを作って");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("回復案を作成");
  await expect(page.locator("#scr-detail-recover.screen.active .status-strip")).toContainText("1 / 1工程");
  await expect(page.locator("#scr-detail-recover.screen.active .status-strip")).toContainText("現在: 作成 / Writer");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("発生工程: 作成 / Writer（失敗）");
  await page.getByRole("button", { name: "回復案を作成" }).click();
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("出力契約の必須項目が不足");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("回復対象: Writer");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("完了済み: 依頼整理 / 初回実行");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("次に渡す内容: 不足項目を補う");

  await page.getByRole("button", { name: "この回復案を採用" }).click();
  await expect(page.locator('#scr-detail-recover textarea[name="prompt"]')).toHaveValue(/不足項目/);
  await page.locator('#scr-detail-recover textarea[name="prompt"]').fill("不足項目を補って再実行してください");
  await page.getByRole("button", { name: "回復して再開" }).click();

  await expect(page.locator("#scr-detail-review.screen.active")).toContainText("回復後にREADMEを更新しました");
  await expect(page.getByRole("button", { name: "この結果を採用する" })).toBeVisible();

  const commands = await page.evaluate(() => window.__nagareDesktopCalls.map((call) => call.command));
  expect(commands).toEqual(expect.arrayContaining([
    "create_work_recovery",
    "accept_work_recovery",
    "apply_work_recovery",
  ]));
});

test("desktop prototype UI keeps work detail operation failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("README の確認操作を試す");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({ approve_work: "approval backend unavailable" }));
  await page.getByRole("button", { name: "この結果を採用する" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "採用して完了" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("結果を採用できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("approval backend unavailable");
  await expect(page.locator("#app-dynamic-modal")).toContainText("結果とレビューの状態を確認");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "戻る" }).click();

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({ reject_work: "reject backend unavailable" }));
  await page.getByRole("button", { name: "コメントを付けて差し戻す" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "差し戻して再実行" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("差し戻しを実行できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("reject backend unavailable");
  await expect(page.locator("#app-dynamic-modal")).toContainText("差し戻しコメントと現在のワーク状態を確認");

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({}));
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "差し戻して再実行" }).click();
  await expect(page.locator("#scr-detail-running.screen.active")).toContainText("差し戻しを受け取り");

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({ advance_work: "advance worker unavailable" }));
  await page.getByRole("button", { name: "次の判断点まで進める" }).click();
  await expect(page.locator("#scr-detail-running.screen.active")).toContainText("ワークを進められませんでした");
  await expect(page.locator("#scr-detail-running.screen.active")).toContainText("advance worker unavailable");
  await expect(page.locator("#scr-detail-running.screen.active")).toContainText("現在の工程と実行環境を確認");

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({}));
  await page.getByRole("link", { name: /ワーク/ }).click();
  await page.locator('#create-work-form textarea[name="description"]').fill("質問が必要なREADMEを作って");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);
  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({ answer_work: "answer backend unavailable" }));
  await page.locator('#scr-detail-question textarea[name="answer"]').fill("開発者向けです");
  await page.getByRole("button", { name: "回答して再開" }).click();
  await expect(page.locator("#scr-detail-question.screen.active")).toContainText("回答を保存できませんでした");
  await expect(page.locator("#scr-detail-question.screen.active")).toContainText("answer backend unavailable");
  await expect(page.locator("#scr-detail-question.screen.active")).toContainText("回答内容と現在のワーク状態を確認");

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({}));
  await page.getByRole("link", { name: /ワーク/ }).click();
  await page.locator('#create-work-form textarea[name="description"]').fill("回復が必要なREADMEを作って");
  await page.getByRole("button", { name: "作業を開始" }).click();
  await openWorkDetail(page);

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({ create_work_recovery: "recovery planner unavailable" }));
  await page.getByRole("button", { name: "回復案を作成" }).click();
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("回復案を作成できませんでした");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("recovery planner unavailable");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("失敗した工程と実行環境の状態を確認");

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({}));
  await page.getByRole("button", { name: "回復案を作成" }).click();
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("出力契約の必須項目が不足");

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({ accept_work_recovery: "recovery accept failed" }));
  await page.getByRole("button", { name: "この回復案を採用" }).click();
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("回復案を採用できませんでした");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("recovery accept failed");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("回復案の状態と現在のワーク状態を確認");

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({}));
  await page.getByRole("button", { name: "この回復案を採用" }).click();
  await expect(page.locator('#scr-detail-recover textarea[name="prompt"]')).toBeVisible();

  await page.evaluate(() => window.__nagareDesktopSetCommandFailures({ apply_work_recovery: "recovery apply failed" }));
  await page.getByRole("button", { name: "回復して再開" }).click();
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("回復を適用できませんでした");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("recovery apply failed");
  await expect(page.locator("#scr-detail-recover.screen.active")).toContainText("追加指示と回復案の状態を確認");
});

test("desktop prototype UI filters work history by status and keyword", async ({ page }) => {
  const initial = baseState(true);
  initial.work_items = [
    { id: "work_done", title: "README更新", description: "READMEを更新", status_label: "完了", status_kind: "done", next_action: "なし", result_summary: "READMEの手順を更新しました · 評価 92 / 100 · 手順は再現可能です", updated_at: "now" },
    { id: "work_run", title: "API調査", description: "APIを調査", project: "外部プロジェクト", status_label: "処理中", status_kind: "running", next_action: "実行中", result_summary: "外部APIを確認中", updated_at: "now" },
    { id: "work_question", title: "FAQ作成", description: "FAQを作成", status_label: "要対応・質問", status_kind: "question", next_action: "質問に回答", result_summary: "対象読者の確認が必要です", updated_at: "now" },
  ];
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await expect(page.locator("#attn-dot")).toBeVisible();
  await expect(page.locator("#attn-dot")).toHaveAttribute("title", "要対応 1件");
  await expect(page.locator("#work-list [data-work-id]")).toHaveCount(3);
  await expect(page.locator("#work-status-filter")).toContainText("状態: すべて (3)");
  await expect(page.locator("#work-status-filter")).toContainText("要対応 (1)");
  await expect(page.locator("#work-status-filter")).toContainText("処理中 (1)");
  await expect(page.locator("#work-status-filter")).toContainText("完了 (1)");
  await expect(page.locator("#work-project-filter")).toContainText("プロジェクト: すべて (3)");
  await expect(page.locator("#work-project-filter")).toContainText("nagare-desktop-e2e (2)");
  await expect(page.locator("#work-project-filter")).toContainText("外部プロジェクト (1)");
  await expect(page.locator('[data-work-id="work_done"] .wr-sum')).toContainText("READMEの手順を更新しました");
  await expect(page.locator('[data-work-id="work_done"] .wr-sum')).not.toContainText("評価 92 / 100");
  await expect(page.locator('[data-work-id="work_done"] .wr-side')).toContainText("92 / 100");
  await expect(page.locator('[data-work-id="work_done"] [data-work-open]')).toHaveText("詳細");
  await page.locator("#work-project-filter").selectOption("nagare-desktop-e2e");
  await expect(page.locator("#filter-result")).toContainText("2件");
  await expect(page.locator('[data-work-id="work_run"]')).toBeHidden();
  await page.locator("#work-project-filter").selectOption("all");
  await page.locator("#work-status-filter").selectOption("attn");
  await expect(page.locator("#filter-result")).toContainText("1件");
  await expect(page.locator('[data-work-id="work_question"]')).toBeVisible();
  await expect(page.locator('[data-work-id="work_done"]')).toBeHidden();

  await page.locator("#work-search-filter").fill("FAQ");
  await expect(page.locator("#filter-result")).toContainText("1件");
  await page.locator("#work-search-filter").fill("README");
  await expect(page.locator("#filter-result")).toContainText("0件");
  await expect(page.locator("#work-empty")).toBeVisible();

  await page.locator("#work-status-filter").selectOption("all");
  await expect(page.locator("#filter-result")).toContainText("1件");
  await expect(page.locator('[data-work-id="work_done"]')).toBeVisible();
});

test("desktop prototype UI deletes a work item from the work list", async ({ page }) => {
  const initial = baseState(true);
  initial.work_items = [
    { id: "work_done", title: "README更新", description: "READMEを更新", status_label: "完了", status_kind: "done", next_action: "なし", result_summary: "READMEの手順を更新しました · 評価 92 / 100", updated_at: "now" },
    { id: "work_question", title: "FAQ作成", description: "FAQを作成", status_label: "要対応・質問", status_kind: "question", next_action: "質問に回答", result_summary: "対象読者の確認が必要です", updated_at: "now" },
  ];
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await expect(page.locator("#attn-dot")).toBeVisible();
  await expect(page.locator("#work-list [data-work-id]")).toHaveCount(2);
  await page.locator('[data-work-id="work_question"] [data-work-delete]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("ワークを削除しますか?");
  await expect(page.locator("#app-dynamic-modal")).toContainText("FAQ作成");
  await expect(page.locator("#app-dynamic-modal")).toContainText("実行記録、成果物記録、レビュー");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();

  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await expect(page.locator('[data-work-id="work_question"]')).toHaveCount(0);
  await expect(page.locator('[data-work-id="work_done"]')).toBeVisible();
  await expect(page.locator("#work-list [data-work-id]")).toHaveCount(1);
  await expect(page.locator("#attn-dot")).toBeHidden();
  await expect(page.locator("#app-notice")).toContainText("ワークを削除しました。");
  const deleteCall = await page.evaluate(() => window.__nagareDesktopCalls.find((call) => call.command === "delete_work"));
  expect(deleteCall.payload.request.id).toBe("work_question");
});

test("desktop prototype UI keeps work delete failures visible in context", async ({ page }) => {
  const initial = baseState(true);
  initial.work_items = [
    { id: "work_done", title: "README更新", description: "READMEを更新", status_label: "完了", status_kind: "done", next_action: "なし", result_summary: "READMEの手順を更新しました", updated_at: "now" },
  ];
  await installTauriMock(page, initial, {
    failCommands: { delete_work: "delete backend unavailable" },
  });
  await page.goto(desktopIndexUrl);

  await page.locator('[data-work-id="work_done"] [data-work-delete]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("ワークを削除できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("delete backend unavailable");
  await expect(page.locator("#app-dynamic-modal")).toContainText("ワークの状態を再読み込み");
  await expect(page.locator('[data-work-id="work_done"]')).toBeVisible();
});

test("desktop prototype UI hides work attention dot when no work needs action", async ({ page }) => {
  const initial = baseState(true);
  initial.work_items = [
    { id: "work_done", title: "README更新", description: "READMEを更新", status_label: "完了", status_kind: "done", next_action: "なし", result_summary: "READMEの手順を更新しました · 評価 92 / 100", updated_at: "now" },
    { id: "work_run", title: "API調査", description: "APIを調査", status_label: "処理中", status_kind: "running", next_action: "実行中", result_summary: "外部APIを確認中", updated_at: "now" },
  ];
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await expect(page.locator("#attn-dot")).toBeHidden();
  await expect(page.locator("#work-status-filter")).toContainText("要対応 (0)");
});

test("desktop prototype UI presents primary navigation before optional integrations", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await expect(page.locator(".nav-label").first()).toHaveText("主要フロー");
  const navTexts = await page.locator(".nav a").evaluateAll((items) => (
    items.map((item) => (item.textContent || "").replace(/\s+/g, " ").trim())
  ));
  expect(navTexts.slice(0, 6)).toEqual(["ワーク", "プロジェクト", "実行環境", "エージェント", "ナレッジ", "分析・改善"]);
  expect(navTexts.slice(6, 8)).toEqual(["スキル", "MCP接続"]);

  await page.locator("#nav-work").click();
  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await page.locator("#nav-project").click();
  await expect(page.locator("#scr-project-list.screen.active")).toBeVisible();
  await page.locator("#nav-runtime").click();
  await expect(page.locator("#scr-settings-runtime.screen.active")).toBeVisible();
  await expect(page.locator(".topbar .crumb")).toHaveText("実行環境");
  await page.locator("#nav-agent").click();
  await expect(page.locator("#scr-agent-list.screen.active")).toBeVisible();
  await page.locator("#nav-knowledge").click();
  await expect(page.locator("#scr-knowledge-list.screen.active")).toBeVisible();
  await page.locator("#nav-insights").click();
  await expect(page.locator("#scr-insights.screen.active")).toBeVisible();
});

test("desktop prototype UI shows insights attention dot only when improvements exist", async ({ page }) => {
  const initial = baseState(true);
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await expect(page.locator("#insights-dot")).toBeVisible();
  await expect(page.locator("#insights-dot")).toHaveAttribute("title", "未対応の改善提案 3件");

  const withoutProposals = baseState(true);
  withoutProposals.insights.proposal_count = 0;
  withoutProposals.insights.proposals = [];
  await page.evaluate((nextState) => window.__nagareDesktopSetMockState(nextState), withoutProposals);
  await page.evaluate(() => NagareApp.loadState());
  await expect(page.locator("#insights-dot")).toBeHidden();
});

test("desktop prototype UI dismisses improvement proposals from the pending list", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /分析・改善/ }).click();
  await page.locator('[data-insights-tab="improvements"]').click();
  await page.locator('[data-preview-proposal="proposal-rubric-readability"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "今回は見送る" }).click();

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const dismissCall = calls.find((call) => call.command === "dismiss_improvement");
  expect(dismissCall.payload.request.proposal_id).toBe("proposal-rubric-readability");

  const pendingImprovements = page.locator("#scr-insights [data-pending-improvements]");
  const appliedImprovements = page.locator("#scr-insights [data-applied-improvements]");
  await expect(pendingImprovements).not.toContainText("README ルーブリック改善");
  await expect(appliedImprovements).not.toContainText("README ルーブリック改善");
  await expect(page.locator("#insights-dot")).toHaveAttribute("title", "未対応の改善提案 2件");
});

test("desktop prototype UI keeps improvement dismiss failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    failCommands: { dismiss_improvement: "dismiss backend unavailable" },
  });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /分析・改善/ }).click();
  await page.locator('[data-insights-tab="improvements"]').click();
  await page.locator('[data-preview-proposal="proposal-rubric-readability"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "今回は見送る" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("改善提案を見送れませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("dismiss backend unavailable");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator("#scr-insights [data-pending-improvements]")).toContainText("README ルーブリック改善");
});

test("desktop prototype UI starts work with project defaults", async ({ page }) => {
  const initial = baseState(true);
  initial.project.workflow_mode = "finish_first";
  initial.project.approval_policy = "manual_on_review_concern";
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("既定ポリシーでREADMEを更新して");
  await page.getByRole("button", { name: "作業を開始" }).click();

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const createCall = calls.find((call) => call.command === "create_work");
  expect(createCall.payload.request.workflow_mode).toBe("finish_first");
  expect(createCall.payload.request.approval_policy).toBe("manual_on_review_concern");
});

test("desktop prototype UI automatically advances a new work to the next decision point", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    backgroundAdvanceDelay: 50,
  });
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("自動進行でREADMEを更新して");
  await page.getByRole("button", { name: "作業を開始" }).click();

  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await expect(page.locator("[data-advance-progress]")).toHaveCount(0);
  await expect(page.locator("#app-notice")).toContainText("ワークを開始しました。進行状況は一覧で更新されます。");
  await expect(page.locator('[data-work-id="work_1"]')).toContainText("処理中");
  await expect(page.locator('[data-work-id="work_1"] .wr-sum')).toContainText("README の内容を整理");
  await expect(page.getByRole("button", { name: "次の判断点まで進める" })).toHaveCount(0);

  const commands = await page.evaluate(() => window.__nagareDesktopCalls.map((call) => call.command));
  expect(commands).toEqual(expect.arrayContaining(["create_work", "start_work_background"]));
  expect(commands).not.toContain("advance_work");
});

test("desktop prototype UI shows a greeting work as an answer-only result", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("おはようございます");
  await page.getByRole("button", { name: "作業を開始" }).click();

  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await openWorkDetail(page);
  await expect(page.locator("#scr-detail-review.screen.active")).toBeVisible();
  await expect(page.locator("#scr-detail-review.screen.active .status-strip")).toContainText("おはようございます。今日進めたい作業があれば");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("依頼への回答");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("オーガナイザーまとめ");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("おはようございます");
  await expect(page.locator("#scr-detail-review.screen.active .answer-box")).toContainText("生成結果");
  await expect(page.locator("#scr-detail-review.screen.active .answer-box")).toContainText("そのまま依頼を書いてください");
  await expect(page.locator("#scr-detail-review.screen.active .result-overview")).toContainText("応答のみ");
  await expect(page.locator("#scr-detail-review.screen.active")).not.toContainText("README.md");
  const calls = await page.evaluate(() => window.__nagareDesktopCalls.map((call) => call.command));
  expect(calls).toContain("create_work");
});

test("desktop prototype UI shows auto-completed work without manual approval actions", async ({ page }) => {
  const initial = baseState(true);
  initial.project.approval_policy = "auto_complete_on_review_pass";
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await page.locator('#create-work-form textarea[name="description"]').fill("自動採用でREADMEを更新して");
  await page.getByRole("button", { name: "作業を開始" }).click();

  await openWorkDetail(page);
  await expect(page.locator("#scr-detail-done.screen.active")).toBeVisible();
  await expect(page.locator("#scr-detail-done.screen.active .status-strip")).toContainText("レビュー合格により自動完了しました");
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("確認");
  await expect(page.locator("#scr-detail-done.screen.active")).toContainText("レビュー合格で自動完了");
  await expect(page.getByRole("button", { name: "この結果を採用する" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "コメントを付けて差し戻す" })).toHaveCount(0);

  const createCall = await page.evaluate(() => window.__nagareDesktopCalls.find((call) => call.command === "create_work"));
  expect(createCall.payload.request.approval_policy).toBe("auto_complete_on_review_pass");
});

test("desktop prototype UI uses project default knowledge for new work", async ({ page }) => {
  const initial = baseState(true);
  initial.domains = [
    ...initial.domains,
    { id: "support-docs", name: "サポート文書", description: "問い合わせ回答を扱う", shared_knowledge: ["問い合わせ分類"], common_rubric: ["回答が具体的"], dispatch_hints: ["support"], artifact_type_count: 1 },
  ];
  initial.artifact_types = [
    ...initial.artifact_types,
    { id: "faq", domain_id: "support-docs", name: "FAQ", description: "よくある質問", knowledge: ["FAQテンプレート"], rubric: ["## 正確性 (100)"], dispatch_hints: ["faq"], rubric_count: 1, rubric_score_total: 100 },
  ];
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await page.getByRole("button", { name: "設定" }).click();
  await page.locator('#scr-project-settings [data-project-tab="knowledge"]').click();
  await page.locator('#scr-project-settings select[name="default_domain_id"]').selectOption("support-docs");
  await expect(page.locator('#scr-project-settings select[name="default_artifact_type_id"]')).not.toContainText("README");
  await page.locator('#scr-project-settings select[name="default_artifact_type_id"]').selectOption("faq");
  await page.locator("#scr-project-settings").getByRole("button", { name: "保存" }).click();

  await page.getByRole("link", { name: /ワーク/ }).click();
  await page.locator('#create-work-form textarea[name="description"]').fill("FAQを整備して");
  await page.getByRole("button", { name: "作業を開始" }).click();

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const saveCall = calls.find((call) => call.command === "save_project_settings");
  expect(saveCall.payload.request.default_domain_id).toBe("support-docs");
  expect(saveCall.payload.request.default_artifact_type_id).toBe("faq");
  const createCall = calls.find((call) => call.command === "create_work");
  expect(createCall.payload.request.domain_id).toBe("support-docs");
  expect(createCall.payload.request.artifact_type_id).toBe("faq");
});

test("desktop prototype UI renders backend state in management screens", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /スキル/ }).click();
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("markdown-tools");

  await page.getByRole("link", { name: /MCP接続/ }).click();
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("GitHub MCP");

  await page.getByRole("link", { name: /実行環境/ }).click();
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("Codex CLI");

  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("Writer");

  await page.getByRole("link", { name: /ナレッジ/ }).click();
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("プロダクト文書");

  await page.getByRole("link", { name: /分析・改善/ }).click();
  await expect(page.locator("#scr-insights.screen.active")).toContainText("Reviewer");
});

test("desktop prototype UI renders the component catalog from the app shell", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /カタログ/ }).click();
  await expect(page.locator("#scr-catalog.screen.active")).toBeVisible();
  await expect(page.locator("#scr-catalog.screen.active")).toContainText("UIカタログ");
  await expect(page.locator("#scr-catalog.screen.active")).toContainText("主要アクション");
  await expect(page.locator("#scr-catalog.screen.active")).toContainText("要対応");
  await expect(page.locator("#scr-catalog.screen.active")).toContainText("ワーク行");
});

test("desktop prototype UI filters skills by source and keyword", async ({ page }) => {
  const initial = baseState(true);
  initial.skill_packages = [
    ...initial.skill_packages,
    { id: "hachiware-labs/hachi-search", source_kind: "vercel", source: "hachiware-labs/hachi-search", provided_skill_sets: ["hachi-search"] },
    { id: "openai/code-review", source_kind: "openai", source: "openai/code-review", provided_skill_sets: ["code-review"] },
  ];
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /スキル/ }).click();
  await expect(page.locator("#skill-filter-result")).toContainText("3件");
  await expect(page.locator('[data-assign-skill]')).toHaveCount(0);
  await expect(page.locator("[data-skill-row]").filter({ has: page.locator('[data-delete-skill="markdown-tools"]') })).toContainText("まだエージェントに割り当てられていません");
  await page.locator("#skill-source-filter").selectOption("vercel");
  await expect(page.locator("#skill-filter-result")).toContainText("1件");
  await expect(page.locator('[data-delete-skill="hachiware-labs/hachi-search"]')).toBeVisible();
  await expect(page.locator('[data-delete-skill="markdown-tools"]')).toBeHidden();

  await page.locator("#skill-search-filter").fill("code");
  await expect(page.locator("#skill-filter-result")).toContainText("0件");
  await expect(page.locator("#skill-empty")).toBeVisible();

  await page.locator("#skill-source-filter").selectOption("all");
  await expect(page.locator("#skill-filter-result")).toContainText("1件");
  await expect(page.locator('[data-delete-skill="openai/code-review"]')).toBeVisible();
});

test("desktop prototype UI manages skills through backend commands", async ({ page }) => {
  await installTauriMock(page, baseState(true), { skillDeleteWarnings: ["外部ツール側は登録だけ外しました"] });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /スキル/ }).click();
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("ライブラリ登録 → エージェント割り当て");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("nagare-desktop-e2e へ自動反映");
  await page.getByRole("button", { name: "スキルを追加" }).click();
  await page.locator('#app-dynamic-modal select[name="source_kind"]').selectOption("vercel");
  await expect(page.locator('#app-dynamic-modal input[name="install_targets"][value="codex"]')).toBeChecked();
  await expect(page.locator('#app-dynamic-modal input[name="install_targets"][value="openclaw"]')).toBeVisible();
  await expect(page.locator('#app-dynamic-modal [data-skill-capabilities-field]')).toBeVisible();
  await expect(page.locator('#app-dynamic-modal textarea[name="required_capabilities"]')).toBeHidden();
  await page.locator('#app-dynamic-modal input[name="install_targets"][value="openclaw"]').check();
  await page.locator('#app-dynamic-modal input[name="source"]').fill("hachiware-labs/hachi-search");
  await page.locator('#app-dynamic-modal [data-skill-capabilities-field] summary').click();
  await expect(page.locator('#app-dynamic-modal textarea[name="required_capabilities"]')).toBeVisible();
  await page.locator('#app-dynamic-modal textarea[name="required_capabilities"]').fill("repo_read\nweb_search");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "追加" }).click();

  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("hachiware-labs/hachi-search");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("次の操作: エージェントへ割り当て");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("スキルを追加しました。 hachiware-labs/hachi-search");
  await expect(page.locator("[data-skill-row]").filter({ has: page.locator('[data-delete-skill="hachiware-labs/hachi-search"]') })).toContainText("まだエージェントに割り当てられていません");
  await page.locator('[data-followup-assign-skill="hachiware-labs/hachi-search"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("反映先: 流 nagare-desktop-e2e");
  await expect(page.locator("#app-dynamic-modal")).toContainText("選択したエージェントに保存すると");
  await page.locator('#app-dynamic-modal input[name="agent_ids"][value="worker"]').check();
  await page.locator('#app-dynamic-modal input[name="agent_ids"][value="reviewer"]').check();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "選択したエージェントへ割り当て" }).click();
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("割り当て済み: Writer、Reviewer");

  await page.locator('[data-delete-skill="hachiware-labs/hachi-search"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("割り当て解除");
  await expect(page.locator("#app-dynamic-modal")).toContainText("Writer");
  await expect(page.locator("#app-dynamic-modal")).toContainText("Reviewer");
  await expect(page.locator("#app-dynamic-modal")).toContainText("過去のワーク履歴は残ります");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator('[data-delete-skill="hachiware-labs/hachi-search"]')).toHaveCount(0);
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("削除結果: hachiware-labs/hachi-search");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("スキル本体も削除しました");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("削除したスキル: hachiware-labs/hachi-search");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("割り当て解除: Writer、Reviewer");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("確認が必要なこと");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("外部ツール側は登録だけ外しました");

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const addCall = calls.find((call) => call.command === "add_skill");
  expect(addCall.payload.request.package_id).toBe("hachiware-labs/hachi-search");
  expect(addCall.payload.request.source_kind).toBe("vercel");
  expect(addCall.payload.request.install).toBe(true);
  expect(addCall.payload.request.install_targets).toEqual(["codex", "openclaw"]);
  const agentCalls = calls.filter((call) => call.command === "save_agent");
  const workerCall = agentCalls.find((call) => call.payload.request.id === "worker");
  const reviewerCall = agentCalls.find((call) => call.payload.request.id === "reviewer");
  expect(workerCall.payload.request.skill_set_ids).toContain("hachiware-labs/hachi-search");
  expect(reviewerCall.payload.request.skill_set_ids).toContain("hachiware-labs/hachi-search");
  const deleteCall = calls.find((call) => call.command === "delete_skill_package_command");
  expect(deleteCall.payload.request.remove_installed_body).toBe(true);
});

test("desktop prototype UI adds multiple preset skills from fixed provider lists", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /スキル/ }).click();
  await page.getByRole("button", { name: "スキルを追加" }).click();
  await expect(page.locator('#app-dynamic-modal [data-skill-targets-field]')).toBeHidden();
  await expect(page.locator('#app-dynamic-modal [data-skill-capabilities-field]')).toBeHidden();
  await expect(page.locator('#app-dynamic-modal input[name="preset_skill"][value="openai/code-review"]')).toBeChecked();
  await page.locator('#app-dynamic-modal input[name="preset_skill"][value="openai/prompt-engineering"]').check();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "追加" }).click();

  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("openai/code-review");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("openai/prompt-engineering");
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("2件のスキルを追加しました。 openai/prompt-engineering");
  await expect(page.locator('[data-followup-assign-skill="openai/prompt-engineering"]')).toBeVisible();
  const calls = await page.evaluate(() => window.__nagareDesktopCalls.filter((call) => call.command === "add_skill"));
  expect(calls.map((call) => call.payload.request.package_id)).toEqual([
    "openai/code-review",
    "openai/prompt-engineering",
  ]);
  expect(calls.every((call) => call.payload.request.source_kind === "openai")).toBe(true);
  expect(calls.every((call) => call.payload.request.install === false)).toBe(true);
  expect(calls.every((call) => call.payload.request.install_targets.length === 0)).toBe(true);
});

test("desktop prototype UI selects a Clawhub skill from searchable candidates", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /スキル/ }).click();
  await page.getByRole("button", { name: "スキルを追加" }).click();
  await page.locator('#app-dynamic-modal select[name="source_kind"]').selectOption("clawhub");
  await expect(page.locator('#app-dynamic-modal input[name="install_targets"][value="openclaw"]')).toBeChecked();
  await expect(page.locator('#app-dynamic-modal input[name="install_targets"][value="codex"]')).toHaveCount(0);
  await expect(page.locator('#app-dynamic-modal [data-skill-capabilities-field]')).toBeHidden();
  await expect(page.locator('#app-dynamic-modal [data-skill-catalog-search]')).toBeVisible();
  await page.locator('#app-dynamic-modal [data-skill-catalog-search]').fill("browser");
  await expect(page.locator("#app-dynamic-modal")).toContainText("browser-automation");
  await expect(page.locator("#app-dynamic-modal")).not.toContainText("markdown-tools");
  await page.locator('#app-dynamic-modal input[name="preset_skill"][value="browser-automation"]').check();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "追加" }).click();

  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("browser-automation");
  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const addCall = calls.find((call) => call.command === "add_skill");
  expect(addCall.payload.request.package_id).toBe("browser-automation");
  expect(addCall.payload.request.source_kind).toBe("clawhub");
  expect(addCall.payload.request.source).toBe("browser-automation");
  expect(addCall.payload.request.install).toBe(true);
  expect(addCall.payload.request.install_targets).toEqual(["openclaw"]);
});

test("desktop prototype UI manages MCP connections through backend commands", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /MCP接続/ }).click();
  await page.getByRole("button", { name: "MCPを追加" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("管理ID（任意）");
  await expect(page.locator('#app-dynamic-modal input[name="id"]')).toHaveAttribute("placeholder", "空欄なら表示名から自動生成");
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("Filesystem MCP");
  await expect(page.locator('#app-dynamic-modal input[name="id"]')).toHaveValue("filesystem");
  await page.locator('#app-dynamic-modal select[name="tool_kind"]').selectOption("codex_cli");
  await page.locator('#app-dynamic-modal input[name="command"]').fill("npx");
  await page.locator('#app-dynamic-modal textarea[name="args"]').fill("-y\n@modelcontextprotocol/server-filesystem");
  await page.locator('#app-dynamic-modal textarea[name="test_args"]').fill("--version");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存" }).click();

  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("Filesystem MCP");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("次の操作: 接続テスト");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("接続テストに成功するまで、エージェントへ割り当てできません");
  await page.locator('[data-edit-mcp="filesystem"]').click();
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("Filesystem MCP Updated");
  await page.locator('#app-dynamic-modal textarea[name="args"]').fill("-y\n@modelcontextprotocol/server-filesystem\nC:/nagare-desktop-e2e");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存" }).click();

  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("Filesystem MCP Updated");
  await page.locator('[data-followup-test-mcp="filesystem"]').click();
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("検証済み");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("接続テスト結果: Filesystem MCP Updated");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("詳細: mock ok");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("次の操作: エージェントへ割り当て");
  await expect(page.locator('[data-result-assign-mcp="filesystem"]')).toBeVisible();
  await page.locator('[data-result-assign-mcp="filesystem"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("反映先: 流 nagare-desktop-e2e");
  await page.locator('#app-dynamic-modal input[name="agent_ids"][value="worker"]').check();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "選択したエージェントへ割り当て" }).click();
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("割り当て: Writer");

  await page.locator('[data-delete-mcp="filesystem"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("割り当て解除");
  await expect(page.locator("#app-dynamic-modal")).toContainText("Writer");
  await expect(page.locator("#app-dynamic-modal")).toContainText("過去のワーク履歴と実行記録は残ります");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator('[data-delete-mcp="filesystem"]')).toHaveCount(0);
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("削除結果: Filesystem MCP Updated");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("MCP接続をライブラリから削除しました");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("割り当て解除: Writer");

  const commands = await page.evaluate(() => window.__nagareDesktopCalls.map((call) => call.command));
  expect(commands).toEqual(expect.arrayContaining([
    "save_mcp_connection",
    "test_mcp_connection_command",
    "delete_mcp_connection_command",
  ]));
  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const editCall = calls.filter((call) => call.command === "save_mcp_connection").at(-1);
  expect(editCall.payload.request.display_name).toBe("Filesystem MCP Updated");
  const agentCall = calls.find((call) => call.command === "save_agent" && call.payload.request.id === "worker");
  expect(agentCall.payload.request.mcp_connection_ids).toContain("filesystem");
});

test("desktop prototype UI keeps failed MCP test details visible", async ({ page }) => {
  await installTauriMock(page, baseState(true), { mcpTestSuccess: false, mcpTestDetail: "auth failed" });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /MCP接続/ }).click();
  await page.locator('[data-test-mcp="github"]').click();

  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("接続テスト結果: GitHub MCP");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("接続テストに失敗しました");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("詳細: auth failed");
  await expect(page.locator('[data-result-edit-mcp="github"]')).toBeVisible();
  await expect(page.locator('[data-result-retest-mcp="github"]')).toBeVisible();
  await expect(page.locator('[data-result-assign-mcp="github"]')).toHaveCount(0);
  await expect(page.locator('[data-assign-mcp]')).toHaveCount(0);
  await page.locator('[data-result-edit-mcp="github"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("MCP接続を編集");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await page.locator('[data-result-retest-mcp="github"]').click();
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("詳細: auth failed");
});

test("desktop prototype UI keeps operation command failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    failCommands: {
      add_skill: "skill registry unavailable",
      test_mcp_connection_command: "mcp runner crashed",
      refresh_runtime_status: "runtime detector crashed",
    },
  });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /スキル/ }).click();
  await page.getByRole("button", { name: "スキルを追加" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "追加" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("スキルを追加できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("skill registry unavailable");
  await expect(page.locator("#app-dynamic-modal")).toContainText("追加元、参照先、インストール先を確認");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();

  await page.getByRole("link", { name: /MCP接続/ }).click();
  await page.locator('[data-test-mcp="github"]').click();
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("接続テスト結果: GitHub MCP");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("接続テストを実行できませんでした");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("mcp runner crashed");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("MCP設定、コマンド、権限、実行環境を確認");
  await expect(page.locator('[data-result-edit-mcp="github"]')).toBeVisible();
  await expect(page.locator('[data-result-retest-mcp="github"]')).toBeVisible();

  await page.getByRole("link", { name: /実行環境/ }).click();
  await page.locator('[data-refresh-runtime="codex"]').click();
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("再確認結果: Codex CLI");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("再確認を実行できませんでした");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("runtime detector crashed");
});

test("desktop prototype UI keeps skill and MCP save or delete failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    failCommands: {
      delete_skill_package_command: "skill uninstall failed",
      save_mcp_connection: "mcp config store locked",
      delete_mcp_connection_command: "mcp is still referenced",
    },
  });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /スキル/ }).click();
  await page.locator('[data-delete-skill="markdown-tools"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("スキルを削除できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("skill uninstall failed");
  await expect(page.locator("#app-dynamic-modal")).toContainText("割り当て状況とスキル本体の保存先を確認");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator('[data-delete-skill="markdown-tools"]')).toBeVisible();

  await page.getByRole("link", { name: /MCP接続/ }).click();
  await page.getByRole("button", { name: "MCPを追加" }).click();
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("Filesystem MCP");
  await expect(page.locator('#app-dynamic-modal input[name="id"]')).toHaveValue("filesystem");
  await page.locator('#app-dynamic-modal input[name="command"]').fill("npx");
  await page.locator('#app-dynamic-modal textarea[name="args"]').fill("-y\n@modelcontextprotocol/server-filesystem");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("MCP接続を保存できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("mcp config store locked");
  await expect(page.locator("#app-dynamic-modal")).toContainText("表示名、コマンド、引数、対象ランタイムを確認");
  await expect(page.locator('#app-dynamic-modal input[name="display_name"]')).toHaveValue("Filesystem MCP");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator('[data-edit-mcp="filesystem"]')).toHaveCount(0);

  await page.locator('[data-delete-mcp="github"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("MCP接続を削除できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("mcp is still referenced");
  await expect(page.locator("#app-dynamic-modal")).toContainText("割り当て状況とMCP接続の登録状態を確認");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator('[data-delete-mcp="github"]')).toBeVisible();
});

test("desktop prototype UI keeps MCP validation failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /MCP接続/ }).click();
  await page.getByRole("button", { name: "MCPを追加" }).click();
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("Filesystem MCP");
  await expect(page.locator('#app-dynamic-modal input[name="id"]')).toHaveValue("filesystem");
  await page.locator('#app-dynamic-modal input[name="command"]').fill("npx");
  await page.locator('#app-dynamic-modal textarea[name="args"]').fill("-y\n@modelcontextprotocol/server-filesystem");
  await page.locator('#app-dynamic-modal textarea[name="env"]').fill("BROKEN_ENV_LINE");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("MCP接続を保存できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("環境変数 `BROKEN_ENV_LINE` は KEY=VALUE 形式で入力してください。");
  await expect(page.locator("#app-dynamic-modal")).toContainText("表示名、コマンド、引数、対象ランタイムを確認");
  await expect(page.locator('#app-dynamic-modal textarea[name="env"]')).toHaveValue("BROKEN_ENV_LINE");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator('[data-edit-mcp="filesystem"]')).toHaveCount(0);
});

test("desktop prototype UI keeps capability assignment failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    failCommands: { save_agent: "agent assignment store locked" },
  });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /スキル/ }).click();
  await page.getByRole("button", { name: "スキルを追加" }).click();
  await page.locator('#app-dynamic-modal select[name="source_kind"]').selectOption("vercel");
  await page.locator('#app-dynamic-modal input[name="source"]').fill("hachiware-labs/hachi-search");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "追加" }).click();
  await expect(page.locator("#scr-settings-skills.screen.active")).toContainText("次の操作: エージェントへ割り当て");

  await page.locator('[data-followup-assign-skill="hachiware-labs/hachi-search"]').click();
  await page.locator('#app-dynamic-modal input[name="agent_ids"][value="worker"]').check();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "選択したエージェントへ割り当て" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("エージェントへ割り当てできませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("agent assignment store locked");
  await expect(page.locator("#app-dynamic-modal")).toContainText("対象エージェント、スキル/MCPの状態、実行環境の対応状況を確認");
  await expect(page.locator('#app-dynamic-modal input[name="agent_ids"][value="worker"]')).toBeChecked();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator("[data-skill-row]").filter({ has: page.locator('[data-delete-skill="hachiware-labs/hachi-search"]') })).toContainText("まだエージェントに割り当てられていません");
});

test("desktop prototype UI filters MCP connections by status, runtime, and keyword", async ({ page }) => {
  const initial = baseState(true);
  initial.mcp_connections = [
    ...initial.mcp_connections,
    { id: "postgres", name: "Postgres MCP", tool_kind: "codex_cli", runtime_label: "Codex CLI", agent_assignable: true, command: "npx", args: ["postgres-mcp"], test_status: "failed", test_detail: "auth failed" },
    { id: "local-search", name: "Local Search MCP", tool_kind: "openclaw", runtime_label: "OpenClaw", agent_assignable: false, command: "npx", args: ["local-search"], test_status: "untested" },
  ];
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /MCP接続/ }).click();
  await expect(page.locator("#mcp-filter-result")).toContainText("4件");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("OpenClawではエージェント個別のMCP割り当てはできません");
  await expect(page.locator("#scr-settings-mcp.screen.active")).toContainText("接続テストに失敗しています: auth failed");
  await expect(page.locator('[data-assign-mcp]')).toHaveCount(0);
  await page.locator("#mcp-status-filter").selectOption("failed");
  await expect(page.locator("#mcp-filter-result")).toContainText("1件");
  await expect(page.locator('[data-edit-mcp="postgres"]')).toBeVisible();
  await expect(page.locator('[data-edit-mcp="github"]')).toBeHidden();

  await page.locator("#mcp-tool-filter").selectOption("openclaw");
  await expect(page.locator("#mcp-filter-result")).toContainText("0件");
  await expect(page.locator("#mcp-empty")).toBeVisible();

  await page.locator("#mcp-status-filter").selectOption("untested");
  await expect(page.locator("#mcp-filter-result")).toContainText("1件");
  await expect(page.locator('[data-edit-mcp="local-search"]')).toBeVisible();

  await page.locator("#mcp-search-filter").fill("postgres");
  await expect(page.locator("#mcp-filter-result")).toContainText("0件");
});

test("desktop prototype UI creates a new project from the project list", async ({ page }) => {
  const initial = baseState(true);
  initial.project.work_count = 3;
  initial.project.domain_count = 2;
  initial.project.artifact_type_count = 4;
  initial.project.status_counts = [
    { label: "要対応・確認", kind: "review", count: 1 },
    { label: "処理中", kind: "running", count: 1 },
    { label: "完了", kind: "done", count: 1 },
  ];
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("C:/nagare-desktop-e2e");
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("ワーク 3件 · エージェント 2件 · ドメイン 2件 · 成果物種別 4件");
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("ワーク状況: 要対応 1件 · 処理中 1件 · 完了 1件");
  await page.getByRole("button", { name: "新規プロジェクト" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("作業対象のフォルダを選びます");
  await expect(page.locator("#app-dynamic-modal")).toContainText("使用する実行環境: Codex CLI");
  await page.locator('#app-dynamic-modal input[name="icon"]').fill("灯");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "選択…" }).click();
  await expect(page.locator('#app-dynamic-modal input[name="root"]')).toHaveValue("C:/nagare-desktop-e2e");
  await page.locator('#app-dynamic-modal input[name="root"]').fill("C:/another-nagare-project");
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "作成" }).click();

  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await expect(page.locator("#proj-badge")).toContainText("灯 another-nagare-project");
  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("another-nagare-project");
  await page.locator("[data-project-open-work]").click();
  await expect(page.locator("#scr-home-active.screen.active")).toBeVisible();
  await expect(page.locator("#scr-home-active.screen.active")).toContainText("新規ワーク依頼");

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const initCall = calls.filter((call) => call.command === "initialize_project_with_runtime").at(-1);
  expect(initCall.payload.request.root).toBe("C:/another-nagare-project");
  expect(initCall.payload.request.display_name).toBe("another-nagare-project");
  expect(initCall.payload.request.icon).toBe("灯");
  expect(initCall.payload.request.runtime_id).toBe("codex");
});

test("desktop prototype UI keeps project creation failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    failCommands: { initialize_project_with_runtime: "project init failed" },
  });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await page.getByRole("button", { name: "新規プロジェクト" }).click();
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("失敗プロジェクト");
  await page.locator('#app-dynamic-modal input[name="root"]').fill("C:/failed-project");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "作成" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("プロジェクトを作成できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("project init failed");
  await expect(page.locator('#app-dynamic-modal input[name="display_name"]')).toHaveValue("失敗プロジェクト");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("nagare-desktop-e2e");
  await expect(page.locator("#proj-badge")).toContainText("流 nagare-desktop-e2e");
});

test("desktop prototype UI refreshes runtime status per runtime row", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /実行環境/ }).click();
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("Codex CLI");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("利用エージェント: Writer、Reviewer");
  await page.locator('[data-refresh-runtime="openclaw"]').click();

  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("OpenClaw");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("利用可能");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("再確認結果: OpenClaw");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("この実行環境は利用できます");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("詳細: mock refreshed");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("モデル: 実行環境側で設定");
  await expect(page.locator('[data-edit-runtime-model="openclaw"]')).toHaveCount(0);

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const refreshCall = calls.find((call) => call.command === "refresh_runtime_status");
  expect(refreshCall.payload.request.runtime_id).toBe("openclaw");
});

test("desktop prototype UI keeps unavailable runtime refresh details visible", async ({ page }) => {
  await installTauriMock(page, baseState(true), { runtimeRefreshAvailable: false, runtimeRefreshDetail: "program not found" });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /実行環境/ }).click();
  await page.locator('[data-refresh-runtime="openclaw"]').click();

  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("再確認結果: OpenClaw");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("この実行環境はまだ利用できません");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("詳細: program not found");
});

test("desktop prototype UI applies runtime model settings to target agents", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /実行環境/ }).click();
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("モデル設定: 2件のエージェントは実行環境既定を使用");
  await page.locator('[data-edit-runtime-model="codex"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("Codex CLI のモデル設定");
  await expect(page.locator("#app-dynamic-modal")).toContainText("Writer、Reviewer");
  await page.locator('#app-dynamic-modal input[name="model"]').fill("gpt-5.1-codex");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "対象エージェントへ適用" }).click();

  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("モデル設定: gpt-5.1-codex（対象 2件）");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("モデル設定結果: Codex CLI");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("gpt-5.1-codex を対象エージェントへ適用しました");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("対象: Writer、Reviewer");
  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await expect(page.locator("#scr-agent-list.screen.active")).not.toContainText("gpt-5.1-codex");
  await page.locator('[data-edit-agent="worker"]').click();
  await page.locator('#scr-agent-settings [data-agent-tab="runtime"]').click();
  await expect(page.locator('#scr-agent-settings input[name="model"]')).toHaveValue("gpt-5.1-codex");
  await page.locator("#scr-agent-settings").getByRole("button", { name: "エージェント一覧へ" }).click();

  await page.getByRole("link", { name: /実行環境/ }).click();
  await page.locator('[data-edit-runtime-model="codex"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "実行環境既定に戻す" }).click();

  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("モデル設定: 2件のエージェントは実行環境既定を使用");
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("実行環境既定 を対象エージェントへ適用しました");
  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await expect(page.locator("#scr-agent-list.screen.active")).not.toContainText("gpt-5.1-codex");
  await expect(page.locator("#scr-agent-list.screen.active")).not.toContainText("実行環境既定");
  await page.locator('[data-edit-agent="worker"]').click();
  await page.locator('#scr-agent-settings [data-agent-tab="runtime"]').click();
  await expect(page.locator('#scr-agent-settings input[name="model"]')).toHaveValue("");

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const saveCalls = calls.filter((call) => call.command === "save_runtime_model_defaults");
  const saveCall = saveCalls[0];
  expect(saveCall.payload.request.runtime_id).toBe("codex");
  expect(saveCall.payload.request.model).toBe("gpt-5.1-codex");
  const resetCall = saveCalls[1];
  expect(resetCall.payload.request.runtime_id).toBe("codex");
  expect(resetCall.payload.request.model).toBe("");
});

test("desktop prototype UI keeps runtime model save failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    failCommands: { save_runtime_model_defaults: "runtime model store locked" },
  });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /実行環境/ }).click();
  await page.locator('[data-edit-runtime-model="codex"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("Codex CLI のモデル設定");
  await page.locator('#app-dynamic-modal input[name="model"]').fill("gpt-5.1-codex");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "対象エージェントへ適用" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("モデル設定を保存できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("runtime model store locked");
  await expect(page.locator("#app-dynamic-modal")).toContainText("モデル名、Provider、Base URLを確認");
  await expect(page.locator('#app-dynamic-modal input[name="model"]')).toHaveValue("gpt-5.1-codex");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator("#scr-settings-runtime.screen.active")).toContainText("モデル設定: 2件のエージェントは実行環境既定を使用");
  await expect(page.locator("#scr-settings-runtime.screen.active")).not.toContainText("gpt-5.1-codex（対象 2件）");
});

test("desktop prototype UI filters runtimes by availability and keyword", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /実行環境/ }).click();
  await expect(page.locator("#runtime-filter-result")).toContainText("2件");
  await page.locator("#runtime-status-filter").selectOption("available");
  await expect(page.locator("#runtime-filter-result")).toContainText("1件");
  await expect(page.locator('[data-refresh-runtime="codex"]')).toBeVisible();
  await expect(page.locator('[data-refresh-runtime="openclaw"]')).toBeHidden();

  await page.locator("#runtime-status-filter").selectOption("missing");
  await expect(page.locator("#runtime-filter-result")).toContainText("1件");
  await expect(page.locator('[data-refresh-runtime="openclaw"]')).toBeVisible();

  await page.locator("#runtime-search-filter").fill("codex");
  await expect(page.locator("#runtime-filter-result")).toContainText("0件");
  await expect(page.locator("#runtime-empty")).toBeVisible();
});

test("desktop prototype UI explains unavailable runtimes used by agents", async ({ page }) => {
  const state = baseState(true);
  state.agents.push({
    id: "openclaw-worker",
    name: "OpenClaw Worker",
    role: "worker",
    description: "OpenClawで作業",
    prompt: "作業する",
    tool_kind: "openclaw",
    runtime: "openclaw",
    model: "gpt-5-codex",
    specialties: ["search"],
    domain_ids: [],
    artifact_type_ids: [],
    skill_set_ids: [],
    mcp_connection_ids: [],
  });
  await installTauriMock(page, state);
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /実行環境/ }).click();
  const openclawRow = page.locator("[data-runtime-row]").filter({ has: page.locator('[data-refresh-runtime="openclaw"]') });
  await expect(openclawRow).toContainText("利用エージェント: OpenClaw Worker");
  await expect(openclawRow).toContainText("OpenClaw Worker は接続確認までワークに割り当てられません。");
  await expect(openclawRow).toContainText("OpenClaw が見つかりません。インストール後に検出を確認してください。");
});

test("desktop prototype UI renders insights and routes improvement previews", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /分析・改善/ }).click();
  await expect(page.locator("#scr-insights.screen.active")).toContainText("82 / 100");
  await expect(page.locator("#scr-insights.screen.active")).toContainText("形式の準拠");
  await page.locator('[data-insights-tab="improvements"]').click();

  await expect(page.locator("#scr-insights.screen.active")).toContainText("Reviewer のプロンプト改善");
  await expect(page.locator("#scr-insights.screen.active")).toContainText("適用済みの改善");
  await expect(page.locator("#scr-insights.screen.active")).toContainText("Writer のプロンプト改善");
  await expect(page.locator("#scr-insights.screen.active")).toContainText("差し戻し率 22% → 12%");
  await page.getByRole("button", { name: "今すぐ見直す" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("形式基準を先に確認");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "エージェントを開く" }).click();
  await expect(page.locator("#scr-agent-list.screen.active")).toBeVisible();
  await expect(page.locator("#app-dynamic-modal")).toContainText("Reviewer");
  await expect(page.locator("#app-dynamic-modal")).toContainText("AI支援");
  await expect(page.locator("#app-dynamic-modal")).toContainText("生成支援 — AIで下書き");
  await expect(page.locator("#app-dynamic-modal")).toContainText("改善提案 — Nagareが実績から検出");
  await expect(page.locator('#app-dynamic-modal textarea[name="prompt"]')).toHaveValue("レビューを担当する");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "プロンプト欄に挿入" }).click();
  await expect(page.locator('#app-dynamic-modal textarea[name="prompt"]')).toHaveValue("形式基準を先に確認してからレビューする");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存" }).click();

  await page.getByRole("link", { name: /分析・改善/ }).click();
  await page.locator('[data-insights-tab="improvements"]').click();
  await page.locator('[data-preview-proposal="proposal-rubric-readability"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "ナレッジを開く" }).click();
  await expect(page.locator("#scr-knowledge-rubric.screen.active")).toBeVisible();
  await expect(page.locator("#scr-knowledge-rubric")).toContainText("README");
  await expect(page.locator("#scr-knowledge-rubric")).toContainText("AI支援");
  await expect(page.locator("#scr-knowledge-rubric")).toContainText("改善提案 — Nagareが実績から検出");
  await expect(page.locator('#scr-knowledge-rubric textarea[name="rubric"]')).toHaveValue(/手順の再現性/);
  await page.locator("#scr-knowledge-rubric").getByRole("button", { name: "ルーブリック欄に挿入" }).click();
  await expect(page.locator('#scr-knowledge-rubric textarea[name="rubric"]')).toHaveValue(/手順が5分以内に追える/);
  await page.locator('#scr-knowledge-rubric textarea[name="rubric"]').fill("## 手順の再現性 (100)\n手順が5分以内に追える");
  await expect(page.locator("#scr-knowledge-rubric")).toContainText("形式OK");
  await page.locator("#scr-knowledge-rubric").getByRole("button", { name: "保存" }).click();

  await page.getByRole("link", { name: /分析・改善/ }).click();
  await page.locator('[data-insights-tab="improvements"]').click();
  await page.locator('[data-preview-proposal="operation-approval-policy"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("確認ポリシー: 懸念がある時だけ確認する");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "設定で確認" }).click();
  await expect(page.locator("#scr-project-settings.screen.active")).toBeVisible();
  await expect(page.locator('#scr-project-settings [data-project-pane="policy"]')).toHaveClass(/active/);
  await expect(page.locator("#scr-project-settings")).toContainText("改善提案 — Nagareが実績から検出");
  await page.locator("#scr-project-settings").getByRole("button", { name: "確認ポリシーに反映" }).click();
  await expect(page.locator('#scr-project-settings select[name="approval_policy"]')).toHaveValue("manual_on_review_concern");
  await page.locator("#scr-project-settings").getByRole("button", { name: "保存" }).click();

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const agentCall = calls.find((call) => call.command === "save_agent");
  const artifactCall = calls.find((call) => call.command === "save_artifact_type");
  const projectSettingsCall = calls.find((call) => call.command === "save_project_settings");
  expect(agentCall.payload.request.id).toBe("reviewer");
  expect(agentCall.payload.request.prompt).toBe("形式基準を先に確認してからレビューする");
  expect(agentCall.payload.request.improvement_proposal_id).toBe("proposal-prompt-reviewer");
  expect(artifactCall.payload.request.id).toBe("readme");
  expect(artifactCall.payload.request.rubric).toContain("手順が5分以内に追える");
  expect(artifactCall.payload.request.improvement_proposal_id).toBe("proposal-rubric-readability");
  expect(projectSettingsCall.payload.request.approval_policy).toBe("manual_on_review_concern");
  expect(projectSettingsCall.payload.request.improvement_proposal_id).toBe("operation-approval-policy");

  await page.getByRole("link", { name: /分析・改善/ }).click();
  await page.locator('[data-insights-tab="improvements"]').click();
  const pendingImprovements = page.locator("#scr-insights [data-pending-improvements]");
  const appliedImprovements = page.locator("#scr-insights [data-applied-improvements]");
  await expect(pendingImprovements).toContainText("未対応の改善提案はありません");
  await expect(pendingImprovements).not.toContainText("確認ポリシー緩和の提案");
  await expect(pendingImprovements).not.toContainText("README ルーブリック改善");
  await expect(pendingImprovements).not.toContainText("Reviewer のプロンプト改善");
  await expect(appliedImprovements).toContainText("Reviewer のプロンプト改善");
  await expect(appliedImprovements).toContainText("README ルーブリック改善");
  await expect(appliedImprovements).toContainText("確認ポリシー緩和の提案");
  await expect(appliedImprovements).toContainText("測定中: 形式の準拠 60%（2件）");
  await expect(appliedImprovements).toContainText("測定中: 平均 82点 / 懸念3件");
  await expect(appliedImprovements).toContainText("効果: 最近の同種懸念なし");
  await expect(page.locator("#insights-dot")).toBeHidden();

  const reviewerApplied = appliedImprovements.locator(".list-item").filter({ hasText: "Reviewer のプロンプト改善" });
  await reviewerApplied.getByRole("button", { name: "対象を確認" }).click();
  await expect(page.locator("#scr-agent-settings.screen.active")).toBeVisible();
  await expect(page.locator("#scr-agent-settings")).toContainText("Reviewer");
  await expect(page.locator('#scr-agent-settings [data-agent-pane="prompt"]')).toHaveClass(/active/);
  await expect(page.locator('#scr-agent-settings textarea[name="prompt"]')).toHaveValue("形式基準を先に確認してからレビューする");
  await page.locator("#scr-agent-settings").getByRole("button", { name: "エージェント一覧へ" }).click();

  await page.getByRole("link", { name: /分析・改善/ }).click();
  await page.locator('[data-insights-tab="improvements"]').click();
  const readmeApplied = page.locator("#scr-insights [data-applied-improvements] .list-item").filter({ hasText: "README ルーブリック改善" });
  await readmeApplied.getByRole("button", { name: "対象を確認" }).click();
  await expect(page.locator("#scr-knowledge-rubric.screen.active")).toBeVisible();
  await expect(page.locator("#scr-knowledge-rubric")).toContainText("README");
  await expect(page.locator('#scr-knowledge-rubric textarea[name="rubric"]')).toHaveValue(/手順が5分以内に追える/);
  await expect(page.locator("#scr-knowledge-rubric")).toContainText("このルーブリックへの改善提案 — 現在なし");

  await page.getByRole("link", { name: /分析・改善/ }).click();
  await page.locator('[data-insights-tab="improvements"]').click();
  const policyApplied = page.locator("#scr-insights [data-applied-improvements] .list-item").filter({ hasText: "確認ポリシー緩和の提案" });
  await policyApplied.getByRole("button", { name: "対象を確認" }).click();
  await expect(page.locator("#scr-project-settings.screen.active")).toBeVisible();
  await expect(page.locator('#scr-project-settings [data-project-pane="policy"]')).toHaveClass(/active/);
  await expect(page.locator('#scr-project-settings select[name="approval_policy"]')).toHaveValue("manual_on_review_concern");
  await expect(page.locator("#scr-project-settings")).not.toContainText("改善提案 — Nagareが実績から検出");
});

test("desktop prototype UI manages agents and capability assignments", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await page.getByRole("button", { name: "新規エージェント" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("まず名前、ロール、実行環境だけを決めます");
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("UI Worker");
  await page.locator('#app-dynamic-modal select[name="role"]').selectOption("worker");
  await page.locator('#app-dynamic-modal select[name="tool_kind"]').selectOption("codex_cli");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "作成して設定へ" }).click();
  const agentSettings = page.locator("#scr-agent-settings.screen.active");
  await expect(agentSettings).toBeVisible();
  await expect(page.locator('#scr-agent-settings input[name="id"]')).toHaveValue("ui-worker");
  await expect(page.locator('#scr-agent-settings input[name="id"]')).toHaveAttribute("placeholder", "空欄なら表示名から自動生成");
  await expect(agentSettings).toContainText("管理ID（任意）");
  await expect(page.locator('#scr-agent-settings input[name="display_name"]')).toHaveValue("UI Worker");
  await page.locator('#scr-agent-settings [data-choose-agent-avatar]').click();
  await expect(page.locator('#scr-agent-settings input[name="avatar"]')).toHaveValue("C:/nagare-desktop-e2e/avatars/ui-worker.svg");
  await page.locator('#scr-agent-settings select[name="role"]').selectOption("worker");
  await page.locator('#scr-agent-settings textarea[name="description"]').fill("UI実装を担当する");
  await page.locator('#scr-agent-settings textarea[name="specialties"]').fill("ui\nimplementation");
  await page.locator('#scr-agent-settings [data-agent-tab="runtime"]').click();
  await page.locator('#scr-agent-settings select[name="tool_kind"]').selectOption("codex_cli");
  await page.locator('#scr-agent-settings input[name="model"]').fill("gpt-5-codex");
  await page.locator('#scr-agent-settings [data-agent-tab="scope"]').click();
  await page.locator('#scr-agent-settings input[name="domain_ids"][value="product-docs"]').check();
  await page.locator('#scr-agent-settings input[name="artifact_type_ids"][value="readme"]').check();
  await page.locator('#scr-agent-settings [data-agent-tab="capabilities"]').click();
  await page.locator('#scr-agent-settings input[name="skill_set_ids"][value="markdown-tools"]').check();
  await page.locator('#scr-agent-settings input[name="mcp_connection_ids"][value="github"]').check();
  await page.locator('#scr-agent-settings [data-agent-tab="prompt"]').click();
  await expect(agentSettings).toContainText("AI支援");
  await expect(agentSettings).toContainText("このプロンプトへの改善提案 — 現在なし");
  await page.locator("#scr-agent-settings").getByRole("button", { name: "AIで下書き" }).click();
  await expect(page.locator('#scr-agent-settings textarea[name="prompt"]')).toHaveValue(/UI Worker/);
  await page.locator("#scr-agent-settings").getByRole("button", { name: "保存" }).click();

  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("UI Worker");
  await expect(page.locator('#scr-agent-list [data-edit-agent="ui-worker"]').locator("..").locator("img.agent-avatar")).toHaveAttribute("src", /ui-worker\.svg/);
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("保存結果: UI Worker");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("ワーカーとして保存しました");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("実行環境: Codex CLI / gpt-5-codex");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("担当範囲: プロダクト文書 / README");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("能力: markdown-tools、GitHub MCP");
  await expect(page.locator('[data-edit-agent="ui-worker"]').locator("..")).not.toContainText("markdown-tools");
  await expect(page.locator('[data-edit-agent="ui-worker"]').locator("..")).not.toContainText("github");

  await page.locator('[data-edit-agent="ui-worker"]').click();
  await expect(agentSettings).toBeVisible();
  await page.locator('#scr-agent-settings [data-agent-tab="capabilities"]').click();
  await expect(page.locator('#scr-agent-settings input[name="skill_set_ids"][value="markdown-tools"]')).toBeChecked();
  await expect(page.locator('#scr-agent-settings input[name="mcp_connection_ids"][value="github"]')).toBeChecked();
  await page.locator("#scr-agent-settings").getByRole("button", { name: "削除" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("UI Worker · ワーカー");
  await expect(page.locator("#app-dynamic-modal")).toContainText("nagare-desktop-e2e 参加候補");
  await expect(page.locator("#app-dynamic-modal")).toContainText("markdown-tools、GitHub MCP");
  await expect(page.locator("#app-dynamic-modal")).toContainText("過去のワーク履歴と実行記録は残ります");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator('[data-edit-agent="ui-worker"]')).toHaveCount(0);
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("削除結果: UI Worker");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("このエージェントを今後の割り当て候補から外しました");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("能力: markdown-tools、GitHub MCP");

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const draftCall = calls.find((call) => call.command === "generate_agent_prompt_draft");
  expect(draftCall.payload.request.display_name).toBe("UI Worker");
  expect(draftCall.payload.request.domain_ids).toEqual(["product-docs"]);
  expect(draftCall.payload.request.artifact_type_ids).toEqual(["readme"]);
  const createCall = calls.find((call) => call.command === "save_agent" && call.payload.request.id === "ui-worker");
  expect(createCall.payload.request.description).toBe("");
  const saveCall = calls.filter((call) => call.command === "save_agent" && call.payload.request.id === "ui-worker").at(-1);
  expect(saveCall.payload.request.id).toBe("ui-worker");
  expect(saveCall.payload.request.avatar).toBe("C:/nagare-desktop-e2e/avatars/ui-worker.svg");
  expect(saveCall.payload.request.prompt).toContain("UI Worker");
  expect(saveCall.payload.request.skill_set_ids).toEqual(["markdown-tools"]);
  expect(saveCall.payload.request.mcp_connection_ids).toEqual(["github"]);
  expect(calls.map((call) => call.command)).toContain("delete_agent_command");
});

test("desktop prototype UI keeps agent operation failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    failCommands: {
      generate_agent_prompt_draft: "prompt generator offline",
      save_agent: "agent store locked",
      delete_agent_command: "agent is still referenced",
    },
  });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await page.getByRole("button", { name: "新規エージェント" }).click();
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("UI Worker");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "作成して設定へ" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("エージェントを作成できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("agent store locked");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();

  await page.locator('[data-edit-agent="worker"]').click();
  await page.locator('#scr-agent-settings textarea[name="description"]').fill("UI実装を担当する");
  await page.locator('#scr-agent-settings [data-agent-tab="prompt"]').click();
  await page.locator("#scr-agent-settings").getByRole("button", { name: "AIで下書き" }).click();
  await expect(page.locator("#scr-agent-settings.screen.active")).toContainText("プロンプト下書きを生成できませんでした");
  await expect(page.locator("#scr-agent-settings.screen.active")).toContainText("prompt generator offline");
  await expect(page.locator("#scr-agent-settings.screen.active")).toContainText("エージェントの説明、担当範囲、ナレッジの状態を確認");

  await page.locator("#scr-agent-settings").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#scr-agent-settings.screen.active")).toContainText("エージェントを保存できませんでした");
  await expect(page.locator("#scr-agent-settings.screen.active")).toContainText("agent store locked");
  await expect(page.locator("#scr-agent-settings.screen.active")).toContainText("表示名、役割、実行環境、モデル、担当範囲を確認");

  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await page.locator('[data-edit-agent="worker"]').click();
  await page.locator("#scr-agent-settings").getByRole("button", { name: "削除" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("エージェントを削除できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("agent is still referenced");
  await expect(page.locator("#app-dynamic-modal")).toContainText("プロジェクト設定の整理役や割り当て状況を確認");
});

test("desktop prototype UI filters agents by role and keyword", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("標準の内蔵オーガナイザー");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("依頼の整理と担当割り当てを行う標準整理役");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("既定ワーカー");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("既定レビュアー");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("利用実績 4件");
  await expect(page.locator("#scr-agent-list.screen.active")).toContainText("担当範囲 全ドメイン");
  await page.locator("#agent-project-filter").selectOption("nagare-desktop-e2e");
  await expect(page.locator("#agent-filter-result")).toContainText("3件");
  await page.locator("#agent-role-filter").selectOption("organizer");
  await expect(page.locator("#agent-filter-result")).toContainText("1件");
  await expect(page.locator('[data-open-organizer-settings]')).toBeVisible();
  await page.locator('[data-open-organizer-settings]').click();
  await expect(page.locator("#scr-project-settings.screen.active")).toBeVisible();
  await expect(page.locator('#scr-project-settings [data-project-pane="organizer"]')).toBeVisible();
  await expect(page.locator("#scr-project-settings")).toContainText("標準の内蔵オーガナイザー");
  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await page.locator("#agent-project-filter").selectOption("nagare-desktop-e2e");
  await page.locator("#agent-role-filter").selectOption("reviewer");
  await expect(page.locator("#agent-filter-result")).toContainText("1件");
  await expect(page.locator('[data-edit-agent="reviewer"]')).toBeVisible();
  await expect(page.locator('[data-edit-agent="worker"]')).toBeHidden();

  await page.locator("#agent-search-filter").fill("文書");
  await expect(page.locator("#agent-filter-result")).toContainText("0件");
  await expect(page.locator("#agent-empty")).toBeVisible();

  await page.locator("#agent-role-filter").selectOption("all");
  await expect(page.locator("#agent-filter-result")).toContainText("1件");
  await expect(page.locator('[data-edit-agent="worker"]')).toBeVisible();
});

test("desktop prototype UI updates agent model and MCP choices when runtime changes", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /^エージェント$/ }).click();
  await page.getByRole("button", { name: "新規エージェント" }).click();
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("OpenClaw Worker");
  await page.locator('#app-dynamic-modal select[name="tool_kind"]').selectOption("openclaw");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "作成して設定へ" }).click();
  await expect(page.locator("#scr-agent-settings.screen.active")).toBeVisible();
  await page.locator('#scr-agent-settings [data-agent-tab="runtime"]').click();

  await expect(page.locator('#scr-agent-settings select[name="model_provider"]')).toHaveCount(0);
  await expect(page.locator('#scr-agent-settings [data-agent-pane="runtime"]')).toContainText("実行環境側で設定します");
  await expect(page.locator('#scr-agent-settings [data-agent-pane="runtime"] input[disabled]')).toHaveValue("実行環境側の設定を使用");
  await page.locator('#scr-agent-settings [data-agent-tab="capabilities"]').click();
  await expect(page.locator('#scr-agent-settings input[name="mcp_connection_ids"][value="github"]')).toHaveCount(0);
  await expect(page.locator('#scr-agent-settings input[name="mcp_connection_ids"][value="openclaw-search"]')).toHaveCount(0);
  await expect(page.locator("#scr-agent-settings")).toContainText("OpenClawではMCPをエージェント個別に割り当てできません");

  await page.locator('#scr-agent-settings [data-agent-tab="runtime"]').click();
  await page.locator('#scr-agent-settings [data-agent-tab="capabilities"]').click();
  await page.locator("#scr-agent-settings").getByRole("button", { name: "保存" }).click();

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const saveCall = calls.find((call) => call.command === "save_agent" && call.payload.request.id === "openclaw-worker");
  expect(saveCall.payload.request.tool_kind).toBe("openclaw");
  expect(saveCall.payload.request.model_provider).toBe("");
  expect(saveCall.payload.request.model).toBe("");
  expect(saveCall.payload.request.model_base_url).toBe("");
  expect(saveCall.payload.request.mcp_connection_ids).toEqual([]);
});

test("desktop prototype UI updates project settings and can delete the project", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await page.getByRole("button", { name: "設定" }).click();
  const projectSettings = page.locator("#scr-project-settings.screen.active");
  await expect(projectSettings).toBeVisible();
  await page.locator('#scr-project-settings input[name="icon"]').fill("灯");
  await page.locator('#scr-project-settings input[name="display_name"]').fill("灯火館 UI");
  await page.locator('#scr-project-settings [data-project-tab="organizer"]').click();
  await expect(projectSettings).toContainText("標準の内蔵オーガナイザー");
  await page.locator('#scr-project-settings [data-project-tab="agents"]').click();
  await expect(projectSettings).toContainText("参加エージェント");
  await expect(projectSettings).toContainText("Writer");
  await expect(projectSettings).toContainText("既定ワーカー");
  await expect(projectSettings).toContainText("Reviewer");
  await page.locator("#scr-project-settings").getByRole("button", { name: "エージェントを追加" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("エージェントを作成");
  await expect(page.locator("#app-dynamic-modal")).toContainText("まず名前、ロール、実行環境だけを決めます");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(projectSettings).toBeVisible();
  await expect(page.locator('#scr-project-settings [data-project-pane="agents"]')).toHaveClass(/active/);
  await expect(page.locator('#scr-project-settings input[name="display_name"]')).toHaveValue("灯火館 UI");
  await page.locator("#scr-project-settings").getByRole("button", { name: "エージェントを追加" }).click();
  await page.locator('#app-dynamic-modal input[name="display_name"]').fill("Project Helper");
  await page.locator('#app-dynamic-modal select[name="role"]').selectOption("worker");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "作成して設定へ" }).click();
  await expect(page.locator('#app-dynamic-modal input[name="id"]')).toHaveValue("project-helper");
  await page.locator('#app-dynamic-modal textarea[name="description"]').fill("プロジェクト設定から追加したエージェント");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存" }).click();
  await expect(projectSettings).toBeVisible();
  await expect(page.locator('#scr-project-settings [data-project-pane="agents"]')).toHaveClass(/active/);
  await expect(projectSettings).toContainText("Project Helper");
  await page.locator('#scr-project-settings select[name="work_agent_id"]').selectOption("project-helper");
  await expect(page.locator('#scr-project-settings select[name="work_agent_id"]')).toHaveValue("project-helper");
  await expect(page.locator('#scr-project-settings select[name="review_agent_id"]')).toHaveValue("reviewer");
  await page.locator('#scr-project-settings [data-project-edit-agent="project-helper"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("Project Helper");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(projectSettings).toBeVisible();
  await expect(page.locator('#scr-project-settings [data-project-pane="agents"]')).toHaveClass(/active/);
  await expect(page.locator('#scr-project-settings input[name="display_name"]')).toHaveValue("灯火館 UI");
  await page.locator('#scr-project-settings [data-project-tab="knowledge"]').click();
  await expect(projectSettings).toContainText("ドメインと成果物種別");
  await expect(projectSettings).toContainText("プロダクト文書");
  await expect(projectSettings).toContainText("README");
  await page.locator("#scr-project-settings").getByRole("button", { name: "ナレッジを開く" }).click();
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("プロジェクト設定から開いています");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("未保存のプロジェクト設定は保持されています");
  await page.getByRole("button", { name: "プロジェクト設定へ戻る" }).click();
  await expect(projectSettings).toBeVisible();
  await expect(page.locator('#scr-project-settings [data-project-pane="knowledge"]')).toHaveClass(/active/);
  await expect(page.locator('#scr-project-settings input[name="display_name"]')).toHaveValue("灯火館 UI");
  await page.locator('#scr-project-settings [data-project-open-domain="product-docs"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("プロダクト文書");
  await page.locator('#app-dynamic-modal [data-domain-tab="artifacts"]').click();
  await page.locator('#app-dynamic-modal [data-edit-artifact="readme"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("README");
  await page.locator('#app-dynamic-modal textarea[name="description"]').fill("セットアップ文書を更新");
  await page.locator('#app-dynamic-modal textarea[name="rubric"]').fill("## 手順の再現性 (100)\n手順が再現できる");
  await expect(page.locator("#app-dynamic-modal")).toContainText("形式OK");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("プロダクト文書");
  await expect(page.locator('#app-dynamic-modal [data-domain-pane="artifacts"]')).toHaveClass(/active/);
  await expect(page.locator("#app-dynamic-modal")).toContainText("セットアップ文書を更新");
  await page.locator("#app-dynamic-modal .modal-foot").getByRole("button", { name: "閉じる" }).click();
  await expect(projectSettings).toBeVisible();
  await expect(page.locator('#scr-project-settings [data-project-pane="knowledge"]')).toHaveClass(/active/);
  await expect(page.locator('#scr-project-settings input[name="display_name"]')).toHaveValue("灯火館 UI");
  await page.locator('#scr-project-settings [data-project-open-domain="product-docs"]').click();
  await page.locator('#app-dynamic-modal textarea[name="shared_knowledge"]').fill("用語集\nUIルール");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存" }).click();
  await expect(projectSettings).toBeVisible();
  await expect(page.locator('#scr-project-settings [data-project-pane="knowledge"]')).toHaveClass(/active/);
  await expect(page.locator('#scr-project-settings input[name="display_name"]')).toHaveValue("灯火館 UI");
  await page.locator('#scr-project-settings [data-project-tab="policy"]').click();
  await page.locator('#scr-project-settings select[name="workflow_mode"]').selectOption("finish_first");
  await page.locator('#scr-project-settings select[name="approval_policy"]').selectOption("manual_on_review_concern");
  await page.locator("#scr-project-settings").getByRole("button", { name: "保存" }).click();

  await expect(page.locator("#scr-project-list.screen.active")).toContainText("灯火館 UI");
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("保存結果: 灯火館 UI");
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("プロジェクト設定を更新しました。次のワークから既定値として反映されます。");
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("整理役: 標準の内蔵オーガナイザー");
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("進め方: 最後まで進めてから確認");
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("確認: 懸念がある時だけ確認");
  await expect(page.locator("#proj-badge")).toContainText("灯 灯火館 UI");

  await page.getByRole("button", { name: "設定" }).click();
  await page.locator("#scr-project-settings").getByRole("button", { name: "プロジェクトを削除" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("灯 灯火館 UI");
  await expect(page.locator("#app-dynamic-modal")).toContainText("C:/nagare-desktop-e2e");
  await expect(page.locator("#app-dynamic-modal")).toContainText("ワーク履歴 0件");
  await expect(page.locator("#app-dynamic-modal")).toContainText("エージェント 2件、ドメイン 1件、成果物種別 1件");
  await expect(page.locator("#app-dynamic-modal")).toContainText("対象フォルダ内の通常ファイルは削除しません");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();

  await expect(page.locator("#scr-home-empty.screen.active")).toBeVisible();
  await expect(page.getByRole("button", { name: "セットアップを開始" })).toBeVisible();
  await expect.poll(() => page.evaluate(() => localStorage.getItem("nagare.root"))).toBeNull();
  await page.evaluate(() => NagareApp.loadState());
  await expect(page.locator("#scr-home-empty.screen.active")).toBeVisible();
  await expect(page.getByRole("button", { name: "セットアップを開始" })).toBeVisible();

  await page.getByRole("button", { name: "セットアップを開始" }).click();
  await page.getByRole("button", { name: "選択…" }).click();
  await page.locator('#app-dynamic-modal input[id="setup-name"]').fill("再作成プロジェクト");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "次へ" }).click();
  await page.locator('#app-dynamic-modal [data-runtime-option="codex"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存して接続確認" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "はじめる" }).click();
  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await expect(page.locator("#scr-project-list.screen.active")).toContainText("再作成プロジェクト");
  await expect(page.locator("#scr-project-list.screen.active")).not.toContainText("保存結果: 灯火館 UI");
  await expect(page.locator("#scr-project-list.screen.active")).not.toContainText("確認: 懸念がある時だけ確認");

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const saveCall = calls.find((call) => call.command === "save_project_settings");
  expect(saveCall.payload.request.display_name).toBe("灯火館 UI");
  expect(saveCall.payload.request.work_agent_id).toBe("project-helper");
  expect(saveCall.payload.request.review_agent_id).toBe("reviewer");
  expect(saveCall.payload.request.workflow_mode).toBe("finish_first");
  expect(saveCall.payload.request.approval_policy).toBe("manual_on_review_concern");
  expect(calls.map((call) => call.command)).toContain("delete_project");
});

test("desktop prototype UI keeps project delete failures visible in context", async ({ page }) => {
  await installTauriMock(page, baseState(true), {
    failCommands: { delete_project: "delete project failed" },
  });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /プロジェクト/ }).click();
  await page.getByRole("button", { name: "設定" }).click();
  await expect(page.locator("#scr-project-settings.screen.active")).toBeVisible();
  await page.locator("#scr-project-settings").getByRole("button", { name: "プロジェクトを削除" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();

  await expect(page.locator("#app-dynamic-modal")).toContainText("プロジェクトを削除できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("delete project failed");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator("#scr-project-settings.screen.active")).toBeVisible();
  await expect(page.locator("#proj-badge")).toContainText("流 nagare-desktop-e2e");
  await expect.poll(() => page.evaluate(() => localStorage.getItem("nagare.root"))).toBe("C:/nagare-desktop-e2e");
});

test("desktop prototype UI filters knowledge by domain and keyword", async ({ page }) => {
  const initial = baseState(true);
  initial.domains = [
    ...initial.domains,
    { id: "support-docs", name: "サポート文書", description: "問い合わせ回答を扱う", shared_knowledge: ["問い合わせ分類"], common_rubric: ["回答が具体的"], dispatch_hints: ["support"], artifact_type_count: 1 },
  ];
  initial.artifact_types = [
    ...initial.artifact_types,
    { id: "faq", domain_id: "support-docs", name: "FAQ", description: "よくある質問", knowledge: ["FAQテンプレート"], rubric: ["## 正確性 (100)"], dispatch_hints: ["faq"], rubric_count: 1, rubric_score_total: 100 },
  ];
  await installTauriMock(page, initial);
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /ナレッジ/ }).click();
  await expect(page.locator("#knowledge-filter-result")).toContainText("2件");
  await expect(page.locator('[data-edit-domain="support-docs"]')).toBeVisible();
  await expect(page.locator("[data-knowledge-row]").filter({ has: page.locator('[data-edit-domain="product-docs"]') })).toContainText("改善提案 1件");
  await expect(page.locator("[data-knowledge-row]").filter({ has: page.locator('[data-edit-domain="product-docs"]') })).toContainText("利用プロジェクト: nagare-desktop-e2e（自動候補）");
  await expect(page.locator("[data-knowledge-row]").filter({ has: page.locator('[data-edit-domain="support-docs"]') })).toContainText("利用プロジェクト: 未割り当て");
  await page.locator('[data-edit-domain="product-docs"]').click();
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("品質記録と改善提案");
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("README ルーブリック改善");
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("読みやすさの懸念が1件");
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("最近のレビュー");
  await page.locator('[data-domain-proposal-preview="proposal-rubric-readability"]').click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("README ルーブリック改善");
  await expect(page.locator("#app-dynamic-modal")).toContainText("手順が5分以内に追える");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "ナレッジ一覧へ" }).click();

  await page.locator("#knowledge-search-filter").fill("FAQテンプレート");
  await expect(page.locator("#knowledge-filter-result")).toContainText("1件");
  await expect(page.locator('[data-edit-domain="support-docs"]')).toBeVisible();
  await expect(page.locator('[data-edit-domain="product-docs"]')).toBeHidden();

  await page.locator("#knowledge-search-filter").fill("存在しない");
  await expect(page.locator("#knowledge-filter-result")).toContainText("0件");
  await expect(page.locator("#knowledge-empty")).toBeVisible();
});

test("desktop prototype UI manages domains and artifact types", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /ナレッジ/ }).click();
  await page.getByRole("button", { name: "ドメインを追加" }).click();
  const domainPage = page.locator("#scr-knowledge-domain.screen.active");
  await expect(domainPage).toBeVisible();
  await expect(domainPage).toContainText("自動注入");
  await page.locator('#scr-knowledge-domain input[name="id"]').fill("support-docs");
  await page.locator('#scr-knowledge-domain input[name="display_name"]').fill("サポート文書");
  await page.locator('#scr-knowledge-domain textarea[name="description"]').fill("問い合わせ回答やFAQを扱う");
  await page.locator('#scr-knowledge-domain textarea[name="shared_knowledge"]').fill("問い合わせ分類\n表記ルール");
  await page.locator("#scr-knowledge-domain summary").click();
  await page.locator('#scr-knowledge-domain textarea[name="common_rubric"]').fill("回答が具体的である");
  await page.locator('#scr-knowledge-domain textarea[name="dispatch_hints"]').fill("support\nfaq");
  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "保存" }).click();

  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("サポート文書");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("利用プロジェクト: 未割り当て");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("次の操作: 成果物種別を追加");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("ワークで使うには、README、FAQ、リリースノートのような成果物種別とルーブリックを設定します");

  await page.locator('[data-followup-add-artifact="support-docs"]').click();
  const rubricPage = page.locator("#scr-knowledge-rubric.screen.active");
  await expect(rubricPage).toBeVisible();
  await page.locator('#scr-knowledge-rubric input[name="id"]').fill("faq");
  await page.locator('#scr-knowledge-rubric input[name="display_name"]').fill("FAQ");
  await expect(page.locator('#scr-knowledge-rubric select[name="domain_id"]')).toHaveValue("support-docs");
  await page.locator('#scr-knowledge-rubric textarea[name="description"]').fill("よくある質問の回答");
  await page.locator('#scr-knowledge-rubric textarea[name="knowledge"]').fill("FAQテンプレート");
  await expect(rubricPage).toContainText("ルーブリックが未入力です");
  await expect(rubricPage).toContainText("AI支援");
  await expect(rubricPage).toContainText("このルーブリックへの改善提案 — 現在なし");
  await page.locator('#scr-knowledge-rubric textarea[name="rubric"]').fill("## 正確性 (60)\n具体的である\n\n## 正確性 (40)\n根拠がある");
  await expect(rubricPage).toContainText("項目名が重複しています: 正確性");
  await page.locator("#scr-knowledge-rubric").getByRole("button", { name: "保存" }).click();
  await expect(rubricPage).toContainText("ルーブリックを保存できませんでした");
  await expect(rubricPage).toContainText("項目名が重複しています: 正確性");
  await page.locator("#scr-knowledge-rubric").getByRole("button", { name: "AIで下書き" }).click();
  await expect(page.locator('#scr-knowledge-rubric textarea[name="rubric"]')).toHaveValue(/ドメイン知識の反映/);
  await expect(rubricPage).toContainText("形式OK");
  await expect(rubricPage).toContainText("4項目 / 合計100点");
  await page.locator('#scr-knowledge-rubric textarea[name="dispatch_hints"]').fill("faq");
  await page.locator("#scr-knowledge-rubric").getByRole("button", { name: "保存" }).click();

  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("保存結果: FAQ");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("成果物種別の知識とルーブリックを更新しました。作成とレビューの両方に渡されます。");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("ドメイン: サポート文書 · 成果物知識 1件 · 4項目 / 合計100点");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("FAQ");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("サポート文書");
  await expect(page.locator('[data-followup-add-artifact="support-docs"]')).toHaveCount(0);

  await page.locator('[data-edit-domain="support-docs"]').click();
  await expect(domainPage).toBeVisible();
  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "削除" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("このドメインはまだ削除できません");
  await expect(page.locator("#app-dynamic-modal")).toContainText("FAQ");
  await expect(page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" })).toBeDisabled();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();

  await expect(domainPage).toBeVisible();
  await page.locator('#scr-knowledge-domain [data-domain-tab="artifacts"]').click();
  await page.locator('#scr-knowledge-domain [data-edit-artifact="faq"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("FAQ を サポート文書 から削除します");
  await expect(page.locator("#app-dynamic-modal")).toContainText("成果物知識");
  await expect(page.locator("#app-dynamic-modal")).toContainText("1件");
  await expect(page.locator("#app-dynamic-modal")).toContainText("4項目 / 100点");
  await expect(page.locator("#app-dynamic-modal")).toContainText("過去のワーク履歴と実行記録は残ります");
  await expect(page.locator("#app-dynamic-modal")).toContainText("新しいワークへ自動注入されません");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(domainPage).toBeVisible();
  await expect(page.locator('#scr-knowledge-domain [data-domain-pane="artifacts"]')).toHaveClass(/active/);
  await expect(page.locator('#scr-knowledge-domain [data-edit-artifact="faq"]')).toHaveCount(0);
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("削除結果: FAQ");
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("成果物種別を削除しました。新しいワークにはこの知識とルーブリックは自動注入されません。");
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("ドメイン: サポート文書 · 成果物知識 1件 · ルーブリック 4項目 / 100点");
  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "ナレッジ一覧へ" }).click();

  await page.locator('[data-edit-domain="support-docs"]').click();
  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "削除" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator('[data-edit-domain="support-docs"]')).toHaveCount(0);
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("削除結果: サポート文書");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("ドメインをナレッジから削除しました。新しいワークには自動注入されません。");
  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("共通知識 2件 · 成果物種別 0件 · 過去のワーク履歴は保持");

  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const domainCall = calls.find((call) => call.command === "save_domain");
  const rubricDraftCall = calls.find((call) => call.command === "generate_rubric_draft");
  const artifactCall = calls.find((call) => call.command === "save_artifact_type");
  expect(domainCall.payload.request.id).toBe("support-docs");
  expect(domainCall.payload.request.shared_knowledge).toContain("問い合わせ分類");
  expect(rubricDraftCall.payload.request.domain_id).toBe("support-docs");
  expect(rubricDraftCall.payload.request.display_name).toBe("FAQ");
  expect(rubricDraftCall.payload.request.knowledge).toEqual(["FAQテンプレート"]);
  expect(artifactCall.payload.request.id).toBe("faq");
  expect(artifactCall.payload.request.domain_id).toBe("support-docs");
  expect(artifactCall.payload.request.rubric).toContain("## ドメイン知識の反映 (20)");
  expect(calls.map((call) => call.command)).toEqual(expect.arrayContaining([
    "delete_artifact_type_command",
    "delete_domain_command",
  ]));
});

test("desktop prototype UI auto-generates domain and artifact ids from display names", async ({ page }) => {
  await installTauriMock(page, baseState(true));
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /ナレッジ/ }).click();
  await page.getByRole("button", { name: "ドメインを追加" }).click();
  const domainPage = page.locator("#scr-knowledge-domain.screen.active");
  await expect(domainPage).toBeVisible();
  await expect(page.locator('#scr-knowledge-domain input[name="id"]')).toHaveAttribute("placeholder", "空欄なら表示名から自動生成");
  await expect(domainPage).toContainText("管理ID（任意）");
  await page.locator('#scr-knowledge-domain input[name="display_name"]').fill("support docs");
  await expect(page.locator('#scr-knowledge-domain input[name="id"]')).toHaveValue("support-docs");
  await page.locator('#scr-knowledge-domain textarea[name="description"]').fill("Support documentation");
  await page.locator('#scr-knowledge-domain textarea[name="shared_knowledge"]').fill("support taxonomy");
  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "保存" }).click();

  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("support docs");
  await page.locator('[data-followup-add-artifact="support-docs"]').click();
  const rubricPage = page.locator("#scr-knowledge-rubric.screen.active");
  await expect(rubricPage).toBeVisible();
  await expect(page.locator('#scr-knowledge-rubric input[name="id"]')).toHaveAttribute("placeholder", "空欄なら表示名から自動生成");
  await expect(rubricPage).toContainText("管理ID（任意）");
  await page.locator('#scr-knowledge-rubric input[name="display_name"]').fill("faq");
  await expect(page.locator('#scr-knowledge-rubric input[name="id"]')).toHaveValue("faq");
  await expect(page.locator('#scr-knowledge-rubric select[name="domain_id"]')).toHaveValue("support-docs");
  await page.locator('#scr-knowledge-rubric textarea[name="description"]').fill("Frequently asked questions");
  await page.locator('#scr-knowledge-rubric textarea[name="knowledge"]').fill("FAQ template");
  await page.locator('#scr-knowledge-rubric textarea[name="rubric"]').fill("## 正確性 (100)\n具体的である");
  await page.locator("#scr-knowledge-rubric").getByRole("button", { name: "保存" }).click();

  await expect(page.locator("#scr-knowledge-list.screen.active")).toContainText("保存結果: faq");
  const calls = await page.evaluate(() => window.__nagareDesktopCalls);
  const domainCall = calls.find((call) => call.command === "save_domain");
  const artifactCall = calls.find((call) => call.command === "save_artifact_type");
  expect(domainCall.payload.request.id).toBe("support-docs");
  expect(domainCall.payload.request.display_name).toBe("support docs");
  expect(artifactCall.payload.request.id).toBe("faq");
  expect(artifactCall.payload.request.domain_id).toBe("support-docs");
  expect(artifactCall.payload.request.display_name).toBe("faq");
});

test("desktop prototype UI keeps knowledge operation failures visible in context", async ({ page }) => {
  const initial = baseState(true);
  initial.domains = [
    ...initial.domains,
    { id: "empty-domain", name: "空ドメイン", description: "削除失敗確認用", shared_knowledge: [], common_rubric: [], dispatch_hints: [], artifact_type_count: 0 },
  ];
  await installTauriMock(page, initial, {
    failCommands: {
      save_domain: "domain store locked",
      generate_rubric_draft: "rubric generator offline",
      save_artifact_type: "artifact store locked",
      delete_artifact_type_command: "artifact is still referenced",
      delete_domain_command: "domain is still referenced",
    },
  });
  await page.goto(desktopIndexUrl);

  await page.getByRole("link", { name: /ナレッジ/ }).click();
  await page.locator('[data-edit-domain="product-docs"]').click();
  await page.locator('#scr-knowledge-domain input[name="display_name"]').fill("プロダクト文書 更新");
  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("ドメインを保存できませんでした");
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("domain store locked");
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("表示名、共通知識、割り当てヒントを確認");

  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "ナレッジ一覧へ" }).click();
  await page.locator('[data-edit-domain="empty-domain"]').click();
  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "削除" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("ドメインを削除できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("domain is still referenced");
  await expect(page.locator("#app-dynamic-modal")).toContainText("成果物種別の有無とナレッジの参照状態を確認");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "閉じる" }).click();
  await expect(page.locator("#scr-knowledge-domain.screen.active")).toContainText("空ドメイン");

  await page.locator("#scr-knowledge-domain").getByRole("button", { name: "ナレッジ一覧へ" }).click();
  await page.locator('[data-edit-domain="product-docs"]').click();
  await page.locator('#scr-knowledge-domain [data-domain-tab="artifacts"]').click();
  await page.locator('#scr-knowledge-domain [data-edit-artifact="readme"]').click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "AIで下書き" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("ルーブリック下書きを生成できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("rubric generator offline");
  await expect(page.locator("#app-dynamic-modal")).toContainText("成果物の説明、成果物知識、ドメイン共通知識を確認");

  await page.locator('#app-dynamic-modal textarea[name="rubric"]').fill("## 手順の再現性 (100)\n手順が再現できる");
  await expect(page.locator("#app-dynamic-modal")).toContainText("形式OK");
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "保存" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("成果物種別を保存できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("artifact store locked");
  await expect(page.locator("#app-dynamic-modal")).toContainText("表示名、ドメイン、説明、ルーブリック形式を確認");

  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await page.locator("#app-dynamic-modal").getByRole("button", { name: "削除" }).click();
  await expect(page.locator("#app-dynamic-modal")).toContainText("成果物種別を削除できませんでした");
  await expect(page.locator("#app-dynamic-modal")).toContainText("artifact is still referenced");
  await expect(page.locator("#app-dynamic-modal")).toContainText("ドメイン内の成果物種別と参照状態を確認");
});
