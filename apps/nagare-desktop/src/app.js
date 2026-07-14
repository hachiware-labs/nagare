const NagareApp = (() => {
  const invoke = window.__TAURI__?.core?.invoke;
  let state = null;
  let currentRoot = localStorage.getItem("nagare.root") || "";
  let pendingWork = null;
  let homeWorkProgress = null;
  const backgroundWorkIds = new Set();
  let workRefreshTimer = null;
  let activeDetail = null;
  let activeInsightsTab = "analysis";
  let pendingSkillFollowup = null;
  let pendingSkillDeleteResult = null;
  let pendingMcpFollowupId = null;
  let pendingMcpResult = null;
  let pendingRuntimeResult = null;
  let pendingDomainFollowupId = null;
  let pendingKnowledgeResult = null;
  let pendingAgentResult = null;
  let pendingProjectResult = null;
  let agentReturnContext = null;
  let knowledgeReturnContext = null;
  let domainReturnContext = null;
  let artifactReturnContext = null;
  let projectSettingsDraft = null;

  function clearProjectScopedUiState() {
    pendingWork = null;
    homeWorkProgress = null;
    backgroundWorkIds.clear();
    stopWorkRefresh();
    activeDetail = null;
    pendingSkillFollowup = null;
    pendingSkillDeleteResult = null;
    pendingMcpFollowupId = null;
    pendingMcpResult = null;
    pendingRuntimeResult = null;
    pendingDomainFollowupId = null;
    pendingKnowledgeResult = null;
    pendingAgentResult = null;
    pendingProjectResult = null;
    agentReturnContext = null;
    knowledgeReturnContext = null;
    domainReturnContext = null;
    artifactReturnContext = null;
    projectSettingsDraft = null;
  }

  function clearStoredRoot() {
    currentRoot = "";
    localStorage.removeItem("nagare.root");
  }

  function isMissingSavedRootError(error) {
    const message = String(error || "").toLowerCase();
    return (
      message.includes("os error 3") ||
      message.includes("path not found") ||
      message.includes("cannot find path") ||
      message.includes("指定されたパスが見つかりません")
    );
  }

  function escapeHtml(value) {
    return String(value ?? "")
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#39;");
  }

  function lines(values) {
    return (values || []).filter(Boolean);
  }

  function rootValue() {
    return currentRoot || state?.root || null;
  }

  function projectName() {
    return state?.project?.name || projectNameFromRoot(rootValue()) || "nagare";
  }

  function projectIcon() {
    return state?.project?.icon || "流";
  }

  function projectNameFromRoot(root) {
    const parts = String(root || "").split(/[\\/]/).filter(Boolean);
    return parts[parts.length - 1] || "";
  }

  function slugFromText(value, fallback) {
    const slug = String(value || "")
      .trim()
      .toLowerCase()
      .replace(/['"]/g, "")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "");
    return slug || fallback;
  }

  function uniqueEntityId(base, existingIds) {
    const existing = new Set(existingIds.filter(Boolean));
    let id = base;
    let index = 2;
    while (existing.has(id)) {
      id = `${base}-${index}`;
      index += 1;
    }
    return id;
  }

  function generatedDomainId(displayName) {
    return uniqueEntityId(
      slugFromText(displayName, `domain-${(state?.domains || []).length + 1}`),
      (state?.domains || []).map((domain) => domain.id),
    );
  }

  function generatedArtifactId(displayName) {
    return uniqueEntityId(
      slugFromText(displayName, `artifact-${(state?.artifact_types || []).length + 1}`),
      (state?.artifact_types || []).map((artifact) => artifact.id),
    );
  }

  function generatedAgentId(displayName) {
    return uniqueEntityId(
      slugFromText(displayName, `agent-${(state?.agents || []).length + 1}`),
      (state?.agents || []).map((agent) => agent.id),
    );
  }

  function generatedMcpId(displayName, command = "", args = "") {
    const argCandidate = textLines(args)
      .find((line) => !line.startsWith("-")) || "";
    const source = String(displayName || argCandidate || command || "").trim()
      .replace(/^@[^/\\]+[/\\]/, "")
      .replace(/\b(mcp|server|connection|接続)\b/gi, "")
      .trim();
    return uniqueEntityId(
      slugFromText(source, `mcp-${(state?.mcp_connections || []).length + 1}`),
      (state?.mcp_connections || []).map((mcp) => mcp.id),
    );
  }

  function bindGeneratedId(form, generator) {
    const idInput = form.querySelector('input[name="id"]');
    const nameInput = form.querySelector('input[name="display_name"]');
    if (!idInput || !nameInput || idInput.readOnly) return;
    let userEditedId = Boolean(idInput.value.trim());
    idInput.addEventListener("input", () => {
      userEditedId = Boolean(idInput.value.trim());
    });
    nameInput.addEventListener("input", () => {
      if (userEditedId) return;
      idInput.value = generator(nameInput.value.trim());
    });
  }

  function bindMcpGeneratedId(form) {
    const idInput = form.querySelector('input[name="id"]');
    const nameInput = form.querySelector('input[name="display_name"]');
    const commandInput = form.querySelector('input[name="command"]');
    const argsInput = form.querySelector('textarea[name="args"]');
    if (!idInput || !nameInput || idInput.readOnly) return;
    let userEditedId = Boolean(idInput.value.trim());
    idInput.addEventListener("input", () => {
      userEditedId = Boolean(idInput.value.trim());
    });
    const update = () => {
      if (userEditedId) return;
      idInput.value = generatedMcpId(nameInput.value.trim(), commandInput?.value || "", argsInput?.value || "");
    };
    nameInput.addEventListener("input", update);
    commandInput?.addEventListener("input", update);
    argsInput?.addEventListener("input", update);
  }

  function setCrumb(html) {
    const crumb = document.getElementById("crumb");
    if (crumb) crumb.innerHTML = html;
  }

  function toast(message, kind = "info") {
    let notice = document.getElementById("app-notice");
    if (!notice) {
      notice = document.createElement("div");
      notice.id = "app-notice";
      notice.className = "card";
      notice.style.cssText = "display:none;margin:0 0 14px;padding:10px 12px;font-size:13px;font-weight:600;";
      document.querySelector(".content-inner")?.prepend(notice);
    }
    notice.style.display = "";
    notice.style.borderColor = kind === "error" ? "var(--danger-line)" : "var(--primary-line)";
    notice.style.background = kind === "error" ? "var(--danger-soft)" : "var(--primary-soft)";
    notice.style.color = kind === "error" ? "var(--danger)" : "var(--primary)";
    notice.textContent = message;
  }

  function showOperationError(container, title, error, next = "") {
    const body = container?.classList?.contains("modal-body")
      ? container
      : container?.querySelector?.(".modal-body") || container;
    if (!body) {
      toast(String(error), "error");
      return;
    }
    body.querySelector("[data-operation-error]")?.remove();
    const panel = document.createElement("div");
    panel.className = "card";
    panel.dataset.operationError = "true";
    panel.style.cssText = "margin:0 0 14px;padding:12px 14px;border-color:var(--danger-line);background:var(--danger-soft);";
    panel.innerHTML = `
      <div style="font-size:13.5px;font-weight:800;color:var(--danger);">${escapeHtml(title)}</div>
      <div style="font-size:12.5px;color:var(--text-body);margin-top:4px;">${escapeHtml(String(error))}</div>
      ${next ? `<div style="font-size:12px;color:var(--text-muted);margin-top:6px;">${escapeHtml(next)}</div>` : ""}
    `;
    body.prepend(panel);
  }

  function refreshNavigationIndicators() {
    setWorkAttentionDot();
    setInsightsAttentionDot();
  }

  function setWorkAttentionDot() {
    const dot = document.getElementById("attn-dot");
    if (!dot) return;
    const attentionCount = (state?.work_items || []).filter((item) => {
      const kind = rowKind(item.status_kind);
      return kind === "attn" || kind === "recover";
    }).length;
    dot.style.display = state?.initialized && attentionCount ? "" : "none";
    dot.title = attentionCount ? `要対応 ${attentionCount}件` : "";
  }

  function setInsightsAttentionDot() {
    const dot = document.getElementById("insights-dot");
    if (!dot) return;
    const insights = state?.insights || {};
    const proposalCount = Number(insights.proposal_count ?? insights.proposals?.length ?? 0);
    dot.style.display = state?.initialized && proposalCount ? "" : "none";
    dot.title = proposalCount ? `未対応の改善提案 ${proposalCount}件` : "";
  }

  async function call(command, payload = {}) {
    if (!invoke) throw new Error("Tauri API が見つかりません。デスクトップアプリとして起動してください。");
    return invoke(command, payload);
  }

  async function loadState({ navigate = true } = {}) {
    if (!invoke) {
      return;
    }
    try {
      state = await call("app_state", { root: currentRoot || null });
      if (state.initialized) {
        currentRoot = state.root || currentRoot;
        if (currentRoot) {
          localStorage.setItem("nagare.root", currentRoot);
        } else {
          localStorage.removeItem("nagare.root");
        }
      } else {
        clearProjectScopedUiState();
        clearStoredRoot();
      }
      refreshNavigationIndicators();
      syncBackgroundWorkState();
      renderProjectScreens();
      renderSettingsScreens();
      renderInsights();
      renderCatalog();
      if (state.initialized) {
        renderHome();
        if (navigate) goApp("home-active");
      } else {
        renderEmptyHome();
        if (navigate) goApp("home-empty");
      }
    } catch (error) {
      if ((currentRoot || localStorage.getItem("nagare.root")) && isMissingSavedRootError(error)) {
        clearProjectScopedUiState();
        clearStoredRoot();
        await loadState({ navigate });
        return;
      }
      renderLoadErrorHome(error);
      if (navigate) goApp("home-empty");
      toast(String(error), "error");
    }
  }

  function goApp(id, crumb = null) {
    window.go(id);
    if (crumb) setCrumb(crumb);
    refreshNavigationIndicators();
  }

  function syncBackgroundWorkState() {
    if (!backgroundWorkIds.size) return;
    const items = state?.work_items || [];
    for (const id of [...backgroundWorkIds]) {
      const item = items.find((work) => work.id === id);
      if (!item || rowKind(item.status_kind) !== "run") {
        backgroundWorkIds.delete(id);
      }
    }
    if (backgroundWorkIds.size) startWorkRefresh();
    else stopWorkRefresh();
  }

  function startWorkRefresh() {
    if (workRefreshTimer) return;
    workRefreshTimer = window.setInterval(() => {
      if (!backgroundWorkIds.size || !state?.initialized) {
        stopWorkRefresh();
        return;
      }
      loadState({ navigate: false }).catch((error) => toast(String(error), "error"));
    }, 1500);
  }

  function stopWorkRefresh() {
    if (!workRefreshTimer) return;
    window.clearInterval(workRefreshTimer);
    workRefreshTimer = null;
  }

  function markWorkAdvancing(id) {
    if (!id) return;
    backgroundWorkIds.add(id);
    startWorkRefresh();
  }

  function renderEmptyHome() {
    const el = document.getElementById("scr-home-empty");
    el.innerHTML = `
      <div class="hero-empty">
        <img src="logo.png" alt="">
        <h2>依頼を書くだけで、AIが作業を進めます</h2>
        <p>Nagare は依頼を受け取り、適切なAIエージェントに割り当てて実行し、品質基準でレビューします。あなたの判断が必要なときだけ、質問と確認が届きます。</p>
        <div class="flow-model">
          ${["依頼", "整理", "実行", "レビュー", "確認"].map((label, index) => `
            <div class="flow-step ${index === 4 ? "human" : ""}">
              <div class="fs-icon">${["✍", "🧭", "⚙", "□", "人"][index]}</div>
              <div class="fs-name">${label}</div>
              <div class="fs-desc">${["自然文で書く", "担当と基準を選ぶ", "エージェントが作業", "基準で検証", "必要時だけ判断"][index]}</div>
            </div>
          `).join("")}
        </div>
        <button class="btn btn-primary" type="button" data-app-setup-open>セットアップを開始</button>
        <p style="font-size:12px;color:var(--text-faint);margin-top:12px;">必要なのはプロジェクトと実行環境の2つだけ。約1分で完了します。</p>
      </div>
    `;
    el.querySelector("[data-app-setup-open]").addEventListener("click", () => openSetup(1));
  }

  function renderLoadErrorHome(error) {
    const el = document.getElementById("scr-home-empty");
    const root = currentRoot || localStorage.getItem("nagare.root") || "";
    el.innerHTML = `
      <div class="hero-empty">
        <img src="logo.png" alt="">
        <h2>状態を読み込めませんでした</h2>
        <p>保存済みのプロジェクト状態を確認できませんでした。再読み込みするか、保存済みの場所を外してセットアップからやり直せます。</p>
        <div class="card" style="max-width:620px;margin:18px auto 0;padding:14px 16px;text-align:left;border-color:var(--danger-line);background:var(--danger-soft);">
          <div style="font-size:13.5px;font-weight:800;color:var(--danger);">読み込みエラー</div>
          <div style="font-size:12.5px;color:var(--text-body);margin-top:4px;">${escapeHtml(String(error))}</div>
          ${root ? `<div style="font-size:12px;color:var(--text-muted);margin-top:8px;">保存済みの場所: <span class="mono">${escapeHtml(root)}</span></div>` : ""}
        </div>
        <div style="display:flex;gap:8px;justify-content:center;margin-top:18px;flex-wrap:wrap;">
          <button class="btn btn-primary" type="button" data-retry-load-state>再読み込み</button>
          <button class="btn btn-secondary" type="button" data-clear-root-and-retry>保存済みの場所を外す</button>
        </div>
      </div>
    `;
    el.querySelector("[data-retry-load-state]").addEventListener("click", () => loadState());
    el.querySelector("[data-clear-root-and-retry]").addEventListener("click", () => {
      clearProjectScopedUiState();
      clearStoredRoot();
      loadState();
    });
  }

  function availableRuntimes() {
    return (state?.runtimes || []).filter((runtime) => runtime.available);
  }

  function openSetup(step = 1, draft = {}) {
    const runtimes = availableRuntimes();
    const runtimeId = draft.runtimeId || runtimes[0]?.id || "codex";
    const setupRuntime = (state?.runtimes || []).find((runtime) => runtime.id === runtimeId) || null;
    const root = draft.root ?? currentRoot ?? state?.root ?? "";
    const name = draft.name ?? (root ? projectNameFromRoot(root) : "");
    const setupError = String(draft.error || "").trim();
    const setupRuntimeDetail = String(draft.runtimeDetail || setupRuntime?.detail || "").trim();
    const modal = dynamicModal(`
      <div class="modal" style="width:540px;">
        <div class="modal-head">
          <div class="m-step">セットアップ ${step} / 3</div>
          <h3>${step === 1 ? "プロジェクトをつくる" : step === 2 ? "実行環境をえらぶ" : "接続確認"}</h3>
          <div class="m-sub">${step === 1 ? "作業対象のフォルダを選びます。表示名は自動で用意できます。" : step === 2 ? "この端末で見つかったAI実行環境から選びます。" : "選択した構成で開始します。"}</div>
        </div>
        <div class="modal-body">
          ${step === 1 ? `
            <div class="field">
              <label>対象フォルダ</label>
              <div style="display:flex;gap:8px;">
                <input id="setup-root" type="text" class="mono" value="${escapeHtml(root)}" style="flex:1;">
                <button class="btn btn-secondary" type="button" data-choose-folder>選択…</button>
              </div>
              <div class="hint">このフォルダにNagareのプロジェクト設定とワーク履歴を作成します。</div>
            </div>
            <div class="field">
              <label>表示名（任意）</label>
              <input id="setup-name" type="text" value="${escapeHtml(name)}" placeholder="空欄ならフォルダ名を使います">
              <div class="hint">あとからプロジェクト設定で変更できます。</div>
            </div>
          ` : step === 2 ? `
            ${runtimes.length ? runtimes.map((runtime) => `
              <label class="rt-opt ${runtime.id === runtimeId ? "sel" : ""}" data-runtime-option="${escapeHtml(runtime.id)}">
                <input class="visually-hidden" type="radio" name="setup-runtime" value="${escapeHtml(runtime.id)}" ${runtime.id === runtimeId ? "checked" : ""}>
                <div class="rt-icon">${runtimeIcon(runtime.id)}</div>
                <div><div class="rt-name">${escapeHtml(runtime.label)}</div><div class="rt-desc">${escapeHtml(runtime.detail || runtime.model_note || "")}</div></div>
                <span class="rt-state badge badge-done">利用可能</span>
              </label>
            `).join("") : `
              <div class="card" style="padding:14px;border-color:var(--warning-line);background:var(--warning-soft);">
                <b>利用できる実行環境が見つかりません</b>
                <p style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">Claude Code / Codex CLI / OpenCode / OpenClaw のいずれかを PATH から実行できる状態にしてください。</p>
                <button class="btn btn-secondary btn-sm" type="button" data-open-runtime-settings style="margin-top:10px;">実行環境を確認</button>
              </div>
            `}
          ` : `
            <div class="conn-result">
              <div class="c-icon">${setupError ? "!" : "✓"}</div>
              <h4>${setupError ? "接続確認に失敗しました" : `${escapeHtml(runtimeLabel(runtimeId))} で開始します`}</h4>
              <p>${setupError ? "実行環境を確認してから、もう一度開始してください。" : "プロジェクトと既定エージェントを初期化します。"}</p>
            </div>
            <div class="conn-check"><span class="${setupError ? "bad" : "ok"}">${setupError ? "!" : "✓"}</span> 実行環境: ${escapeHtml(runtimeLabel(runtimeId))}${setupRuntimeDetail ? ` <span class="mono">${escapeHtml(setupRuntimeDetail)}</span>` : ""}</div>
            <div class="conn-check"><span class="ok">✓</span> プロジェクト: ${escapeHtml(name || projectNameFromRoot(root))}</div>
            <div class="conn-check"><span class="ok">✓</span> フォルダ: <span class="mono">${escapeHtml(root)}</span></div>
            ${setupError ? `
              <div class="card" style="padding:12px;margin-top:10px;border-color:var(--danger-line);background:var(--danger-soft);">
                <b style="color:var(--danger);">失敗理由</b>
                <div style="font-size:12.5px;color:var(--text-body);margin-top:4px;">${escapeHtml(setupError)}</div>
                <div class="hint" style="margin-top:8px;">インストール、PATH、認証を確認してください。別の実行環境を選ぶ場合は戻るを押します。</div>
                <button class="btn btn-secondary btn-sm" type="button" data-open-runtime-settings style="margin-top:10px;">実行環境を確認</button>
              </div>
            ` : ""}
          `}
        </div>
        <div class="modal-foot">
          ${step === 1 ? `<button class="btn btn-secondary" data-close>閉じる</button><button class="btn btn-primary" data-next>次へ</button>` : ""}
          ${step === 2 ? `<button class="btn btn-secondary left" data-back>戻る</button><button class="btn btn-primary" data-next ${runtimes.length ? "" : "disabled"}>保存して接続確認</button>` : ""}
          ${step === 3 ? `<button class="btn btn-secondary left" data-back>戻る</button><button class="btn btn-primary" data-complete ${setupError ? "disabled" : ""}>はじめる</button>` : ""}
        </div>
      </div>
    `);
    modal.querySelectorAll("[data-runtime-option]").forEach((option) => {
      option.addEventListener("click", () => {
        modal.querySelectorAll("[data-runtime-option]").forEach((item) => item.classList.remove("sel"));
        option.classList.add("sel");
        option.querySelector("input").checked = true;
      });
    });
    modal.querySelector("[data-close]")?.addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-back]")?.addEventListener("click", () => openSetup(step - 1, { root, name, runtimeId }));
    modal.querySelector("[data-open-runtime-settings]")?.addEventListener("click", () => {
      closeDynamicModal();
      renderRuntimes();
      goApp("settings-runtime", "エージェント / <b>実行環境</b>");
    });
    modal.querySelector("[data-choose-folder]")?.addEventListener("click", async () => {
      try {
        const folder = await call("choose_project_folder");
        if (!folder) return;
        const input = modal.querySelector("#setup-root");
        input.value = folder;
        const nameInput = modal.querySelector("#setup-name");
        if (nameInput && !nameInput.value.trim()) {
          nameInput.value = projectNameFromRoot(folder);
        }
      } catch (error) {
        toast(String(error), "error");
      }
    });
    modal.querySelector("[data-next]")?.addEventListener("click", async () => {
      const nextRoot = modal.querySelector("#setup-root")?.value?.trim() ?? root;
      const nextName = modal.querySelector("#setup-name")?.value?.trim() ?? name;
      const nextRuntime = modal.querySelector('input[name="setup-runtime"]:checked')?.value ?? runtimeId;
      if (step === 1 && !nextRoot) {
        toast("対象フォルダを入力してください。", "error");
        return;
      }
      if (step === 2) {
        try {
          const response = await call("refresh_runtime_status", { request: { root: nextRoot, runtime_id: nextRuntime } });
          state = response.state || state;
          if (response.runtime) {
            const currentRuntimes = state.runtimes || [];
            state.runtimes = currentRuntimes.map((runtime) => runtime.id === response.runtime.id ? response.runtime : runtime);
          }
          if (response.runtime && !response.runtime.available) {
            openSetup(3, {
              root: nextRoot,
              name: nextName,
              runtimeId: nextRuntime,
              runtimeDetail: response.runtime.detail,
              error: `${response.runtime.label || nextRuntime} が見つかりません。インストールとPATHを確認してください。`,
            });
            return;
          }
          openSetup(3, { root: nextRoot, name: nextName, runtimeId: nextRuntime, runtimeDetail: response.runtime?.detail });
          return;
        } catch (error) {
          openSetup(3, { root: nextRoot, name: nextName, runtimeId: nextRuntime, error: String(error) });
          return;
        }
      }
      openSetup(step + 1, { root: nextRoot, name: nextName, runtimeId: nextRuntime });
    });
    modal.querySelector("[data-complete]")?.addEventListener("click", async () => {
      try {
        const nextState = await call("initialize_project_with_runtime", {
          request: { root, runtime_id: runtimeId, display_name: name || projectNameFromRoot(root), icon: "流" },
        });
        state = nextState;
        currentRoot = state.root || root;
        localStorage.setItem("nagare.root", currentRoot);
        clearProjectScopedUiState();
        closeDynamicModal();
        toast("セットアップを完了しました。最初の依頼を作成できます。");
        renderHome();
        goApp("home-active");
      } catch (error) {
        const message = String(error);
        openSetup(3, { root, name, runtimeId, error: message });
        toast(message, "error");
      }
    });
  }

  function runtimeIcon(id) {
    return { claude: "C", codex: "⌘", opencode: "O", openclaw: "爪" }[id] || "AI";
  }

  function runtimeLabel(id) {
    return state?.runtimes?.find((runtime) => runtime.id === id)?.label || id;
  }

  function renderHome() {
    const items = state?.work_items || [];
    const currentProject = projectName();
    const statusCounts = workStatusFilterCounts(items);
    const projectOptions = workProjectFilterOptions(items, currentProject);
    const el = document.getElementById("scr-home-active");
    el.innerHTML = `
      <div class="group-head" style="margin-top:0;"><h3>新規ワーク依頼</h3></div>
      <form class="ask-box" id="create-work-form">
        <textarea name="description" required placeholder="依頼を書いてください。例: リリースノート v0.4 の下書きを作成して"></textarea>
        <div class="ask-foot">
          <div class="dd" style="display:flex;align-items:center;gap:6px;">
            <label class="visually-hidden" for="work-project-select">対象プロジェクト</label>
            <span class="dd-pre">プロジェクト:</span>
            <select id="work-project-select" name="project" style="width:auto;min-width:220px;">
              <option value="${escapeHtml(currentProject)}">${escapeHtml(currentProject)}</option>
            </select>
          </div>
          <div style="flex:1"></div>
          <button class="btn btn-primary" type="submit">作業を開始</button>
        </div>
      </form>
      <div class="filters">
        <label class="visually-hidden" for="work-status-filter">状態</label>
        <select id="work-status-filter" style="width:auto;min-width:150px;">
          <option value="all">状態: すべて (${statusCounts.all})</option>
          <option value="attn">要対応 (${statusCounts.attn})</option>
          <option value="run">処理中 (${statusCounts.run})</option>
          <option value="done">完了 (${statusCounts.done})</option>
        </select>
        <label class="visually-hidden" for="work-project-filter">プロジェクト</label>
        <select id="work-project-filter" style="width:auto;min-width:200px;">
          <option value="all">プロジェクト: すべて (${items.length})</option>
          ${projectOptions.map((option) => `<option value="${escapeHtml(option.value)}">${escapeHtml(option.label)} (${option.count})</option>`).join("")}
        </select>
        <label class="visually-hidden" for="work-search-filter">ワーク検索</label>
        <input id="work-search-filter" type="search" placeholder="ワークを検索" style="width:220px;">
        <span style="flex:1"></span>
        <span class="filter-result" id="filter-result">${items.length}件を表示</span>
      </div>
      <div class="list" id="work-list">
        ${items.length ? items.map(workRow).join("") : `<div class="list-empty">まだワークはありません。上の入力欄から依頼を作成してください。</div>`}
        <div class="list-empty" id="work-empty" style="display:none;">条件に一致するワークはありません</div>
      </div>
    `;
    el.querySelector("#create-work-form").addEventListener("submit", (event) => {
      event.preventDefault();
      const form = new FormData(event.currentTarget);
      const description = String(form.get("description") || "").trim();
      const selectedProject = String(form.get("project") || currentProject);
      if (!description) return toast("依頼内容を入力してください。", "error");
      const artifact = primaryArtifact();
      const domain = primaryDomain();
      pendingWork = {
        description,
        project: selectedProject,
        domain_id: domain?.id || "",
        artifact_type_id: artifact?.id || "",
        workflow_mode: state?.project?.workflow_mode || "confirm_first",
        approval_policy: state?.project?.approval_policy || "manual_final_approval",
      };
      startWork(event.currentTarget.querySelector('button[type="submit"]'));
    });
    el.querySelectorAll("[data-work-id]").forEach((row) => {
      row.addEventListener("click", () => openWork(row.dataset.workId));
    });
    el.querySelectorAll("[data-work-open]").forEach((button) => {
      button.addEventListener("click", (event) => {
        event.stopPropagation();
        openWork(button.dataset.workOpen);
      });
    });
    el.querySelectorAll("[data-work-delete]").forEach((button) => {
      button.addEventListener("click", (event) => {
        event.stopPropagation();
        openDeleteWorkDialog(button.dataset.workDelete);
      });
    });
    el.querySelector("#work-status-filter").addEventListener("input", applyDynamicWorkFilter);
    el.querySelector("#work-project-filter").addEventListener("input", applyDynamicWorkFilter);
    el.querySelector("#work-search-filter").addEventListener("input", applyDynamicWorkFilter);
    applyDynamicWorkFilter();
  }

  function workStatusFilterCounts(items) {
    return items.reduce((counts, item) => {
      counts.all += 1;
      const kind = rowKind(item.status_kind);
      if (kind === "attn" || kind === "recover") counts.attn += 1;
      if (kind === "run") counts.run += 1;
      if (kind === "done") counts.done += 1;
      return counts;
    }, { all: 0, attn: 0, run: 0, done: 0 });
  }

  function workProjectFilterOptions(items, currentProject) {
    const counts = new Map();
    const add = (project) => {
      const label = String(project || currentProject || projectName()).trim() || projectName();
      const value = label.toLowerCase();
      counts.set(value, { value, label, count: (counts.get(value)?.count || 0) + 1 });
    };
    if (!items.length) add(currentProject);
    items.forEach((item) => add(workProjectName(item)));
    return [...counts.values()].sort((a, b) => a.label.localeCompare(b.label, "ja"));
  }

  function workProjectName(item) {
    return item.project || projectName();
  }

  function workRow(item) {
    const kind = rowKind(item.status_kind);
    const itemProject = workProjectName(item);
    const summary = workResultSummary(item);
    const score = workReviewScore(item);
    return `
      <div class="list-item work-item ${kind === "attn" ? "attn" : kind === "recover" ? "attn-fix" : ""}" data-status="${kind}" data-project="${escapeHtml(itemProject.toLowerCase())}" data-search="${escapeHtml([item.title, item.result_summary, item.description, itemProject].join(" ").toLowerCase())}" data-work-id="${escapeHtml(item.id)}">
        <div class="wr-picon">${escapeHtml(projectIcon())}</div>
        <div class="wr-body">
          <div class="wr-title">${escapeHtml(item.title)} <span class="wr-projtag">${escapeHtml(itemProject)}</span></div>
          <div class="wr-sum">${escapeHtml(summary)}</div>
        </div>
        <div class="wr-state" aria-label="ワークの状態">
          <div class="wr-state-badges">
            <span class="${badgeClass(item.status_kind)}">${escapeHtml(item.status_label || "未処理")}</span>
            ${score ? `<span class="badge badge-done">${escapeHtml(score)}</span>` : ""}
          </div>
          <span class="wr-next">${escapeHtml(item.next_action ? `次: ${item.next_action}` : item.updated_at || "")}</span>
        </div>
        <div class="wr-actions" aria-label="ワークの操作">
          <button class="btn btn-secondary btn-sm" type="button" data-work-open="${escapeHtml(item.id)}">詳細</button>
          <button class="btn btn-danger-soft btn-sm" type="button" data-work-delete="${escapeHtml(item.id)}">削除</button>
        </div>
      </div>
    `;
  }

  function workResultSummary(item) {
    const summary = String(item.result_summary || "").trim();
    if (summary) return summary.replace(/\s*·\s*評価\s*\d+\s*\/\s*\d+/u, "");
    return item.description || "結果はまだありません";
  }

  function workReviewScore(item) {
    const explicit = item.score_label || item.review_score_label || item.evaluation_label;
    if (explicit) return String(explicit);
    const match = String(item.result_summary || "").match(/評価\s*(\d+\s*\/\s*\d+)/u);
    return match ? match[1].replace(/\s+/g, " ") : "";
  }

  function rowKind(status) {
    if (["question", "review"].includes(status)) return "attn";
    if (status === "recover") return "recover";
    if (status === "running") return "run";
    if (status === "done") return "done";
    return "all";
  }

  function badgeClass(status) {
    if (status === "done") return "badge badge-done";
    if (status === "running") return "badge badge-run";
    if (status === "recover") return "badge badge-fix";
    if (status === "question" || status === "review") return "badge badge-ask";
    return "badge badge-neutral";
  }

  function applyDynamicWorkFilter() {
    const value = document.getElementById("work-status-filter")?.value || "all";
    const project = document.getElementById("work-project-filter")?.value || "all";
    const query = (document.getElementById("work-search-filter")?.value || "").trim().toLowerCase();
    let count = 0;
    document.querySelectorAll("#work-list [data-work-id]").forEach((row) => {
      const statusVisible = value === "all" || row.dataset.status === value || (value === "attn" && row.dataset.status === "recover");
      const projectVisible = project === "all" || row.dataset.project === project;
      const queryVisible = !query || row.dataset.search.includes(query);
      const visible = statusVisible && projectVisible && queryVisible;
      row.style.display = visible ? "" : "none";
      if (visible) count += 1;
    });
    const result = document.getElementById("filter-result");
    const empty = document.getElementById("work-empty");
    if (result) result.textContent = `${count}件を表示`;
    if (empty) empty.style.display = count ? "none" : "";
  }

  function agentForRole(role, projectLabel = "") {
    return findAgentByLabel(projectLabel) || (state?.agents || []).find((agent) => agent.role === role) || null;
  }

  function primaryDomain() {
    const artifact = primaryArtifact();
    return findDomain(state?.project?.default_domain_id || artifact?.domain_id) || (state?.domains || [])[0] || null;
  }

  function primaryArtifact() {
    const artifacts = state?.artifact_types || [];
    const defaultArtifact = findArtifact(state?.project?.default_artifact_type_id);
    if (defaultArtifact) return defaultArtifact;
    const defaultDomainId = state?.project?.default_domain_id;
    return artifacts.find((artifact) => artifact.domain_id === defaultDomainId) || artifacts[0] || null;
  }

  function runtimeSummary(agent) {
    if (!agent) return "実行時に選択";
    const runtime = (state?.runtimes || []).find((item) => item.id === agent.runtime || item.id === agent.tool_kind);
    const label = runtime?.label || toolKindLabel(agent.tool_kind || agent.runtime);
    const status = runtime?.available === false ? "未接続" : "接続済み";
    const badge = runtime?.available === false ? "badge-fix" : "badge-done";
    return `${escapeHtml(label)} <span class="badge ${badge}" style="margin-left:4px;">${status}</span>`;
  }

  async function startWork(trigger = null) {
    const button = trigger instanceof HTMLElement ? trigger : null;
    try {
      if (!pendingWork?.description) {
        toast("依頼内容を入力してください。", "error");
        return;
      }
      if (button) {
        button.disabled = true;
        button.dataset.originalText = button.textContent || "作業を開始";
        button.textContent = "開始中...";
      }
      const detail = await call("create_work", {
        request: {
          root: rootValue(),
          description: pendingWork.description,
          project: pendingWork.project,
          domain_id: pendingWork.domain_id,
          artifact_type_id: pendingWork.artifact_type_id,
          workflow_mode: pendingWork.workflow_mode,
          approval_policy: pendingWork.approval_policy,
        },
      });
      activeDetail = detail;
      await loadState({ navigate: false });
      if (shouldOfferAdvance(detail)) {
        markWorkAdvancing(detail.item.id);
        pendingWork = null;
        homeWorkProgress = null;
        renderHome();
        goApp("home-active");
        toast("ワークを開始しました。進行状況は一覧で更新されます。");
        call("start_work_background", { request: { root: rootValue(), id: detail.item.id, max_steps: 8 } })
          .then((nextState) => {
            if (nextState) state = nextState;
            syncBackgroundWorkState();
            renderHome();
            startWorkRefresh();
          })
          .catch((error) => {
            backgroundWorkIds.delete(detail.item.id);
            stopWorkRefresh();
            renderHome();
            showOperationError(document.getElementById("scr-home-active"), "ワークを進められませんでした", error, "ワーク一覧から詳細を開き、現在の工程と実行環境を確認してください。");
            toast(String(error), "error");
          });
        return;
      }
      homeWorkProgress = null;
      pendingWork = null;
      renderHome();
      goApp("home-active");
      toast("ワークを作成しました。");
    } catch (error) {
      homeWorkProgress = null;
      if (button) {
        button.disabled = false;
        button.textContent = button.dataset.originalText || "作業を開始";
      }
      showOperationError(document.getElementById("scr-home-active"), "ワークを開始できませんでした", error, "依頼内容、プロジェクト、実行環境の状態を確認してから、もう一度開始してください。");
      toast(String(error), "error");
    }
  }

  async function openWork(id) {
    try {
      const detail = await call("get_work_detail", { root: rootValue(), id });
      activeDetail = detail;
      renderDetail(detail);
      goDetail(detail);
    } catch (error) {
      toast(String(error), "error");
    }
  }

  function openDeleteWorkDialog(id) {
    const item = (state?.work_items || []).find((work) => work.id === id);
    if (!item) return toast("削除するワークが見つかりません。", "error");
    const modal = dynamicModal(`
      <div class="modal">
        <div class="modal-head"><h3>ワークを削除しますか?</h3><div class="m-sub">${escapeHtml(item.title || id)} を一覧から削除します。</div></div>
        <div class="modal-body">
          <div class="card" style="padding:12px 14px;margin-bottom:10px;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text);">${escapeHtml(item.title || id)}</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">${escapeHtml(workResultSummary(item))}</div>
          </div>
          <div class="hint">削除すると、このワークの実行記録、成果物記録、レビュー、差し戻し履歴も一覧と詳細から削除されます。実ファイルは別途管理してください。</div>
        </div>
        <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button><button class="btn btn-primary" type="button" data-confirm>削除</button></div>
      </div>
    `);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-confirm]").addEventListener("click", async () => {
      try {
        const nextState = await call("delete_work", { request: { root: rootValue(), id } });
        state = nextState;
        if (activeDetail?.item?.id === id) activeDetail = null;
        closeDynamicModal();
        renderHome();
        goApp("home-active");
        toast("ワークを削除しました。");
      } catch (error) {
        showOperationError(modal, "ワークを削除できませんでした", error, "ワークの状態を再読み込みしてから、もう一度削除してください。");
        toast(String(error), "error");
      }
    });
  }

  function goDetail(detail) {
    const title = escapeHtml(detail?.item?.title || "ワーク詳細");
    goApp(detailScreenId(detail), `ワーク / <b>${title}</b>`);
  }

  function detailScreenId(detail) {
    if (detail?.item?.status_kind === "done" || detail?.next_action_kind === "done") return "detail-done";
    if (detail?.question || detail?.next_action_kind === "answer_question") return "detail-question";
    if (hasActiveRecovery(detail)) return "detail-recover";
    if (detail?.item?.status_kind === "running" || ["dispatch", "accept_dispatch"].includes(detail?.next_action_kind)) return "detail-running";
    return "detail-review";
  }

  function renderDetail(detail) {
    const el = document.getElementById(`scr-${detailScreenId(detail)}`);
    const needsHuman = detailNeedsHuman(detail);
    const needsApproval = detail.approval_ready || detail.next_action_kind === "approve";
    const displaySteps = detailDisplaySteps(detail);
    const progress = detailProgress(detail);
    const action = actionSurface(detail);
    const actionBeforeResult = detail.question || hasActiveRecovery(detail);
    el.innerHTML = `
      <div class="detail-head">
        <h2>${escapeHtml(detail.item.title)}</h2>
        <span class="${badgeClass(detail.item.status_kind)}">${escapeHtml(detail.item.status_label)}</span>
      </div>
      <div class="card status-strip ${detail.item.status_kind === "done" ? "strip-ok" : detail.item.status_kind === "recover" ? "strip-danger" : "strip-warn"}">
        <div class="ss-main">
          <div class="ss-now">${escapeHtml(statusSentence(detail))}</div>
          <div class="ss-next">${escapeHtml(detailNextText(detail))}</div>
        </div>
        <div class="ss-prog">
          <div>${escapeHtml(progress.position)}</div>
          <div>${escapeHtml(progress.current)}</div>
          <div>${escapeHtml(progress.elapsed)}</div>
        </div>
      </div>
      ${actionBeforeResult ? action : ""}
      ${detailResultSections(detail)}
      ${!actionBeforeResult && !needsApproval ? action : ""}
      <div class="group-head" style="margin-top:26px;"><h3>実行の流れ</h3><span class="count">${escapeHtml(String(displaySteps.length))}工程 · 行をクリックで詳細</span></div>
      <div class="trace">${displaySteps.map((step, index) => stepTemplate(step, index, detail)).join("") || `<p class="page-sub">実行の記録はまだありません。</p>`}</div>
      ${shouldOfferAdvance(detail) ? `<div data-advance-area style="display:flex;flex-direction:column;align-items:flex-end;gap:8px;margin-top:14px;"><button class="btn btn-primary" data-advance>次の判断点まで進める</button></div>` : ""}
      <div style="margin-top:18px;"><button class="btn btn-secondary" data-back-home>ワーク一覧へ戻る</button></div>
    `;
    el.querySelector("[data-back-home]").addEventListener("click", async () => {
      await loadState({ navigate: false });
      renderHome();
      goApp("home-active");
    });
    el.querySelector("[data-advance]")?.addEventListener("click", (event) => advanceWork(detail.item.id, {}, event.currentTarget));
    el.querySelector("[data-approve]")?.addEventListener("click", () => openApprove(detail));
    el.querySelector("[data-reject]")?.addEventListener("click", () => openReject(detail));
    el.querySelector("[data-answer]")?.addEventListener("submit", (event) => answerQuestion(event, detail));
    el.querySelector("[data-create-recovery]")?.addEventListener("click", () => createRecovery(detail));
    el.querySelector("[data-accept-recovery]")?.addEventListener("click", () => acceptRecovery(detail));
    el.querySelector("[data-apply-recovery]")?.addEventListener("submit", (event) => applyRecovery(event, detail));
    el.querySelectorAll("[data-artifact-detail]").forEach((button) => {
      button.addEventListener("click", () => openArtifactPreview(detail, Number(button.dataset.artifactDetail)));
    });
    el.querySelector("[data-artifact-list]")?.addEventListener("click", () => openArtifactList(detail));
    el.querySelector("[data-copy-review]")?.addEventListener("click", () => copyReviewResult(detail));
    el.querySelector("[data-edit-review-rubric]")?.addEventListener("click", () => openReviewRubric(detail));
    el.querySelectorAll(".step-top").forEach((header) => {
      header.addEventListener("click", (event) => {
        if (event.target.closest("button,a,input,textarea,select,label") || hasTextSelection()) return;
        header.closest(".step")?.classList.toggle("open");
      });
    });
    el.querySelectorAll("[data-step-diagnostics]").forEach((button) => {
      button.addEventListener("click", (event) => {
        event.stopPropagation();
        openStepDiagnostics(detail, Number(button.dataset.stepDiagnostics));
      });
    });
    el.querySelectorAll("[data-knowledge-ref]").forEach((button) => {
      button.addEventListener("click", (event) => {
        event.stopPropagation();
        openKnowledgeReference(button.dataset.knowledgeRef);
      });
    });
  }

  function detailNeedsHuman(detail) {
    return Boolean(detail?.question || hasActiveRecovery(detail) || detail?.approval_ready || ["answer_question", "approve"].includes(detail?.next_action_kind));
  }

  function hasActiveRecovery(detail) {
    return Boolean(
      detail
      && detail.item?.status_kind !== "done"
      && ["recover", "apply_recovery"].includes(detail.next_action_kind)
    );
  }

  function shouldOfferAdvance(detail) {
    return Boolean(detail && !detailNeedsHuman(detail) && detail.item?.status_kind !== "done");
  }

  function detailResultSections(detail) {
    const artifacts = lines(detail.artifacts);
    const needsApproval = detail.approval_ready || detail.next_action_kind === "approve";
    const done = detail.item?.status_kind === "done" || detail.next_action_kind === "done";
    const hasAnswer = Boolean(String(detail.answer || ((needsApproval || done) ? detail.item?.result_summary : "") || "").trim());
    const hasArtifacts = artifacts.length > 0;
    const hasReview = Boolean(detail.review);
    const dispatchingAfterReview = ["dispatch", "accept_dispatch"].includes(detail.next_action_kind) && hasReview;
    const hasConcreteResult = hasAnswer || hasArtifacts || hasReview || needsApproval || done || dispatchingAfterReview;
    if (!hasConcreteResult) return "";

    const parts = [
      `<div class="group-head" style="margin-top:18px;"><h3>結果</h3><span class="count">サマリー</span></div>`,
      resultOverview(detail),
    ];
    if (hasReview) {
      parts.push(`<div class="group-head" style="margin-top:18px;"><h3>レビュー</h3><span class="count">${escapeHtml(detail.review?.score_label ? `最終レビュー ${detail.review.score_label}点` : "実施済み")}</span></div>`);
      parts.push(reviewSummary(detail));
    }
    return parts.join("");
  }

  function detailProgress(detail) {
    const steps = detailDisplaySteps(detail);
    const total = steps.length;
    if (!total) {
      return {
        position: "0工程",
        current: "現在工程なし",
        elapsed: detailElapsedLabel(detail),
      };
    }
    const activeIndex = activeStepIndex(steps, detail);
    const step = steps[Math.max(0, activeIndex)];
    const completed = detail?.item?.status_kind === "done";
    const awaitingApproval = detail?.approval_ready || detail?.next_action_kind === "approve";
    return {
      position: `${Math.min(activeIndex + 1, total)} / ${total}工程`,
      current: completed
        ? "全工程完了"
        : awaitingApproval
          ? "AI工程完了・確認待ち"
          : step
            ? `現在: ${stepTitleLabel(step)} / ${stepActorLabel(step)}`
            : "現在工程なし",
      elapsed: detailElapsedLabel(detail),
    };
  }

  function detailDisplaySteps(detail) {
    const steps = lines(detail?.steps).map((step) => ({ ...step }));
    const hasOrganizerStep = steps.some((step) => {
      const label = `${step.kind || ""} ${step.title || ""}`.toLowerCase();
      return step.kind === "synthesis" || /summary|synthesis|まとめ|最終結果/.test(label);
    });
    const finalState = Boolean(
      detail?.approval_ready
      || detail?.next_action_kind === "approve"
      || detail?.next_action_kind === "done"
      || detail?.item?.status_kind === "done"
    );
    const hasFinalResult = finalState && Boolean(primaryResultText(detail));
    if (!hasOrganizerStep && hasFinalResult) {
      steps.push({
        kind: "organizer",
        title: "最終結果をまとめる",
        state: "記録済み",
        actor: "オーガナイザー",
        summary: "保存されている最終回答を、依頼者向けのまとめとして表示しています。",
        rationale: "このワークは過去形式の実行記録のため、保存された最終回答から補完しています。",
        input: detail?.review?.summary || "作業結果とレビュー結果",
        output: primaryResultText(detail),
        knowledge_refs: [],
        diagnostics: "",
        review_items: [],
      });
    }
    return steps;
  }

  function activeStepIndex(steps, detail) {
    if (detail?.item?.status_kind === "done") return Math.max(0, steps.length - 1);
    const firstActive = steps.findIndex((step) => ["now", "wait", "fail"].includes(stepClass(step)));
    if (firstActive >= 0) return firstActive;
    const firstOpen = steps.findIndex((step) => stepClass(step) !== "done");
    if (firstOpen >= 0) return firstOpen;
    return Math.max(0, steps.length - 1);
  }

  function detailElapsedLabel(detail) {
    const explicit = detail?.elapsed_label || detail?.item?.elapsed_label || detail?.duration_label;
    if (explicit) return `経過: ${explicit}`;
    if (detail?.item?.updated_at) return `更新: ${detail.item.updated_at}`;
    return "経過: 未記録";
  }

  function statusSentence(detail) {
    if (detail.question) return "エージェントから質問が届いています";
    if (hasActiveRecovery(detail)) return "実行が止まったため、回復方法を選べます";
    if (detail.approval_ready) return primaryResultText(detail) || "レビューが完了し、あなたの確認待ちです";
    if (detail.item.status_kind === "done" && approvalPolicyValue(detail) === "auto_complete_on_review_pass") return "レビュー合格により自動完了しました";
    if (detail.item.status_kind === "done" && approvalPolicyValue(detail) === "manual_on_review_concern") return "レビュー懸念がないため自動完了しました";
    if (detail.item.status_kind === "done") return "この作業は完了し、結果は採用済みです";
    if (["dispatch", "accept_dispatch"].includes(detail.next_action_kind)) {
      return detail.review
        ? "差し戻しを受け取り、担当を整理しています"
        : "担当エージェントを整理しています";
    }
    if (detail.item.status_kind === "running") return "エージェントが作業を進めています";
    return detail.item.status_label || "ワークの状態を確認できます";
  }

  function detailNextText(detail) {
    if (detail.approval_ready || detail.next_action_kind === "approve") {
      return "レビュー済み。必要なら採用または差し戻しできます。";
    }
    return detail.item.next_action || "次の操作はありません";
  }

  function primaryResultText(detail) {
    return String(detail?.answer || detail?.item?.result_summary || "").trim();
  }

  function resultOverview(detail) {
    const concerns = reviewConcernTexts(detail);
    const resultText = primaryResultText(detail);
    const needsApproval = detail.approval_ready || detail.next_action_kind === "approve";
    const done = detail.item.status_kind === "done";
    const dispatching = ["dispatch", "accept_dispatch"].includes(detail.next_action_kind);
    const reviewLine = detail.review?.score_label ? `最終レビュー ${detail.review.score_label}点` : (detail.review ? "レビュー済み" : "レビュー待ち");
    const policy = approvalPolicyLabel(detail);
    const requestText = workRequestText(detail);
    const organizerText = organizerResultText(detail, resultText, dispatching);
    const conclusion = organizerConclusion(detail, resultText, dispatching);
    const headline = dispatching ? conclusion : resultText || conclusion;
    const summaryText = dispatching && detail.review
      ? "差し戻しコメントを受け取り、担当エージェントへ再実行の指示を渡します。"
      : dispatching
        ? "依頼内容から担当エージェントを整理しています。"
        : resultText || detail.item.next_action || "結果が出るとここに要点が表示されます。";
    return `
      <div class="card result-overview ${needsApproval ? "needs-decision" : done ? "is-done" : ""}">
        <div class="ro-main">
          <div class="ro-kicker">依頼への回答</div>
          <h3>${escapeHtml(headline)}</h3>
          <div class="ro-dialogue">
            <div class="ro-line"><span>依頼</span><p>${escapeHtml(requestText)}</p></div>
          </div>
          ${resultArtifactLinks(detail, resultText)}
          <div class="ro-facts">
            <span><b>レビュー</b>${escapeHtml(reviewLine)}</span>
            <span><b>判断</b>${escapeHtml(needsApproval ? reviewRecommendation(detail) : detail.item.status_label || "確認中")}</span>
            <span><b>懸念</b>${escapeHtml(concerns.length ? `${concerns.length}件` : "なし")}</span>
            <span><b>確認</b>${escapeHtml(policy)}</span>
          </div>
          ${!resultText && organizerText !== headline ? `<p class="ro-note">${escapeHtml(organizerText)}</p>` : ""}
          ${summaryText && summaryText !== organizerText ? `<p class="ro-note">${escapeHtml(summaryText)}</p>` : ""}
        </div>
        ${needsApproval ? `
          <div class="ro-actions">
            <button class="btn btn-primary" data-approve>この結果を採用する</button>
            <button class="btn btn-danger-soft" data-reject>コメントを付けて差し戻す</button>
          </div>
        ` : ""}
      </div>
      ${needsApproval && concerns.length ? `
        <div class="mini-rubric result-concerns">
          ${concerns.map((concern) => `
            <div class="mr-row warn">
              <span class="mr-mark">!</span>
              <span class="mr-name">確認点</span>
              <span class="mr-evi">${escapeHtml(concern)}</span>
              <span class="mr-score">要確認</span>
            </div>
          `).join("")}
        </div>
        <div class="hint" style="margin-top:8px;">差し戻しコメントは次の実行で担当エージェントへの指示として使われます。</div>
      ` : ""}`;
  }

  function resultArtifactLinks(detail, resultText) {
    const artifacts = lines(detail.artifacts);
    if (!artifacts.length) {
      return `
        <div class="ro-artifacts">
          <span class="ro-artifacts-label">成果物</span>
          <span class="ro-artifacts-empty">${escapeHtml(resultText ? "応答のみ" : "未生成")}</span>
        </div>`;
    }
    const visible = artifacts.slice(0, 5);
    const remaining = artifacts.length - visible.length;
    return `
      <div class="ro-artifacts">
        <span class="ro-artifacts-label">成果物</span>
        <span class="ro-artifact-links">
          ${visible.map((artifact, index) => `
            <button class="result-artifact-link" type="button" data-artifact-detail="${index}">${escapeHtml(artifact.title || artifact.uri || `成果物${index + 1}`)}</button>
          `).join("")}
          ${remaining > 0 ? `<button class="result-artifact-link artifact-more-link" type="button" data-artifact-list>ほか${remaining}件</button>` : ""}
        </span>
      </div>`;
  }

  function workRequestText(detail) {
    return String(detail?.request || detail?.item?.description || detail?.item?.title || "").trim() || "依頼内容は記録されていません。";
  }

  function organizerResultText(detail, resultText, dispatching) {
    if (dispatching && detail.review) return "差し戻し内容を受け取りました。担当エージェントへ再実行の指示として渡します。";
    if (dispatching) return "依頼内容を整理し、担当エージェントを決めています。";
    if (detail.question) return "作業を進めるために、追加の確認が必要です。";
    if (hasActiveRecovery(detail)) return "実行が止まっています。回復方法を選ぶと、このワークを続きから再開できます。";
    if (resultText) return resultText;
    return detail.item?.next_action || "結果が出ると、ここに依頼への回答を表示します。";
  }

  function organizerConclusion(detail, resultText, dispatching) {
    if (dispatching && detail.review) return "差し戻しを受け取り、再実行待ちです";
    if (dispatching) return "担当を整理しています";
    if (detail.question) return "確認が必要です";
    if (hasActiveRecovery(detail)) return "回復が必要です";
    if (detail.item?.status_kind === "done") return "依頼への対応が完了しました";
    if (resultText) return "依頼への回答を作成しました";
    return "結果を作成中です";
  }

  function approvalPolicyValue(detail) {
    return detail?.item?.approval_policy || detail?.approval_policy || "manual_final_approval";
  }

  function approvalPolicyLabel(detail) {
    return {
      manual_final_approval: "最後に確認",
      manual_on_review_concern: "懸念がある時だけ確認",
      auto_complete_on_review_pass: "レビュー合格で自動完了",
    }[approvalPolicyValue(detail)] || approvalPolicyValue(detail);
  }

  function reviewConcernTexts(detail) {
    const isNoConcern = (value) => {
      const normalized = String(value || "").trim().replace(/[。.!！]+$/, "");
      return /^(none|null|n\/a|no concerns?|なし|特になし|該当なし|懸念なし)$/i.test(normalized);
    };
    const direct = lines(detail.review?.concerns).filter((concern) => !isNoConcern(concern));
    const itemConcerns = lines(detail.review?.items)
      .filter((item) => Boolean(item.concern_note)
        || /fail|reject|ng|concern|warning|needs?_changes?/i.test(item.verdict || ""))
      .map((item) => [item.item, item.concern_note || item.evidence].filter(Boolean).join(": "))
      .filter((concern) => !isNoConcern(concern));
    return unique([...direct, ...itemConcerns]).slice(0, 4);
  }

  function reviewRecommendation(detail) {
    const review = detail.review || {};
    const hasConcerns = reviewConcernTexts(detail).length > 0;
    const verdict = String(review.verdict || "").toLowerCase();
    if (/fail|reject|ng/.test(verdict)) return "差し戻しを推奨";
    if (/pass|ok|approved/.test(verdict)) return "採用を推奨";
    if (hasConcerns) return "採用前に懸念を確認";
    return "内容を確認";
  }

  function actionSurface(detail) {
    if (detail.question) {
      const options = questionOptions(detail);
      return `
        <div class="action-panel" style="margin-top:16px;">
          <h3>質問に回答</h3>
          ${detail.question_source ? `<div class="hint" style="margin-bottom:8px;">質問元: ${escapeHtml(detail.question_source)}</div>` : ""}
          <p class="q-body">${escapeHtml(detail.question)}</p>
          <form data-answer class="field" style="margin:12px 0 0;">
            ${options.length ? `
              <div class="list" style="margin-bottom:10px;">
                ${options.map((option, index) => `
                  <label class="opt" style="margin-bottom:${index === options.length - 1 ? "0" : "6px"};">
                    <input type="radio" name="answer_choice" value="${escapeHtml(option)}" ${index === 0 ? "checked" : ""}>
                    <div><div class="o-name">${escapeHtml(option)}</div></div>
                  </label>
                `).join("")}
              </div>
            ` : ""}
            <textarea name="answer" ${options.length ? "" : "required"} placeholder="${options.length ? "補足があれば書いてください" : "回答を書いてください"}"></textarea>
            <div style="display:flex;justify-content:flex-end;margin-top:10px;"><button class="btn btn-primary" type="submit">回答して再開</button></div>
          </form>
        </div>`;
    }
    if (hasActiveRecovery(detail) && detail.recovery) {
      const accepted = detail.recovery.status === "accepted";
      const failed = recoveryFailedStep(detail);
      return `
        <div class="action-panel recover-panel" style="margin-top:16px;">
          <h3>回復方法を選ぶ</h3>
          <div class="q-body">
            ${failed ? `<b>発生工程:</b> ${escapeHtml(failed.title)} / ${escapeHtml(failed.actor)}（${escapeHtml(failed.state)}）<br>` : ""}
            ${detail.recovery.target_agent ? `<b>回復対象:</b> ${escapeHtml(detail.recovery.target_agent)}<br>` : ""}
            <b>原因:</b> ${escapeHtml(detail.recovery.reason || detail.recovery.failure_class)}<br>
            <b>影響:</b> ${escapeHtml(detail.recovery.impact || "")}<br>
            <b>完了済み:</b> ${escapeHtml(lines(detail.recovery.handoff_completed).join(" / ") || "-")}<br>
            <b>次に渡す内容:</b> ${escapeHtml(lines(detail.recovery.handoff_pending).join(" / ") || detail.recovery.summary || "-")}
          </div>
          ${detail.recovery.warnings?.length ? `<div class="card" style="padding:8px 10px;margin-top:10px;border-color:var(--warning-line);background:var(--warning-soft);">${detail.recovery.warnings.map((warning) => escapeHtml(warning)).join("<br>")}</div>` : ""}
          ${accepted ? `
            <form data-apply-recovery class="field" style="margin:12px 0 0;">
              <textarea name="prompt" placeholder="回復時に追加で伝えること（任意）">${escapeHtml(detail.recovery.prompt_hint || "")}</textarea>
              <div style="display:flex;justify-content:flex-end;margin-top:10px;"><button class="btn btn-primary" type="submit">回復して再開</button></div>
            </form>
          ` : `
            <button class="btn btn-primary" style="margin-top:12px;" data-accept-recovery>この回復案を採用</button>
          `}
        </div>`;
    }
    if (hasActiveRecovery(detail)) {
      const failed = recoveryFailedStep(detail);
      return `
        <div class="action-panel recover-panel" style="margin-top:16px;">
          <h3>回復案を作成</h3>
          <p class="q-body">
            実行が止まっています。原因、影響、引き継ぎ内容を整理した回復案を作成します。<br>
            ${failed ? `<b>発生工程:</b> ${escapeHtml(failed.title)} / ${escapeHtml(failed.actor)}（${escapeHtml(failed.state)}）` : ""}
          </p>
          <button class="btn btn-primary" type="button" data-create-recovery>回復案を作成</button>
        </div>`;
    }
    if (detail.approval_ready || detail.next_action_kind === "approve") {
      const concerns = reviewConcernTexts(detail);
      const artifactNames = lines(detail.artifacts).map((artifact) => artifact.title || artifact.uri).slice(0, 3);
      return `
        <div class="action-panel approval-panel" style="margin-top:16px;">
          <div style="display:flex;align-items:flex-start;gap:14px;">
            <div style="flex:1;min-width:0;">
              <h3>あなたの確認</h3>
              <div class="q-body" style="margin-top:10px;">
                <b>判断:</b> ${escapeHtml(reviewRecommendation(detail))}<br>
                <b>レビュー:</b> ${escapeHtml([detail.review?.score_label, detail.review?.summary].filter(Boolean).join(" — ") || "レビュー結果を確認してください")}<br>
                <b>確認対象:</b> ${escapeHtml(artifactNames.join(" / ") || "成果物はまだありません")}
              </div>
              ${concerns.length ? `
                <div class="mini-rubric" style="margin-top:10px;">
                  ${concerns.map((concern) => `
                    <div class="mr-row warn">
                      <span class="mr-mark">!</span>
                      <span class="mr-name">懸念</span>
                      <span class="mr-evi">${escapeHtml(concern)}</span>
                      <span class="mr-score">確認</span>
                    </div>
                  `).join("")}
                </div>
              ` : `<div class="hint" style="margin-top:8px;">レビュー懸念はありません。採用するとワークは完了になります。</div>`}
              <div class="hint" style="margin-top:8px;">差し戻しコメントは次の実行で担当エージェントへの指示として使われます。</div>
            </div>
            <div style="display:flex;flex-direction:column;gap:8px;min-width:190px;">
              <button class="btn btn-primary" data-approve>この結果を採用する</button>
              <button class="btn btn-danger-soft" data-reject>コメントを付けて差し戻す</button>
            </div>
          </div>
        </div>`;
    }
    return "";
  }

  function recoveryFailedStep(detail) {
    const steps = lines(detail?.steps);
    const failed = [...steps].reverse().find((step) => {
      const text = `${step.state || ""} ${step.summary || ""} ${step.title || ""}`;
      return /失敗|failed|error|blocked|停止/i.test(text);
    });
    if (!failed) return null;
    return {
      title: failed.title || "工程",
      actor: failed.actor || detail?.recovery?.target_agent || "担当未設定",
      state: failed.state || "停止",
    };
  }

  function questionOptions(detail) {
    const raw = detail.question_options || detail.answer_options || detail.options || [];
    return lines(raw).map((option) => {
      if (typeof option === "string") return option;
      return option.label || option.title || option.value || "";
    }).filter(Boolean);
  }

  function openArtifactList(detail) {
    const artifacts = lines(detail.artifacts);
    if (!artifacts.length) return;
    const modal = dynamicModal(`
      <div class="modal" style="width:620px;">
        <div class="modal-head">
          <h3>成果物</h3>
          <div class="m-sub">${escapeHtml(detail.item?.title || "ワーク")} · ${escapeHtml(String(artifacts.length))}件</div>
        </div>
        <div class="modal-body">
          <div class="artifact-link-list">
            ${artifacts.map((artifact, index) => `
              <button class="artifact-list-link" type="button" data-artifact-list-item="${index}">
                <span>${escapeHtml(artifact.title || artifact.uri || `成果物${index + 1}`)}</span>
                <small>${escapeHtml(artifact.uri || "")}</small>
              </button>
            `).join("")}
          </div>
        </div>
        <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button></div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelectorAll("[data-artifact-list-item]").forEach((button) => {
      button.addEventListener("click", () => openArtifactPreview(detail, Number(button.dataset.artifactListItem)));
    });
  }

  async function openArtifactPreview(detail, index) {
    const artifact = (detail.artifacts || [])[index];
    if (!artifact) return;
    const modal = dynamicModal(`
      <div class="modal" style="width:620px;">
        <div class="modal-head">
          <h3>成果物の詳細</h3>
          <div class="m-sub">${escapeHtml(detail.item?.title || "ワーク")}</div>
        </div>
        <div class="modal-body">
          <div class="card" style="padding:12px;">
            <div class="kv"><div class="k">成果物</div><div class="v">${escapeHtml(artifact.title || artifact.uri || "-")}</div></div>
            <div class="kv"><div class="k">保存先</div><div class="v mono">${escapeHtml(artifact.uri || "-")}</div></div>
            <div class="kv"><div class="k">結果要約</div><div class="v">${escapeHtml(detail.answer || detail.item?.result_summary || "-")}</div></div>
          </div>
          <div data-artifact-content style="margin-top:12px;">
            <div class="hint">成果物の内容を読み込んでいます。</div>
          </div>
        </div>
        <div class="modal-foot"><button class="btn btn-primary" type="button" data-close>閉じる</button></div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    const contentEl = modal.querySelector("[data-artifact-content]");
    try {
      const content = await call("read_artifact_content", { request: { root: rootValue(), uri: artifact.uri } });
      contentEl.innerHTML = artifactContentTemplate(content);
    } catch (error) {
      contentEl.innerHTML = `
        <div class="card" style="padding:12px;border-color:var(--warning-line);background:var(--warning-soft);">
          <b style="color:var(--warning);">ファイル内容を読み込めませんでした</b>
          <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">${escapeHtml(String(error))}</div>
        </div>`;
    }
  }

  function artifactContentTemplate(content) {
    const sizeLabel = `${Number(content.size_bytes || 0).toLocaleString()} bytes`;
    return `
      ${content.display_path ? `
        <div class="card" style="padding:10px 12px;margin-bottom:10px;background:#fff;">
          <div class="kv"><div class="k">読み込んだファイル</div><div class="v mono">${escapeHtml(content.display_path)}</div></div>
        </div>
      ` : ""}
      <div class="field" style="margin:0;">
        <div style="display:flex;align-items:center;gap:8px;margin-bottom:6px;">
          <label style="margin:0;">内容プレビュー</label>
          <span style="font-size:12px;color:var(--text-muted);">${escapeHtml(sizeLabel)}${content.truncated ? " · 先頭のみ表示" : ""}</span>
        </div>
        <pre class="mono" style="white-space:pre-wrap;max-height:360px;overflow:auto;border:1px solid var(--border);border-radius:var(--radius);background:#f8fafc;padding:12px;font-size:12px;line-height:1.55;">${escapeHtml(content.content || "内容は空です。")}</pre>
      </div>
      ${content.truncated ? `<div class="hint" style="margin-top:8px;">ファイルが大きいため、先頭だけを表示しています。</div>` : ""}`;
  }

  function reviewSummary(detail) {
    if (!detail.review) return "";
    const review = detail.review;
    const concerns = reviewConcernTexts(detail);
    const artifact = findArtifact(detail.artifact_type_id);
    const domain = findDomain(detail.domain_id || artifact?.domain_id);
    const criteria = reviewCriteriaProgress(review);
    const resolvedReviews = resolvedReviewSummary(detail);
    return `
      <div class="card review-summary" style="margin-top:8px;">
        <div class="review-summary-head">
          <div class="review-total">
            <span>最終レビュー評価</span>
            <strong>${escapeHtml(review.score_label || review.verdict || "未記録")}<small>点</small></strong>
          </div>
          <div class="review-summary-copy">
            <p>${escapeHtml(review.summary || "レビュー要約はありません。")}</p>
            <div>${escapeHtml([criteria, resolvedReviews, concerns.length ? `懸念 ${concerns.length}件` : "懸念なし"].filter(Boolean).join(" · "))}</div>
          </div>
          <div class="review-summary-actions">
            <button class="btn btn-secondary btn-sm" type="button" data-copy-review>レビューをコピー</button>
            ${artifact ? `<button class="btn btn-secondary btn-sm" type="button" data-edit-review-rubric>ルーブリックを編集</button>` : ""}
          </div>
        </div>
        ${artifact ? `<div class="hint" style="margin-top:8px;">${escapeHtml(domain?.name || "ナレッジ")} / ${escapeHtml(artifact.name || artifact.id)} のルーブリックで評価しています。基準を直す場合はナレッジで編集します。</div>` : ""}
        ${review.items?.length ? `<div class="mini-rubric" style="margin-top:12px;">${review.items.map(reviewItem).join("")}</div>` : ""}
      </div>
    `;
  }

  function reviewCriteriaProgress(review) {
    const items = lines(review?.items);
    if (!items.length) return "";
    const passed = items.filter((item) => /^(pass|passed|ok|approved)$/i.test(String(item.verdict || "").trim())).length;
    return `評価項目 ${passed} / ${items.length}件合格`;
  }

  function resolvedReviewSummary(detail) {
    const steps = detailDisplaySteps(detail);
    const finalReviewIndex = steps.findLastIndex((step) => isReviewStep(step));
    if (finalReviewIndex < 0 || reviewRequiresRevision(steps[finalReviewIndex])) return "";
    const resolvedCount = steps
      .slice(0, finalReviewIndex)
      .filter((step) => isReviewStep(step) && reviewRequiresRevision(step))
      .length;
    return resolvedCount ? `途中の要修正 ${resolvedCount}回は対応済み` : "";
  }

  async function copyReviewResult(detail) {
    const text = reviewCopyText(detail);
    if (!text.trim()) {
      toast("コピーできるレビュー結果がありません。", "error");
      return;
    }
    try {
      await copyTextToClipboard(text);
      toast("レビュー結果をコピーしました。");
    } catch (error) {
      toast(`レビュー結果をコピーできませんでした: ${String(error)}`, "error");
    }
  }

  function reviewCopyText(detail) {
    const review = detail?.review;
    if (!review) return "";
    const artifacts = lines(detail.artifacts).map((artifact) => artifact.title || artifact.uri).filter(Boolean);
    const concerns = reviewConcernTexts(detail);
    const sections = [
      "レビュー結果",
      "",
      `ワーク: ${detail.item?.title || ""}`,
      `依頼: ${workRequestText(detail)}`,
      artifacts.length ? `成果物: ${artifacts.join(" / ")}` : "",
      `評価: ${review.score_label || review.verdict || "未記録"}`,
      `判断: ${reviewRecommendation(detail)}`,
      review.summary ? `要約: ${review.summary}` : "",
      concerns.length ? ["懸念:", ...concerns.map((concern) => `- ${concern}`)].join("\n") : "懸念: なし",
    ].filter(Boolean);
    const items = lines(review.items);
    if (items.length) {
      sections.push(
        "",
        "評価項目:",
        ...items.map((item) => {
          const head = [item.item, item.score_label].filter(Boolean).join(" ");
          const evidence = item.evidence || item.concern_note || "";
          const verdict = item.verdict ? `判定: ${item.verdict}` : "";
          return `- ${head}${verdict ? ` / ${verdict}` : ""}${evidence ? `\n  根拠: ${evidence}` : ""}${item.concern_note && item.evidence ? `\n  懸念: ${item.concern_note}` : ""}`;
        }),
      );
    }
    return sections.join("\n");
  }

  async function copyTextToClipboard(text) {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return;
    }
    const textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.left = "-9999px";
    textarea.style.top = "0";
    document.body.appendChild(textarea);
    textarea.select();
    const copied = document.execCommand("copy");
    textarea.remove();
    if (!copied) throw new Error("clipboard unavailable");
  }

  function openReviewRubric(detail) {
    const artifact = findArtifact(detail.artifact_type_id);
    if (!artifact) {
      toast("対象の成果物が見つかりません。ナレッジ一覧から確認してください。", "error");
      return;
    }
    artifactReturnContext = { kind: "work-detail", workId: detail.item?.id };
    openArtifactDialog(artifact);
  }

  function reviewItem(item) {
    const concern = !/pass|ok|approved/i.test(item.verdict || "");
    return `
      <div class="mr-row ${concern ? "warn" : ""}">
        <span class="mr-mark">${concern ? "!" : "✓"}</span>
        <span class="mr-name">${escapeHtml(item.item)}</span>
        <span class="mr-evi">${escapeHtml(item.evidence || item.concern_note || "")}</span>
        <span class="mr-score">${escapeHtml(item.score_label || "")}</span>
      </div>
    `;
  }

  function stepTemplate(step, index, detail = null) {
    const cls = stepClass(step);
    const reviewItems = stepReviewItems(step, detail, index);
    const resolvedReview = reviewIssueResolvedLater(step, index, detail);
    const reviewStep = isReviewStep(step);
    const reviewScore = reviewStepScore(step);
    const reviewCriteria = String(step?.criteria_label || "").trim();
    return `
      <div class="step ${cls}">
        <div class="card step-card">
          <div class="step-top st-click">
            <span class="st-name">${index + 1}. ${escapeHtml(stepTitleLabel(step))}</span>
            <span class="st-agent">担当: <b>${escapeHtml(stepActorLabel(step))}</b></span>
            <span class="st-status-group">
              ${reviewStep ? `<span class="st-score ${reviewScore ? "" : "missing"}">${escapeHtml(reviewScore ? `${reviewScore}点` : "得点未記録")}</span>` : ""}
              ${reviewCriteria ? `<span class="st-criteria">${escapeHtml(reviewCriteria)}</span>` : ""}
              <span class="st-state ${stepBadgeClass(cls)}">${escapeHtml(stepStateLabel(step, cls))}</span>
              ${resolvedReview ? `<span class="step-resolution">後続工程で対応済み</span>` : ""}
            </span>
            ${step.diagnostics ? `<button class="btn btn-ghost btn-sm" type="button" data-step-diagnostics="${index}">診断ログ</button>` : ""}
            <span class="st-toggle">▾</span>
          </div>
          ${reviewItems.length ? `<div class="mini-rubric">${reviewItems.map(reviewItem).join("")}</div>` : ""}
          <div class="step-body">
            ${step.summary ? `<div class="step-why">${escapeHtml(step.summary)}</div>` : ""}
            <div class="step-io">
              ${step.rationale ? `<div class="io-row"><span class="io-k">根拠</span><span class="io-v">${escapeHtml(step.rationale)}</span></div>` : ""}
              ${step.input ? `<div class="io-row"><span class="io-k">入力</span><span class="io-v">${escapeHtml(step.input)}</span></div>` : ""}
              ${step.output ? `<div class="io-row"><span class="io-k">出力</span><span class="io-v">${escapeHtml(step.output)}</span></div>` : ""}
            </div>
            ${step.knowledge_refs?.length ? `<div class="step-know"><span class="sk-label">使用した知識:</span>${step.knowledge_refs.map((ref) => `<button class="chip k" type="button" data-knowledge-ref="${escapeHtml(ref)}">${escapeHtml(ref)}</button>`).join("")}</div>` : ""}
          </div>
        </div>
      </div>
    `;
  }

  function openKnowledgeReference(ref) {
    const artifact = findArtifactByKnowledgeRef(ref);
    const domain = findDomainByLabel(ref) || findDomain(artifact?.domain_id);
    if (artifact && /ルーブリック|rubric|成果物|readme|faq/i.test(String(ref || ""))) {
      openArtifactDialog(artifact);
      return;
    }
    if (domain) {
      openDomainDialog(domain);
      return;
    }
    if (artifact) {
      openArtifactDialog(artifact);
      return;
    }
    renderKnowledge();
    goApp("knowledge-list", "<b>ナレッジ</b>");
    toast("対応するナレッジが見つかりません。ナレッジ一覧から確認してください。", "error");
  }

  function stepReviewItems(step, detail, index) {
    const ownItems = lines(step.review_items);
    if (ownItems.length) return ownItems;
    const steps = detailDisplaySteps(detail);
    const latestReviewIndex = steps.findLastIndex((candidate) => isReviewStep(candidate));
    if (index === latestReviewIndex && isReviewStep(step)) {
      return lines(detail?.review?.items);
    }
    return [];
  }

  function openStepDiagnostics(detail, index) {
    const step = (detail.steps || [])[index];
    if (!step) return;
    const modal = dynamicModal(`
      <div class="modal" style="width:680px;">
        <div class="modal-head">
          <h3>診断ログ</h3>
          <div class="m-sub">${escapeHtml(index + 1)}. ${escapeHtml(stepTitleLabel(step))} / ${escapeHtml(stepActorLabel(step))}</div>
        </div>
        <div class="modal-body">
          <div class="card" style="padding:12px;margin-bottom:10px;">
            <div class="kv"><div class="k">状態</div><div class="v">${escapeHtml(stepStateLabel(step, stepClass(step)))}</div></div>
            ${step.rationale ? `<div class="kv"><div class="k">根拠</div><div class="v">${escapeHtml(step.rationale)}</div></div>` : ""}
            ${step.input ? `<div class="kv"><div class="k">入力</div><div class="v">${escapeHtml(step.input)}</div></div>` : ""}
            ${step.output ? `<div class="kv"><div class="k">出力</div><div class="v">${escapeHtml(step.output)}</div></div>` : ""}
          </div>
          <div class="field" style="margin:0;">
            <label>診断ログ</label>
            <pre class="mono" style="white-space:pre-wrap;max-height:320px;overflow:auto;border:1px solid var(--border);border-radius:var(--radius);background:#f8fafc;padding:12px;font-size:12px;line-height:1.5;">${escapeHtml(step.diagnostics || "診断ログはありません。")}</pre>
          </div>
        </div>
        <div class="modal-foot"><button class="btn btn-primary" type="button" data-close>閉じる</button></div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
  }

  function stepClass(step) {
    const kind = String(step.kind || "").toLowerCase();
    const title = String(step.title || "").toLowerCase();
    const state = String(step.state || "").toLowerCase();
    const outcome = String(step.outcome || "").toLowerCase();
    const isReview = kind === "review" || /review|レビュー/.test(title);

    if (isReview && /revise|request[_ -]?changes|changes[_ -]?requested|要修正|修正要求|差し戻し/.test(outcome)) return "fail";
    if (/fail|error|blocked|rejected|request[_ -]?changes|changes[_ -]?requested|失敗|停止|拒否/.test(state)) return "fail";
    if (/running|progress|処理中|実行中/.test(state)) return "now";
    if (/done|complete|completed|succeeded|passed|approved|accepted|applied|superseded|answered|recorded|完了|成功|合格|採用|記録済み/.test(state)) return "done";
    if (/question|needs_input|waiting|draft|質問|回答待ち|入力待ち/.test(state)) return "wait";

    if (!state) {
      const fallback = `${kind} ${title}`;
      if (/fail|error|失敗|停止/.test(fallback)) return "fail";
      if (/running|progress|処理中|実行中/.test(fallback)) return "now";
      if (/done|complete|完了/.test(fallback)) return "done";
      if (/question|質問|回答待ち|差し戻し|回復/.test(fallback)) return "wait";
    }
    return "";
  }

  function isReviewStep(step) {
    const kind = String(step?.kind || "").toLowerCase();
    const title = String(step?.title || "").toLowerCase();
    return kind === "review" || /review|レビュー/.test(title);
  }

  function reviewStepScore(step) {
    if (!isReviewStep(step)) return "";
    const explicit = String(step?.score_label || "").trim();
    const source = explicit || String(step?.output || "").match(/\d+\s*\/\s*\d+/)?.[0] || "";
    const match = source.match(/(\d+)\s*\/\s*(\d+)/);
    if (!match) return "";
    const total = Number(match[1]);
    const max = Number(match[2]);
    if (!Number.isFinite(total) || !Number.isFinite(max) || max !== 100) return "";
    return `${Math.min(100, total)} / 100`;
  }

  function reviewRequiresRevision(step) {
    const outcome = String(step?.outcome || "").toLowerCase();
    const state = String(step?.state || "").toLowerCase();
    return /revise|request[_ -]?changes|changes[_ -]?requested|要修正|修正要求|差し戻し/.test(`${outcome} ${state}`);
  }

  function reviewIssueResolvedLater(step, index, detail) {
    if (!isReviewStep(step) || !reviewRequiresRevision(step)) return false;
    return detailDisplaySteps(detail)
      .slice(index + 1)
      .some((laterStep) => isReviewStep(laterStep) && !reviewRequiresRevision(laterStep) && stepClass(laterStep) === "done");
  }

  function stepBadgeClass(cls) {
    if (cls === "done") return "badge badge-done";
    if (cls === "fail") return "badge badge-fix";
    if (cls === "wait") return "badge badge-ask";
    if (cls === "now") return "badge badge-run";
    return "badge badge-neutral";
  }

  function stepTitleLabel(step) {
    const title = String(step?.title || "工程").trim();
    if (/^(オーガナイザーまとめ|organizer summary)$/i.test(title)) return "最終結果をまとめる";
    return title;
  }

  function stepActorLabel(step) {
    const actor = String(step?.actor || "").trim();
    const normalized = actor.toLowerCase();
    return {
      dispatcher: "整理役",
      organizer: "オーガナイザー",
      worker: "作業担当",
      reviewer: "レビュー担当",
      supervisor: "進行管理",
    }[normalized] || actor || roleLabel(step?.kind) || "担当未設定";
  }

  function stepStateLabel(step, cls = stepClass(step)) {
    const stateLabel = String(step?.state || "").trim();
    if (/記録済み|recorded/i.test(stateLabel)) return "記録済み";
    if (cls === "fail" && /revise|request[_ -]?changes|changes[_ -]?requested|要修正|修正要求|差し戻し/i.test(String(step?.outcome || ""))) return "要修正";
    if (/完了|completed?|done|answered|approved/i.test(stateLabel)) return "完了";
    if (/処理中|実行中|running|progress/i.test(stateLabel)) return "処理中";
    if (/質問|回答待ち|入力待ち|needs_input|waiting/i.test(stateLabel)) return "入力待ち";
    if (/request[_ -]?changes|changes[_ -]?requested/i.test(stateLabel)) return "要修正";
    if (/失敗|error|failed|blocked|rejected/i.test(stateLabel)) return "失敗";
    if (stateLabel) return stateLabel;
    return { done: "完了", now: "処理中", wait: "入力待ち", fail: "失敗" }[cls] || "未開始";
  }

  async function advanceWork(id, options = {}, trigger = null) {
    const button = trigger instanceof HTMLElement ? trigger : null;
    let progressPanel = null;
    try {
      if (button) {
        button.disabled = true;
        button.dataset.originalText = button.textContent || "次の判断点まで進める";
        button.textContent = "進行中...";
        const area = button.closest("[data-advance-area]") || button.parentElement;
        if (area) {
          progressPanel = document.createElement("div");
          progressPanel.dataset.advanceProgress = "true";
          progressPanel.className = "card advance-progress";
          progressPanel.innerHTML = `
            <div style="font-weight:800;color:var(--text);font-size:13px;">次の判断点まで処理しています</div>
            <div style="color:var(--text-muted);font-size:12.5px;margin-top:3px;">エージェント実行、レビュー、整理工程が終わるまでこのまま待ってください。</div>
          `;
          area.prepend(progressPanel);
        }
      }
      toast("ワークを進めています。");
      const detail = await call("advance_work", { request: { root: rootValue(), id, max_steps: 8, ...options } });
      activeDetail = detail;
      await loadState({ navigate: false });
      renderDetail(detail);
      goDetail(detail);
      toast("ワークを進めました。");
    } catch (error) {
      progressPanel?.remove();
      if (button) {
        button.disabled = false;
        button.textContent = button.dataset.originalText || "次の判断点まで進める";
      }
      showOperationError(document.querySelector(".screen.active"), "ワークを進められませんでした", error, "現在の工程と実行環境を確認してから、もう一度進めてください。");
      toast(String(error), "error");
    }
  }

  async function advanceWorkInList(id, options = {}) {
    try {
      toast("ワークを進めています。");
      const detail = await call("advance_work", { request: { root: rootValue(), id, max_steps: 8, ...options } });
      activeDetail = detail;
      await loadState({ navigate: false });
      homeWorkProgress = null;
      renderHome();
      goApp("home-active");
      toast("ワークを進めました。");
    } catch (error) {
      homeWorkProgress = null;
      await loadState({ navigate: false }).catch(() => {});
      renderHome();
      goApp("home-active");
      showOperationError(document.getElementById("scr-home-active"), "ワークを進められませんでした", error, "ワーク一覧から詳細を開き、現在の工程と実行環境を確認してください。");
      toast(String(error), "error");
    }
  }

  async function answerQuestion(event, detail) {
    event.preventDefault();
    try {
      const form = new FormData(event.currentTarget);
      const selected = String(form.get("answer_choice") || "").trim();
      const note = String(form.get("answer") || "").trim();
      const answer = [selected, note].filter(Boolean).join("\n");
      if (!answer) {
        toast("回答を入力してください。", "error");
        return;
      }
      const updated = await call("answer_work", { request: { root: rootValue(), id: detail.item.id, question: detail.question, answer } });
      await loadState({ navigate: false });
      renderDetail(updated);
      goDetail(updated);
      toast("回答を保存しました。");
    } catch (error) {
      showOperationError(event.currentTarget, "回答を保存できませんでした", error, "回答内容と現在のワーク状態を確認してから、もう一度送信してください。");
      toast(String(error), "error");
    }
  }

  async function createRecovery(detail) {
    try {
      const updated = await call("create_work_recovery", { request: { root: rootValue(), id: detail.item.id } });
      await loadState({ navigate: false });
      renderDetail(updated);
      goDetail(updated);
      toast("回復案を作成しました。");
    } catch (error) {
      showOperationError(document.querySelector(".screen.active"), "回復案を作成できませんでした", error, "失敗した工程と実行環境の状態を確認してから、もう一度作成してください。");
      toast(String(error), "error");
    }
  }

  async function acceptRecovery(detail) {
    try {
      const updated = await call("accept_work_recovery", { request: { root: rootValue(), id: detail.item.id, recovery_plan_id: detail.recovery?.id } });
      await loadState({ navigate: false });
      renderDetail(updated);
      goDetail(updated);
      toast("回復案を採用しました。");
    } catch (error) {
      showOperationError(document.querySelector(".screen.active"), "回復案を採用できませんでした", error, "回復案の状態と現在のワーク状態を確認してから、もう一度採用してください。");
      toast(String(error), "error");
    }
  }

  async function applyRecovery(event, detail) {
    event.preventDefault();
    try {
      const form = new FormData(event.currentTarget);
      const updated = await call("apply_work_recovery", {
        request: {
          root: rootValue(),
          id: detail.item.id,
          recovery_plan_id: detail.recovery?.id,
          prompt: String(form.get("prompt") || ""),
        },
      });
      await loadState({ navigate: false });
      renderDetail(updated);
      goDetail(updated);
      toast("回復を適用しました。");
    } catch (error) {
      showOperationError(event.currentTarget, "回復を適用できませんでした", error, "追加指示と回復案の状態を確認してから、もう一度再開してください。");
      toast(String(error), "error");
    }
  }

  function openApprove(detail) {
    const modal = dynamicModal(`
      <div class="modal">
        <div class="modal-head"><h3>この結果を採用する</h3><div class="m-sub">採用すると作業は完了になります。</div></div>
        <form class="modal-body" data-approve-form>
          <div class="field"><label>採用メモ（任意）</label><textarea name="rationale" placeholder="確認内容を残せます"></textarea></div>
          <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>戻る</button><button class="btn btn-primary" type="submit">採用して完了</button></div>
        </form>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-approve-form]").addEventListener("submit", async (event) => {
      event.preventDefault();
      await humanDecision("approve_work", detail, new FormData(event.currentTarget).get("rationale"));
    });
  }

  function openReject(detail) {
    const concerns = reviewConcernTexts(detail);
    const defaultComment = concerns.length
      ? `次の懸念に対応してください:\n${concerns.map((concern) => `- ${concern}`).join("\n")}`
      : "";
    const modal = dynamicModal(`
      <div class="modal">
        <div class="modal-head"><h3>コメントを付けて差し戻す</h3><div class="m-sub">コメントは次の実行への指示として渡されます。</div></div>
        <form class="modal-body" data-reject-form>
          ${concerns.length ? `
            <div class="field">
              <label>引用するレビュー懸念</label>
              ${concerns.map((concern, index) => `
                <label class="opt sel" style="margin-bottom:6px;">
                  <input type="checkbox" name="concern" value="${escapeHtml(concern)}" checked data-reject-concern="${index}">
                  <div><div class="o-name">${escapeHtml(concern)}</div><div class="o-desc">次の実行への指示に含めます</div></div>
                </label>
              `).join("")}
            </div>
          ` : ""}
          <div class="field"><label>コメント</label><textarea name="rationale" required data-auto-draft="true" placeholder="不足している点、直してほしい点を書いてください">${escapeHtml(defaultComment)}</textarea></div>
          <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>戻る</button><button class="btn btn-primary" type="submit">差し戻して再実行</button></div>
        </form>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector('textarea[name="rationale"]')?.addEventListener("input", (event) => {
      event.currentTarget.dataset.autoDraft = "false";
    });
    modal.querySelectorAll("[data-reject-concern]").forEach((checkbox) => {
      checkbox.addEventListener("change", () => updateRejectDraft(modal));
    });
    modal.querySelector("[data-reject-form]").addEventListener("submit", async (event) => {
      event.preventDefault();
      await humanDecision("reject_work", detail, new FormData(event.currentTarget).get("rationale"));
    });
  }

  function updateRejectDraft(modal) {
    const textarea = modal.querySelector('textarea[name="rationale"]');
    if (!textarea || textarea.dataset.autoDraft === "false") return;
    const concerns = [...modal.querySelectorAll('input[name="concern"]:checked')].map((input) => input.value);
    textarea.value = concerns.length
      ? `次の懸念に対応してください:\n${concerns.map((concern) => `- ${concern}`).join("\n")}`
      : "";
  }

  async function humanDecision(command, detail, rationale) {
    try {
      const updated = await call(command, { request: { root: rootValue(), id: detail.item.id, rationale: String(rationale || "") } });
      closeDynamicModal();
      await loadState({ navigate: false });
      renderDetail(updated);
      goDetail(updated);
      toast(command === "approve_work" ? "採用しました。" : "差し戻しました。");
    } catch (error) {
      const modal = document.getElementById("app-dynamic-modal");
      showOperationError(
        modal,
        command === "approve_work" ? "結果を採用できませんでした" : "差し戻しを実行できませんでした",
        error,
        command === "approve_work"
          ? "結果とレビューの状態を確認してから、もう一度採用してください。"
          : "差し戻しコメントと現在のワーク状態を確認してから、もう一度実行してください。",
      );
      toast(String(error), "error");
    }
  }

  function dynamicModal(html) {
    closeDynamicModal();
    const overlay = document.createElement("div");
    overlay.className = "overlay open";
    overlay.id = "app-dynamic-modal";
    overlay.innerHTML = html;
    let backdropClickStarted = false;
    overlay.addEventListener("mousedown", (event) => {
      backdropClickStarted = event.target === overlay && !hasTextSelection();
    });
    overlay.addEventListener("click", (event) => {
      if (event.target === overlay && backdropClickStarted && !hasTextSelection()) closeDynamicModal();
      backdropClickStarted = false;
    });
    document.body.appendChild(overlay);
    return overlay;
  }

  function closeDynamicModal() {
    document.getElementById("app-dynamic-modal")?.remove();
  }

  function hasTextSelection() {
    return Boolean(window.getSelection?.()?.toString().trim());
  }

  function renderProjectScreens() {
    const project = state?.project;
    const list = document.getElementById("scr-project-list");
    if (!list) return;
    const projectSummary = project ? projectListSummary(project) : "";
    const projectStatus = project ? projectWorkStatusSummary(project) : "";
    list.innerHTML = `
      <div style="display:flex;align-items:center;gap:12px;">
        <h2 class="page-title" style="margin-bottom:0;">プロジェクト</h2>
        <span style="flex:1"></span>
        <button class="btn btn-primary" type="button" data-add-project>新規プロジェクト</button>
      </div>
      <p class="page-sub" style="margin-top:4px;">作業フォルダごとに、参加エージェントと知識の範囲を持ちます。</p>
      ${pendingProjectResult ? projectOperationResultPanel(pendingProjectResult) : ""}
      <div class="list">
        ${project ? `
          <div class="list-item">
            <div class="wr-picon">${escapeHtml(project.icon)}</div>
            <div class="wr-body">
              <div class="wr-title">${escapeHtml(project.name)}</div>
              <div class="wr-sum">${escapeHtml(project.root)} · ${escapeHtml(projectSummary)}</div>
              <div style="font-size:12px;color:var(--text-faint);margin-top:4px;">${escapeHtml(projectStatus)}</div>
            </div>
            <div style="display:flex;gap:8px;">
              <button class="btn btn-primary btn-sm" type="button" data-project-open-work>ワークへ</button>
              <button class="btn btn-secondary btn-sm" type="button" data-project-settings>設定</button>
            </div>
          </div>
        ` : `<div class="list-empty">プロジェクトはまだありません。新規プロジェクトから開始してください。</div>`}
      </div>`;
    list.querySelector("[data-add-project]").addEventListener("click", () => openProjectCreateDialog());
    list.querySelector("[data-project-open-work]")?.addEventListener("click", () => {
      renderHome();
      goApp("home-active", "<b>ワーク</b>");
    });
    list.querySelector("[data-project-settings]")?.addEventListener("click", () => openProjectSettingsDialog());
    list.querySelector("[data-clear-project-result]")?.addEventListener("click", () => {
      pendingProjectResult = null;
      renderProjectScreens();
    });
  }

  function projectListSummary(project) {
    const workCount = project.work_count ?? (state?.work_items || []).length;
    const agentCount = project.agent_count ?? (state?.agents || []).length;
    const domainCount = project.domain_count ?? (state?.domains || []).length;
    const artifactCount = project.artifact_type_count ?? (state?.artifact_types || []).length;
    return `ワーク ${workCount}件 · エージェント ${agentCount}件 · ドメイン ${domainCount}件 · 成果物 ${artifactCount}件`;
  }

  function projectWorkStatusSummary(project) {
    const counts = projectStatusCounts(project);
    const attention = (counts.question || 0) + (counts.review || 0) + (counts.recover || 0);
    const running = counts.running || 0;
    const done = counts.done || 0;
    const total = attention + running + done;
    if (!total) return "ワーク状況: まだワークはありません";
    const parts = [];
    if (attention) parts.push(`要対応 ${attention}件`);
    if (running) parts.push(`処理中 ${running}件`);
    if (done) parts.push(`完了 ${done}件`);
    return `ワーク状況: ${parts.join(" · ")}`;
  }

  function projectStatusCounts(project) {
    const result = {};
    const statusCounts = project?.status_counts || [];
    statusCounts.forEach((item) => {
      result[item.kind] = Number(item.count || 0);
    });
    if (Object.keys(result).length) return result;
    (state?.work_items || []).forEach((item) => {
      const kind = item.status_kind || workStatusKind(item.status_label);
      result[kind] = (result[kind] || 0) + 1;
    });
    return result;
  }

  function openProjectCreateDialog(draft = {}) {
    const runtime = availableRuntimes()[0] || null;
    const root = draft.root || "";
    const name = draft.name || projectNameFromRoot(root) || "";
    const icon = draft.icon || "流";
    const modal = dynamicModal(`
      <div class="modal" style="width:540px;">
        <div class="modal-head">
          <h3>新規プロジェクト</h3>
          <div class="m-sub">作業対象のフォルダを選びます。表示名は自動で用意できます。実行環境は現在の接続設定を使います。</div>
        </div>
        <form class="modal-body" data-project-create-form>
          <div class="field">
            <label>対象フォルダ</label>
            <div style="display:flex;gap:8px;">
              <input name="root" type="text" class="mono" value="${escapeHtml(root)}" style="flex:1;" required>
              <button class="btn btn-secondary" type="button" data-choose-folder>選択…</button>
            </div>
            <div class="hint">このフォルダにNagareのプロジェクト設定とワーク履歴を作成します。</div>
          </div>
          <div style="display:grid;grid-template-columns:96px 1fr;gap:12px;">
            <div class="field"><label>アイコン</label><input name="icon" type="text" maxlength="4" value="${escapeHtml(icon)}"></div>
            <div class="field">
              <label>表示名（任意）</label>
              <input name="display_name" type="text" value="${escapeHtml(name)}" placeholder="空欄ならフォルダ名を使います">
              <div class="hint">あとからプロジェクト設定で変更できます。</div>
            </div>
          </div>
          ${runtime ? `
            <div class="card" style="padding:10px 12px;background:#f8fafc;margin-bottom:12px;">
              <div style="font-size:12.5px;font-weight:700;color:var(--text-body);">使用する実行環境: ${escapeHtml(runtime.label || runtime.id)}</div>
              <div style="font-size:12px;color:var(--text-muted);margin-top:2px;">実行環境の変更は、作成後に「実行環境」またはエージェント設定で行います。</div>
            </div>
          ` : `
            <div class="card" style="padding:12px;border-color:var(--warning-line);background:var(--warning-soft);margin-bottom:12px;">
              <b style="color:var(--warning);">利用できる実行環境が見つかりません</b>
              <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">先に実行環境の接続状態を確認してください。</div>
            </div>
          `}
          <div class="modal-foot" style="padding:16px 0 0;">
            <button class="btn btn-secondary" type="button" data-close>閉じる</button>
            <button class="btn btn-primary" type="submit" ${runtime ? "" : "disabled"}>作成</button>
          </div>
        </form>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-choose-folder]").addEventListener("click", async () => {
      try {
        const folder = await call("choose_project_folder");
        if (!folder) return;
        const rootInput = modal.querySelector('input[name="root"]');
        const nameInput = modal.querySelector('input[name="display_name"]');
        rootInput.value = folder;
        if (!nameInput.value.trim()) nameInput.value = projectNameFromRoot(folder);
      } catch (error) {
        showOperationError(modal, "フォルダを選択できませんでした", error, "対象フォルダを直接入力するか、もう一度選択してください。");
        toast(String(error), "error");
      }
    });
    modal.querySelector("[data-project-create-form]").addEventListener("submit", async (event) => {
      event.preventDefault();
      if (!runtime) return;
      const form = new FormData(event.currentTarget);
      const request = {
        root: String(form.get("root") || "").trim(),
        runtime_id: runtime.id,
        display_name: String(form.get("display_name") || "").trim(),
        icon: String(form.get("icon") || "流").trim() || "流",
      };
      if (!request.root) {
        showOperationError(event.currentTarget, "対象フォルダを入力してください", "フォルダが未入力です。", "プロジェクトの作業フォルダを選択または入力してください。");
        return;
      }
      if (!request.display_name) request.display_name = projectNameFromRoot(request.root);
      try {
        state = await call("initialize_project_with_runtime", { request });
        currentRoot = state.root || request.root;
        localStorage.setItem("nagare.root", currentRoot);
        clearProjectScopedUiState();
        closeDynamicModal();
        renderProjectScreens();
        renderSettingsScreens();
        renderHome();
        goApp("home-active");
        toast("プロジェクトを作成しました。");
      } catch (error) {
        showOperationError(event.currentTarget, "プロジェクトを作成できませんでした", error, "名前、対象フォルダ、実行環境の接続状態を確認してから、もう一度作成してください。");
        toast(String(error), "error");
      }
    });
  }

  function projectOperationResultPanel(result) {
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:var(--success-line);background:var(--success-soft);">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">${escapeHtml(result.icon || "流")}</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">保存結果: ${escapeHtml(result.name)}</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">プロジェクト設定を更新しました。次のワークから既定値として反映されます。</div>
            <div style="font-size:12px;color:var(--text-muted);margin-top:6px;">
              整理役: ${escapeHtml(result.organizer)} ·
              進め方: ${escapeHtml(result.workflow)} ·
              確認: ${escapeHtml(result.approval)}
            </div>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-clear-project-result>閉じる</button>
        </div>
      </div>`;
  }

  function openProjectSettingsDialog(initialTabOverride = "basic", proposal = null) {
    const project = state?.project;
    if (!project) return;
    const page = document.getElementById("scr-project-settings");
    if (!page) return;
    const organizerAgents = (state.agents || []).filter((agent) => agent.role === "organizer");
    const draft = projectSettingsDraft || {};
    const organizerAgentId = draft.organizer_agent_id === "__builtin__" ? "" : (draft.organizer_agent_id ?? project.organizer_agent_id ?? "");
    const active = (tab) => tab === initialTabOverride ? "active" : "";
    page.innerHTML = `
      <div style="display:flex;align-items:flex-start;gap:12px;margin-bottom:14px;">
        <div style="flex:1;min-width:0;">
          <h2 class="page-title" style="margin-bottom:0;">${escapeHtml(project.icon || projectIcon())} ${escapeHtml(project.name || projectName())}</h2>
          <p class="page-sub" style="margin:4px 0 0;">プロジェクト名、整理役、参加エージェント、知識、既定の進め方を設定します。</p>
        </div>
        <button class="btn btn-secondary btn-sm" type="button" data-close>プロジェクト一覧へ</button>
      </div>
      <form class="card" style="padding:16px;" data-project-form>
          ${proposalHiddenInputs(proposal)}
          <div class="tabs" style="margin-bottom:16px;">
            <button class="tab ${active("basic")}" type="button" data-project-tab="basic">基本情報</button>
            <button class="tab ${active("organizer")}" type="button" data-project-tab="organizer">整理役</button>
            <button class="tab ${active("agents")}" type="button" data-project-tab="agents">参加エージェント</button>
            <button class="tab ${active("knowledge")}" type="button" data-project-tab="knowledge">知識・成果物</button>
            <button class="tab ${active("policy")}" type="button" data-project-tab="policy">進め方</button>
          </div>

          <div class="tabpane ${active("basic")}" data-project-pane="basic">
            <div style="display:grid;grid-template-columns:96px 1fr;gap:12px;">
              <div class="field"><label>アイコン</label><input name="icon" type="text" maxlength="4" value="${escapeHtml(draft.icon ?? project.icon ?? "流")}"></div>
              <div class="field"><label>プロジェクト名</label><input name="display_name" type="text" value="${escapeHtml(draft.display_name ?? project.name ?? "")}"></div>
            </div>
            <div class="field"><label>対象フォルダ</label><input type="text" class="mono" value="${escapeHtml(project.root || state.root || "")}" readonly></div>
            <div class="hint">対象フォルダはこのプロジェクトの作業場所です。変更が必要な場合は新しいプロジェクトとして作成します。</div>
          </div>

          <div class="tabpane ${active("organizer")}" data-project-pane="organizer">
            <div class="field">
              <label>整理役（オーガナイザー）</label>
              <select name="organizer_agent_id">
                <option value="__builtin__" ${!organizerAgentId ? "selected" : ""}>標準（内蔵オーガナイザー）</option>
                ${organizerAgents.map((agent) => `<option value="${escapeHtml(agent.id)}" ${organizerAgentId === agent.id ? "selected" : ""}>${escapeHtml(agent.name)}</option>`).join("")}
              </select>
              <div class="hint">未設定の場合は標準の内蔵オーガナイザーを使います。専用エージェントは、依頼の分解や担当割り当ての方針を細かく調整したい場合に選びます。</div>
            </div>
            <div class="list">
              <div class="list-item">
                <div class="wr-picon">内</div>
                <div class="wr-body"><div class="wr-title">標準の内蔵オーガナイザー</div><div class="wr-sum">設定なしで使える既定の整理役です。</div></div>
                <span class="badge badge-neutral">${!project.organizer_agent_id ? "使用中" : "候補"}</span>
              </div>
              ${organizerAgents.map((agent) => `
                <div class="list-item">
                  ${agentAvatarMarkup(agent)}
                  <div class="wr-body"><div class="wr-title">${escapeHtml(agent.name)}</div><div class="wr-sum">${escapeHtml(agent.description || "専用オーガナイザー")}</div></div>
                  <span class="badge badge-neutral">${project.organizer_agent_id === agent.id ? "使用中" : "候補"}</span>
                </div>
              `).join("") || `<div class="list-empty">専用オーガナイザー候補はありません。必要な場合はエージェント画面で作成します。</div>`}
            </div>
          </div>

          <div class="tabpane ${active("agents")}" data-project-pane="agents">
            <div style="display:flex;align-items:center;gap:10px;margin-bottom:10px;">
              <div style="flex:1;">
                <h4 style="font-size:13.5px;margin-bottom:3px;">このプロジェクトで使うエージェント</h4>
                <div class="hint" style="margin:0;">ワーク開始時は、整理役が依頼内容・役割・得意分野・担当範囲から自動で割り当てます。</div>
              </div>
              <button class="btn btn-secondary btn-sm" type="button" data-project-add-agent>エージェントを追加</button>
            </div>
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-bottom:12px;">
              <div class="field">
                <label>既定ワーカー</label>
                <select name="work_agent_id">${projectAgentDefaultOptions(draft.work_agent_id ?? project.work_agent, "worker")}</select>
                <div class="hint">通常の作成担当です。依頼内容に合う専用エージェントがあれば整理役が上書きします。</div>
              </div>
              <div class="field">
                <label>既定レビュアー</label>
                <select name="review_agent_id">${projectAgentDefaultOptions(draft.review_agent_id ?? project.review_agent, "reviewer")}</select>
                <div class="hint">通常のレビュー担当です。成果物のルーブリックで評価します。</div>
              </div>
            </div>
            <div class="list">${projectAgentRows(project)}</div>
          </div>

          <div class="tabpane ${active("knowledge")}" data-project-pane="knowledge">
            <div style="display:flex;align-items:center;gap:10px;margin-bottom:10px;">
              <div style="flex:1;">
                <h4 style="font-size:13.5px;margin-bottom:3px;">ドメインと成果物</h4>
                <div class="hint" style="margin:0;">共有資産です。ここでは新しいワークで最初に使う既定値だけを選び、内容の編集はナレッジ画面で行います。</div>
              </div>
              <button class="btn btn-secondary btn-sm" type="button" data-project-open-knowledge>ナレッジを開く</button>
            </div>
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-bottom:12px;">
              <div class="field">
                <label>既定ドメイン</label>
                <select name="default_domain_id" data-project-default-domain>${projectDomainDefaultOptions(draft.default_domain_id ?? project.default_domain_id)}</select>
                <div class="hint">新規ワークの開始確認で最初に使うドメインです。依頼内容から整理役が必要に応じて見直します。</div>
              </div>
              <div class="field">
                <label>既定成果物</label>
                <select name="default_artifact_type_id" data-project-default-artifact>${projectArtifactDefaultOptions(draft.default_artifact_type_id ?? project.default_artifact_type_id, draft.default_domain_id ?? project.default_domain_id)}</select>
                <div class="hint">レビュー基準と作成指示の初期候補です。ワークごとに開始確認で確認できます。</div>
              </div>
            </div>
            <div class="list">${projectKnowledgeRows()}</div>
          </div>

          <div class="tabpane ${active("policy")}" data-project-pane="policy">
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
            <div class="field"><label>進め方</label><select name="workflow_mode">${workflowModeOptions(draft.workflow_mode ?? project.workflow_mode)}</select></div>
            <div class="field"><label>確認ポリシー</label><select name="approval_policy">${approvalPolicyOptions(draft.approval_policy ?? project.approval_policy)}</select></div>
            </div>
            <div class="hint">ここで選んだ既定値は、新しいワークの開始確認に反映されます。ワークごとに個別変更できます。</div>
            ${proposal ? projectPolicyProposalPanel(proposal) : ""}
          </div>

          <div class="modal-foot">
            <button class="btn btn-danger-soft left" type="button" data-delete-project>プロジェクトを削除</button>
            <button class="btn btn-secondary" type="button" data-close>閉じる</button>
            <button class="btn btn-primary" type="submit">保存</button>
          </div>
      </form>`;
    goApp("project-settings", `プロジェクト / <b>${escapeHtml(project.name || projectName())}</b>`);
    const modal = page;
    modal.querySelectorAll("[data-close]").forEach((button) => button.addEventListener("click", closeProjectSettingsDialog));
    modal.querySelector("[data-project-form]").addEventListener("submit", saveProjectSettingsFromDialog);
    modal.querySelectorAll("[data-project-tab]").forEach((button) => {
      button.addEventListener("click", () => switchProjectTab(modal, button.dataset.projectTab));
    });
    modal.querySelector("[data-project-add-agent]")?.addEventListener("click", () => {
      projectSettingsDraft = projectSettingsDraftFromModal(modal);
      agentReturnContext = { kind: "project", tab: "agents" };
      openAgentCreateDialog();
    });
    modal.querySelector("[data-project-open-knowledge]")?.addEventListener("click", () => {
      projectSettingsDraft = projectSettingsDraftFromModal(modal);
      knowledgeReturnContext = { kind: "project", tab: "knowledge" };
      closeDynamicModal();
      renderKnowledge();
      goApp("knowledge-list", "エージェント / <b>ナレッジ</b>");
    });
    modal.querySelectorAll("[data-project-edit-agent]").forEach((button) => {
      button.addEventListener("click", () => {
        const agent = findAgent(button.dataset.projectEditAgent);
        if (agent) {
          projectSettingsDraft = projectSettingsDraftFromModal(modal);
          agentReturnContext = { kind: "project", tab: "agents" };
          openAgentDialog(agent);
        }
      });
    });
    modal.querySelectorAll("[data-project-open-domain]").forEach((button) => {
      button.addEventListener("click", () => {
        const domain = findDomain(button.dataset.projectOpenDomain);
        if (domain) {
          projectSettingsDraft = projectSettingsDraftFromModal(modal);
          domainReturnContext = { kind: "project", tab: "knowledge" };
          openDomainDialog(domain);
        }
      });
    });
    modal.querySelector("[data-delete-project]").addEventListener("click", openDeleteProjectDialog);
    modal.querySelector("[data-insert-project-policy]")?.addEventListener("click", () => insertProjectPolicyProposal(modal, proposal));
    modal.querySelector("[data-project-default-domain]")?.addEventListener("input", () => refreshProjectDefaultArtifactOptions(modal));
  }

  function projectPolicyProposalPanel(proposal) {
    return `
      <div class="card" style="margin-top:12px;padding:12px 14px;background:var(--warning-soft);border-color:var(--warning-line);">
        <div style="display:flex;gap:10px;align-items:flex-start;">
          <div class="li-icon" style="background:var(--warning-soft);color:var(--warning);">!</div>
          <div style="flex:1;">
            <div class="li-title" style="font-size:13px;">改善提案 — Nagareが実績から検出</div>
            <div class="li-desc"><b>${escapeHtml(proposal.title || "確認ポリシーの提案")}</b> · ${escapeHtml(proposal.summary || proposal.evidence || "")}</div>
            <div class="hint" style="margin-top:6px;">${escapeHtml(proposal.evidence || "")}</div>
          </div>
          <button class="btn btn-primary btn-sm" type="button" data-insert-project-policy>確認ポリシーに反映</button>
        </div>
      </div>`;
  }

  function insertProjectPolicyProposal(modal, proposal) {
    const select = modal.querySelector('select[name="approval_policy"]');
    if (!select) return;
    select.value = approvalPolicyFromProposal(proposal);
    select.dispatchEvent(new Event("input", { bubbles: true }));
    toast("提案を確認ポリシーに反映しました。保存すると既定値になります。");
  }

  function approvalPolicyFromProposal(proposal) {
    const text = `${proposal?.suggested_text || ""} ${proposal?.summary || ""} ${proposal?.title || ""}`;
    if (text.includes("自動")) return "auto_complete_on_review_pass";
    if (text.includes("懸念") || text.includes("重要")) return "manual_on_review_concern";
    return "manual_final_approval";
  }

  function proposalHiddenInputs(proposal) {
    if (!proposal?.id) return "";
    return `
      <input type="hidden" name="improvement_proposal_id" value="${escapeHtml(proposal.id)}">
      <input type="hidden" name="improvement_kind" value="${escapeHtml(proposal.kind || "")}">
      <input type="hidden" name="improvement_title" value="${escapeHtml(proposal.title || "")}">
      <input type="hidden" name="improvement_target_label" value="${escapeHtml(proposal.target_label || "")}">
      <input type="hidden" name="improvement_summary" value="${escapeHtml(proposal.summary || "")}">
      <input type="hidden" name="improvement_evidence" value="${escapeHtml(proposal.evidence || "")}">
    `;
  }

  function proposalRequestFields(form) {
    return {
      improvement_proposal_id: String(form.get("improvement_proposal_id") || "").trim(),
      improvement_kind: String(form.get("improvement_kind") || "").trim(),
      improvement_title: String(form.get("improvement_title") || "").trim(),
      improvement_target_label: String(form.get("improvement_target_label") || "").trim(),
      improvement_summary: String(form.get("improvement_summary") || "").trim(),
      improvement_evidence: String(form.get("improvement_evidence") || "").trim(),
    };
  }

  function reconcileAppliedImprovement(request) {
    const id = String(request?.improvement_proposal_id || "").trim();
    if (!id || !state?.insights) return;
    const proposals = Array.isArray(state.insights.proposals) ? state.insights.proposals : [];
    state.insights.proposals = proposals.filter((proposal) => proposal.id !== id);
    state.insights.proposal_count = state.insights.proposals.length;
  }

  function syncStateChrome() {
    refreshNavigationIndicators();
    renderInsights();
  }

  function projectAgentRows(project) {
    const agents = state?.agents || [];
    if (!agents.length) return `<div class="list-empty">参加エージェントはまだありません。エージェントを追加すると、このプロジェクトの候補になります。</div>`;
    return agents.map((agent) => {
      const isWorker = agent.name === project.work_agent || agent.id === project.work_agent;
      const isReviewer = agent.name === project.review_agent || agent.id === project.review_agent;
      const labels = [
        isWorker ? "既定ワーカー" : "",
        isReviewer ? "既定レビュアー" : "",
        roleLabel(agent.role),
      ].filter(Boolean);
      const domainLabels = lines(agent.domain_ids).map(domainName);
      const artifactLabels = lines(agent.artifact_type_ids).map(artifactName);
      const scope = [...domainLabels, ...artifactLabels].join(" / ") || "全体候補";
      return `
        <div class="list-item">
          ${agentAvatarMarkup(agent)}
          <div class="wr-body">
            <div class="wr-title">${escapeHtml(agent.name || agent.id)} <span class="wr-projtag">${escapeHtml(labels.join(" / "))}</span></div>
            <div class="wr-sum">${escapeHtml(agent.description || "説明なし")} · ${escapeHtml(toolKindLabel(agent.tool_kind))} · ${escapeHtml(scope)}</div>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-project-edit-agent="${escapeHtml(agent.id)}">設定</button>
        </div>
      `;
    }).join("");
  }

  function projectKnowledgeRows() {
    const domains = state?.domains || [];
    if (!domains.length) return `<div class="list-empty">ドメインはまだありません。ナレッジ画面で追加できます。</div>`;
    return domains.map((domain) => {
      const artifacts = (state?.artifact_types || []).filter((artifact) => artifact.domain_id === domain.id);
      const artifactSummary = artifacts.length
        ? artifacts.map((artifact) => `${artifact.name}${artifact.rubric_count ? ` (${artifact.rubric_count}項目)` : ""}`).join(" / ")
        : "成果物なし";
      return `
        <div class="list-item">
          <div class="wr-picon">知</div>
          <div class="wr-body">
            <div class="wr-title">${escapeHtml(domain.name)} <span class="wr-projtag">${escapeHtml(String(artifacts.length))}種別</span></div>
            <div class="wr-sum">${escapeHtml(domain.description || "説明なし")} · ${escapeHtml(artifactSummary)}</div>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-project-open-domain="${escapeHtml(domain.id)}">編集</button>
        </div>
      `;
    }).join("");
  }

  function projectDomainDefaultOptions(current) {
    const selected = String(current || "").trim();
    const domains = state?.domains || [];
    return [
      `<option value="" ${selected ? "" : "selected"}>自動で選ぶ</option>`,
      ...domains.map((domain) => `<option value="${escapeHtml(domain.id)}" ${domain.id === selected ? "selected" : ""}>${escapeHtml(domain.name || domain.id)}</option>`),
    ].join("");
  }

  function projectArtifactDefaultOptions(current, domainId = "") {
    const selected = String(current || "").trim();
    const selectedDomain = String(domainId || "").trim();
    const artifacts = (state?.artifact_types || []).filter((artifact) => !selectedDomain || artifact.domain_id === selectedDomain);
    return [
      `<option value="" ${selected ? "" : "selected"}>自動で選ぶ</option>`,
      ...artifacts.map((artifact) => `<option value="${escapeHtml(artifact.id)}" ${artifact.id === selected ? "selected" : ""}>${escapeHtml(artifact.name || artifact.id)}</option>`),
    ].join("");
  }

  function refreshProjectDefaultArtifactOptions(modal) {
    const domainSelect = modal.querySelector("[data-project-default-domain]");
    const artifactSelect = modal.querySelector("[data-project-default-artifact]");
    if (!domainSelect || !artifactSelect) return;
    const current = artifactSelect.value;
    const domainId = domainSelect.value;
    const currentArtifact = findArtifact(current);
    const nextCurrent = currentArtifact && (!domainId || currentArtifact.domain_id === domainId) ? current : "";
    artifactSelect.innerHTML = projectArtifactDefaultOptions(nextCurrent, domainId);
  }

  function switchProjectTab(modal, tab) {
    modal.querySelectorAll("[data-project-tab]").forEach((button) => button.classList.toggle("active", button.dataset.projectTab === tab));
    modal.querySelectorAll("[data-project-pane]").forEach((pane) => pane.classList.toggle("active", pane.dataset.projectPane === tab));
  }

  function closeProjectSettingsDialog() {
    projectSettingsDraft = null;
    knowledgeReturnContext = null;
    domainReturnContext = null;
    artifactReturnContext = null;
    closeDynamicModal();
    renderProjectScreens();
    goApp("project-list", "<b>プロジェクト</b>");
  }

  function projectSettingsDraftFromModal(modal) {
    const form = modal.querySelector("[data-project-form]");
    if (!form) return null;
    const data = new FormData(form);
    return {
      icon: String(data.get("icon") || "").trim(),
      display_name: String(data.get("display_name") || "").trim(),
      organizer_agent_id: String(data.get("organizer_agent_id") || "__builtin__"),
      work_agent_id: String(data.get("work_agent_id") || "").trim(),
      review_agent_id: String(data.get("review_agent_id") || "").trim(),
      default_domain_id: String(data.get("default_domain_id") || "").trim(),
      default_artifact_type_id: String(data.get("default_artifact_type_id") || "").trim(),
      workflow_mode: String(data.get("workflow_mode") || "confirm_first"),
      approval_policy: String(data.get("approval_policy") || "manual_final_approval"),
    };
  }

  function projectDefaultAgentId(current, role) {
    const agents = state?.agents || [];
    const value = String(current || "").trim();
    const byCurrent = agents.find((agent) => agent.id === value || agent.name === value);
    if (byCurrent) return byCurrent.id;
    return agents.find((agent) => agent.role === role)?.id || "";
  }

  function projectAgentDefaultOptions(current, role) {
    const agents = state?.agents || [];
    const selected = projectDefaultAgentId(current, role);
    const roleAgents = agents.filter((agent) => agent.role === role);
    const options = roleAgents.length ? roleAgents : agents;
    if (!options.length) return `<option value="">候補なし</option>`;
    return options.map((agent) => `<option value="${escapeHtml(agent.id)}" ${agent.id === selected ? "selected" : ""}>${escapeHtml(agent.name || agent.id)}</option>`).join("");
  }

  function workflowModeOptions(current) {
    return [
      ["confirm_first", "確認しながら進める"],
      ["finish_first", "最後まで進めてから確認"],
    ].map(([value, label]) => `<option value="${value}" ${current === value ? "selected" : ""}>${label}</option>`).join("");
  }

  function approvalPolicyOptions(current) {
    return [
      ["manual_final_approval", "最後に確認する"],
      ["manual_on_review_concern", "懸念がある時だけ確認"],
      ["auto_complete_on_review_pass", "レビュー合格で自動完了"],
    ].map(([value, label]) => `<option value="${value}" ${current === value ? "selected" : ""}>${label}</option>`).join("");
  }

  async function saveProjectSettingsFromDialog(event) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const request = {
      root: rootValue(),
      display_name: String(form.get("display_name") || "").trim(),
      icon: String(form.get("icon") || "").trim(),
      organizer_agent_id: String(form.get("organizer_agent_id") || "__builtin__"),
      work_agent_id: String(form.get("work_agent_id") || "").trim(),
      review_agent_id: String(form.get("review_agent_id") || "").trim(),
      default_domain_id: String(form.get("default_domain_id") || "").trim(),
      default_artifact_type_id: String(form.get("default_artifact_type_id") || "").trim(),
      workflow_mode: String(form.get("workflow_mode") || "confirm_first"),
      approval_policy: String(form.get("approval_policy") || "manual_final_approval"),
      ...proposalRequestFields(form),
    };
    try {
      state = await call("save_project_settings", {
        request,
      });
      reconcileAppliedImprovement(request);
      pendingProjectResult = projectSaveResultFromRequest(request);
      projectSettingsDraft = null;
      closeDynamicModal();
      renderProjectScreens();
      renderHome();
      syncStateChrome();
      goApp("project-list", "<b>プロジェクト</b>");
      toast("プロジェクト設定を保存しました。");
    } catch (error) {
      showOperationError(event.currentTarget, "プロジェクト設定を保存できませんでした", error, "入力内容と対象フォルダの状態を確認してから、もう一度保存してください。");
      toast(String(error), "error");
    }
  }

  function projectSaveResultFromRequest(request) {
    const organizer = request.organizer_agent_id && request.organizer_agent_id !== "__builtin__"
      ? findAgent(request.organizer_agent_id)?.name || request.organizer_agent_id
      : "標準の内蔵オーガナイザー";
    const workflow = {
      confirm_first: "開始前に確認",
      finish_first: "最後まで進めてから確認",
    }[request.workflow_mode] || request.workflow_mode || "開始前に確認";
    const approval = {
      manual_final_approval: "最後に確認する",
      manual_on_review_concern: "懸念がある時だけ確認",
      auto_complete_on_review_pass: "レビュー合格で自動完了",
    }[request.approval_policy] || request.approval_policy || "最後に確認する";
    return {
      name: request.display_name || state?.project?.name || "プロジェクト",
      icon: request.icon || state?.project?.icon || projectIcon(),
      organizer,
      worker: findAgent(request.work_agent_id)?.name || request.work_agent_id || state?.project?.work_agent || "",
      reviewer: findAgent(request.review_agent_id)?.name || request.review_agent_id || state?.project?.review_agent || "",
      workflow,
      approval,
    };
  }

  function openDeleteProjectDialog() {
    const project = state?.project;
    const root = project?.root || state?.root || rootValue() || "";
    const agentCount = project?.agent_count ?? (state?.agents || []).length;
    const domainCount = project?.domain_count ?? (state?.domains || []).length;
    const artifactCount = project?.artifact_type_count ?? (state?.artifact_types || []).length;
    const workCount = project?.work_count ?? (state?.work_items || []).length;
    const modal = dynamicModal(`
      <div class="modal">
        <div class="modal-head"><h3>プロジェクトを削除しますか?</h3><div class="m-sub">${escapeHtml(project?.name || "このプロジェクト")} のNagare管理情報を削除します。</div></div>
        <div class="modal-body">
          <div class="card" style="padding:10px 12px;">
            <div class="kv"><div class="k">対象</div><div class="v">${escapeHtml(project?.icon || projectIcon())} ${escapeHtml(project?.name || projectName())}</div></div>
            <div class="kv"><div class="k">対象フォルダ</div><div class="v mono">${escapeHtml(root || "-")}</div></div>
            <div class="kv"><div class="k">削除される情報</div><div class="v">ワーク履歴 ${escapeHtml(String(workCount))}件、プロジェクト設定、整理役と確認ポリシー</div></div>
            <div class="kv"><div class="k">関連設定</div><div class="v">エージェント ${escapeHtml(String(agentCount))}件、ドメイン ${escapeHtml(String(domainCount))}件、成果物 ${escapeHtml(String(artifactCount))}件のプロジェクト紐づけ</div></div>
            <div class="kv"><div class="k">残るもの</div><div class="v">対象フォルダ内の通常ファイルは削除しません</div></div>
          </div>
          <div class="hint" style="margin-top:8px;">削除後は空状態ホームに戻ります。必要になった場合は同じフォルダで新しいプロジェクトを作成できます。</div>
        </div>
        <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button><button class="btn btn-primary" type="button" data-confirm>削除</button></div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-confirm]").addEventListener("click", async () => {
      try {
        state = await call("delete_project", { request: { root: rootValue() } });
        clearProjectScopedUiState();
        clearStoredRoot();
        closeDynamicModal();
        renderEmptyHome();
        refreshNavigationIndicators();
        goApp("home-empty");
        toast("プロジェクトを削除しました。");
      } catch (error) {
        showOperationError(modal, "プロジェクトを削除できませんでした", error, "対象フォルダとNagare管理情報の状態を確認してから、もう一度削除してください。");
        toast(String(error), "error");
      }
    });
  }

  function renderSettingsScreens() {
    renderSkills();
    renderMcp();
    renderRuntimes();
    renderAgents();
    renderKnowledge();
  }

  function renderCatalog() {
    const el = document.getElementById("scr-catalog");
    if (!el) return;
    el.innerHTML = `
      <h2 class="page-title">UIカタログ</h2>
      <p class="page-sub">Nagare の画面で使う基本部品です。実装画面と同じCSSトークンで描画します。</p>
      <div class="group-head" style="margin-top:0;"><h3>アクション</h3></div>
      <div class="card" style="padding:16px;margin-bottom:16px;display:flex;gap:10px;flex-wrap:wrap;align-items:center;">
        <button class="btn btn-primary" type="button">主要アクション</button>
        <button class="btn btn-secondary" type="button">副次アクション</button>
        <button class="btn btn-danger-soft" type="button">削除</button>
        <button class="btn btn-primary" type="button" disabled>実行不可</button>
      </div>
      <div class="group-head"><h3>状態</h3></div>
      <div class="card" style="padding:16px;margin-bottom:16px;display:flex;gap:8px;flex-wrap:wrap;">
        <span class="badge badge-run"><span class="bdot"></span>処理中</span>
        <span class="badge badge-ask"><span class="bdot"></span>要対応</span>
        <span class="badge badge-fix"><span class="bdot"></span>回復が必要</span>
        <span class="badge badge-done">完了</span>
        <span class="badge badge-neutral">未確認</span>
      </div>
      <div class="group-head"><h3>入力</h3></div>
      <div class="card" style="padding:16px;margin-bottom:16px;">
        <div class="field"><label>依頼文</label><textarea placeholder="依頼を書いてください"></textarea><div class="hint">長い入力は広い編集面を使います。</div></div>
        <div class="field"><label>対象プロジェクト</label><select><option>流 Nagare</option><option>社内ドキュメント</option></select></div>
      </div>
      <div class="group-head"><h3>一覧</h3></div>
      <div class="list">
        <div class="list-item">
          <div class="wr-picon">流</div>
          <div class="wr-body"><div class="wr-title">ワーク行</div><div class="wr-sum">結果や次の操作が読める要約を表示します。</div></div>
          <span class="badge badge-ask">要対応</span>
        </div>
        <div class="list-item">
          <div class="wr-picon">評</div>
          <div class="wr-body"><div class="wr-title">レビュー行</div><div class="wr-sum">根拠と懸念を簡潔に表示します。</div></div>
          <span class="badge badge-done">92 / 100</span>
        </div>
      </div>
    `;
  }

  function renderSkills() {
    const el = document.getElementById("scr-settings-skills");
    if (!el || !state) return;
    const pendingSkill = findSkillPackage(pendingSkillFollowup?.id);
    el.innerHTML = `
      <h2 class="page-title">スキル</h2>
      <p class="page-sub">Nagare 全体で使うスキルのライブラリです。割り当て操作はエージェント設定に集約します。</p>
      <div style="display:flex;justify-content:flex-end;margin:0 0 10px;"><button class="btn btn-primary" type="button" data-add-skill>スキルを追加</button></div>
      ${pendingSkill ? skillFollowupPanel(pendingSkill, pendingSkillFollowup) : ""}
      ${pendingSkillDeleteResult ? skillDeleteResultPanel(pendingSkillDeleteResult) : ""}
      <div class="card" style="padding:12px 14px;margin-bottom:12px;background:#f8fafc;">
        <div style="font-size:13px;color:var(--text-body);font-weight:700;">能力の流れ: ライブラリ登録 → エージェント割り当て → ${escapeHtml(projectName())} へ自動反映</div>
        <div style="font-size:12px;color:var(--text-muted);margin-top:3px;">この画面では追加・削除・割り当て状況だけを管理します。実行時は割り当て済みエージェントの専用環境へ渡されます。</div>
      </div>
      <div class="filters">
        <label class="visually-hidden" for="skill-source-filter">追加元</label>
        <select id="skill-source-filter" style="width:auto;min-width:160px;">
          <option value="all">追加元: すべて</option>
          ${unique((state.skill_packages || []).map((pkg) => pkg.source_kind || "manual")).map((kind) => `<option value="${escapeHtml(kind)}">${escapeHtml(sourceKindLabel(kind))}</option>`).join("")}
        </select>
        <label class="visually-hidden" for="skill-search-filter">スキル検索</label>
        <input id="skill-search-filter" type="search" placeholder="スキル名・追加元で検索" style="width:240px;">
        <span style="flex:1"></span>
        <span class="filter-result" id="skill-filter-result">${(state.skill_packages || []).length}件を表示</span>
      </div>
      <div class="list">${(state.skill_packages || []).map((pkg) => `
        <div class="list-item" data-skill-row data-source-kind="${escapeHtml(pkg.source_kind || "manual")}" data-search="${escapeHtml([pkg.id, pkg.source, ...(pkg.provided_skill_sets || [])].join(" ").toLowerCase())}">
          <div class="wr-picon">技</div>
          <div class="wr-body">
            <div class="wr-title">${escapeHtml(pkg.id)} <span class="wr-projtag">${escapeHtml(sourceKindLabel(pkg.source_kind))}</span></div>
            <div class="wr-sum">${escapeHtml(skillListSummary(pkg))}</div>
          </div>
          <div style="display:flex;gap:8px;">
            <button class="btn btn-danger-soft btn-sm" type="button" data-delete-skill="${escapeHtml(pkg.id)}">削除</button>
          </div>
        </div>
      `).join("") || `<div class="list-empty">登録済みスキルはありません。</div>`}<div class="list-empty" id="skill-empty" style="display:none;">条件に一致するスキルはありません。</div></div>`;
    el.querySelector("[data-add-skill]").addEventListener("click", () => openSkillDialog());
    el.querySelector("[data-followup-assign-skill]")?.addEventListener("click", (event) => {
      pendingSkillFollowup = null;
      openCapabilityAssignmentDialog("skill", event.currentTarget.dataset.followupAssignSkill);
    });
    el.querySelector("[data-clear-skill-followup]")?.addEventListener("click", () => {
      pendingSkillFollowup = null;
      renderSkills();
    });
    el.querySelector("[data-clear-skill-delete-result]")?.addEventListener("click", () => {
      pendingSkillDeleteResult = null;
      renderSkills();
    });
    el.querySelector("#skill-source-filter").addEventListener("input", applySkillFilter);
    el.querySelector("#skill-search-filter").addEventListener("input", applySkillFilter);
    el.querySelectorAll("[data-delete-skill]").forEach((button) => {
      button.addEventListener("click", () => openDeleteSkillDialog(button.dataset.deleteSkill));
    });
    applySkillFilter();
  }

  function findSkillPackage(packageId) {
    if (!packageId) return null;
    return (state?.skill_packages || []).find((item) => item.id === packageId) || null;
  }

  function skillListSummary(pkg) {
    const assigned = assignedSkillAgentNames(pkg);
    if (assigned.length) return `割り当て済み: ${assigned.join("、")}`;
    const provided = (pkg.provided_skill_sets || []).join("、");
    return `${provided || "スキルセット未検出"} · まだエージェントに割り当てられていません`;
  }

  function skillFollowupPanel(pkg, followup = {}) {
    const assigned = assignedSkillAgentNames(pkg);
    const countText = Number(followup.count || 0) > 1 ? `${followup.count}件のスキルを追加しました。` : "スキルを追加しました。";
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:var(--primary-line);background:var(--primary-soft);">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">技</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">次の操作: エージェントへ割り当て</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">
              ${escapeHtml(countText)} ${escapeHtml(pkg.id)} を必要なエージェントへ割り当てると、ワーク実行時に利用できます。${assigned.length ? ` 現在の割り当て: ${escapeHtml(assigned.join("、"))}` : ""}
            </div>
          </div>
          <div style="display:flex;gap:8px;align-items:center;">
            <button class="btn btn-primary btn-sm" type="button" data-followup-assign-skill="${escapeHtml(pkg.id)}">エージェントへ割り当て</button>
            <button class="btn btn-secondary btn-sm" type="button" data-clear-skill-followup>後で</button>
          </div>
        </div>
      </div>`;
  }

  function skillDeleteResultPanel(result) {
    const detached = (result.detached_agents || []).map((id) => findAgent(id)?.name || id).filter(Boolean);
    const removedSets = result.removed_skill_sets || [];
    const warnings = result.warnings || [];
    const bodyText = result.remove_installed_body
      ? result.installed_body_removed
        ? "スキル本体も削除しました"
        : "登録と割り当てを外しました。本体ファイルの削除対象はありませんでした"
      : "登録と割り当てだけを外しました。本体ファイルは残しています";
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:${warnings.length ? "var(--warning-line)" : "var(--success-line)"};background:${warnings.length ? "var(--warning-soft)" : "var(--success-soft)"};">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">技</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">削除結果: ${escapeHtml(result.package_id)}</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">${escapeHtml(bodyText)}</div>
            <div style="font-size:12px;color:var(--text-muted);margin-top:6px;">
              ${removedSets.length ? `削除したスキル: ${escapeHtml(removedSets.join("、"))}` : "削除したスキル: なし"} ·
              ${detached.length ? `割り当て解除: ${escapeHtml(detached.join("、"))}` : "割り当て解除: 対象なし"}
            </div>
            ${warnings.length ? `
              <div class="card" style="padding:8px 10px;margin-top:10px;border-color:var(--warning-line);background:#fff;">
                <b style="font-size:12px;color:var(--warning);">確認が必要なこと</b>
                <div style="font-size:12px;color:var(--text-body);margin-top:4px;">${warnings.map((warning) => escapeHtml(warning)).join("<br>")}</div>
              </div>
            ` : ""}
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-clear-skill-delete-result>閉じる</button>
        </div>
      </div>`;
  }

  function applySkillFilter() {
    const sourceKind = document.getElementById("skill-source-filter")?.value || "all";
    const query = (document.getElementById("skill-search-filter")?.value || "").trim().toLowerCase();
    let count = 0;
    document.querySelectorAll("[data-skill-row]").forEach((row) => {
      const sourceVisible = sourceKind === "all" || row.dataset.sourceKind === sourceKind;
      const queryVisible = !query || row.dataset.search.includes(query);
      const visible = sourceVisible && queryVisible;
      row.style.display = visible ? "" : "none";
      if (visible) count += 1;
    });
    const result = document.getElementById("skill-filter-result");
    const empty = document.getElementById("skill-empty");
    if (result) result.textContent = `${count}件を表示`;
    if (empty) empty.style.display = count ? "none" : "";
  }

  function renderMcp() {
    const el = document.getElementById("scr-settings-mcp");
    if (!el || !state) return;
    const pendingMcp = findMcp(pendingMcpFollowupId);
    el.innerHTML = `
      <h2 class="page-title">MCP接続</h2>
      <p class="page-sub">接続済みMCPを確認します。割り当てはエージェント設定で行います。</p>
      <div style="display:flex;justify-content:flex-end;margin:0 0 10px;"><button class="btn btn-primary" type="button" data-add-mcp>MCPを追加</button></div>
      ${pendingMcp ? mcpFollowupPanel(pendingMcp) : ""}
      ${pendingMcpResult ? mcpOperationResultPanel(pendingMcpResult) : ""}
      <div class="filters">
        <label class="visually-hidden" for="mcp-status-filter">状態</label>
        <select id="mcp-status-filter" style="width:auto;min-width:150px;">
          <option value="all">状態: すべて</option>
          <option value="passed">検証済み</option>
          <option value="untested">未テスト</option>
          <option value="failed">失敗</option>
        </select>
        <label class="visually-hidden" for="mcp-tool-filter">対象ランタイム</label>
        <select id="mcp-tool-filter" style="width:auto;min-width:170px;">
          <option value="all">ランタイム: すべて</option>
          ${unique((state.mcp_connections || []).map((mcp) => mcp.tool_kind || "codex_cli")).map((tool) => `<option value="${escapeHtml(tool)}">${escapeHtml(toolKindLabel(tool))}</option>`).join("")}
        </select>
        <label class="visually-hidden" for="mcp-search-filter">MCP検索</label>
        <input id="mcp-search-filter" type="search" placeholder="名前・ランタイムで検索" style="width:220px;">
        <span style="flex:1"></span>
        <span class="filter-result" id="mcp-filter-result">${(state.mcp_connections || []).length}件を表示</span>
      </div>
      <div class="list">${(state.mcp_connections || []).map((mcp) => `
        <div class="list-item" data-mcp-row data-status="${escapeHtml(normalizeMcpStatus(mcp.test_status))}" data-tool-kind="${escapeHtml(mcp.tool_kind || "codex_cli")}" data-search="${escapeHtml([mcp.id, mcp.name, mcp.command, ...(mcp.args || []), mcp.runtime_label, mcp.test_detail].join(" ").toLowerCase())}">
          <div class="wr-picon">M</div>
          <div class="wr-body">
            <div class="wr-title">${escapeHtml(mcp.name || mcp.id)} <span class="${mcpPassed(mcp) ? "badge badge-done" : "badge badge-neutral"}">${escapeHtml(testStatusLabel(mcp.test_status))}</span></div>
            <div class="wr-sum">${escapeHtml(mcpListSummary(mcp))}</div>
            <div style="font-size:11.5px;color:${mcpCanAssignToAgent(mcp) ? "var(--text-faint)" : "var(--warning)"};margin-top:3px;">${escapeHtml(mcpAssignmentSummary(mcp))}</div>
          </div>
          <div style="display:flex;gap:8px;">
            <button class="btn btn-secondary btn-sm" type="button" data-edit-mcp="${escapeHtml(mcp.id)}">編集</button>
            <button class="btn btn-secondary btn-sm" type="button" data-test-mcp="${escapeHtml(mcp.id)}">接続テスト</button>
            <button class="btn btn-danger-soft btn-sm" type="button" data-delete-mcp="${escapeHtml(mcp.id)}">削除</button>
          </div>
        </div>
      `).join("") || `<div class="list-empty">登録済みMCP接続はありません。</div>`}<div class="list-empty" id="mcp-empty" style="display:none;">条件に一致するMCP接続はありません。</div></div>`;
    el.querySelector("[data-add-mcp]").addEventListener("click", () => openMcpDialog());
    el.querySelector("[data-followup-test-mcp]")?.addEventListener("click", (event) => testMcpConnection(event.currentTarget.dataset.followupTestMcp));
    el.querySelector("[data-followup-assign-mcp]")?.addEventListener("click", (event) => {
      pendingMcpFollowupId = null;
      openCapabilityAssignmentDialog("mcp", event.currentTarget.dataset.followupAssignMcp);
    });
    el.querySelector("[data-clear-mcp-followup]")?.addEventListener("click", () => {
      pendingMcpFollowupId = null;
      renderMcp();
    });
    el.querySelector("[data-clear-mcp-result]")?.addEventListener("click", () => {
      pendingMcpResult = null;
      renderMcp();
    });
    el.querySelector("[data-result-edit-mcp]")?.addEventListener("click", (event) => {
      openMcpDialog(findMcp(event.currentTarget.dataset.resultEditMcp));
    });
    el.querySelector("[data-result-retest-mcp]")?.addEventListener("click", (event) => {
      testMcpConnection(event.currentTarget.dataset.resultRetestMcp);
    });
    el.querySelector("[data-result-assign-mcp]")?.addEventListener("click", (event) => {
      pendingMcpFollowupId = null;
      openCapabilityAssignmentDialog("mcp", event.currentTarget.dataset.resultAssignMcp);
    });
    el.querySelector("#mcp-status-filter").addEventListener("input", applyMcpFilter);
    el.querySelector("#mcp-tool-filter").addEventListener("input", applyMcpFilter);
    el.querySelector("#mcp-search-filter").addEventListener("input", applyMcpFilter);
    el.querySelectorAll("[data-edit-mcp]").forEach((button) => {
      button.addEventListener("click", () => openMcpDialog(findMcp(button.dataset.editMcp)));
    });
    el.querySelectorAll("[data-test-mcp]").forEach((button) => {
      button.addEventListener("click", () => testMcpConnection(button.dataset.testMcp));
    });
    el.querySelectorAll("[data-delete-mcp]").forEach((button) => {
      button.addEventListener("click", () => openDeleteMcpDialog(button.dataset.deleteMcp));
    });
    applyMcpFilter();
  }

  function mcpFollowupPanel(mcp) {
    const passed = mcpPassed(mcp);
    const assignable = mcpCanAssignToAgent(mcp);
    const assigned = assignedMcpAgentNames(mcp.id);
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:${passed ? "var(--success-line)" : "var(--warning-line)"};background:${passed ? "var(--success-soft)" : "var(--warning-soft)"};">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">M</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">${passed && assignable ? "次の操作: エージェントへ割り当て" : passed ? "登録済み" : "次の操作: 接続テスト"}</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">
              ${passed && assignable
                ? `${escapeHtml(mcp.name || mcp.id)} は接続テストに成功しました。必要なエージェントへ割り当てると、ワーク実行時に利用できます。${assigned.length ? ` 現在の割り当て: ${escapeHtml(assigned.join("、"))}` : ""}`
                : passed
                  ? `${escapeHtml(mcp.name || mcp.id)} は接続テストに成功しました。${escapeHtml(mcpAssignmentSummary(mcp))}`
                : `${escapeHtml(mcp.name || mcp.id)} は保存されました。接続テストに成功するまで、エージェントへ割り当てできません。`}
            </div>
          </div>
          <div style="display:flex;gap:8px;align-items:center;">
            ${passed && assignable
              ? `<button class="btn btn-primary btn-sm" type="button" data-followup-assign-mcp="${escapeHtml(mcp.id)}">エージェントへ割り当て</button>`
              : passed
                ? ""
              : `<button class="btn btn-primary btn-sm" type="button" data-followup-test-mcp="${escapeHtml(mcp.id)}">接続テストを実行</button>`}
            <button class="btn btn-secondary btn-sm" type="button" data-clear-mcp-followup>後で</button>
          </div>
        </div>
      </div>`;
  }

  function mcpOperationResultPanel(result) {
    const isDelete = result.kind === "delete";
    const ok = result.success !== false;
    const assigned = result.assigned_agents || [];
    const canEdit = !isDelete && Boolean(findMcp(result.id));
    const canRetry = canEdit && !ok;
    const canAssign = canEdit && ok && result.assignable;
    const border = ok ? "var(--success-line)" : result.error ? "var(--danger-line)" : "var(--warning-line)";
    const background = ok ? "var(--success-soft)" : result.error ? "var(--danger-soft)" : "var(--warning-soft)";
    const title = isDelete ? `削除結果: ${result.name || result.id}` : `接続テスト結果: ${result.name || result.id}`;
    const summary = isDelete
      ? "MCP接続をライブラリから削除しました"
      : result.error
        ? "接続テストを実行できませんでした"
      : ok
        ? "接続テストに成功しました"
        : "接続テストに失敗しました";
    const next = isDelete
      ? "必要になった場合は、同じMCPを再登録してください"
      : result.error
        ? "MCP設定、コマンド、権限、実行環境を確認してから再テストしてください"
      : ok
        ? (result.assignable ? "必要なエージェントへ割り当てるとワークで利用できます" : "このランタイムではエージェント個別割り当てはできません")
        : "コマンド、引数、認証情報を確認してから再テストしてください";
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:${border};background:${background};">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">M</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">${escapeHtml(title)}</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">${escapeHtml(summary)}。${escapeHtml(next)}。</div>
            <div style="font-size:12px;color:var(--text-muted);margin-top:6px;">
              ${result.runtime_label ? `対象ランタイム: ${escapeHtml(result.runtime_label)} · ` : ""}
              ${isDelete ? `割り当て解除: ${escapeHtml(assigned.length ? assigned.join("、") : "対象なし")}` : `詳細: ${escapeHtml(result.error || result.detail || "-")}`}
            </div>
          </div>
          <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;justify-content:flex-end;">
            ${canAssign ? `<button class="btn btn-primary btn-sm" type="button" data-result-assign-mcp="${escapeHtml(result.id)}">エージェントへ割り当て</button>` : ""}
            ${canEdit ? `<button class="btn btn-secondary btn-sm" type="button" data-result-edit-mcp="${escapeHtml(result.id)}">設定を編集</button>` : ""}
            ${canRetry ? `<button class="btn btn-secondary btn-sm" type="button" data-result-retest-mcp="${escapeHtml(result.id)}">再テスト</button>` : ""}
            <button class="btn btn-secondary btn-sm" type="button" data-clear-mcp-result>閉じる</button>
          </div>
        </div>
      </div>`;
  }

  function mcpListSummary(mcp) {
    const runtime = mcp.runtime_label || toolKindLabel(mcpToolKind(mcp));
    if (normalizeMcpStatus(mcp.test_status) === "failed") {
      return `${runtime} · 接続テストに失敗しています${mcp.test_detail ? `: ${mcp.test_detail}` : ""}`;
    }
    if (!mcpPassed(mcp)) {
      return `${runtime} · 接続テスト後にエージェントへ割り当てできます`;
    }
    if (!mcpAgentAssignable(mcp)) {
      return `${runtime} · このランタイムではエージェント個別割り当てはできません`;
    }
    return `${runtime} · エージェントへ割り当てできます`;
  }

  function applyMcpFilter() {
    const status = document.getElementById("mcp-status-filter")?.value || "all";
    const toolKind = document.getElementById("mcp-tool-filter")?.value || "all";
    const query = (document.getElementById("mcp-search-filter")?.value || "").trim().toLowerCase();
    let count = 0;
    document.querySelectorAll("[data-mcp-row]").forEach((row) => {
      const statusVisible = status === "all" || row.dataset.status === status;
      const toolVisible = toolKind === "all" || row.dataset.toolKind === toolKind;
      const queryVisible = !query || row.dataset.search.includes(query);
      const visible = statusVisible && toolVisible && queryVisible;
      row.style.display = visible ? "" : "none";
      if (visible) count += 1;
    });
    const result = document.getElementById("mcp-filter-result");
    const empty = document.getElementById("mcp-empty");
    if (result) result.textContent = `${count}件を表示`;
    if (empty) empty.style.display = count ? "none" : "";
  }

  function sourceKindLabel(kind) {
    return {
      openai: "OpenAI",
      anthropic: "Anthropic",
      clawhub: "Clawhub",
      vercel: "Vercel/GitHub",
      github: "GitHub",
      local: "ローカル",
      manual: "手入力",
    }[kind] || kind || "追加元";
  }

  function testStatusLabel(status) {
    return {
      success: "検証済み",
      passed: "検証済み",
      failed: "失敗",
      untested: "未テスト",
      "": "未テスト",
    }[status || ""] || status;
  }

  function findMcp(id) {
    return (state.mcp_connections || []).find((mcp) => mcp.id === id) || null;
  }

  function mcpToolKind(mcp) {
    return mcp?.tool_kind || "codex_cli";
  }

  function runtimeSupportsAgentMcp(toolKind) {
    return ["codex", "codex_cli"].includes(toolKind || "codex_cli");
  }

  function mcpAgentAssignable(mcp) {
    if (typeof mcp?.agent_assignable === "boolean") return mcp.agent_assignable;
    return runtimeSupportsAgentMcp(mcpToolKind(mcp));
  }

  function mcpCanAssignToAgent(mcp) {
    return mcpPassed(mcp) && mcpAgentAssignable(mcp);
  }

  function mcpAssignmentSummary(mcp) {
    if (!mcpAgentAssignable(mcp)) {
      return `${toolKindLabel(mcpToolKind(mcp))}ではエージェント個別のMCP割り当てはできません`;
    }
    if (!mcpPassed(mcp)) return "接続テストに成功するまで割り当てできません";
    return `割り当て: ${assignedMcpAgentNames(mcp.id).join("、") || "未割り当て"}`;
  }

  function skillPackageSkillIds(pkg) {
    return (pkg?.provided_skill_sets?.length ? pkg.provided_skill_sets : [pkg?.id]).filter(Boolean);
  }

  function assignedSkillAgentNames(pkg) {
    const skillIds = new Set(skillPackageSkillIds(pkg));
    return (state.agents || [])
      .filter((agent) => (agent.skill_set_ids || []).some((id) => skillIds.has(id)))
      .map((agent) => agent.name || agent.id)
      .filter(Boolean);
  }

  function assignedMcpAgentNames(id) {
    return (state.agents || [])
      .filter((agent) => (agent.mcp_connection_ids || []).includes(id))
      .map((agent) => agent.name || agent.id)
      .filter(Boolean);
  }

  function arrayValues(value) {
    return Array.isArray(value) ? value.filter(Boolean) : textLines(value);
  }

  function agentSaveRequest(agent, overrides = {}) {
    const toolKind = agent?.tool_kind || runtimeToolKind(agent?.runtime) || "codex_cli";
    const model = agent?.model && agent.model !== "実行環境既定" ? agent.model : "";
    return {
      root: rootValue(),
      id: agent?.id || "",
      display_name: agent?.name || agent?.display_name || agent?.id || "",
      avatar: agent?.avatar || "",
      role: agent?.role || "worker",
      tool_kind: toolKind,
      model,
      model_provider: agent?.model_provider || "",
      model_base_url: agent?.model_base_url || "",
      description: agent?.description || "",
      prompt: agent?.prompt || "",
      specialties: arrayValues(agent?.specialties),
      domain_ids: arrayValues(agent?.domain_ids),
      artifact_type_ids: arrayValues(agent?.artifact_type_ids),
      skill_set_ids: arrayValues(agent?.skill_set_ids),
      mcp_connection_ids: arrayValues(agent?.mcp_connection_ids),
      ...overrides,
    };
  }

  function openCapabilityAssignmentDialog(kind, id) {
    const isMcp = kind === "mcp";
    const mcp = isMcp ? findMcp(id) : null;
    const pkg = isMcp ? null : (state.skill_packages || []).find((item) => item.id === id);
    const skillIds = pkg ? skillPackageSkillIds(pkg) : [];
    const candidates = (state.agents || []).filter((agent) => {
      if (!isMcp) return true;
      return mcpCanAssignToAgent(mcp) && (agent.tool_kind || agent.runtime) === mcpToolKind(mcp);
    });
    const alreadyAssigned = (agent) => isMcp
      ? (agent.mcp_connection_ids || []).includes(id)
      : skillIds.some((skillId) => (agent.skill_set_ids || []).includes(skillId));
    const emptyMessage = isMcp && !mcpAgentAssignable(mcp)
      ? "このMCPは対象ランタイムではエージェントへ個別割り当てできません。"
      : isMcp && !mcpPassed(mcp)
        ? "接続テストに成功するまでエージェントへ割り当てできません。"
        : "このMCPの対象ランタイムに合うエージェントはありません。";
    const modal = dynamicModal(`
      <div class="modal" style="width:600px;">
        <div class="modal-head">
          <h3>エージェントへ割り当て</h3>
          <div class="m-sub">${escapeHtml(isMcp ? (mcp?.name || id) : (pkg?.id || id))} を使うエージェントを選びます。</div>
        </div>
        <form class="modal-body" data-capability-assignment-form>
          <div class="card" style="padding:10px 12px;margin-bottom:10px;background:#f8fafc;">
            <div style="font-size:12.5px;color:var(--text-body);font-weight:700;">反映先: ${escapeHtml(projectIcon())} ${escapeHtml(projectName())}</div>
            <div style="font-size:12px;color:var(--text-muted);margin-top:2px;">選択したエージェントに保存すると、このプロジェクトでそのエージェントが使う能力として自動反映されます。</div>
          </div>
          <div class="list">
            ${candidates.map((agent) => `
              <div class="list-item">
                <input type="checkbox" name="agent_ids" value="${escapeHtml(agent.id)}" ${alreadyAssigned(agent) ? "checked" : ""} aria-label="${escapeHtml(agent.name || agent.id)}へ割り当て">
                ${agentAvatarMarkup(agent)}
                <div class="wr-body">
                  <div class="wr-title">${escapeHtml(agent.name)} <span class="wr-projtag">${escapeHtml(roleLabel(agent.role))}</span></div>
                  <div class="wr-sum">${escapeHtml(toolKindLabel(agent.tool_kind || agent.runtime))} · ${escapeHtml(agent.description || "")}${alreadyAssigned(agent) ? " · 割り当て済み" : ""}</div>
                </div>
                <button class="btn btn-secondary btn-sm" type="button" data-assign-agent="${escapeHtml(agent.id)}">詳細設定</button>
              </div>
            `).join("") || `<div class="list-empty">${isMcp ? escapeHtml(emptyMessage) : "エージェントはまだありません。"}</div>`}
          </div>
          <div class="modal-foot" style="padding:16px 0 0;">
            <button class="btn btn-secondary" type="button" data-close>閉じる</button>
            <button class="btn btn-primary" type="submit" ${candidates.length ? "" : "disabled"}>選択したエージェントへ割り当て</button>
          </div>
        </form>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-capability-assignment-form]").addEventListener("submit", async (event) => {
      event.preventDefault();
      const form = new FormData(event.currentTarget);
      const selectedIds = form.getAll("agent_ids").map(String);
      if (!selectedIds.length) {
        showOperationError(event.currentTarget, "割り当て先を選んでください", "エージェントが未選択です。", "少なくとも1つのエージェントを選択してから保存してください。");
        return;
      }
      try {
        const assignedNames = [];
        for (const agentId of selectedIds) {
          const agent = findAgent(agentId);
          if (!agent) continue;
          const request = isMcp
            ? agentSaveRequest(agent, { mcp_connection_ids: unique([...arrayValues(agent.mcp_connection_ids), id]) })
            : agentSaveRequest(agent, { skill_set_ids: unique([...arrayValues(agent.skill_set_ids), ...skillIds]) });
          state = await call("save_agent", { request });
          assignedNames.push(request.display_name || request.id);
        }
        if (isMcp) pendingMcpFollowupId = null;
        else pendingSkillFollowup = null;
        closeDynamicModal();
        renderSettingsScreens();
        toast(`${assignedNames.join("、")}へ割り当てました。`);
      } catch (error) {
        showOperationError(event.currentTarget, "エージェントへ割り当てできませんでした", error, "対象エージェント、スキル/MCPの状態、実行環境の対応状況を確認してから、もう一度保存してください。");
        toast(String(error), "error");
      }
    });
    modal.querySelectorAll("[data-assign-agent]").forEach((button) => {
      button.addEventListener("click", () => {
        const agent = findAgent(button.dataset.assignAgent);
        const preselect = isMcp ? { mcpId: id } : { skillId: skillIds[0] };
        openAgentDialog(agent, null, "capabilities", preselect);
        toast("保存するとエージェントへ割り当てます。");
      });
    });
  }

  function mcpPassed(mcp) {
    return ["success", "passed"].includes(mcp?.test_status || "");
  }

  function normalizeMcpStatus(status) {
    if (["success", "passed"].includes(status || "")) return "passed";
    if (status === "failed") return "failed";
    return "untested";
  }

  function openSkillDialog() {
    const modal = dynamicModal(`
      <div class="modal" style="width:560px;">
        <div class="modal-head"><h3>スキルを追加</h3><div class="m-sub">追加元を選び、必要な情報だけ入力します。割り当てはエージェント設定で行います。</div></div>
        <form class="modal-body" data-skill-form>
          <div class="field">
            <label>追加元</label>
            <select name="source_kind" data-skill-source>
              <option value="openai">OpenAI リスト</option>
              <option value="anthropic">Anthropic リスト</option>
              <option value="clawhub">Clawhub</option>
              <option value="vercel">Vercel / GitHub</option>
              <option value="local">ローカル</option>
              <option value="manual">手入力</option>
            </select>
            <div class="hint" data-skill-hint></div>
          </div>
          <div class="field" data-skill-list-field>
            <label>追加するスキル</label>
            <div data-skill-search-field hidden style="margin-bottom:8px;">
              <input type="search" data-skill-catalog-search placeholder="Clawhubから検索" style="width:100%;">
            </div>
            <div class="list" style="max-height:180px;overflow:auto;">
              ${skillCatalogOptions("openai")}
            </div>
          </div>
          <div class="field" data-skill-source-field hidden>
            <label>GitHub リポジトリ / パッケージID / パス</label>
            <input name="source" type="text" placeholder="例: hachiware-labs/hachi-search">
          </div>
          <div class="field" data-skill-targets-field>
            <label>インストール先</label>
            <div data-skill-targets></div>
            <div class="hint" data-skill-targets-hint></div>
          </div>
          <div class="field" data-skill-capabilities-field hidden>
            <details>
              <summary style="cursor:pointer;font-weight:700;color:var(--text-body);">詳細設定（任意）</summary>
              <div style="margin-top:10px;">
                <label>必要な能力</label>
                <textarea name="required_capabilities" placeholder="例: repo_read&#10;web_search" style="min-height:64px;"></textarea>
                <div class="hint">通常は空欄で問題ありません。手で追加するスキルに能力条件を付けたい場合だけ入力します。</div>
              </div>
            </details>
          </div>
          <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button><button class="btn btn-primary" type="submit">追加</button></div>
        </form>
      </div>`);
    const select = modal.querySelector("[data-skill-source]");
    const refresh = () => updateSkillDialogSource(modal, select.value);
    select.addEventListener("input", refresh);
    modal.querySelector("[data-skill-catalog-search]").addEventListener("input", () => renderSkillCatalog(modal, select.value));
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-skill-form]").addEventListener("submit", saveSkillFromDialog);
    modal.addEventListener("change", (event) => {
      if (event.target?.name === "install_targets") {
        event.target.closest(".opt")?.classList.toggle("sel", event.target.checked);
      }
    });
    refresh();
  }

  function skillCatalog(kind) {
    return {
      openai: [
        ["openai/code-review", "コードレビュー支援"],
        ["openai/prompt-engineering", "プロンプト改善支援"],
      ],
      anthropic: [
        ["anthropic/artifacts", "成果物作成支援"],
        ["anthropic/research", "調査支援"],
      ],
      clawhub: [
        ["markdown-tools", "Markdownの整形、見出し整理、リンク確認"],
        ["web-search", "Web検索とURL調査の補助"],
        ["browser-automation", "ブラウザ操作と画面確認の補助"],
        ["git-worktree", "Git差分、ブランチ、作業ツリー整理"],
        ["api-client", "API仕様確認とHTTPリクエスト作成"],
      ],
    }[kind] || [];
  }

  function skillCatalogOptions(kind, query = "") {
    const normalized = query.trim().toLowerCase();
    const presets = skillCatalog(kind).filter(([id, desc]) => {
      if (!normalized) return true;
      return `${id} ${desc}`.toLowerCase().includes(normalized);
    });
    if (!presets.length) return `<div class="list-empty">条件に一致するスキルはありません。</div>`;
    const type = kind === "clawhub" ? "radio" : "checkbox";
    return presets.map(([id, desc], index) => `
      <label class="list-item" style="cursor:pointer;">
        <input type="${type}" name="preset_skill" value="${escapeHtml(id)}" ${index === 0 ? "checked" : ""}>
        <div class="wr-body"><div class="wr-title">${escapeHtml(id)}</div><div class="wr-sum">${escapeHtml(desc)}</div></div>
      </label>
    `).join("");
  }

  function renderSkillCatalog(modal, sourceKind) {
    const listField = modal.querySelector("[data-skill-list-field]");
    const searchInput = modal.querySelector("[data-skill-catalog-search]");
    const list = listField.querySelector(".list");
    list.innerHTML = skillCatalogOptions(sourceKind, sourceKind === "clawhub" ? searchInput.value : "");
  }

  function updateSkillDialogSource(modal, sourceKind) {
    const listField = modal.querySelector("[data-skill-list-field]");
    const searchField = modal.querySelector("[data-skill-search-field]");
    const searchInput = modal.querySelector("[data-skill-catalog-search]");
    const sourceField = modal.querySelector("[data-skill-source-field]");
    const targetsField = modal.querySelector("[data-skill-targets-field]");
    const capabilitiesField = modal.querySelector("[data-skill-capabilities-field]");
    const targets = modal.querySelector("[data-skill-targets]");
    const targetsHint = modal.querySelector("[data-skill-targets-hint]");
    const hint = modal.querySelector("[data-skill-hint]");
    const usesCatalog = ["openai", "anthropic", "clawhub"].includes(sourceKind);
    const usesInstaller = ["vercel", "clawhub"].includes(sourceKind);
    listField.hidden = !usesCatalog;
    searchField.hidden = sourceKind !== "clawhub";
    sourceField.hidden = usesCatalog;
    targetsField.hidden = !usesInstaller;
    capabilitiesField.hidden = !["vercel", "local", "manual"].includes(sourceKind);
    if (sourceKind !== "clawhub") searchInput.value = "";
    if (usesCatalog) renderSkillCatalog(modal, sourceKind);
    if (usesInstaller) {
      targets.innerHTML = skillInstallTargetOptions(sourceKind);
      targetsHint.textContent = sourceKind === "clawhub"
        ? "ClawhubはOpenClaw側へ取り込みます。Codexで使う場合はVercel/GitHubまたはローカル追加を使います。"
        : "このプロジェクト内で、選んだ実行環境用のスキル領域へ取り込みます。";
    } else {
      targets.innerHTML = "";
      targetsHint.textContent = "";
    }
    hint.textContent = {
      openai: "定義済みリストから選びます。検索や追加情報は不要です。",
      anthropic: "定義済みリストから選びます。検索や追加情報は不要です。",
      clawhub: "検索して候補から1つ選びます。追加後、必要なエージェントへ割り当てます。",
      vercel: "GitHubリポジトリ形式で入力します。例: hachiware-labs/hachi-search",
      local: "ローカルのスキルフォルダパスを入力します。",
      manual: "パッケージIDまたは参照元を直接入力します。",
    }[sourceKind] || "";
  }

  function skillInstallTargetOptions(sourceKind) {
    const targets = sourceKind === "clawhub"
      ? [["openclaw", "OpenClaw / プロジェクト内", "OpenClaw のスキル領域へ取り込みます"]]
      : [
          ["codex", "Codex / プロジェクト内", "このプロジェクトの Codex エージェントから使えるようにします"],
          ["openclaw", "OpenClaw / プロジェクト内", "このプロジェクトの OpenClaw エージェントから使えるようにします"],
        ];
    return targets.map(([value, name, desc], index) => `
      <label class="opt ${index === 0 ? "sel" : ""}" style="margin-bottom:8px;">
        <input type="checkbox" name="install_targets" value="${escapeHtml(value)}" ${index === 0 ? "checked" : ""}>
        <div><div class="o-name">${escapeHtml(name)}</div><div class="o-desc">${escapeHtml(desc)}</div></div>
      </label>
    `).join("");
  }

  async function saveSkillFromDialog(event) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const sourceKind = String(form.get("source_kind") || "manual");
    const presets = form.getAll("preset_skill").map(String).filter(Boolean);
    const sourceInput = String(form.get("source") || "").trim();
    const sources = ["openai", "anthropic", "clawhub"].includes(sourceKind) ? presets : [sourceInput].filter(Boolean);
    if (!sources.length) {
      toast("追加するスキルを選ぶか、参照元を入力してください。", "error");
      return;
    }
    const installTargets = form.getAll("install_targets").map(String);
    const shouldInstall = ["vercel", "clawhub"].includes(sourceKind);
    if (shouldInstall && !installTargets.length) {
      toast("インストール先を選択してください。", "error");
      return;
    }
    try {
      for (const source of sources) {
        state = await call("add_skill", {
          request: {
            root: rootValue(),
            package_id: source,
            source_kind: sourceKind,
            source,
            path: sourceKind === "local" ? source : null,
            install: shouldInstall,
            install_scope: "project",
            install_targets: shouldInstall ? installTargets : [],
            skill_set_id: source,
            skill_paths: "",
            required_capabilities: String(form.get("required_capabilities") || ""),
            optional_capabilities: "",
          },
        });
      }
      pendingSkillFollowup = { id: sources[sources.length - 1], count: sources.length };
      pendingSkillDeleteResult = null;
      closeDynamicModal();
      renderSettingsScreens();
      refreshNavigationIndicators();
      toast(sources.length === 1 ? "スキルを追加しました。" : `${sources.length}件のスキルを追加しました。`);
    } catch (error) {
      showOperationError(event.currentTarget, "スキルを追加できませんでした", error, "追加元、参照先、インストール先を確認してから、もう一度追加してください。");
      toast(String(error), "error");
    }
  }

  function openDeleteSkillDialog(packageId) {
    const pkg = (state.skill_packages || []).find((item) => item.id === packageId);
    const assigned = pkg ? assignedSkillAgentNames(pkg) : [];
    const modal = dynamicModal(`
      <div class="modal">
        <div class="modal-head"><h3>スキルを削除しますか?</h3><div class="m-sub">${escapeHtml(packageId)} をこのプロジェクトから外します。</div></div>
        <div class="modal-body">
          <div class="card" style="padding:10px 12px;margin-bottom:10px;">
            <div class="kv"><div class="k">割り当て解除</div><div class="v">${escapeHtml(assigned.length ? assigned.join("、") : "対象エージェントなし")}</div></div>
            <div class="kv"><div class="k">履歴</div><div class="v">過去のワーク履歴は残ります</div></div>
          </div>
          <label class="opt sel"><input type="checkbox" name="remove_installed_body" checked><div><div class="o-name">スキル本体も削除する</div><div class="o-desc">このプロジェクト用に取り込んだスキル本体を削除します。エージェント割り当ても外れます。</div></div></label>
        </div>
      <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button><button class="btn btn-primary" type="button" data-confirm>削除</button></div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-confirm]").addEventListener("click", async () => {
      const remove = modal.querySelector('input[name="remove_installed_body"]').checked;
      try {
        const response = await call("delete_skill_package_command", {
          request: { root: rootValue(), package_id: packageId, remove_installed_body: remove },
        });
        state = response.state;
        if (pendingSkillFollowup?.id === packageId) pendingSkillFollowup = null;
        pendingSkillDeleteResult = {
          package_id: response.package_id || packageId,
          removed_skill_sets: response.removed_skill_sets || [],
          detached_agents: response.detached_agents || [],
          installed_body_removed: Boolean(response.installed_body_removed),
          remove_installed_body: remove,
          warnings: response.warnings || [],
        };
        closeDynamicModal();
        renderSettingsScreens();
        toast(response.warnings?.length ? "スキルを削除しました。確認が必要な内容があります。" : "スキルを削除しました。");
      } catch (error) {
        showOperationError(modal, "スキルを削除できませんでした", error, "割り当て状況とスキル本体の保存先を確認してから、もう一度削除してください。");
        toast(String(error), "error");
      }
    });
  }

  function openMcpDialog(mcp = null) {
    const modal = dynamicModal(`
      <div class="modal" style="width:560px;">
        <div class="modal-head"><h3>${mcp ? "MCP接続を編集" : "MCPを追加"}</h3><div class="m-sub">接続はライブラリに登録し、テスト成功後にエージェントへ割り当てます。</div></div>
        <form class="modal-body" data-mcp-form>
          <div class="field"><label>表示名</label><input name="display_name" type="text" value="${escapeHtml(mcp?.name || "")}" placeholder="例: GitHub MCP"></div>
          <div class="field"><label>管理ID（任意）</label><input name="id" type="text" value="${escapeHtml(mcp?.id || "")}" ${mcp ? "readonly" : ""} placeholder="空欄なら表示名から自動生成"><div class="hint">通常は変更不要です。外部設定で固定IDが必要な場合だけ入力します。</div></div>
          <div class="field">
            <label>対象ランタイム</label>
            <select name="tool_kind">
              ${mcpToolKindOptions(mcp?.tool_kind || "codex_cli")}
            </select>
          </div>
          <div class="field"><label>コマンド</label><input name="command" type="text" value="${escapeHtml(mcp?.command || "")}" placeholder="例: npx"></div>
          <div class="field"><label>引数（1行に1つ）</label><textarea name="args" style="min-height:72px;" placeholder="-y&#10;@modelcontextprotocol/server-github">${escapeHtml((mcp?.args || []).join("\n"))}</textarea></div>
          <div class="field"><label>環境変数（任意）</label><textarea name="env" style="min-height:56px;" placeholder="GITHUB_TOKEN=...">${escapeHtml((mcp?.env || []).join("\n"))}</textarea></div>
          <div class="field"><label>テスト引数（任意）</label><textarea name="test_args" style="min-height:48px;" placeholder="--version">${escapeHtml((mcp?.test_args || []).join("\n"))}</textarea></div>
          <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button><button class="btn btn-primary" type="submit">保存</button></div>
        </form>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    const mcpForm = modal.querySelector("[data-mcp-form]");
    bindMcpGeneratedId(mcpForm);
    mcpForm.addEventListener("submit", saveMcpFromDialog);
  }

  function mcpToolKindOptions(current) {
    return [
      ["codex_cli", "Codex CLI"],
      ["claude_code", "Claude Code"],
      ["opencode", "OpenCode"],
      ["openclaw", "OpenClaw"],
    ].map(([value, label]) => `<option value="${value}" ${current === value ? "selected" : ""}>${label}</option>`).join("");
  }

  async function saveMcpFromDialog(event) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const displayName = String(form.get("display_name") || "").trim();
    const command = String(form.get("command") || "").trim();
    const args = String(form.get("args") || "");
    const id = String(form.get("id") || "").trim() || generatedMcpId(displayName, command, args);
    try {
      state = await call("save_mcp_connection", {
        request: {
          root: rootValue(),
          id,
          display_name: displayName,
          tool_kind: String(form.get("tool_kind") || "codex_cli"),
          command,
          args,
          env: String(form.get("env") || ""),
          test_args: String(form.get("test_args") || ""),
        },
      });
      pendingMcpFollowupId = id;
      pendingMcpResult = null;
      closeDynamicModal();
      renderSettingsScreens();
      toast("MCP接続を保存しました。接続テストを実行してください。");
    } catch (error) {
      showOperationError(event.currentTarget, "MCP接続を保存できませんでした", error, "表示名、コマンド、引数、対象ランタイムを確認してから、もう一度保存してください。");
      toast(String(error), "error");
    }
  }

  async function testMcpConnection(id) {
    try {
      const response = await call("test_mcp_connection_command", { request: { root: rootValue(), id } });
      state = response.state;
      pendingMcpFollowupId = id;
      const mcp = findMcp(id);
      pendingMcpResult = {
        kind: "test",
        id,
        name: mcp?.name || id,
        runtime_label: mcp?.runtime_label || toolKindLabel(mcpToolKind(mcp)),
        success: Boolean(response.success),
        detail: response.detail || mcp?.test_detail || "",
        assignable: mcpCanAssignToAgent(mcp),
      };
      renderSettingsScreens();
      toast(response.success ? "MCP接続テストに成功しました。" : `MCP接続テストに失敗しました。${response.detail}`, response.success ? "info" : "error");
    } catch (error) {
      const mcp = findMcp(id);
      pendingMcpFollowupId = id;
      pendingMcpResult = {
        kind: "test",
        id,
        name: mcp?.name || id,
        runtime_label: mcp?.runtime_label || toolKindLabel(mcpToolKind(mcp)),
        success: false,
        error: String(error),
        detail: "",
        assignable: false,
      };
      renderSettingsScreens();
      toast(String(error), "error");
    }
  }

  function openDeleteMcpDialog(id) {
    const mcp = findMcp(id);
    const assigned = assignedMcpAgentNames(id);
    const modal = dynamicModal(`
      <div class="modal">
        <div class="modal-head"><h3>MCP接続を削除しますか?</h3><div class="m-sub">${escapeHtml(mcp?.name || id)} をライブラリから削除します。</div></div>
        <div class="modal-body">
          <div class="card" style="padding:10px 12px;">
            <div class="kv"><div class="k">対象</div><div class="v">${escapeHtml(mcp?.name || id)}${mcp?.runtime_label ? ` · ${escapeHtml(mcp.runtime_label)}` : ""}</div></div>
            <div class="kv"><div class="k">割り当て解除</div><div class="v">${escapeHtml(assigned.length ? assigned.join("、") : "対象エージェントなし")}</div></div>
            <div class="kv"><div class="k">履歴</div><div class="v">過去のワーク履歴と実行記録は残ります</div></div>
          </div>
          <div class="hint" style="margin-top:8px;">削除後、このMCPは新しいワークでは使われません。必要になった場合は再登録してください。</div>
        </div>
        <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button><button class="btn btn-primary" type="button" data-confirm>削除</button></div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-confirm]").addEventListener("click", async () => {
      try {
        const deletedName = mcp?.name || id;
        const deletedRuntime = mcp?.runtime_label || toolKindLabel(mcpToolKind(mcp));
        state = await call("delete_mcp_connection_command", { request: { root: rootValue(), id } });
        if (pendingMcpFollowupId === id) pendingMcpFollowupId = null;
        pendingMcpResult = {
          kind: "delete",
          id,
          name: deletedName,
          runtime_label: deletedRuntime,
          success: true,
          assigned_agents: assigned,
        };
        closeDynamicModal();
        renderSettingsScreens();
        toast("MCP接続を削除しました。");
      } catch (error) {
        showOperationError(modal, "MCP接続を削除できませんでした", error, "割り当て状況とMCP接続の登録状態を確認してから、もう一度削除してください。");
        toast(String(error), "error");
      }
    });
  }

  function renderRuntimes() {
    const el = document.getElementById("scr-settings-runtime");
    if (!el || !state) return;
    el.innerHTML = `
      <h2 class="page-title">実行環境</h2>
      <p class="page-sub">この端末で利用できるランタイムと接続状態を確認します。モデルはエージェントごとに設定します。</p>
      ${pendingRuntimeResult ? runtimeOperationResultPanel(pendingRuntimeResult) : ""}
      <div class="filters">
        <label class="visually-hidden" for="runtime-status-filter">状態</label>
        <select id="runtime-status-filter" style="width:auto;min-width:150px;">
          <option value="all">状態: すべて</option>
          <option value="available">利用可能</option>
          <option value="missing">未検出</option>
        </select>
        <label class="visually-hidden" for="runtime-search-filter">実行環境検索</label>
        <input id="runtime-search-filter" type="search" placeholder="名前・利用エージェントで検索" style="width:240px;">
        <span style="flex:1"></span>
        <span class="filter-result" id="runtime-filter-result">${(state.runtimes || []).length}件を表示</span>
      </div>
      <div class="list">${(state.runtimes || []).map((runtime) => {
        const agentNames = runtimeAgentNames(runtime);
        return `
        <div class="list-item" data-runtime-row data-status="${runtime.available ? "available" : "missing"}" data-search="${escapeHtml([runtime.id, runtime.label, runtime.command, runtime.detail, ...agentNames].join(" ").toLowerCase())}">
          <div class="wr-picon">${runtimeIcon(runtime.id)}</div>
          <div class="wr-body">
            <div class="wr-title">${escapeHtml(runtime.label)} <span class="${runtime.available ? "badge badge-done" : "badge badge-neutral"}">${runtime.available ? "利用可能" : "未検出"}</span></div>
            <div class="wr-sum">${escapeHtml(runtimeGuidance(runtime, agentNames))}</div>
            <div style="font-size:11.5px;color:${runtime.available || !agentNames.length ? "var(--text-faint)" : "var(--warning)"};margin-top:3px;">利用エージェント: ${escapeHtml(agentNames.length ? agentNames.join("、") : "なし")}</div>
            <div style="font-size:11.5px;color:var(--text-faint);margin-top:3px;">モデル: エージェント設定で個別に選択</div>
          </div>
          <div style="display:flex;gap:8px;align-items:center;">
            <button class="btn ${runtime.available ? "btn-secondary" : "btn-primary"} btn-sm" type="button" data-refresh-runtime="${escapeHtml(runtime.id)}">${runtime.available ? "再確認" : "検出を確認"}</button>
          </div>
        </div>
      `;
      }).join("")}<div class="list-empty" id="runtime-empty" style="display:none;">条件に一致する実行環境はありません。</div></div>`;
    el.querySelector("#runtime-status-filter").addEventListener("input", applyRuntimeFilter);
    el.querySelector("#runtime-search-filter").addEventListener("input", applyRuntimeFilter);
    el.querySelectorAll("[data-refresh-runtime]").forEach((button) => {
      button.addEventListener("click", () => refreshRuntime(button.dataset.refreshRuntime));
    });
    el.querySelector("[data-clear-runtime-result]")?.addEventListener("click", () => {
      pendingRuntimeResult = null;
      renderRuntimes();
    });
    applyRuntimeFilter();
  }

  function runtimeOperationResultPanel(result) {
    const ok = result.success !== false;
    const border = ok ? "var(--success-line)" : result.error ? "var(--danger-line)" : "var(--warning-line)";
    const background = ok ? "var(--success-soft)" : result.error ? "var(--danger-soft)" : "var(--warning-soft)";
    const title = `再確認結果: ${result.label || result.id}`;
    const summary = result.error
      ? "再確認を実行できませんでした"
      : ok
        ? "この実行環境は利用できます"
        : "この実行環境はまだ利用できません";
    const detail = `詳細: ${result.error || result.detail || "-"}`;
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:${border};background:${background};">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">${runtimeIcon(result.id)}</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">${escapeHtml(title)}</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">${escapeHtml(summary)}</div>
            <div style="font-size:12px;color:var(--text-muted);margin-top:6px;">${escapeHtml(detail)}</div>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-clear-runtime-result>閉じる</button>
        </div>
      </div>`;
  }

  function findRuntime(id) {
    return (state.runtimes || []).find((runtime) => runtime.id === id) || null;
  }

  function runtimeAgentNames(runtime) {
    const toolKind = runtimeToolKind(runtime?.id);
    return (state.agents || [])
      .filter((agent) => runtimeMatchesAgent(runtime, agent, toolKind))
      .map((agent) => agent.name || agent.id)
      .filter(Boolean);
  }

  function runtimeMatchesAgent(runtime, agent, toolKind = runtimeToolKind(runtime?.id)) {
    const runtimeId = runtime?.id || "";
    const agentRuntime = agent?.runtime || "";
    const agentTool = agent?.tool_kind || runtimeToolKind(agentRuntime);
    return agentTool === toolKind || agentRuntime === runtimeId || runtimeToolKind(agentRuntime) === toolKind;
  }

  function runtimeGuidance(runtime, agentNames) {
    const agents = agentNames.length ? `${agentNames.join("、")} は` : "この実行環境は";
    if (runtime.available) {
      return `${agents}ワークで利用できます。接続が不安定な場合は再確認してください。`;
    }
    if (agentNames.length) {
      return `${runtime.label || runtime.id} が見つかりません。インストール後に検出を確認してください。${agents}接続確認までワークに割り当てられません。`;
    }
    return `${runtime.label || runtime.id} が見つかりません。必要になったらインストール後に検出を確認してください。`;
  }

  function runtimeToolKind(id) {
    return {
      claude: "claude_code",
      codex: "codex_cli",
      opencode: "opencode",
      openclaw: "openclaw",
    }[id] || id;
  }

  function applyRuntimeFilter() {
    const status = document.getElementById("runtime-status-filter")?.value || "all";
    const query = (document.getElementById("runtime-search-filter")?.value || "").trim().toLowerCase();
    let count = 0;
    document.querySelectorAll("[data-runtime-row]").forEach((row) => {
      const statusVisible = status === "all" || row.dataset.status === status;
      const queryVisible = !query || row.dataset.search.includes(query);
      const visible = statusVisible && queryVisible;
      row.style.display = visible ? "" : "none";
      if (visible) count += 1;
    });
    const result = document.getElementById("runtime-filter-result");
    const empty = document.getElementById("runtime-empty");
    if (result) result.textContent = `${count}件を表示`;
    if (empty) empty.style.display = count ? "none" : "";
  }

  async function refreshRuntime(id) {
    try {
      const response = await call("refresh_runtime_status", { request: { root: rootValue(), runtime_id: id } });
      state = response.state || state;
      if (response.runtime) {
        const runtimes = state.runtimes || [];
        state.runtimes = runtimes.map((runtime) => runtime.id === response.runtime.id ? response.runtime : runtime);
      }
      pendingRuntimeResult = {
        kind: "refresh",
        id,
        label: response.runtime?.label || id,
        success: Boolean(response.runtime?.available),
        detail: response.runtime?.detail || "",
      };
      renderRuntimes();
      toast(`${response.runtime?.label || id} を再確認しました。`);
    } catch (error) {
      const runtime = findRuntime(id);
      pendingRuntimeResult = {
        kind: "refresh",
        id,
        label: runtime?.label || id,
        success: false,
        error: String(error),
      };
      renderRuntimes();
      toast(String(error), "error");
    }
  }

  function openRuntimeModelDialog(runtime) {
    if (!runtime) return;
    const switchable = runtimeModelSwitchable(runtime);
    const isProviderRuntime = switchable && ["opencode"].includes(runtime.id);
    const agentNames = runtimeAgentNames(runtime);
    const modal = dynamicModal(`
      <div class="modal" style="width:640px;">
        <div class="modal-head">
          <h3>${escapeHtml(runtime.label)} のモデル設定</h3>
          <div class="m-sub">${switchable ? "この実行環境を使うエージェントへモデル設定を適用します。" : "この実行環境はNagareからのモデル切替に対応していません。"}</div>
        </div>
        <form class="modal-body" data-runtime-model-form>
          <div class="card" style="padding:10px 12px;margin-bottom:12px;">
            <div class="kv"><div class="k">対象</div><div class="v">${escapeHtml(agentNames.length ? agentNames.join("、") : "対象エージェントなし")}</div></div>
            <div class="kv"><div class="k">現在</div><div class="v">${escapeHtml(runtimeConfiguredModel(runtime) || "実行環境既定")}</div></div>
            <div class="kv"><div class="k">切替</div><div class="v">${escapeHtml(switchable ? "Nagareから設定できます" : "実行環境側の設定を使用")}</div></div>
          </div>
          ${!switchable ? `
            <div class="card empty-subtle" style="margin-bottom:12px;">${escapeHtml(runtime.label)} はモデルの切替を実行環境側で行います。Nagareでは利用エージェントとの関係だけを管理します。</div>
            <input name="model_provider" type="hidden" value="">
            <input name="model" type="hidden" value="">
            <input name="model_base_url" type="hidden" value="">
          ` : isProviderRuntime ? `
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
              <div class="field"><label>Provider</label><select name="model_provider" data-runtime-provider>${["", "OpenAI", "Ollama", "LMStudio"].map((value) => `<option value="${escapeHtml(value)}" ${runtime.configured_provider === value ? "selected" : ""}>${escapeHtml(value || "実行環境既定")}</option>`).join("")}</select></div>
              <div class="field"><label>モデル</label><input name="model" type="text" value="${escapeHtml(runtime.configured_model || "")}" placeholder="空なら実行環境既定"></div>
              <div class="field" data-runtime-base-url-field style="grid-column:1 / -1;"><label>Base URL</label><input name="model_base_url" type="text" value="${escapeHtml(runtime.configured_base_url || "")}" placeholder="例: http://localhost:11434"></div>
            </div>
          ` : `
            <input name="model_provider" type="hidden" value="">
            <input name="model_base_url" type="hidden" value="">
            ${runtimeModelSelect(runtime)}
          `}
          <div class="hint">${switchable ? "保存すると、対象エージェントのモデル設定が更新されます。エージェント個別の設定はあとから上書きできます。" : "モデル名をNagareへ重複入力しないため、ここでは保存操作を行いません。"}</div>
          <div class="modal-foot">
            <button class="btn btn-secondary left" type="button" data-reset-runtime-model ${agentNames.length && switchable ? "" : "disabled"}>実行環境既定に戻す</button>
            <button class="btn btn-secondary" type="button" data-close>閉じる</button>
            <button class="btn btn-primary" type="submit" ${agentNames.length && switchable ? "" : "disabled"}>対象エージェントへ適用</button>
          </div>
        </form>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    bindRuntimeProviderBaseUrlToggle(modal);
    bindRuntimeModelChoice(modal);
    modal.querySelector("[data-reset-runtime-model]")?.addEventListener("click", () => {
      const modelChoice = modal.querySelector('[data-runtime-model-choice]');
      if (modelChoice) {
        modelChoice.value = "";
        modelChoice.dispatchEvent(new Event("input", { bubbles: true }));
      } else {
        const model = modal.querySelector('[name="model"]');
        if (model) model.value = "";
      }
      const customModel = modal.querySelector('input[name="model_custom"]');
      if (customModel) customModel.value = "";
      const provider = modal.querySelector('[name="model_provider"]');
      if (provider) provider.value = "";
      const baseUrl = modal.querySelector('input[name="model_base_url"]');
      if (baseUrl) baseUrl.value = "";
      provider?.dispatchEvent(new Event("input", { bubbles: true }));
      modal.querySelector("[data-runtime-model-form]").requestSubmit();
    });
    modal.querySelector("[data-runtime-model-form]").addEventListener("submit", (event) => saveRuntimeModelDefaults(event, runtime));
  }

  function bindRuntimeProviderBaseUrlToggle(modal) {
    const provider = modal.querySelector("[data-runtime-provider]");
    const field = modal.querySelector("[data-runtime-base-url-field]");
    if (!provider || !field) return;
    const update = () => {
      const needsBaseUrl = ["Ollama", "LMStudio"].includes(provider.value);
      field.hidden = !needsBaseUrl;
      if (!needsBaseUrl) field.querySelector('input[name="model_base_url"]').value = "";
    };
    provider.addEventListener("input", update);
    update();
  }

  function runtimeModelSelect(runtime) {
    const choices = modelChoices(runtime);
    const configured = runtime.configured_model || "";
    const usesCustomValue = Boolean(configured) && !choices.includes(configured);
    return `
      <div class="field">
        <label>モデル</label>
        <select name="model" data-runtime-model-choice>
          <option value="" ${configured ? "" : "selected"}>実行環境既定</option>
          ${choices.map((choice) => `<option value="${escapeHtml(choice)}" ${configured === choice ? "selected" : ""}>${escapeHtml(choice)}</option>`).join("")}
          <option value="__custom__" ${usesCustomValue ? "selected" : ""}>手入力...</option>
        </select>
      </div>
      <div class="field" data-runtime-custom-model-field ${usesCustomValue ? "" : "hidden"}>
        <label>モデル名</label>
        <input name="model_custom" type="text" value="${escapeHtml(usesCustomValue ? configured : "")}" placeholder="例: gpt-5.3-codex">
      </div>
      <div class="hint">候補はNagareで選べる設定値です。利用可否はCodex CLIの契約・ログイン状態に従います。</div>
    `;
  }

  function bindRuntimeModelChoice(modal) {
    const choice = modal.querySelector("[data-runtime-model-choice]");
    const field = modal.querySelector("[data-runtime-custom-model-field]");
    if (!choice || !field) return;
    const input = field.querySelector('input[name="model_custom"]');
    const update = () => {
      const usesCustomValue = choice.value === "__custom__";
      field.hidden = !usesCustomValue;
      if (input) input.required = usesCustomValue;
    };
    choice.addEventListener("input", update);
    update();
  }

  async function saveRuntimeModelDefaults(event, runtime) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const selectedModel = String(form.get("model") || "").trim();
    const model = selectedModel === "__custom__"
      ? String(form.get("model_custom") || "").trim()
      : selectedModel;
    try {
      state = await call("save_runtime_model_defaults", {
        request: {
          root: rootValue(),
          runtime_id: runtime.id,
          model_provider: String(form.get("model_provider") || "").trim(),
          model,
          model_base_url: String(form.get("model_base_url") || "").trim(),
        },
      });
      const nextRuntime = findRuntime(runtime.id) || runtime;
      const agentNames = runtimeAgentNames(nextRuntime);
      pendingRuntimeResult = {
        kind: "model",
        id: runtime.id,
        label: runtime.label,
        success: true,
        model_summary: runtimeConfiguredModel(nextRuntime) || "実行環境既定",
        agent_names: agentNames,
      };
      closeDynamicModal();
      renderSettingsScreens();
      toast(`${runtime.label} のモデル設定を保存しました。`);
    } catch (error) {
      showOperationError(event.currentTarget, "モデル設定を保存できませんでした", error, "モデル名、Provider、Base URLを確認してから、もう一度適用してください。");
      toast(String(error), "error");
    }
  }

  function renderAgents() {
    const el = document.getElementById("scr-agent-list");
    if (!el || !state) return;
    const currentProject = projectName();
    const totalAgents = (state.agents || []).length;
    el.innerHTML = `
      <h2 class="page-title">エージェント</h2>
      <p class="page-sub">役割、担当範囲、プロジェクト上の状態を確認します。実行環境やモデルは編集画面で設定します。</p>
      <div style="display:flex;justify-content:flex-end;margin:0 0 10px;"><button class="btn btn-primary" type="button" data-add-agent>新規エージェント</button></div>
      ${pendingAgentResult ? agentOperationResultPanel(pendingAgentResult) : ""}
      <div class="filters">
        <label class="visually-hidden" for="agent-role-filter">ロール</label>
        <select id="agent-role-filter" style="width:auto;min-width:150px;">
          <option value="all">ロール: すべて</option>
          <option value="organizer">オーガナイザー</option>
          <option value="worker">ワーカー</option>
          <option value="reviewer">レビュアー</option>
        </select>
        <label class="visually-hidden" for="agent-project-filter">参加プロジェクト</label>
        <select id="agent-project-filter" style="width:auto;min-width:190px;">
          <option value="all">プロジェクト: すべて</option>
          <option value="${escapeHtml(currentProject.toLowerCase())}">${escapeHtml(currentProject)}</option>
        </select>
        <label class="visually-hidden" for="agent-search-filter">エージェント検索</label>
        <input id="agent-search-filter" type="search" placeholder="名前・役割・担当範囲で検索" style="width:260px;">
        <span style="flex:1"></span>
        <span class="filter-result" id="agent-filter-result">${totalAgents}件を表示</span>
      </div>
      <div class="list">
        ${(state.agents || []).map((agent) => {
        const projectStatus = agentProjectStatus(agent);
        const scope = agentScopeSummary(agent);
        const usage = agentUsageSummary(agent);
        const searchText = [
          agent.name,
          agent.description,
          agent.tool_kind,
          agent.model,
          scope,
          usage,
          projectStatus,
          currentProject,
          agent.builtin ? "ビルトイン 標準" : "",
          ...(agent.specialties || []),
        ].join(" ").toLowerCase();
        return `
        <div class="list-item" data-agent-row data-role="${escapeHtml(agent.role || "")}" data-project="${escapeHtml(currentProject.toLowerCase())}" data-search="${escapeHtml(searchText)}">
          ${agentAvatarMarkup(agent)}
          <div class="wr-body">
            <div class="wr-title">${escapeHtml(agent.name)}${agent.builtin ? ' <span class="badge badge-neutral" title="Nagareが標準で用意するエージェント">ビルトイン</span>' : ""} <span class="wr-projtag">${escapeHtml(roleLabel(agent.role))}</span></div>
            <div class="wr-sum">${escapeHtml(agentListSummary(scope, usage))}</div>
          </div>
          <div class="wr-side">
            <span class="badge badge-neutral">${escapeHtml(projectStatus)}</span>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-edit-agent="${escapeHtml(agent.id)}">編集</button>
        </div>
      `;
      }).join("")}<div class="list-empty" id="agent-empty" style="display:none;">条件に一致するエージェントはいません。</div></div>`;
    el.querySelector("[data-add-agent]").addEventListener("click", () => openAgentCreateDialog());
    el.querySelector("[data-clear-agent-result]")?.addEventListener("click", () => {
      pendingAgentResult = null;
      renderAgents();
    });
    el.querySelector("#agent-role-filter").addEventListener("input", applyAgentFilter);
    el.querySelector("#agent-project-filter").addEventListener("input", applyAgentFilter);
    el.querySelector("#agent-search-filter").addEventListener("input", applyAgentFilter);
    el.querySelectorAll("[data-edit-agent]").forEach((button) => {
      button.addEventListener("click", () => openAgentDialog(findAgent(button.dataset.editAgent)));
    });
    applyAgentFilter();
  }

  function applyAgentFilter() {
    const role = document.getElementById("agent-role-filter")?.value || "all";
    const project = document.getElementById("agent-project-filter")?.value || "all";
    const query = (document.getElementById("agent-search-filter")?.value || "").trim().toLowerCase();
    let count = 0;
    document.querySelectorAll("[data-agent-row]").forEach((row) => {
      const roleVisible = role === "all" || row.dataset.role === role;
      const projectVisible = project === "all" || row.dataset.project === project;
      const queryVisible = !query || row.dataset.search.includes(query);
      const visible = roleVisible && projectVisible && queryVisible;
      row.style.display = visible ? "" : "none";
      if (visible) count += 1;
    });
    const result = document.getElementById("agent-filter-result");
    const empty = document.getElementById("agent-empty");
    if (result) result.textContent = `${count}件を表示`;
    if (empty) empty.style.display = count ? "none" : "";
  }

  function roleLabel(role) {
    return { organizer: "オーガナイザー", worker: "ワーカー", reviewer: "レビュアー" }[role] || role || "エージェント";
  }

  function agentListSummary(scope, usage) {
    return `担当範囲 ${scope} · ${usage}`;
  }

  function agentOperationResultPanel(result) {
    const isDelete = result.kind === "delete";
    const border = isDelete ? "var(--warning-line)" : "var(--success-line)";
    const background = isDelete ? "var(--warning-soft)" : "var(--success-soft)";
    const title = isDelete ? `削除結果: ${result.name || result.id}` : `保存結果: ${result.name || result.id}`;
    const summary = isDelete
      ? "このエージェントを今後の割り当て候補から外しました"
      : `${roleLabel(result.role)}として保存しました`;
    const runtime = result.tool_kind ? toolKindLabel(result.tool_kind) : "";
    const model = result.model && result.model !== "実行環境既定" ? result.model : "実行環境既定";
    const scope = result.scope || "全ドメイン";
    const capabilities = result.capabilities || [];
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:${border};background:${background};">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">${escapeHtml((result.name || result.id || "A").slice(0, 1))}</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">${escapeHtml(title)}</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">${escapeHtml(summary)}</div>
            <div style="font-size:12px;color:var(--text-muted);margin-top:6px;">
              ${runtime ? `実行環境: ${escapeHtml(runtime)} / ${escapeHtml(model)} · ` : ""}
              担当範囲: ${escapeHtml(scope)} ·
              能力: ${escapeHtml(capabilities.length ? capabilities.join("、") : "割り当てなし")}
            </div>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-clear-agent-result>閉じる</button>
        </div>
      </div>`;
  }

  function agentProjectStatus(agent) {
    const labels = [];
    if (agent.name === state?.project?.work_agent || agent.id === state?.project?.work_agent) labels.push("既定ワーカー");
    if (agent.name === state?.project?.review_agent || agent.id === state?.project?.review_agent) labels.push("既定レビュアー");
    if (agent.id === state?.project?.organizer_agent_id) labels.push("既定オーガナイザー");
    if (!labels.length && agent.builtin && lines(agent.domain_ids).includes("general")) labels.push("汎用フォールバック");
    if (!labels.length) labels.push(`${projectName()} 参加候補`);
    return labels.join(" / ");
  }

  function agentScopeSummary(agent) {
    const domains = lines(agent.domain_ids).map(domainName);
    const artifacts = lines(agent.artifact_type_ids).map(artifactName);
    return unique([...domains, ...artifacts]).join(" / ") || "全ドメイン";
  }

  function agentUsageSummary(agent) {
    const usageCount = Number(agent.usage_count || 0);
    const score = (state?.insights?.agent_scores || []).find((item) => item.agent_id === agent.id || item.agent_name === agent.name);
    if (!usageCount && !score) return "利用実績なし";
    const usage = usageCount ? `利用実績 ${usageCount}工程` : "";
    const evaluation = score ? `評価 ${score.review_count || 0}件 · 平均 ${score.average_score_label || "-"}` : "";
    return [usage, evaluation].filter(Boolean).join(" · ");
  }

  function agentAvatarMarkup(agent, options = {}) {
    const value = String(agent?.avatar || "").trim();
    const name = agent?.name || agent?.display_name || agent?.id || "A";
    if (isAvatarImage(value)) {
      const sizeClass = options.large ? " lg" : "";
      return `<img class="agent-avatar${sizeClass}" src="${escapeHtml(avatarImageSource(value))}" alt="">`;
    }
    return `<div class="${escapeHtml(options.className || "wr-picon")}">${escapeHtml(value || name.slice(0, 1) || "A")}</div>`;
  }

  function isAvatarImage(value) {
    const trimmed = String(value || "").trim();
    return /^data:image\//i.test(trimmed) || /\.(png|jpe?g|svg|webp)(\?.*)?$/i.test(trimmed);
  }

  function avatarImageSource(value) {
    const trimmed = String(value || "").trim();
    if (/^(data:|blob:|https?:|file:)/i.test(trimmed)) return trimmed;
    const normalized = trimmed.replace(/\\/g, "/");
    if (/^[A-Za-z]:\//.test(normalized)) return `file:///${normalized}`;
    if (normalized.startsWith("/")) return `file://${normalized}`;
    return normalized;
  }

  function toolKindLabel(value) {
    return {
      codex_cli: "Codex CLI",
      codex: "Codex",
      claude_code: "Claude Code",
      opencode: "OpenCode",
      openclaw: "OpenClaw",
    }[value] || value || "実行環境";
  }

  function findAgent(id) {
    return (state.agents || []).find((agent) => agent.id === id) || null;
  }

  function suggestedAgentId(name) {
    return generatedAgentId(name);
  }

  function openAgentCreateDialog() {
    const modal = dynamicModal(`
      <div class="modal" style="width:520px;">
        <div class="modal-head">
          <h3>エージェントを作成</h3>
          <div class="m-sub">まず名前、ロール、実行環境だけを決めます。作成後に詳細設定を続けます。</div>
        </div>
        <form class="modal-body" data-agent-create-form>
          <div class="field"><label>表示名</label><input name="display_name" type="text" placeholder="例: UI Worker" required></div>
          <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
            <div class="field"><label>ロール</label><select name="role">${agentRoleOptions("worker")}</select></div>
            <div class="field"><label>実行環境</label><select name="tool_kind">${agentToolOptions("codex_cli")}</select></div>
          </div>
          <div class="hint">説明、得意分野、担当範囲、プロンプト、スキル/MCPは次の設定画面で調整できます。</div>
          <div class="modal-foot" style="padding:16px 0 0;">
            <button class="btn btn-secondary" type="button" data-close>閉じる</button>
            <button class="btn btn-primary" type="submit">作成して設定へ</button>
          </div>
        </form>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-agent-create-form]").addEventListener("submit", async (event) => {
      event.preventDefault();
      const form = new FormData(event.currentTarget);
      const displayName = String(form.get("display_name") || "").trim();
      const request = {
        root: rootValue(),
        id: suggestedAgentId(displayName),
        display_name: displayName,
        avatar: "",
        role: String(form.get("role") || "worker"),
        tool_kind: String(form.get("tool_kind") || "codex_cli"),
        model: "",
        model_provider: "",
        model_base_url: "",
        description: "",
        prompt: "",
        specialties: [],
        domain_ids: [],
        artifact_type_ids: [],
        skill_set_ids: [],
        mcp_connection_ids: [],
      };
      try {
        state = await call("save_agent", { request });
        renderSettingsScreens();
        const created = findAgent(request.id);
        openAgentDialog(created, null, "basic");
        toast("エージェントを作成しました。詳細を設定してください。");
      } catch (error) {
        showOperationError(event.currentTarget, "エージェントを作成できませんでした", error, "表示名、ロール、実行環境を確認してから、もう一度作成してください。");
        toast(String(error), "error");
      }
    });
  }

  function findAgentByLabel(label) {
    const candidates = labelCandidates(label);
    if (!candidates.length) return null;
    return (state.agents || []).find((agent) => {
      const values = [agent.id, agent.name, agent.display_name].map(normalizeLabel).filter(Boolean);
      return candidates.some((candidate) => values.includes(candidate));
    }) || null;
  }

  function openAgentDialog(agent = null, proposal = null, initialTabOverride = "", preselect = {}) {
    const isEdit = Boolean(agent);
    const initialTab = initialTabOverride || (proposal ? "prompt" : "basic");
    const active = (tab) => tab === initialTab ? "active" : "";
    const selectedSkillIds = unique([...(agent?.skill_set_ids || []), preselect.skillId].filter(Boolean));
    const selectedMcpIds = unique([...(agent?.mcp_connection_ids || []), preselect.mcpId].filter(Boolean));
    const usePage = !agentReturnContext?.kind && !proposal && !preselect.skillId && !preselect.mcpId;
    const title = isEdit ? (agent.name || agent.id) : "新規エージェント";
    const builtinNotice = agent?.builtin
      ? '<p class="page-sub" style="margin:4px 0 0;">ビルトインエージェント: Nagareが標準で用意する役割です。プロジェクトに合わせて設定を調整できます。</p>'
      : "";
    const formBody = `
          ${proposalHiddenInputs(proposal)}
          <div class="tabs" style="margin-bottom:16px;">
            <button class="tab ${active("basic")}" type="button" data-agent-tab="basic">基本情報</button>
            <button class="tab ${active("runtime")}" type="button" data-agent-tab="runtime">実行環境</button>
            <button class="tab ${active("scope")}" type="button" data-agent-tab="scope">担当範囲</button>
            <button class="tab ${active("prompt")}" type="button" data-agent-tab="prompt">プロンプト</button>
            <button class="tab ${active("capabilities")}" type="button" data-agent-tab="capabilities">スキル・MCP</button>
          </div>

          <div class="tabpane ${active("basic")}" data-agent-pane="basic">
            <div class="card" style="padding:12px;margin-bottom:14px;">
              <div style="display:flex;gap:14px;align-items:flex-start;">
                <div data-agent-avatar-preview>${agentAvatarMarkup(agent || { name: title }, { large: true })}</div>
                <div style="flex:1;min-width:0;">
                  <div class="field" style="margin-bottom:8px;">
                    <label>アイコン画像</label>
                    <div style="display:flex;gap:8px;align-items:center;">
                      <input name="avatar" type="text" value="${escapeHtml(agent?.avatar || "")}" placeholder="PNG / JPG / SVG ファイルを選択">
                      <button class="btn btn-secondary btn-sm" type="button" data-choose-agent-avatar>画像を選択</button>
                      <button class="btn btn-secondary btn-sm" type="button" data-clear-agent-avatar>クリア</button>
                    </div>
                    <div class="hint">未設定の場合は表示名の先頭文字を使います。画像はPNG/JPG/SVGを選択できます。</div>
                  </div>
                </div>
              </div>
            </div>
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
              <div class="field"><label>表示名</label><input name="display_name" type="text" value="${escapeHtml(agent?.name || "")}" placeholder="例: UI Worker"></div>
              <div class="field"><label>ロール</label><select name="role">${agentRoleOptions(agent?.role || "worker")}</select></div>
            </div>
            <div class="field"><label>説明</label><textarea name="description" style="min-height:96px;" placeholder="このエージェントの役割や専門性">${escapeHtml(agent?.description || "")}</textarea></div>
            <div class="field"><label>得意分野（1行に1つ）</label><textarea name="specialties" style="min-height:88px;" placeholder="ui&#10;review">${escapeHtml((agent?.specialties || []).join("\n"))}</textarea></div>
            <details class="advanced-settings">
              <summary>詳細設定</summary>
              <div class="field"><label>管理ID（任意）</label><input name="id" type="text" value="${escapeHtml(agent?.id || "")}" ${isEdit ? "readonly" : ""} placeholder="空欄なら表示名から自動生成"><div class="hint">通常は変更不要です。外部設定で固定IDが必要な場合だけ入力します。</div></div>
            </details>
          </div>

          <div class="tabpane ${active("runtime")}" data-agent-pane="runtime">
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
              <div class="field"><label>実行環境</label><select name="tool_kind" data-agent-tool>${agentToolOptions(agent?.tool_kind || "codex_cli")}</select></div>
              <div data-agent-model-area style="display:contents;">${agentModelFields(agent?.tool_kind || "codex_cli", agent)}</div>
            </div>
            <div class="hint">モデルが空の場合は実行環境側の既定値を使います。MCPの候補は選択した実行環境に連動します。</div>
          </div>

          <div class="tabpane ${active("scope")}" data-agent-pane="scope">
            <div class="field"><label>担当ドメイン</label><div class="list" style="max-height:150px;overflow:auto;">${checkboxList("domain_ids", state.domains || [], agent?.domain_ids || [])}</div></div>
            <div class="field"><label>成果物</label><div class="list" style="max-height:150px;overflow:auto;">${checkboxList("artifact_type_ids", state.artifact_types || [], agent?.artifact_type_ids || [])}</div></div>
            <div class="hint">担当範囲はオーガナイザーがエージェントを選ぶ制約です。範囲外の依頼には割り当てられません。未選択の場合は全体候補として扱います。</div>
          </div>

          <div class="tabpane ${active("prompt")}" data-agent-pane="prompt">
            ${aiSupportPanel({
              generateTitle: "生成支援 — AIで下書き",
              generateDescription: "役割・得意分野・担当範囲からプロンプト案を作り、挿入して編集します。",
              generateAttr: "data-agent-prompt-draft",
              proposal,
              proposalEmptyText: "このプロンプトへの改善提案 — 現在なし",
              proposalActionLabel: "プロンプト欄に挿入",
            })}
            <div class="field">
              <div style="display:flex;align-items:center;gap:8px;margin-bottom:6px;">
                <label style="margin:0;">プロンプト</label>
                <span style="flex:1"></span>
              </div>
              <textarea name="prompt" class="mono" style="min-height:280px;" placeholder="このエージェントに常に守ってほしい振る舞い">${escapeHtml(agent?.prompt || "")}</textarea>
              <div class="hint">実行時にシステムプロンプトへ追記されます。知識やルーブリックは担当範囲から自動で渡されます。</div>
            </div>
          </div>

          <div class="tabpane ${active("capabilities")}" data-agent-pane="capabilities">
            <div class="field"><label>スキル</label><div class="list" style="max-height:150px;overflow:auto;">${skillCheckboxes(selectedSkillIds)}</div></div>
            <div class="field"><label>MCP接続</label><div class="list" data-agent-mcp-list style="max-height:150px;overflow:auto;">${mcpCheckboxes(selectedMcpIds, agent?.tool_kind || "codex_cli")}</div></div>
            <div class="hint">スキルとMCPはライブラリに登録済みのものだけ割り当てできます。MCPは接続テスト済みのものだけ選択できます。</div>
          </div>

          <div class="modal-foot">
            ${isEdit ? `<button class="btn btn-danger-soft left" type="button" data-delete-agent>削除</button>` : ""}
            <button class="btn btn-secondary" type="button" data-close>閉じる</button>
            <button class="btn btn-primary" type="submit">保存</button>
          </div>`;
    let modal;
    if (usePage) {
      closeDynamicModal();
      const page = document.getElementById("scr-agent-settings");
      if (!page) return;
      page.innerHTML = `
        <div style="display:flex;align-items:flex-start;gap:12px;margin-bottom:14px;">
          <div style="flex:1;min-width:0;">
            <h2 class="page-title" style="margin-bottom:0;">${escapeHtml(title)}</h2>
            <p class="page-sub" style="margin:4px 0 0;">役割、実行環境、担当範囲、プロンプト、スキル/MCPを設定します。</p>
            ${builtinNotice}
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-close>エージェント一覧へ</button>
        </div>
        <form class="card" style="padding:16px;" data-agent-form>${formBody}</form>`;
      goApp("agent-settings", `エージェント / <b>${escapeHtml(title)}</b>`);
      modal = page;
    } else {
      modal = dynamicModal(`
        <div class="modal" style="width:860px;">
          <div class="modal-head"><h3>${escapeHtml(title)}</h3><div class="m-sub">役割、実行環境、担当範囲、スキル/MCPを設定します。</div>${agent?.builtin ? '<div class="m-sub">ビルトインエージェント</div>' : ""}</div>
          <form class="modal-body" data-agent-form>${formBody}</form>
        </div>`);
    }
    modal.querySelectorAll("[data-close]").forEach((button) => button.addEventListener("click", closeAgentDialog));
    const agentForm = modal.querySelector("[data-agent-form]");
    bindGeneratedId(agentForm, generatedAgentId);
    agentForm.addEventListener("submit", saveAgentFromDialog);
    modal.querySelector("[data-agent-prompt-draft]").addEventListener("click", () => insertAgentPromptDraft(modal));
    modal.querySelector("[data-insert-proposal]")?.addEventListener("click", () => insertProposalDraft(modal, proposal, "agent"));
    modal.querySelector("[data-agent-tool]").addEventListener("input", () => refreshAgentRuntimeFields(modal));
    modal.querySelector('[name="avatar"]')?.addEventListener("input", () => updateAgentAvatarPreview(modal));
    modal.querySelector('[name="display_name"]')?.addEventListener("input", () => updateAgentAvatarPreview(modal));
    modal.querySelector("[data-choose-agent-avatar]")?.addEventListener("click", () => chooseAgentAvatar(modal));
    modal.querySelector("[data-clear-agent-avatar]")?.addEventListener("click", () => {
      const input = modal.querySelector('[name="avatar"]');
      if (input) input.value = "";
      updateAgentAvatarPreview(modal);
    });
    modal.querySelectorAll("[data-agent-tab]").forEach((button) => {
      button.addEventListener("click", () => switchAgentTab(modal, button.dataset.agentTab));
    });
    bindAgentModelChoice(modal);
    modal.querySelector("[data-delete-agent]")?.addEventListener("click", () => openDeleteAgentDialog(agent.id));
  }

  function updateAgentAvatarPreview(modal) {
    const target = modal.querySelector("[data-agent-avatar-preview]");
    if (!target) return;
    const avatar = modal.querySelector('[name="avatar"]')?.value || "";
    const name = modal.querySelector('[name="display_name"]')?.value || modal.querySelector('[name="id"]')?.value || "A";
    target.innerHTML = agentAvatarMarkup({ avatar, name }, { large: true });
  }

  async function chooseAgentAvatar(modal) {
    try {
      const file = await call("choose_agent_avatar_file");
      if (!file) return;
      const input = modal.querySelector('[name="avatar"]');
      if (input) input.value = file;
      updateAgentAvatarPreview(modal);
    } catch (error) {
      showOperationError(modal.querySelector("[data-agent-form]") || modal, "アイコン画像を選択できませんでした", error, "PNG/JPG/SVGの画像ファイルを選択してから、もう一度試してください。");
      toast(String(error), "error");
    }
  }

  function closeAgentDialog() {
    if (agentReturnContext?.kind === "project") {
      const tab = agentReturnContext.tab || "agents";
      agentReturnContext = null;
      closeDynamicModal();
      renderProjectScreens();
      goApp("project-list", "<b>プロジェクト</b>");
      openProjectSettingsDialog(tab);
      return;
    }
    agentReturnContext = null;
    closeDynamicModal();
    if (document.querySelector("#scr-agent-settings.screen.active [data-agent-form]")) {
      renderAgents();
      goApp("agent-list", "<b>エージェント</b>");
    }
  }

  function switchAgentTab(modal, tab) {
    modal.querySelectorAll("[data-agent-tab]").forEach((button) => button.classList.toggle("active", button.dataset.agentTab === tab));
    modal.querySelectorAll("[data-agent-pane]").forEach((pane) => pane.classList.toggle("active", pane.dataset.agentPane === tab));
  }

  function agentRoleOptions(current) {
    return [
      ["organizer", "オーガナイザー"],
      ["worker", "ワーカー"],
      ["reviewer", "レビュアー"],
    ].map(([value, label]) => `<option value="${value}" ${current === value ? "selected" : ""}>${label}</option>`).join("");
  }

  function agentToolOptions(current) {
    return [
      ["codex_cli", "Codex CLI"],
      ["claude_code", "Claude Code"],
      ["opencode", "OpenCode"],
      ["openclaw", "OpenClaw"],
    ].map(([value, label]) => `<option value="${value}" ${current === value ? "selected" : ""}>${label}</option>`).join("");
  }

  function agentModelFields(toolKind, values = {}) {
    const model = values?.model === "実行環境既定" ? "" : values?.model || "";
    const provider = values?.model_provider || "";
    if (!agentModelSwitchable(toolKind)) {
      return `
        <input name="model_provider" type="hidden" value="">
        <input name="model" type="hidden" value="">
        <input name="model_base_url" type="hidden" value="">
        <div class="field" style="grid-column:1 / -1;">
          <label>モデル</label>
          <input type="text" value="実行環境側の設定を使用" disabled>
          <div class="hint">${escapeHtml(toolKindLabel(toolKind))} はNagareからのモデル切替に対応していません。モデルは実行環境側で設定します。</div>
        </div>
      `;
    }
    const modelRef = toolKind === "opencode" && model && provider && !model.includes("/")
      ? `${provider.toLowerCase()}/${model}`
      : model;
    return `
      <div class="field" style="grid-column:1 / -1;"><label>モデル</label>${agentModelSelect(toolKind, modelRef)}</div>
      <input name="model_provider" type="hidden" value="">
      <input name="model_base_url" type="hidden" value="">
    `;
  }

  function agentModelSwitchable(toolKind) {
    return toolKind !== "openclaw";
  }

  function runtimeModelSwitchable(runtime) {
    return runtime?.id !== "openclaw";
  }

  function agentModelSelect(toolKind, model) {
    const runtime = runtimeForToolKind(toolKind);
    const choices = modelChoices(runtime);
    const usesCustomValue = Boolean(model) && !choices.includes(model);
    const hint = toolKind === "opencode"
      ? (choices.length ? "OpenCodeのローカル設定から読み込んだモデルです。" : "OpenCodeのローカル設定に候補がありません。モデル参照を手入力できます。")
      : "候補はNagareで選べる設定値です。利用可否はCodex CLIの契約・ログイン状態に従います。";
    return `
      <select name="model" data-agent-model-choice>
        <option value="" ${model ? "" : "selected"}>実行環境既定</option>
        ${choices.map((choice) => `<option value="${escapeHtml(choice)}" ${model === choice ? "selected" : ""}>${escapeHtml(choice)}</option>`).join("")}
        <option value="__custom__" ${usesCustomValue ? "selected" : ""}>手入力...</option>
      </select>
      <div class="field" data-agent-custom-model-field ${usesCustomValue ? "" : "hidden"} style="margin-top:10px;">
        <label>モデル名</label>
        <input name="model_custom" type="text" value="${escapeHtml(usesCustomValue ? model : "")}" placeholder="${toolKind === "opencode" ? "例: openai/gpt-5.6" : "例: gpt-5.6"}">
      </div>
      <div class="hint">${hint}</div>
    `;
  }

  function runtimeModelInput(runtime, model, scope) {
    const choices = modelChoices(runtime);
    const listId = `model-choices-${String(scope || runtime?.id || "runtime").replace(/[^a-z0-9_-]/gi, "-")}`;
    const placeholder = runtime?.id === "codex" || runtimeToolKind(runtime?.id) === "codex_cli"
      ? "例: gpt-5-codex。空なら実行環境既定"
      : "空なら実行環境既定";
    return `
      <input name="model" type="text" value="${escapeHtml(model || "")}" placeholder="${escapeHtml(placeholder)}" ${choices.length ? `list="${escapeHtml(listId)}"` : ""}>
      ${choices.length ? `<datalist id="${escapeHtml(listId)}">${choices.map((choice) => `<option value="${escapeHtml(choice)}"></option>`).join("")}</datalist>` : ""}
      ${choices.length ? `<div class="hint">候補: ${escapeHtml(choices.join(" / "))}。空欄は実行環境既定です。</div>` : ""}
    `;
  }

  function modelChoices(runtime) {
    return unique(lines(runtime?.model_choices).filter((choice) => !["手入力", "実行環境既定"].includes(choice)));
  }

  function runtimeForToolKind(toolKind) {
    return (state?.runtimes || []).find((runtime) => runtimeToolKind(runtime.id) === toolKind || runtime.id === toolKind) || null;
  }

  function refreshAgentRuntimeFields(modal) {
    const form = modal.querySelector("[data-agent-form]");
    const toolKind = form.querySelector('select[name="tool_kind"]')?.value || "codex_cli";
    const selectedMcpIds = new FormData(form).getAll("mcp_connection_ids").map(String);
    const values = {
      model: selectedAgentModelValue(form),
      model_provider: form.querySelector('[name="model_provider"]')?.value || "",
      model_base_url: form.querySelector('[name="model_base_url"]')?.value || "",
    };
    modal.querySelector("[data-agent-model-area]").innerHTML = agentModelFields(toolKind, values);
    modal.querySelector("[data-agent-mcp-list]").innerHTML = mcpCheckboxes(selectedMcpIds, toolKind);
    bindAgentModelChoice(modal);
  }

  function selectedAgentModelValue(form) {
    const model = form.querySelector('[name="model"]');
    if (!model) return "";
    if (model.value !== "__custom__") return model.value || "";
    return form.querySelector('[name="model_custom"]')?.value || "";
  }

  function bindAgentModelChoice(modal) {
    const choice = modal.querySelector("[data-agent-model-choice]");
    const field = modal.querySelector("[data-agent-custom-model-field]");
    if (!choice || !field) return;
    const input = field.querySelector('input[name="model_custom"]');
    const update = () => {
      const usesCustomValue = choice.value === "__custom__";
      field.hidden = !usesCustomValue;
      if (input) input.required = usesCustomValue;
    };
    choice.addEventListener("input", update);
    update();
  }

  async function insertAgentPromptDraft(modal) {
    const button = modal.querySelector("[data-agent-prompt-draft]");
    const form = modal.querySelector("[data-agent-form]");
    if (!form) return;
    const data = new FormData(form);
    try {
      if (button) button.disabled = true;
      const response = await call("generate_agent_prompt_draft", {
        request: {
          root: rootValue(),
          display_name: String(data.get("display_name") || "").trim(),
          role: String(data.get("role") || "worker"),
          description: String(data.get("description") || "").trim(),
          specialties: textLines(data.get("specialties")),
          domain_ids: data.getAll("domain_ids").map(String),
          artifact_type_ids: data.getAll("artifact_type_ids").map(String),
        },
      });
      modal.querySelector('textarea[name="prompt"]').value = response.text || "";
    } catch (error) {
      showOperationError(form, "プロンプト下書きを生成できませんでした", error, "エージェントの説明、担当範囲、ナレッジの状態を確認してから、もう一度実行してください。");
      toast(String(error), "error");
    } finally {
      if (button) button.disabled = false;
    }
  }

  function checkboxList(name, items, selectedIds) {
    if (!items.length) return `<div class="list-empty">候補はありません。</div>`;
    return items.map((item) => `
      <label class="list-item" style="cursor:pointer;">
        <input type="checkbox" name="${name}" value="${escapeHtml(item.id)}" ${selectedIds.includes(item.id) ? "checked" : ""}>
        <div class="wr-body"><div class="wr-title">${escapeHtml(item.name || item.id)}</div><div class="wr-sum">${escapeHtml(item.description || "")}</div></div>
      </label>
    `).join("");
  }

  function skillCheckboxes(selectedIds) {
    const skillIds = unique((state.skill_packages || []).flatMap((pkg) => pkg.provided_skill_sets?.length ? pkg.provided_skill_sets : [pkg.id]));
    if (!skillIds.length) return `<div class="list-empty">登録済みスキルはありません。</div>`;
    return skillIds.map((id) => `
      <label class="list-item" style="cursor:pointer;">
        <input type="checkbox" name="skill_set_ids" value="${escapeHtml(id)}" ${selectedIds.includes(id) ? "checked" : ""}>
        <div class="wr-body"><div class="wr-title">${escapeHtml(id)}</div><div class="wr-sum">このエージェントへ割り当てます</div></div>
      </label>
    `).join("");
  }

  function mcpCheckboxes(selectedIds, toolKind) {
    if (!runtimeSupportsAgentMcp(toolKind)) {
      return `<div class="list-empty">${escapeHtml(toolKindLabel(toolKind))}ではMCPをエージェント個別に割り当てできません。</div>`;
    }
    const mcps = (state.mcp_connections || []).filter((mcp) => (!mcp.tool_kind || !toolKind || mcp.tool_kind === toolKind) && mcpAgentAssignable(mcp));
    if (!mcps.length) return `<div class="list-empty">この実行環境で使えるMCPはありません。</div>`;
    return mcps.map((mcp) => `
      <label class="list-item" style="cursor:pointer;">
        <input type="checkbox" name="mcp_connection_ids" value="${escapeHtml(mcp.id)}" ${selectedIds.includes(mcp.id) ? "checked" : ""} ${mcpCanAssignToAgent(mcp) ? "" : "disabled"}>
        <div class="wr-body"><div class="wr-title">${escapeHtml(mcp.name || mcp.id)}</div><div class="wr-sum">${escapeHtml(testStatusLabel(mcp.test_status))} · ${escapeHtml(mcp.runtime_label || "")}${mcpCanAssignToAgent(mcp) ? "" : ` · ${escapeHtml(mcpAssignmentSummary(mcp))}`}</div></div>
      </label>
    `).join("");
  }

  function unique(values) {
    return [...new Set(values.filter(Boolean))];
  }

  function textLines(value) {
    return String(value || "").split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  }

  async function saveAgentFromDialog(event) {
    event.preventDefault();
    const fromAgentPage = Boolean(event.currentTarget.closest("#scr-agent-settings"));
    const form = new FormData(event.currentTarget);
    const displayName = String(form.get("display_name") || "").trim();
    const request = {
      root: rootValue(),
      id: String(form.get("id") || "").trim() || generatedAgentId(displayName),
      display_name: displayName,
      avatar: String(form.get("avatar") || "").trim(),
      role: String(form.get("role") || "worker"),
      tool_kind: String(form.get("tool_kind") || "codex_cli"),
      model: String(form.get("model") || "") === "__custom__"
        ? String(form.get("model_custom") || "").trim()
        : String(form.get("model") || "").trim(),
      model_provider: String(form.get("model_provider") || "").trim(),
      model_base_url: String(form.get("model_base_url") || "").trim(),
      description: String(form.get("description") || "").trim(),
      prompt: String(form.get("prompt") || "").trim(),
      specialties: textLines(form.get("specialties")),
      domain_ids: form.getAll("domain_ids").map(String),
      artifact_type_ids: form.getAll("artifact_type_ids").map(String),
      skill_set_ids: form.getAll("skill_set_ids").map(String),
      mcp_connection_ids: form.getAll("mcp_connection_ids").map(String),
      ...proposalRequestFields(form),
    };
    try {
      state = await call("save_agent", {
        request,
      });
      reconcileAppliedImprovement(request);
      pendingAgentResult = agentSaveResultFromRequest(request);
      closeDynamicModal();
      renderSettingsScreens();
      syncStateChrome();
      if (agentReturnContext?.kind === "project") {
        const tab = agentReturnContext.tab || "agents";
        agentReturnContext = null;
        renderProjectScreens();
        goApp("project-list", "<b>プロジェクト</b>");
        openProjectSettingsDialog(tab);
        toast("エージェントを保存しました。プロジェクト設定に戻りました。");
        return;
      }
      agentReturnContext = null;
      if (fromAgentPage) {
        renderAgents();
        goApp("agent-list", "<b>エージェント</b>");
      }
      toast("エージェントを保存しました。");
    } catch (error) {
      showOperationError(event.currentTarget, "エージェントを保存できませんでした", error, "表示名、役割、実行環境、モデル、担当範囲を確認してから、もう一度保存してください。");
      toast(String(error), "error");
    }
  }

  function agentSaveResultFromRequest(request) {
    const skillNames = lines(request.skill_set_ids);
    const mcpNames = lines(request.mcp_connection_ids).map((mcpId) => findMcp(mcpId)?.name || mcpId);
    const domains = lines(request.domain_ids).map(domainName);
    const artifacts = lines(request.artifact_type_ids).map(artifactName);
    return {
      kind: "save",
      id: request.id,
      name: request.display_name || request.id,
      role: request.role,
      tool_kind: request.tool_kind,
      model: request.model || "実行環境既定",
      scope: [...domains, ...artifacts].join(" / ") || "全ドメイン",
      capabilities: [...skillNames, ...mcpNames],
    };
  }

  function openDeleteAgentDialog(id) {
    const agent = findAgent(id);
    const fromAgentPage = Boolean(document.querySelector("#scr-agent-settings.screen.active [data-agent-form]"));
    const projectStatus = agent ? agentProjectStatus(agent) : "割り当て候補";
    const skillNames = lines(agent?.skill_set_ids);
    const mcpNames = lines(agent?.mcp_connection_ids).map((mcpId) => findMcp(mcpId)?.name || mcpId);
    const capabilities = [...skillNames, ...mcpNames];
    const modal = dynamicModal(`
      <div class="modal">
        <div class="modal-head"><h3>エージェントを削除しますか?</h3><div class="m-sub">${escapeHtml(agent?.name || id)} を今後の割り当て候補から外します。</div></div>
        <div class="modal-body">
          <div class="card" style="padding:10px 12px;">
            <div class="kv"><div class="k">対象</div><div class="v">${escapeHtml(agent?.name || id)}${agent?.role ? ` · ${escapeHtml(roleLabel(agent.role))}` : ""}</div></div>
            <div class="kv"><div class="k">プロジェクト</div><div class="v">${escapeHtml(projectStatus)}</div></div>
            <div class="kv"><div class="k">能力</div><div class="v">${escapeHtml(capabilities.length ? capabilities.join("、") : "割り当てなし")}</div></div>
            <div class="kv"><div class="k">履歴</div><div class="v">過去のワーク履歴と実行記録は残ります</div></div>
          </div>
          <div class="hint" style="margin-top:8px;">削除後、このエージェントは新しいワークの候補に入りません。必要になった場合は再作成してください。</div>
        </div>
        <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button><button class="btn btn-primary" type="button" data-confirm>削除</button></div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-confirm]").addEventListener("click", async () => {
      try {
        const deletedResult = {
          kind: "delete",
          id,
          name: agent?.name || id,
          role: agent?.role || "",
          tool_kind: agent?.tool_kind || "",
          model: agent?.model || "実行環境既定",
          scope: agent ? agentScopeSummary(agent) : "全ドメイン",
          capabilities,
        };
        state = await call("delete_agent_command", { request: { root: rootValue(), id } });
        pendingAgentResult = deletedResult;
        closeDynamicModal();
        renderSettingsScreens();
        if (fromAgentPage) {
          renderAgents();
          goApp("agent-list", "<b>エージェント</b>");
        }
        toast("エージェントを削除しました。");
      } catch (error) {
        showOperationError(modal, "エージェントを削除できませんでした", error, "プロジェクト設定の整理役や割り当て状況を確認してから、もう一度削除してください。");
        toast(String(error), "error");
      }
    });
  }

  function renderKnowledge() {
    const el = document.getElementById("scr-knowledge-list");
    if (!el || !state) return;
    const domains = state.domains || [];
    const artifacts = state.artifact_types || [];
    const pendingDomain = findDomain(pendingDomainFollowupId);
    el.innerHTML = `
      <h2 class="page-title">ナレッジ</h2>
      <p class="page-sub">ドメインごとに、共通知識と成果物を管理します。</p>
      <div style="display:flex;justify-content:flex-end;margin:0 0 10px;">
        <button class="btn btn-primary" type="button" data-add-domain>ドメインを追加</button>
      </div>
      ${knowledgeReturnContext?.kind === "project" ? knowledgeReturnPanel() : ""}
      ${pendingDomain ? domainFollowupPanel(pendingDomain) : ""}
      ${pendingKnowledgeResult ? knowledgeOperationResultPanel(pendingKnowledgeResult) : ""}
      <div class="filters">
        <label class="visually-hidden" for="knowledge-search-filter">ナレッジ検索</label>
        <input id="knowledge-search-filter" type="search" placeholder="ドメイン・成果物・知識を検索" style="width:300px;">
        <span style="flex:1"></span>
        <span class="filter-result" id="knowledge-filter-result">${domains.length}件を表示</span>
      </div>
      <div class="list">${domains.map((domain) => {
        const domainArtifacts = artifacts.filter((artifact) => artifact.domain_id === domain.id);
        const artifactCount = domainArtifacts.length || domain.artifact_type_count || 0;
        const artifactNames = domainArtifacts.slice(0, 3).map((artifact) => artifact.name).filter(Boolean).join(" / ");
        const knowledgeCount = lines(domain.shared_knowledge).length + domainArtifacts.reduce((count, artifact) => count + lines(artifact.knowledge).length, 0);
        const rubricCount = lines(domain.common_rubric).length + domainArtifacts.reduce((count, artifact) => count + (artifact.rubric_count ?? lines(artifact.rubric).length), 0);
        const usage = domainProjectUsage(domain, domainArtifacts);
        const proposalCount = domainImprovementProposals(domain, domainArtifacts).length;
        const searchText = [
          domain.name,
          domain.description,
          usage.label,
          proposalCount ? "改善提案" : "",
          ...(domain.shared_knowledge || []),
          ...(domain.common_rubric || []),
          ...(domain.dispatch_hints || []),
          ...domainArtifacts.flatMap((artifact) => [
            artifact.name,
            artifact.description,
            ...(artifact.knowledge || []),
            ...(artifact.rubric || []),
            ...(artifact.dispatch_hints || []),
          ]),
        ].join(" ").toLowerCase();
        return `
        <div class="list-item" data-knowledge-row data-search="${escapeHtml(searchText)}">
          <div class="wr-picon">知</div>
            <div class="wr-body">
              <div class="wr-title">${escapeHtml(domain.name)}${proposalCount ? ` <span class="badge badge-ask" style="margin-left:4px;"><span class="bdot"></span>改善提案 ${proposalCount}件</span>` : ""}</div>
            <div class="wr-sum">${escapeHtml(domain.description || "")} · 知識 ${knowledgeCount}件 · 成果物 ${artifactCount}件${artifactNames ? `（${escapeHtml(artifactNames)}）` : ""} · ルーブリック ${rubricCount}項目 · ${escapeHtml(usage.label)}</div>
          </div>
          <div style="display:flex;gap:8px;align-items:center;">
            <button class="btn btn-secondary btn-sm" type="button" data-domain-detail="${escapeHtml(domain.id)}">詳細</button>
            <button class="btn btn-danger-soft btn-sm" type="button" data-delete-domain-row="${escapeHtml(domain.id)}">削除</button>
          </div>
        </div>
      `;
      }).join("") || `<div class="list-empty">ドメインはまだありません。</div>`}<div class="list-empty" id="knowledge-empty" style="display:none;">条件に一致するドメインはありません。</div></div>`;
    el.querySelector("[data-add-domain]").addEventListener("click", () => openDomainDialog());
    el.querySelector("[data-return-project-settings]")?.addEventListener("click", () => {
      const tab = knowledgeReturnContext?.tab || "knowledge";
      knowledgeReturnContext = null;
      renderProjectScreens();
      goApp("project-list", "<b>プロジェクト</b>");
      openProjectSettingsDialog(tab);
    });
    el.querySelector("[data-clear-knowledge-return]")?.addEventListener("click", () => {
      knowledgeReturnContext = null;
      renderKnowledge();
    });
    el.querySelector("[data-followup-add-artifact]")?.addEventListener("click", (event) => {
      pendingDomainFollowupId = null;
      openArtifactDialog(null, null, event.currentTarget.dataset.followupAddArtifact);
    });
    el.querySelector("[data-clear-domain-followup]")?.addEventListener("click", () => {
      pendingDomainFollowupId = null;
      renderKnowledge();
    });
    el.querySelector("[data-clear-knowledge-result]")?.addEventListener("click", () => {
      pendingKnowledgeResult = null;
      renderKnowledge();
    });
    el.querySelector("#knowledge-search-filter").addEventListener("input", applyKnowledgeFilter);
    el.querySelectorAll("[data-domain-detail]").forEach((button) => {
      button.addEventListener("click", () => openDomainDialog(findDomain(button.dataset.domainDetail), "basic"));
    });
    el.querySelectorAll("[data-delete-domain-row]").forEach((button) => {
      button.addEventListener("click", () => openDeleteDomainDialog(button.dataset.deleteDomainRow));
    });
    applyKnowledgeFilter();
  }

  function domainProjectUsage(domain, domainArtifacts = []) {
    const project = state?.project;
    if (!project || !domain) return { used: false, label: "利用プロジェクト: 未割り当て" };
    const artifactIds = new Set(domainArtifacts.map((artifact) => artifact.id));
    const defaultArtifact = findArtifact(project.default_artifact_type_id);
    const explicitDefault = project.default_domain_id === domain.id || defaultArtifact?.domain_id === domain.id;
    if (explicitDefault) {
      return { used: true, label: `利用プロジェクト: ${projectName()}（既定）` };
    }

    const domains = state?.domains || [];
    const implicitDefault = !project.default_domain_id && !project.default_artifact_type_id && domains[0]?.id === domain.id;
    if (implicitDefault) {
      return { used: true, label: `利用プロジェクト: ${projectName()}（自動候補）` };
    }

    const assignedAgents = (state?.agents || []).filter((agent) => {
      const domainMatch = lines(agent.domain_ids).includes(domain.id);
      const artifactMatch = lines(agent.artifact_type_ids).some((id) => artifactIds.has(id));
      return domainMatch || artifactMatch;
    });
    if (assignedAgents.length) {
      const agentNames = assignedAgents.slice(0, 2).map((agent) => agent.name || agent.id).join("・");
      const suffix = assignedAgents.length > 2 ? `ほか${assignedAgents.length - 2}件` : "担当範囲";
      return { used: true, label: `利用プロジェクト: ${projectName()}（${agentNames} ${suffix}）` };
    }

    return { used: false, label: "利用プロジェクト: 未割り当て" };
  }

  function domainImprovementProposalCount(domain, domainArtifacts = []) {
    return domainImprovementProposals(domain, domainArtifacts).length;
  }

  function domainImprovementProposals(domain, domainArtifacts = []) {
    if (!domain) return [];
    const terms = new Set([
      domain.id,
      domain.name,
      ...domainArtifacts.flatMap((artifact) => [artifact.id, artifact.name]),
    ].filter(Boolean).map((term) => String(term).toLowerCase()));
    return (state?.insights?.proposals || []).filter((proposal) => {
      const kind = String(proposal.kind || "").toLowerCase();
      if (!["ルーブリック", "知識", "rubric", "knowledge"].includes(kind)) return false;
      const haystack = [
        proposal.title,
        proposal.target_label,
        proposal.summary,
        proposal.evidence,
        proposal.next_step,
      ].filter(Boolean).join(" ").toLowerCase();
      return [...terms].some((term) => haystack.includes(term));
    });
  }

  function knowledgeReturnPanel() {
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:var(--primary-line);background:var(--primary-soft);">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">戻</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">プロジェクト設定から開いています</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">ナレッジを確認したら、プロジェクト設定の知識・成果物タブへ戻れます。未保存のプロジェクト設定は保持されています。</div>
          </div>
          <div style="display:flex;gap:8px;align-items:center;">
            <button class="btn btn-primary btn-sm" type="button" data-return-project-settings>プロジェクト設定へ戻る</button>
            <button class="btn btn-secondary btn-sm" type="button" data-clear-knowledge-return>ここで続ける</button>
          </div>
        </div>
      </div>`;
  }

  function domainFollowupPanel(domain) {
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:var(--primary-line);background:var(--primary-soft);">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">知</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">次の操作: 成果物を追加</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">
              ${escapeHtml(domain.name || domain.id)} を追加しました。ワークで使うには、README、FAQ、リリースノートのような成果物とルーブリックを設定します。
            </div>
          </div>
          <div style="display:flex;gap:8px;align-items:center;">
            <button class="btn btn-primary btn-sm" type="button" data-followup-add-artifact="${escapeHtml(domain.id)}">成果物を追加</button>
            <button class="btn btn-secondary btn-sm" type="button" data-clear-domain-followup>後で</button>
          </div>
        </div>
      </div>`;
  }

  function knowledgeOperationResultPanel(result) {
    const isDelete = result.kind === "delete";
    const border = isDelete ? "var(--warning-line)" : "var(--success-line)";
    const background = isDelete ? "var(--warning-soft)" : "var(--success-soft)";
    const title = `${isDelete ? "削除結果" : "保存結果"}: ${result.name || result.id}`;
    return `
      <div class="card" style="margin:0 0 14px;padding:14px 16px;border-color:${border};background:${background};">
        <div style="display:flex;gap:14px;align-items:flex-start;">
          <div class="wr-picon" style="background:#fff;">${escapeHtml(result.entity === "artifact" ? "成" : "知")}</div>
          <div style="flex:1;min-width:0;">
            <div style="font-size:13.5px;font-weight:800;color:var(--text-body);">${escapeHtml(title)}</div>
            <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">${escapeHtml(result.summary || "")}</div>
            <div style="font-size:12px;color:var(--text-muted);margin-top:6px;">${escapeHtml(result.detail || "")}</div>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-clear-knowledge-result>閉じる</button>
        </div>
      </div>`;
  }

  function applyKnowledgeFilter() {
    const query = (document.getElementById("knowledge-search-filter")?.value || "").trim().toLowerCase();
    let count = 0;
    document.querySelectorAll("[data-knowledge-row]").forEach((row) => {
      const queryVisible = !query || row.dataset.search.includes(query);
      const visible = queryVisible;
      row.style.display = visible ? "" : "none";
      if (visible) count += 1;
    });
    const result = document.getElementById("knowledge-filter-result");
    const empty = document.getElementById("knowledge-empty");
    if (result) result.textContent = `${count}件を表示`;
    if (empty) empty.style.display = count ? "none" : "";
  }

  function domainName(id) {
    return (state.domains || []).find((domain) => domain.id === id)?.name || id || "未設定";
  }

  function findDomain(id) {
    return (state.domains || []).find((domain) => domain.id === id) || null;
  }

  function findDomainByLabel(label) {
    const candidates = labelCandidates(label);
    if (!candidates.length) return null;
    return (state.domains || []).find((domain) => {
      const values = [domain.id, domain.name, domain.display_name].map(normalizeLabel).filter(Boolean);
      return candidates.some((candidate) => values.includes(candidate) || values.some((value) => candidate.includes(value) || value.includes(candidate)));
    }) || null;
  }

  function findArtifact(id) {
    return (state.artifact_types || []).find((artifact) => artifact.id === id) || null;
  }

  function findArtifactByLabel(label) {
    const candidates = labelCandidates(label);
    if (!candidates.length) return null;
    return (state.artifact_types || []).find((artifact) => {
      const values = [artifact.id, artifact.name, artifact.display_name].map(normalizeLabel).filter(Boolean);
      return candidates.some((candidate) => values.includes(candidate) || values.some((value) => candidate.includes(value) || value.includes(candidate)));
    }) || (state.artifact_types || []).find((artifact) => {
      const rubricText = normalizeLabel(lines(artifact.rubric).join(" "));
      const knowledgeText = normalizeLabel(lines(artifact.knowledge).join(" "));
      return candidates.some((candidate) => candidate.length >= 2 && (rubricText.includes(candidate) || knowledgeText.includes(candidate)));
    }) || null;
  }

  function findArtifactByKnowledgeRef(label) {
    const artifact = findArtifactByLabel(label);
    if (artifact) return artifact;
    const normalized = normalizeLabel(label);
    if (!normalized) return null;
    return (state.artifact_types || []).find((item) => {
      const domain = findDomain(item.domain_id);
      const haystack = normalizeLabel([
        item.id,
        item.name,
        item.description,
        ...(item.knowledge || []),
        ...(item.rubric || []),
        domain?.id,
        domain?.name,
      ].join(" "));
      return normalized.length >= 2 && haystack.includes(normalized);
    }) || null;
  }

  function labelCandidates(label) {
    const raw = String(label || "");
    return unique(
      [raw, ...raw.split(/[\/／>＞:：|｜]/)]
        .map(normalizeLabel)
        .filter(Boolean),
    );
  }

  function normalizeLabel(value) {
    return String(value || "").trim().toLowerCase();
  }

  function artifactName(id) {
    return findArtifact(id)?.name || id || "未設定";
  }

  function openDomainDialog(domain = null, initialTabOverride = "basic") {
    const isEdit = Boolean(domain);
    const domainArtifacts = isEdit ? (state.artifact_types || []).filter((artifact) => artifact.domain_id === domain.id) : [];
    const domainProposals = isEdit ? domainImprovementProposals(domain, domainArtifacts) : [];
    const active = (tab) => tab === initialTabOverride ? "active" : "";
    const usePage = !domainReturnContext?.kind;
    const title = isEdit ? (domain.name || domain.id) : "ドメインを追加";
    const domainResult = isEdit && pendingKnowledgeResult?.domain_id === domain.id ? pendingKnowledgeResult : null;
    const formBody = `
          ${domainResult ? knowledgeOperationResultPanel(domainResult) : ""}
          <div class="tabs" style="margin-bottom:16px;">
            <button class="tab ${active("basic")}" type="button" data-domain-tab="basic">基本情報</button>
            <button class="tab ${active("artifacts")}" type="button" data-domain-tab="artifacts">成果物</button>
          </div>
          <div class="tabpane ${active("basic")}" data-domain-pane="basic">
            <div class="field"><label>ドメイン名 <span class="required">必須</span></label><input name="display_name" type="text" value="${escapeHtml(domain?.name || "")}" placeholder="例: プロダクト開発" required></div>
            <div class="field"><label>知識体系の説明 <span class="required">必須</span></label><textarea name="description" style="min-height:88px;" placeholder="例: 製品仕様、導入ガイド、FAQなど、プロダクトに関する知識と成果物をまとめる" required>${escapeHtml(domain?.description || "")}</textarea><div class="hint">単独の文書名ではなく、複数の成果物と共通知識を束ねる範囲を記述します。</div></div>
            <div class="field"><label>共通知識（1行に1つ）</label><textarea name="shared_knowledge" style="min-height:180px;" placeholder="例: 用語集&#10;禁止表現&#10;対象読者">${escapeHtml((domain?.shared_knowledge || []).join("\n"))}</textarea><div class="hint">このドメイン内の全成果物に適用され、担当エージェントへ自動注入されます。</div></div>
            <input name="id" type="hidden" value="${escapeHtml(domain?.id || "")}">
            <textarea name="common_rubric" hidden>${escapeHtml((domain?.common_rubric || []).join("\n"))}</textarea>
            <textarea name="dispatch_hints" hidden>${escapeHtml((domain?.dispatch_hints || []).join("\n"))}</textarea>
            ${isEdit ? domainQualityPanel(domain, domainArtifacts, domainProposals) : ""}
            <div class="modal-foot">
              <button class="btn btn-secondary" type="button" data-close>閉じる</button>
              <button class="btn btn-primary" type="submit" data-save-domain disabled>保存</button>
            </div>
          </div>
          <div class="tabpane ${active("artifacts")}" data-domain-pane="artifacts">
              <div style="display:flex;align-items:center;gap:12px;margin-bottom:10px;">
                <div>
                  <h4 style="font-size:14px;">${escapeHtml(domain?.name || "このドメイン")} の成果物</h4>
                  <div class="hint">このドメイン内で作る成果物ごとの知識とルーブリックを管理します。</div>
                </div>
                <span style="flex:1"></span>
                ${isEdit
                  ? `<button class="btn btn-primary btn-sm" type="button" data-add-domain-artifact>成果物を追加</button>`
                  : `<button class="btn btn-primary btn-sm" type="button" disabled>成果物を追加</button>`}
              </div>
              <div class="list">
                ${!isEdit ? `<div class="list-empty">先に基本情報を保存してください。保存後もこの編集画面に留まり、成果物とルーブリックを追加できます。</div>` : domainArtifacts.map((artifact) => `
                  <div class="list-item">
                    <div class="wr-picon">成</div>
                    <div class="wr-body">
                      <div class="wr-title">${escapeHtml(artifact.name)}</div>
                      <div class="wr-sum">${escapeHtml(artifact.description || "")} · ルーブリック ${artifact.rubric_count || 0}項目 / ${artifact.rubric_score_total || 0}点</div>
                    </div>
                    <div style="display:flex;gap:8px;align-items:center;">
                      <button class="btn btn-secondary btn-sm" type="button" data-edit-artifact="${escapeHtml(artifact.id)}">編集</button>
                      <button class="btn btn-danger-soft btn-sm" type="button" data-delete-artifact-row="${escapeHtml(artifact.id)}">削除</button>
                    </div>
                  </div>
                `).join("") || `<div class="list-empty">このドメインの成果物はまだありません。</div>`}
              </div>
              ${usePage ? "" : `<div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button></div>`}
            </div>
          `;
    let modal;
    if (usePage) {
      closeDynamicModal();
      const page = document.getElementById("scr-knowledge-domain");
      if (!page) return;
      page.innerHTML = `
        <div style="display:flex;align-items:flex-start;gap:12px;margin-bottom:14px;">
          <div style="flex:1;min-width:0;">
            <h2 class="page-title" style="margin-bottom:0;">${escapeHtml(title)}</h2>
            <p class="page-sub" style="margin:4px 0 0;">複数の成果物に共通する知識をまとめ、ワーク実行時に担当エージェントへ自動注入します。</p>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-close>ナレッジ一覧へ</button>
        </div>
        <form class="card" style="padding:16px;" data-domain-form>${formBody}</form>`;
      goApp("knowledge-domain", `ナレッジ / <b>${escapeHtml(title)}</b>`);
      modal = page;
    } else {
      modal = dynamicModal(`
        <div class="modal" style="width:860px;">
          <div class="modal-head"><h3>${escapeHtml(title)}</h3><div class="m-sub">複数の成果物、共通知識、品質基準をまとめる知識体系です。</div></div>
          <form class="modal-body" data-domain-form>${formBody}</form>
        </div>`);
    }
    modal.querySelectorAll("[data-close]").forEach((button) => button.addEventListener("click", closeDomainDialog));
    const domainForm = modal.querySelector("[data-domain-form]");
    bindGeneratedId(domainForm, generatedDomainId);
    bindDomainSaveReadiness(domainForm);
    domainForm.addEventListener("submit", saveDomainFromDialog);
    modal.querySelector("[data-clear-knowledge-result]")?.addEventListener("click", () => {
      pendingKnowledgeResult = null;
      openDomainDialog(findDomain(domain.id), initialTabOverride);
    });
    modal.querySelectorAll("[data-domain-proposal-preview]").forEach((button) => {
      button.addEventListener("click", () => openImprovementProposalDialog(button.dataset.domainProposalPreview));
    });
    modal.querySelectorAll("[data-domain-tab]").forEach((button) => {
      button.addEventListener("click", () => {
        const tab = button.dataset.domainTab;
        modal.querySelectorAll("[data-domain-tab]").forEach((item) => item.classList.toggle("active", item.dataset.domainTab === tab));
        modal.querySelectorAll("[data-domain-pane]").forEach((pane) => pane.classList.toggle("active", pane.dataset.domainPane === tab));
      });
    });
    modal.querySelector("[data-add-domain-artifact]")?.addEventListener("click", () => {
      artifactReturnContext = { kind: "domain", domainId: domain.id, tab: "artifacts" };
      openArtifactDialog(null, null, domain.id);
    });
    modal.querySelectorAll("[data-edit-artifact]").forEach((button) => {
      button.addEventListener("click", () => {
        artifactReturnContext = { kind: "domain", domainId: domain.id, tab: "artifacts" };
        openArtifactDialog(findArtifact(button.dataset.editArtifact));
      });
    });
    modal.querySelectorAll("[data-delete-artifact-row]").forEach((button) => {
      button.addEventListener("click", () => {
        artifactReturnContext = { kind: "domain", domainId: domain.id, tab: "artifacts" };
        openDeleteArtifactDialog(button.dataset.deleteArtifactRow);
      });
    });
  }

  function bindDomainSaveReadiness(form) {
    const displayName = form.querySelector('input[name="display_name"]');
    const description = form.querySelector('textarea[name="description"]');
    const saveButton = form.querySelector("[data-save-domain]");
    if (!displayName || !description || !saveButton) return;
    const update = () => {
      saveButton.disabled = !displayName.value.trim() || !description.value.trim();
    };
    displayName.addEventListener("input", update);
    description.addEventListener("input", update);
    update();
  }

  function domainQualityPanel(domain, domainArtifacts, proposals) {
    const insights = state?.insights || {};
    const reviews = (insights.recent_reviews || []).filter((review) => {
      const text = [review.title, review.agent_name, ...(review.concerns || [])].join(" ").toLowerCase();
      return [domain.id, domain.name, ...domainArtifacts.flatMap((artifact) => [artifact.id, artifact.name])]
        .filter(Boolean)
        .some((term) => text.includes(String(term).toLowerCase()));
    });
    const reviewCount = reviews.length || domainArtifacts.reduce((count, artifact) => count + Number(artifact.review_count || 0), 0) || Number(insights.review_count || 0);
    const concernCount = proposals.length || reviews.reduce((count, review) => count + lines(review.concerns).length, 0);
    return `
      <div class="card" style="padding:12px 14px;margin-top:14px;background:#fff;">
        <div style="display:flex;align-items:flex-start;gap:10px;margin-bottom:10px;">
          <div style="flex:1;">
            <h4 style="font-size:13.5px;margin-bottom:3px;">品質記録と改善提案</h4>
            <div class="hint" style="margin:0;">レビュー履歴から繰り返し出る懸念を見つけ、ルーブリックや知識の改善候補として確認できます。</div>
          </div>
          <span class="badge ${proposals.length ? "badge-ask" : "badge-neutral"}">${escapeHtml(proposals.length ? `改善提案 ${proposals.length}件` : "提案なし")}</span>
        </div>
        <div class="ro-facts" style="margin:0 0 10px;">
          <span><b>最近のレビュー</b>${escapeHtml(`${reviewCount || 0}件`)}</span>
          <span><b>懸念</b>${escapeHtml(concernCount ? `${concernCount}件` : "なし")}</span>
          <span><b>成果物</b>${escapeHtml(`${domainArtifacts.length}件`)}</span>
        </div>
        <div class="list">
          ${proposals.length ? proposals.map((proposal) => `
            <div class="list-item" style="background:#fffdf5;">
              <div class="li-icon" style="background:var(--warning-soft);color:var(--warning);">!</div>
              <div style="flex:1;min-width:0;">
                <div class="li-title">${escapeHtml(proposal.title || "改善提案")}</div>
                <div class="li-desc">${escapeHtml(proposal.summary || "")} · 根拠: ${escapeHtml(proposal.evidence || "レビュー履歴")}</div>
              </div>
              <button class="btn btn-secondary btn-sm" type="button" data-domain-proposal-preview="${escapeHtml(proposal.id)}">Diffを見る</button>
            </div>
          `).join("") : `<div class="list-empty">このドメインに関連する未対応の改善提案はありません。</div>`}
        </div>
      </div>`;
  }

  function closeDomainDialog() {
    if (domainReturnContext?.kind === "project") {
      const tab = domainReturnContext.tab || "knowledge";
      domainReturnContext = null;
      closeDynamicModal();
      renderProjectScreens();
      goApp("project-list", "<b>プロジェクト</b>");
      openProjectSettingsDialog(tab);
      return;
    }
    domainReturnContext = null;
    closeDynamicModal();
    if (document.querySelector("#scr-knowledge-domain.screen.active [data-domain-form]")) {
      renderKnowledge();
      goApp("knowledge-list", "<b>ナレッジ</b>");
    }
  }

  async function saveDomainFromDialog(event) {
    event.preventDefault();
    const fromDomainPage = Boolean(event.currentTarget.closest("#scr-knowledge-domain"));
    const form = new FormData(event.currentTarget);
    const displayName = String(form.get("display_name") || "").trim();
    const id = String(form.get("id") || "").trim() || generatedDomainId(displayName);
    const wasExisting = Boolean(findDomain(id));
    const request = {
      root: rootValue(),
      id,
      display_name: displayName,
      description: String(form.get("description") || ""),
      shared_knowledge: String(form.get("shared_knowledge") || ""),
      common_rubric: String(form.get("common_rubric") || ""),
      dispatch_hints: String(form.get("dispatch_hints") || ""),
    };
    try {
      state = await call("save_domain", {
        request,
      });
      if (!wasExisting) pendingDomainFollowupId = id;
      pendingKnowledgeResult = domainSaveResultFromRequest(request);
      closeDynamicModal();
      renderSettingsScreens();
      if (domainReturnContext?.kind === "project") {
        const tab = domainReturnContext.tab || "knowledge";
        domainReturnContext = null;
        renderProjectScreens();
        goApp("project-list", "<b>プロジェクト</b>");
        openProjectSettingsDialog(tab);
        toast("ドメインを保存しました。プロジェクト設定に戻りました。");
        return;
      }
      if (fromDomainPage) {
        openDomainDialog(findDomain(id), "basic");
      }
      toast("ドメインを保存しました。");
    } catch (error) {
      showOperationError(event.currentTarget, "ドメインを保存できませんでした", error, "表示名、共通知識、割り当てヒントを確認してから、もう一度保存してください。");
      toast(String(error), "error");
    }
  }

  function domainSaveResultFromRequest(request) {
    const knowledgeCount = textLines(request.shared_knowledge).length;
    const rubricCount = textLines(request.common_rubric).length;
    const hintCount = textLines(request.dispatch_hints).length;
    return {
      kind: "save",
      entity: "domain",
      id: request.id,
      domain_id: request.id,
      name: request.display_name || request.id,
      summary: "ドメインの共通知識と品質基準を更新しました。次のワークから自動注入されます。",
      detail: `共通知識 ${knowledgeCount}件 · 共通ルーブリック ${rubricCount}件 · 割り当てヒント ${hintCount}件`,
    };
  }

  function openDeleteDomainDialog(id) {
    const domain = findDomain(id);
    const artifacts = (state.artifact_types || []).filter((artifact) => artifact.domain_id === id);
    const blocked = artifacts.length > 0;
    const modal = dynamicModal(`
      <div class="modal">
        <div class="modal-head"><h3>ドメインを削除しますか?</h3><div class="m-sub">${escapeHtml(domain?.name || id)} をナレッジから削除します。</div></div>
        <div class="modal-body">
          <div class="card" style="padding:10px 12px;">
            <div class="kv"><div class="k">対象</div><div class="v">${escapeHtml(domain?.name || id)}</div></div>
            <div class="kv"><div class="k">共通知識</div><div class="v">${escapeHtml(String((domain?.shared_knowledge || []).length))}件</div></div>
            <div class="kv"><div class="k">成果物</div><div class="v">${escapeHtml(artifacts.length ? artifacts.map((artifact) => artifact.name || artifact.id).join("、") : "なし")}</div></div>
            <div class="kv"><div class="k">履歴</div><div class="v">過去のワーク履歴と実行記録は残ります</div></div>
          </div>
          ${blocked ? `
            <div class="card" style="padding:10px 12px;margin-top:10px;border-color:var(--warning-line);background:var(--warning-soft);">
              <b style="color:var(--warning);">このドメインはまだ削除できません</b>
              <div style="font-size:12.5px;color:var(--text-muted);margin-top:4px;">先に成果物を削除または別ドメインへ移動してください。</div>
            </div>
          ` : `<div class="hint" style="margin-top:8px;">削除後、このドメインの共通知識は新しいワークへ自動注入されません。</div>`}
        </div>
        <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>閉じる</button><button class="btn btn-primary" type="button" data-confirm ${blocked ? "disabled" : ""}>削除</button></div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-confirm]").addEventListener("click", async () => {
      try {
        const deletedResult = {
          kind: "delete",
          entity: "domain",
          id,
          domain_id: id,
          name: domain?.name || id,
          summary: "ドメインをナレッジから削除しました。新しいワークには自動注入されません。",
          detail: `共通知識 ${(domain?.shared_knowledge || []).length}件 · 成果物 ${artifacts.length}件 · 過去のワーク履歴は保持`,
        };
        state = await call("delete_domain_command", { request: { root: rootValue(), id } });
        if (pendingDomainFollowupId === id) pendingDomainFollowupId = null;
        pendingKnowledgeResult = deletedResult;
        closeDynamicModal();
        renderSettingsScreens();
        toast("ドメインを削除しました。");
      } catch (error) {
        showOperationError(modal, "ドメインを削除できませんでした", error, "成果物の有無とナレッジの参照状態を確認してから、もう一度削除してください。");
        toast(String(error), "error");
      }
    });
  }

  function openArtifactDialog(artifact = null, proposal = null, defaultDomainId = "") {
    const isEdit = Boolean(artifact);
    const selectedDomainId = artifact?.domain_id || defaultDomainId || state.domains?.[0]?.id || "";
    const usePage = !artifactReturnContext?.kind;
    const title = isEdit ? (artifact.name || artifact.id) : "成果物を追加";
    const formBody = `
          ${proposalHiddenInputs(proposal)}
          <div class="field"><label>表示名</label><input name="display_name" type="text" value="${escapeHtml(artifact?.name || "")}" placeholder="例: README"></div>
          <input name="domain_id" type="hidden" value="${escapeHtml(selectedDomainId)}">
          <div class="field"><label>説明</label><textarea name="description" style="min-height:72px;" placeholder="何を、誰に、どこへ出す成果物かを書いてください">${escapeHtml(artifact?.description || "")}</textarea></div>
          <div class="field"><label>作成指示（1行に1つ）</label><textarea name="knowledge" style="min-height:160px;" placeholder="例: 読み手が判断に必要な前提と結論を明記する&#10;指定テンプレートと表記規則に従う">${escapeHtml((artifact?.knowledge || []).join("\n"))}</textarea><div class="hint">この成果物を作るエージェントへ、ドメインの共通知識と一緒に渡されます。</div></div>
          <input name="id" type="hidden" value="${escapeHtml(artifact?.id || "")}">
          <textarea name="dispatch_hints" hidden>${escapeHtml((artifact?.dispatch_hints || []).join("\n"))}</textarea>
          <div class="field">
            <div style="display:flex;align-items:center;gap:8px;margin-bottom:6px;">
              <label style="margin:0;">ルーブリック（評価基準）</label>
              <span style="flex:1"></span>
              <button class="btn btn-secondary btn-sm" type="button" data-rubric-draft>評価基準をAIで作成</button>
            </div>
            <textarea name="rubric" class="mono" style="min-height:260px;">${escapeHtml((artifact?.rubric || []).join("\n"))}</textarea>
            <div class="hint">成果物の合格条件をMarkdownで記述します。AIで作成すると現在の入力を置き換えますが、保存するまでは確定しません。</div>
            <div class="card" data-rubric-status style="padding:10px 12px;margin-top:8px;"></div>
            ${proposal ? `
              <div class="card" style="padding:10px 12px;margin-top:8px;border-color:var(--warning-line);background:var(--warning-soft);">
                <div style="display:flex;align-items:center;gap:10px;">
                  <div style="flex:1;min-width:0;">
                    <div style="font-size:12.5px;font-weight:700;">過去のレビューに基づく評価基準の改善案</div>
                    <div class="hint" style="margin-top:3px;">${escapeHtml(proposal.summary || proposal.evidence || proposal.title || "評価基準に追加できる改善案があります。")} 反映後も、保存するまでは確定しません。</div>
                  </div>
                  <button class="btn btn-secondary btn-sm" type="button" data-insert-proposal>評価基準に追加</button>
                </div>
              </div>` : ""}
          </div>
          <div class="modal-foot">
            <button class="btn btn-secondary" type="button" data-close>閉じる</button>
            <button class="btn btn-primary" type="submit">保存</button>
          </div>`;
    let modal;
    if (usePage) {
      closeDynamicModal();
      const page = document.getElementById("scr-knowledge-rubric");
      if (!page) return;
      const domain = findDomain(selectedDomainId);
      page.innerHTML = `
        <div style="display:flex;align-items:flex-start;gap:12px;margin-bottom:14px;">
          <div style="flex:1;min-width:0;">
            <h2 class="page-title" style="margin-bottom:0;">${escapeHtml(title)}</h2>
            <p class="page-sub" style="margin:4px 0 0;">${escapeHtml(domain?.name || "ドメイン")} の作成指示とルーブリックを編集します。作成とレビューの両方に渡されます。</p>
          </div>
          <button class="btn btn-secondary btn-sm" type="button" data-close>ナレッジ一覧へ</button>
        </div>
        <form class="card" style="padding:16px;" data-artifact-form>${formBody}</form>`;
      goApp("knowledge-rubric", `ナレッジ / ${escapeHtml(domain?.name || "ドメイン")} / <b>${escapeHtml(title)}</b>`);
      modal = page;
    } else {
      modal = dynamicModal(`
        <div class="modal" style="width:760px;">
          <div class="modal-head"><h3>${escapeHtml(title)}</h3><div class="m-sub">成果物ごとの知識とルーブリックを設定します。説明・知識・ルーブリックは作成とレビューの両方に渡されます。</div></div>
          <form class="modal-body" data-artifact-form>${formBody}</form>
        </div>`);
    }
    modal.querySelectorAll("[data-close]").forEach((button) => button.addEventListener("click", closeArtifactDialog));
    const artifactForm = modal.querySelector("[data-artifact-form]");
    bindGeneratedId(artifactForm, generatedArtifactId);
    artifactForm.addEventListener("submit", saveArtifactFromDialog);
    modal.querySelector("[data-rubric-draft]").addEventListener("click", () => insertRubricDraft(modal));
    modal.querySelector('textarea[name="rubric"]')?.addEventListener("input", () => updateRubricStatus(modal));
    modal.querySelector("[data-insert-proposal]")?.addEventListener("click", () => insertProposalDraft(modal, proposal, "artifact"));
    updateRubricStatus(modal);
  }

  function aiSupportPanel({ generateTitle, generateDescription, generateAttr, proposal, proposalEmptyText, proposalActionLabel }) {
    return `
      <div class="card" style="padding:12px;margin:4px 0 12px;">
        <div style="font-size:12px;font-weight:800;color:var(--text-body);margin-bottom:8px;">AI支援</div>
        <div class="list" style="gap:8px;">
          <div class="list-item" style="padding:10px 12px;">
            <div class="li-icon" style="background:var(--primary-soft);color:var(--primary);">AI</div>
            <div style="flex:1;min-width:0;">
              <div class="li-title" style="font-size:13px;">${escapeHtml(generateTitle)}</div>
              <div class="li-desc">${escapeHtml(generateDescription)}</div>
            </div>
            <button class="btn btn-secondary btn-sm" type="button" ${generateAttr}>AIで下書き</button>
          </div>
          <div class="list-item" style="padding:10px 12px;${proposal ? "background:var(--warning-soft);" : ""}">
            <div class="li-icon" style="background:${proposal ? "var(--warning-soft)" : "#f8fafc"};color:${proposal ? "var(--warning)" : "var(--text-muted)"};">!</div>
            ${proposal ? `
          <div style="flex:1;min-width:0;">
            <div class="li-title" style="font-size:13px;">改善提案 — Nagareが実績から検出</div>
            <div class="li-desc"><b>${escapeHtml(proposal.title || "改善提案")}</b> · ${escapeHtml(proposal.summary || proposal.evidence || "")}</div>
          </div>
          <button class="btn btn-primary btn-sm" type="button" data-insert-proposal>${escapeHtml(proposalActionLabel)}</button>
            ` : `
          <div style="flex:1;">
            <div class="li-title" style="font-size:13px;color:var(--text-muted);">${escapeHtml(proposalEmptyText)}</div>
            <div class="li-desc">レビュー履歴から必要になった時だけ、分析・改善に提案が届きます。</div>
          </div>
            `}
          </div>
        </div>
      </div>`;
  }

  function insertProposalDraft(modal, proposal, target) {
    const suggested = String(proposal?.suggested_text || proposal?.summary || "").trim();
    if (!suggested) {
      toast("挿入できる提案本文がありません。", "error");
      return;
    }
    if (target === "agent") {
      modal.querySelector('textarea[name="prompt"]').value = suggested;
      return;
    }
    const textarea = modal.querySelector('textarea[name="rubric"]');
    const current = textarea.value.trim();
    textarea.value = current.includes(suggested) ? current : current ? `${current}\n\n${suggested}` : suggested;
  }

  function domainOptions(current) {
    return (state.domains || []).map((domain) => `<option value="${escapeHtml(domain.id)}" ${current === domain.id ? "selected" : ""}>${escapeHtml(domain.name)}</option>`).join("");
  }

  async function insertRubricDraft(modal) {
    const button = modal.querySelector("[data-rubric-draft]");
    const form = modal.querySelector("[data-artifact-form]");
    if (!form) return;
    const data = new FormData(form);
    try {
      if (button) button.disabled = true;
      const response = await call("generate_rubric_draft", {
        request: {
          root: rootValue(),
          domain_id: String(data.get("domain_id") || ""),
          display_name: String(data.get("display_name") || "").trim(),
          description: String(data.get("description") || "").trim(),
          knowledge: textLines(data.get("knowledge")),
        },
      });
      modal.querySelector('textarea[name="rubric"]').value = response.text || "";
      updateRubricStatus(modal);
    } catch (error) {
      showOperationError(form, "評価基準をAIで作成できませんでした", error, "成果物の説明、作成指示、ドメイン共通知識を確認してから、もう一度実行してください。");
      toast(String(error), "error");
    } finally {
      if (button) button.disabled = false;
    }
  }

  function summarizeRubricMarkdown(raw) {
    const text = String(raw || "").trim();
    if (!text) return { itemCount: 0, total: 0, valid: false, message: "ルーブリックが未入力です。" };
    const headings = [...text.matchAll(/^##\s+(.+?)\s*\((\d+)\)\s*$/gm)];
    if (!headings.length) return { itemCount: 0, total: 0, valid: false, message: "見出し形式が見つかりません。" };
    const total = headings.reduce((sum, match) => sum + Number(match[2] || 0), 0);
    const names = headings.map((match) => String(match[1] || "").trim());
    const duplicate = names.find((name, index) => names.indexOf(name) !== index);
    if (duplicate) return { itemCount: headings.length, total, valid: false, message: `項目名が重複しています: ${duplicate}` };
    if (total !== 100) return { itemCount: headings.length, total, valid: false, message: `配点合計が ${total}点です。100点にしてください。` };
    return { itemCount: headings.length, total, valid: true, message: `${headings.length}項目 / 合計100点` };
  }

  function updateRubricStatus(modal) {
    const target = modal.querySelector("[data-rubric-status]");
    if (!target) return;
    const summary = summarizeRubricMarkdown(modal.querySelector('textarea[name="rubric"]')?.value);
    target.style.borderColor = summary.valid ? "var(--success-line)" : "var(--warning-line)";
    target.style.background = summary.valid ? "var(--success-soft)" : "var(--warning-soft)";
    target.innerHTML = `
      <div style="display:flex;align-items:center;gap:8px;">
        <span class="${summary.valid ? "badge badge-done" : "badge badge-ask"}">${summary.valid ? "形式OK" : "要確認"}</span>
        <span style="font-size:12.5px;color:var(--text-body);">${escapeHtml(summary.message)}</span>
      </div>
    `;
  }

  async function saveArtifactFromDialog(event) {
    event.preventDefault();
    const fromArtifactPage = Boolean(event.currentTarget.closest("#scr-knowledge-rubric"));
    const form = new FormData(event.currentTarget);
    const domainId = String(form.get("domain_id") || "").trim();
    const displayName = String(form.get("display_name") || "").trim();
    const request = {
      root: rootValue(),
      id: String(form.get("id") || "").trim() || generatedArtifactId(displayName),
      domain_id: domainId,
      display_name: displayName,
      description: String(form.get("description") || ""),
      knowledge: String(form.get("knowledge") || ""),
      rubric: String(form.get("rubric") || ""),
      dispatch_hints: String(form.get("dispatch_hints") || ""),
      ...proposalRequestFields(form),
    };
    const rubric = summarizeRubricMarkdown(request.rubric);
    if (!rubric.valid) {
      updateRubricStatus(event.currentTarget.closest(".screen, .overlay") || document);
      showOperationError(event.currentTarget, "ルーブリックを保存できませんでした", new Error(rubric.message), "配点合計を100点にし、項目名の重複がないMarkdown形式に直してから保存してください。");
      toast(rubric.message, "error");
      return;
    }
    try {
      state = await call("save_artifact_type", {
        request,
      });
      reconcileAppliedImprovement(request);
      if (pendingDomainFollowupId === domainId) pendingDomainFollowupId = null;
      pendingKnowledgeResult = artifactSaveResultFromRequest(request);
      closeDynamicModal();
      renderSettingsScreens();
      syncStateChrome();
      if (artifactReturnContext?.kind === "domain") {
        const returnDomainId = domainId || artifactReturnContext.domainId;
        const tab = artifactReturnContext.tab || "artifacts";
        artifactReturnContext = null;
        openDomainDialog(findDomain(returnDomainId), tab);
        toast("成果物を保存しました。");
        return;
      }
      if (fromArtifactPage) {
        renderKnowledge();
        goApp("knowledge-list", "<b>ナレッジ</b>");
      }
      toast("成果物を保存しました。");
    } catch (error) {
      showOperationError(event.currentTarget, "成果物を保存できませんでした", error, "表示名、ドメイン、説明、ルーブリック形式を確認してから、もう一度保存してください。");
      toast(String(error), "error");
    }
  }

  function artifactSaveResultFromRequest(request) {
    const rubric = summarizeRubricMarkdown(request.rubric);
    const knowledgeCount = textLines(request.knowledge).length;
    const hintCount = textLines(request.dispatch_hints).length;
    return {
      kind: "save",
      entity: "artifact",
      id: request.id,
      domain_id: request.domain_id,
      name: request.display_name || request.id,
      summary: "成果物の作成指示とルーブリックを更新しました。作成とレビューの両方に渡されます。",
      detail: `ドメイン: ${domainName(request.domain_id)} · 作成指示 ${knowledgeCount}件 · ${rubric.valid ? rubric.message : `ルーブリック要確認: ${rubric.message}`} · 割り当てヒント ${hintCount}件`,
    };
  }

  function closeArtifactDialog() {
    if (artifactReturnContext?.kind === "domain") {
      const domainId = artifactReturnContext.domainId;
      const tab = artifactReturnContext.tab || "artifacts";
      artifactReturnContext = null;
      closeDynamicModal();
      openDomainDialog(findDomain(domainId), tab);
      return;
    }
    artifactReturnContext = null;
    closeDynamicModal();
    if (document.querySelector("#scr-knowledge-rubric.screen.active [data-artifact-form]")) {
      renderKnowledge();
      goApp("knowledge-list", "<b>ナレッジ</b>");
    }
  }

  function openDeleteArtifactDialog(id) {
    const artifact = findArtifact(id);
    const domain = findDomain(artifact?.domain_id);
    const fromArtifactPage = Boolean(document.querySelector("#scr-knowledge-rubric.screen.active [data-artifact-form]"));
    const rubric = summarizeRubricMarkdown(lines(artifact?.rubric).join("\n"));
    const rubricCount = artifact?.rubric_count || rubric.itemCount || 0;
    const rubricScore = artifact?.rubric_score_total || rubric.total || 0;
    const modal = dynamicModal(`
      <div class="modal">
        <div class="modal-head"><h3>成果物を削除しますか?</h3><div class="m-sub">${escapeHtml(artifact?.name || id)} を ${escapeHtml(domain?.name || domainName(artifact?.domain_id))} から削除します。</div></div>
        <div class="modal-body">
          <div class="card" style="padding:10px 12px;">
            <div class="kv"><div class="k">対象</div><div class="v">${escapeHtml(artifact?.name || id)}</div></div>
            <div class="kv"><div class="k">ドメイン</div><div class="v">${escapeHtml(domain?.name || domainName(artifact?.domain_id))}</div></div>
            <div class="kv"><div class="k">作成指示</div><div class="v">${escapeHtml(String((artifact?.knowledge || []).length))}件</div></div>
            <div class="kv"><div class="k">ルーブリック</div><div class="v">${escapeHtml(`${rubricCount}項目 / ${rubricScore}点`)}</div></div>
            <div class="kv"><div class="k">履歴</div><div class="v">過去のワーク履歴と実行記録は残ります</div></div>
          </div>
          <div class="hint" style="margin-top:8px;">削除後、この成果物の作成指示とルーブリックは新しいワークへ自動注入されません。</div>
        </div>
        <div class="modal-foot"><button class="btn btn-secondary" type="button" data-close>キャンセル</button><button class="btn btn-danger-soft" type="button" data-confirm>削除する</button></div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", () => {
      const returnContext = artifactReturnContext?.kind === "domain"
        ? { domainId: artifactReturnContext.domainId || artifact?.domain_id, tab: artifactReturnContext.tab || "artifacts" }
        : null;
      artifactReturnContext = null;
      closeDynamicModal();
      if (returnContext) openDomainDialog(findDomain(returnContext.domainId), returnContext.tab);
    });
    modal.querySelector("[data-confirm]").addEventListener("click", async () => {
      try {
        const returnContext = artifactReturnContext?.kind === "domain"
          ? { domainId: artifactReturnContext.domainId || artifact?.domain_id, tab: artifactReturnContext.tab || "artifacts" }
          : null;
        const deletedResult = {
          kind: "delete",
          entity: "artifact",
          id,
          domain_id: artifact?.domain_id,
          name: artifact?.name || id,
          summary: "成果物を削除しました。新しいワークにはこの作成指示とルーブリックは自動注入されません。",
          detail: `ドメイン: ${domain?.name || domainName(artifact?.domain_id)} · 作成指示 ${(artifact?.knowledge || []).length}件 · ルーブリック ${rubricCount}項目 / ${rubricScore}点 · 過去のワーク履歴は保持`,
        };
        state = await call("delete_artifact_type_command", { request: { root: rootValue(), id } });
        pendingKnowledgeResult = deletedResult;
        closeDynamicModal();
        renderSettingsScreens();
        if (returnContext) {
          artifactReturnContext = null;
          openDomainDialog(findDomain(returnContext.domainId), returnContext.tab);
        }
        if (fromArtifactPage) {
          renderKnowledge();
          goApp("knowledge-list", "<b>ナレッジ</b>");
        }
        toast("成果物を削除しました。");
      } catch (error) {
        showOperationError(modal, "成果物を削除できませんでした", error, "ドメイン内の成果物と参照状態を確認してから、もう一度削除してください。");
        toast(String(error), "error");
      }
    });
  }

  function renderInsights() {
    const el = document.getElementById("scr-insights");
    if (!el || !state) return;
    const insights = state.insights || {};
    const proposalCount = insights.proposal_count ?? insights.proposals?.length ?? 0;
    el.innerHTML = `
      <h2 class="page-title" style="margin-bottom:0;">分析・改善</h2>
      <p class="page-sub" style="margin-top:4px;">レビュー履歴から、成績が落ちているエージェントと失点の原因を見つけ、プロンプトやルーブリックの改善につなげます。</p>

      <div class="tabs">
        <button class="tab ${activeInsightsTab === "analysis" ? "active" : ""}" type="button" data-insights-tab="analysis">分析</button>
        <button class="tab ${activeInsightsTab === "improvements" ? "active" : ""}" type="button" data-insights-tab="improvements">改善 ${proposalCount ? `<span class="badge badge-ask" style="margin-left:4px;"><span class="bdot"></span>${proposalCount}件</span>` : ""}</button>
      </div>

      <div class="tabpane ${activeInsightsTab === "analysis" ? "active" : ""}" data-insights-pane="analysis">
        <div class="kpi-row">
          <div class="card stat-tile"><div class="st-label">レビュー済みワーク</div><div class="st-value">${escapeHtml(insights.review_count ?? 0)}<span class="unit">件</span></div><div class="st-delta up">現在の記録</div></div>
          <div class="card stat-tile"><div class="st-label">平均評価点</div><div class="st-value">${escapeHtml(insights.average_score_label || "-")}</div><div class="st-delta up">レビュー集計</div></div>
          <div class="card stat-tile"><div class="st-label">懸念</div><div class="st-value">${escapeHtml(insights.concern_count ?? 0)}<span class="unit">件</span></div><div class="st-delta ${Number(insights.concern_count || 0) ? "" : "up"}">${Number(insights.concern_count || 0) ? "確認対象" : "懸念なし"}</div></div>
          <div class="card stat-tile"><div class="st-label">改善提案</div><div class="st-value">${escapeHtml(proposalCount)}<span class="unit">件</span></div><div class="st-delta ${proposalCount ? "" : "up"}">${proposalCount ? "未対応" : "対応不要"}</div></div>
        </div>

        <div class="group-head" style="margin-top:6px;"><h3>エージェント別成績</h3></div>
        <div class="card" style="overflow:hidden;margin-bottom:16px;">
          ${insights.agent_scores?.length ? `
            <table class="itable">
              <thead><tr><th>エージェント</th><th class="num">レビュー</th><th class="num">平均評価点</th><th>状態</th><th>最も失点する項目</th><th></th></tr></thead>
              <tbody>${insights.agent_scores.map((agent) => `
                <tr ${agent.status_label === "要改善" ? `style="background:#fffdf5;"` : ""}>
                  <td><b>${escapeHtml(agent.agent_name)}</b> · ${escapeHtml(roleLabel(agent.role))}</td>
                  <td class="num">${escapeHtml(agent.review_count)}</td>
                  <td class="num"><b>${escapeHtml(agent.average_score_label || `${agent.average_score ?? "-"} / 100`)}</b></td>
                  <td>${agent.status_label ? `<span class="${agent.status_label === "要改善" ? "badge badge-ask" : "badge badge-neutral"}">${escapeHtml(agent.status_label)}</span>` : ""}</td>
                  <td>${escapeHtml(agent.top_issue || "-")}</td>
                  <td style="text-align:right;">${agent.status_label === "要改善" ? `<button class="btn btn-primary btn-sm" type="button" data-open-improvements>改善案を見る</button>` : ""}</td>
                </tr>
              `).join("")}</tbody>
            </table>
          ` : `<div class="list-empty">レビュー履歴が増えると、エージェント別の傾向が表示されます。</div>`}
        </div>

        <div class="group-head"><h3>失点の内訳</h3><span class="count">75%未満が改善候補</span></div>
        <div class="card" style="overflow:hidden;margin-bottom:16px;">
          ${insights.issue_matrix?.length ? `
            <table class="itable">
              <thead><tr><th>エージェント</th><th>項目</th><th class="num">獲得率</th><th class="num">発生</th><th>改善候補</th></tr></thead>
              <tbody>${insights.issue_matrix.map((issue) => `
                <tr ${Number(issue.rate || 0) < 75 ? `style="background:var(--warning-soft);"` : ""}>
                  <td><b>${escapeHtml(issue.agent_name)}</b></td>
                  <td>${escapeHtml(issue.item)}</td>
                  <td class="num"><b>${escapeHtml(issue.rate_label || `${issue.rate}%`)}</b></td>
                  <td class="num">${escapeHtml(issue.occurrences)}件</td>
                  <td>${escapeHtml(issue.suggestion_kind || "-")}</td>
                </tr>
              `).join("")}</tbody>
            </table>
          ` : `<div class="list-empty">失点傾向はまだありません。</div>`}
        </div>

        <div class="group-head"><h3>最近のレビュー</h3></div>
        <div class="list">${insights.recent_reviews?.length ? insights.recent_reviews.map((review) => `
          <div class="list-item">
            <div class="wr-picon">評</div>
            <div class="wr-body">
              <div class="wr-title">${escapeHtml(review.title)} <span class="wr-projtag">${escapeHtml(review.score_label || review.verdict)}</span></div>
              <div class="wr-sum">${escapeHtml(review.agent_name)} · ${escapeHtml((review.concerns || []).join(" / ") || "懸念なし")}</div>
            </div>
          </div>
        `).join("") : `<div class="list-empty">最近のレビューはありません。</div>`}</div>
      </div>

      <div class="tabpane ${activeInsightsTab === "improvements" ? "active" : ""}" data-insights-pane="improvements">
        <div class="card" style="padding:12px 16px;margin:4px 0 14px;display:flex;gap:10px;align-items:center;background:var(--primary-soft);border-color:var(--primary-line);">
          <span style="font-size:14px;">!</span>
          <span style="font-size:12.5px;color:var(--text-body);">改善提案は自動適用しません。Diffと根拠を確認し、対象画面で人が編集します。</span>
        </div>
        <div class="group-head" style="margin-top:4px;">
          <h3>未対応の改善提案</h3>
          <span class="count">${escapeHtml(proposalCount)}件</span>
          <span style="flex:1"></span>
          ${insights.proposals?.length ? `<button class="btn btn-secondary btn-sm" type="button" data-open-first-proposal>今すぐ見直す</button>` : ""}
        </div>
        <div class="list" data-pending-improvements>${insights.proposals?.length ? insights.proposals.map((proposal) => `
          <div class="list-item" style="background:#fffdf5;">
            <div class="li-icon" style="background:var(--warning-soft);color:var(--warning);">!</div>
            <div style="flex:1;">
              <div class="li-title">${escapeHtml(proposal.title)} <span class="badge badge-neutral" style="margin-left:4px;">${escapeHtml(proposal.kind)}</span></div>
              <div class="li-desc">${escapeHtml(proposal.summary)} · 根拠: ${escapeHtml(proposal.evidence)}</div>
            </div>
            <button class="btn btn-primary btn-sm" type="button" data-preview-proposal="${escapeHtml(proposal.id)}">プレビュー</button>
          </div>
        `).join("") : `<div class="list-empty">未対応の改善提案はありません。</div>`}</div>
        <div class="group-head">
          <h3>適用済みの改善</h3>
          <span class="count">${escapeHtml(String((insights.applied_improvements || []).length))}件</span>
        </div>
        <div class="list" data-applied-improvements>
          ${insights.applied_improvements?.length ? insights.applied_improvements.map((item) => `
            <div class="list-item">
              <div class="li-icon" style="background:var(--success-soft);color:var(--success);">✓</div>
              <div style="flex:1;">
                <div class="li-title">${escapeHtml(item.title)}</div>
                <div class="li-desc">${escapeHtml(item.summary || "")}${item.applied_at ? ` · 適用: ${escapeHtml(item.applied_at)}` : ""}</div>
              </div>
              <div style="display:flex;align-items:center;gap:8px;">
                <span style="font-size:12px;color:var(--success);font-weight:600;">${escapeHtml(item.effect_label || "効果測定中")}</span>
                <button class="btn btn-secondary btn-sm" type="button" data-open-applied-improvement="${escapeHtml(item.id || item.proposal_id || item.title)}">対象を確認</button>
              </div>
            </div>
          `).join("") : `<div class="list-empty">適用済みの改善はまだありません。提案を対象画面で反映すると、効果測定の候補として扱えます。</div>`}
        </div>
        <p style="font-size:12px;color:var(--text-faint);margin-top:12px;">適用後は次のワーク結果を見ながら、悪化していないかを確認します。</p>
      </div>
    `;
    el.querySelectorAll("[data-insights-tab]").forEach((button) => {
      button.addEventListener("click", () => {
        activeInsightsTab = button.dataset.insightsTab;
        renderInsights();
      });
    });
    el.querySelectorAll("[data-open-improvements]").forEach((button) => {
      button.addEventListener("click", () => {
        activeInsightsTab = "improvements";
        renderInsights();
      });
    });
    el.querySelectorAll("[data-preview-proposal]").forEach((button) => {
      button.addEventListener("click", () => openImprovementProposalDialog(button.dataset.previewProposal));
    });
    el.querySelector("[data-open-first-proposal]")?.addEventListener("click", () => {
      const firstProposal = state?.insights?.proposals?.[0];
      if (firstProposal?.id) openImprovementProposalDialog(firstProposal.id);
    });
    el.querySelectorAll("[data-open-applied-improvement]").forEach((button) => {
      button.addEventListener("click", () => {
        const key = button.dataset.openAppliedImprovement;
        const item = (state?.insights?.applied_improvements || []).find((candidate) =>
          [candidate.id, candidate.proposal_id, candidate.title].includes(key),
        );
        if (item) goImprovementTarget(item, { applied: true });
      });
    });
  }

  function openImprovementProposalDialog(id) {
    const proposal = (state?.insights?.proposals || []).find((item) => item.id === id);
    if (!proposal) return;
    const modal = dynamicModal(`
      <div class="modal" style="width:640px;">
        <div class="modal-head">
          <div class="m-step">改善プレビュー</div>
          <h3>${escapeHtml(proposal.title)}</h3>
          <div class="m-sub">対象: ${escapeHtml(proposal.target_label || proposal.kind)}</div>
        </div>
        <div class="modal-body">
          <div style="margin-bottom:12px;">
            <div style="font-size:11px;font-weight:700;letter-spacing:.08em;color:var(--text-faint);text-transform:uppercase;margin-bottom:5px;">変更内容</div>
            <div class="diff" style="max-height:340px;overflow-y:auto;">
              ${(proposal.diff_lines?.length ? proposal.diff_lines : diffFromProposal(proposal)).map(diffLine).join("")}
            </div>
          </div>
          <div style="font-size:12.5px;color:var(--text-muted);border-top:1px solid var(--border);padding-top:10px;">
            <b style="color:var(--text-body);">提案の根拠:</b> ${escapeHtml(proposal.evidence || "")}
          </div>
          ${proposal.next_step ? `<div class="hint" style="margin-top:10px;">次の操作: ${escapeHtml(proposal.next_step)}</div>` : ""}
        </div>
        <div class="modal-foot">
          <button class="btn btn-secondary" type="button" data-close>閉じる</button>
          <button class="btn btn-secondary" type="button" data-dismiss-proposal>今回は見送る</button>
          <button class="btn btn-primary" type="button" data-open-target>${escapeHtml(proposal.action_label || "対象を開く")}</button>
        </div>
      </div>`);
    modal.querySelector("[data-close]").addEventListener("click", closeDynamicModal);
    modal.querySelector("[data-dismiss-proposal]").addEventListener("click", () => dismissImprovementProposal(modal, proposal));
    modal.querySelector("[data-open-target]").addEventListener("click", () => {
      closeDynamicModal();
      goImprovementTarget(proposal);
    });
  }

  async function dismissImprovementProposal(modal, proposal) {
    try {
      state = await call("dismiss_improvement", {
        request: {
          root: rootValue(),
          proposal_id: proposal.id,
          kind: proposal.kind || "",
          title: proposal.title || "改善提案を見送り",
          target_label: proposal.target_label || "",
          summary: proposal.summary || "",
          evidence: proposal.evidence || "",
        },
      });
      activeInsightsTab = "improvements";
      closeDynamicModal();
      syncStateChrome();
      toast("改善提案を見送りました。");
    } catch (error) {
      showOperationError(modal, "改善提案を見送れませんでした", error, "提案の状態を確認してから、もう一度実行してください。");
      toast(String(error), "error");
    }
  }

  function diffFromProposal(proposal) {
    return [
      `- ${proposal.current_text || "現在の設定"}`,
      `+ ${proposal.suggested_text || proposal.summary || "提案内容"}`,
    ];
  }

  function diffLine(line) {
    const text = String(line || "");
    const cls = text.startsWith("+") ? "add" : text.startsWith("-") ? "del" : text.startsWith("@") ? "hunk" : "ctx";
    return `<div class="dl ${cls}">${escapeHtml(text.replace(/^[+-]\s?/, ""))}</div>`;
  }

  function goImprovementTarget(proposal, options = {}) {
    const kind = `${proposal.kind || ""} ${proposal.title || ""} ${proposal.target_label || ""} ${proposal.next_step || ""} ${proposal.summary || ""}`.toLowerCase();
    if (kind.includes("agent") || kind.includes("prompt") || kind.includes("エージェント") || kind.includes("プロンプト")) {
      goApp("agent-list", "<b>エージェント</b>");
      const agent = findAgentByLabel(proposal.target_label || proposal.title);
      if (agent) {
        openAgentDialog(agent, options.applied ? null : proposal, "prompt");
      } else {
        toast("対象エージェントが見つかりません。エージェント一覧から確認してください。", "error");
      }
    } else if (kind.includes("rubric") || kind.includes("knowledge") || kind.includes("ルーブリック") || kind.includes("知識")) {
      goApp("knowledge-list", "<b>ナレッジ</b>");
      const artifact = findArtifactByLabel(proposal.target_label || proposal.title);
      if (artifact) {
        openArtifactDialog(artifact, options.applied ? null : proposal);
      } else {
        toast("対象の成果物が見つかりません。ナレッジ一覧から確認してください。", "error");
      }
    } else if (kind.includes("policy") || kind.includes("確認") || kind.includes("運用")) {
      goApp("project-list", "<b>プロジェクト</b>");
      openProjectSettingsDialog("policy", options.applied ? null : proposal);
    } else {
      goApp("insights", "<b>分析・改善</b>");
    }
  }

  function bindNavigation() {
    const bindings = {
      "nav-work": () => {
        if (state?.initialized) {
          renderHome();
          goApp("home-active");
        } else {
          renderEmptyHome();
          goApp("home-empty");
        }
      },
      "nav-project": () => {
        renderProjectScreens();
        goApp("project-list", "<b>プロジェクト</b>");
      },
      "nav-skills": () => goApp("settings-skills", "追加機能 / <b>スキル</b>"),
      "nav-mcp": () => goApp("settings-mcp", "追加機能 / <b>MCP接続</b>"),
      "nav-runtime": () => goApp("settings-runtime", "<b>実行環境</b>"),
      "nav-agent": () => goApp("agent-list", "<b>エージェント</b>"),
      "nav-knowledge": () => goApp("knowledge-list", "<b>ナレッジ</b>"),
      "nav-insights": () => {
        renderInsights();
        goApp("insights", "<b>分析・改善</b>");
      },
      "nav-catalog": () => {
        renderCatalog();
        goApp("catalog", "デザイン / <b>カタログ</b>");
      },
    };
    Object.entries(bindings).forEach(([id, handler]) => {
      const el = document.getElementById(id);
      if (!el) return;
      el.onclick = (event) => {
        event.preventDefault();
        handler();
      };
    });
  }

  async function init() {
    bindNavigation();
    if (invoke) {
      try {
        const root = await call("launch_root");
        if (root) {
          currentRoot = root;
          localStorage.setItem("nagare.root", root);
        }
      } catch (_) {
        // Older desktop binaries do not expose a launch root.
      }
      await loadState();
    }
  }

  return { init, loadState, openSetup };
})();

NagareApp.init();
