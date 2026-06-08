import { ChildProcess, execFileSync, spawn } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { expect, test } from "@playwright/test";

const repoRoot = path.resolve(__dirname, "../..");
const nagareBin = path.join(
  repoRoot,
  "target",
  "debug",
  process.platform === "win32" ? "nagare.exe" : "nagare",
);

function runNagare(args: string[], cwd: string = repoRoot): string {
  return execFileSync(nagareBin, args, {
    cwd,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

async function waitForServer(url: string, timeoutMs = 5000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let lastError: unknown;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`server did not become ready: ${String(lastError)}`);
}

function getFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        server.close(() => reject(new Error("failed to allocate port")));
        return;
      }
      const port = address.port;
      server.close(() => resolve(port));
    });
  });
}

function startUiServer(root: string, port: number): ChildProcess {
  return spawn(
    nagareBin,
    ["ui", "serve", "--root", root, "--host", "127.0.0.1", "--port", String(port), "--open", "false"],
    { cwd: repoRoot, stdio: ["ignore", "pipe", "pipe"] },
  );
}

function stopProcess(process: ChildProcess): Promise<void> {
  return new Promise((resolve) => {
    if (process.exitCode !== null) {
      resolve();
      return;
    }
    process.once("exit", () => resolve());
    process.kill();
    setTimeout(() => resolve(), 1000);
  });
}

function parseCommandLine(command: string): string[] {
  const args: string[] = [];
  let current = "";
  let quote: string | null = null;
  let escaping = false;
  for (const char of command.trim()) {
    if (escaping) {
      current += char;
      escaping = false;
      continue;
    }
    if (char === "\\") {
      escaping = true;
      continue;
    }
    if (quote) {
      if (char === quote) {
        quote = null;
      } else {
        current += char;
      }
      continue;
    }
    if (char === "\"" || char === "'") {
      quote = char;
      continue;
    }
    if (/\s/.test(char)) {
      if (current.length > 0) {
        args.push(current);
        current = "";
      }
      continue;
    }
    current += char;
  }
  if (escaping) {
    current += "\\";
  }
  if (quote) {
    throw new Error(`unterminated quote in command: ${command}`);
  }
  if (current.length > 0) {
    args.push(current);
  }
  return args;
}

function runDisplayedNagareCommand(command: string, cwd: string): string {
  const args = parseCommandLine(command);
  expect(args[0]).toBe("nagare");
  return runNagare(args.slice(1), cwd);
}

function runDisplayedAgentCommandWithFixture(
  command: string,
  cwd: string,
  fixtureCommand: string,
): string {
  const args = parseCommandLine(command);
  expect(args[0]).toBe("nagare");
  const nagareArgs = args.slice(1);
  const promptIndex = nagareArgs.indexOf("--prompt");
  if (promptIndex >= 0) {
    nagareArgs.splice(promptIndex, 2, "--command", fixtureCommand);
  } else {
    nagareArgs.push("--command", fixtureCommand);
  }
  return runNagare(nagareArgs, cwd);
}

function typeCommand(filePath: string): string {
  return process.platform === "win32" ? `type ${filePath}` : `cat "${filePath}"`;
}

test.beforeAll(() => {
  execFileSync("cargo", ["build", "-p", "nagare-cli"], {
    cwd: repoRoot,
    stdio: "inherit",
  });
});

test("static UI exposes attention queue and command builder interaction", async ({ page }) => {
  const root = mkdtempSync(path.join(tmpdir(), "nagare-ui-e2e-"));
  runNagare(["init", "--root", root]);
  writeFileSync(
    path.join(root, "question.md"),
    [
      "## Nagare Result",
      "status: blocked",
      "completed:",
      "- checked requirements",
      "questions:",
      "- 追加の方針は？",
      "next_notes:",
      "- waiting for user direction",
      "next_action: answer_question",
      "",
    ].join("\n"),
    "utf8",
  );
  writeFileSync(
    path.join(root, "agent-success.md"),
    [
      "## Nagare Result",
      "status: completed",
      "summary:",
      "- browser server run completed",
      "completed:",
      "- reflected browser answer",
      "questions:",
      "next_notes:",
      "- ready for review",
      "next_action: review",
      "",
    ].join("\n"),
    "utf8",
  );
  writeFileSync(
    path.join(root, "review-pass.md"),
    [
      "## Nagare Review",
      "verdict: pass",
      "summary:",
      "- looks good",
      "completed:",
      "- reviewed browser run",
      "findings:",
      "referenced_artifacts:",
      "requested_changes:",
      "questions:",
      "next_notes:",
      "- ready for approval",
      "next_action: approve",
      "",
    ].join("\n"),
    "utf8",
  );
  writeFileSync(
    path.join(root, "agent-success.md"),
    [
      "## Nagare Result",
      "status: completed",
      "summary:",
      "- continued after human feedback",
      "completed:",
      "- applied user direction",
      "questions:",
      "next_notes:",
      "- ready for review",
      "next_action: review",
      "",
    ].join("\n"),
    "utf8",
  );
  const createOutput = runNagare([
    "item",
    "create",
    "--root",
    root,
    "--title",
    "Needs user answer",
  ]);
  const workItemId = createOutput.match(/work_\d+/)?.[0];
  expect(workItemId).toBeTruthy();
  runNagare([
    "item",
    "run",
    workItemId!,
    "--root",
    root,
    "--command",
    "type question.md",
  ]);
  runNagare(["ui", "open", "--root", root, "--out", path.join(root, "ui"), "--open", "false"]);

  await page.goto(pathToFileURL(path.join(root, "ui", "index.html")).toString());
  await expect(page.getByRole("heading", { name: "確認キュー" })).toBeVisible();
  await expect(page.locator(".attention-row")).toContainText("Needs user answer");
  await page.getByRole("link", { name: "Needs user answer" }).click();

  await expect(page.getByRole("heading", { name: "Human Input Panel" })).toBeVisible();
  await expect(page.locator(".human-input")).toContainText("追加の方針は？");
  const input = page.locator(".human-input textarea");
  await input.fill("ユーザー確認済み。source URL を追加して続行してください。");

  const command = page.locator("#human-command-" + workItemId);
  await expect(command).toContainText(`nagare item answer ${workItemId} --answer`);
  await expect(command).toContainText("source URL を追加して続行してください");
  await expect(page.getByRole("button", { name: "Copy" })).toBeVisible();

  const commandText = (await command.textContent()) ?? "";
  const answerOutput = runDisplayedNagareCommand(commandText, root);
  expect(answerOutput).toContain("item_status=ready");

  runNagare(["ui", "open", "--root", root, "--out", path.join(root, "ui"), "--open", "false"]);
  await page.goto(pathToFileURL(path.join(root, "ui", "items", `${workItemId}.html`)).toString());
  await expect(page.locator(".timeline")).toContainText("ユーザー回答");
  await expect(page.locator(".timeline")).toContainText("ユーザー確認済み");
  await expect(page.locator(".next-action")).toContainText("run_agent");
  await expect(page.locator(".next-action")).toContainText(`nagare item run ${workItemId}`);

  const runInput = page.locator(".human-input textarea");
  await runInput.fill("回答内容を反映して次の作業を実行してください。");
  const runCommand = page.locator("#human-command-" + workItemId);
  await expect(runCommand).toContainText(`nagare item run ${workItemId} --prompt`);
  const runCommandText = (await runCommand.textContent()) ?? "";
  const runOutput = runDisplayedAgentCommandWithFixture(
    runCommandText,
    root,
    "type agent-success.md",
  );
  expect(runOutput).toContain("item_status=ready_for_review");

  runNagare(["ui", "open", "--root", root, "--out", path.join(root, "ui"), "--open", "false"]);
  await page.goto(pathToFileURL(path.join(root, "ui", "items", `${workItemId}.html`)).toString());
  await expect(page.locator(".timeline")).toContainText("completed");
  await expect(page.locator(".timeline")).toContainText("Agent Profile `worker` の実行が成功した");
  await expect(page.locator(".next-action")).toContainText("review");
});

test("local UI server creates a work item from the browser", async ({ page }) => {
  const root = mkdtempSync(path.join(tmpdir(), "nagare-ui-serve-e2e-"));
  runNagare(["init", "--root", root]);
  const port = await getFreePort();
  const server = startUiServer(root, port);
  try {
    await waitForServer(`http://127.0.0.1:${port}/`);
    await page.goto(`http://127.0.0.1:${port}/`);
    await expect(page.getByRole("heading", { name: "Nagare" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Work Itemを作成" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Agent Defaults" })).toHaveCount(0);
    await page.getByRole("link", { name: /Settings|設定/ }).click();
    await expect(page.getByRole("heading", { name: "Agent Defaults" })).toHaveCount(0);
    await expect(page.locator("#agent-defaults-form")).toHaveCount(0);
    await expect(page.getByRole("tab", { name: /Workflow|ワークフロー/ })).toHaveAttribute("aria-selected", "true");
    await expect(page.getByRole("tab", { name: "Domain Groups" })).toHaveCount(0);
    await page.getByRole("tab", { name: /Domains|ドメイン/, exact: true }).click();
    await expect(page.getByRole("heading", { name: "ドメイン", exact: true })).toBeVisible();
    await page.locator('a[href="/settings/domain-groups/new"]').click();
    await expect(page.locator("#domain-group-form")).toBeVisible();
    await page.locator('#domain-group-form input[name="id"]').fill("software-development");
    await page.locator('#domain-group-form input[name="display_name"]').fill("Software Development");
    await page
      .locator('#domain-group-form textarea[name="description"]')
      .fill("Software product changes.");
    await page
      .locator('#domain-group-form textarea[name="shared_knowledge"]')
      .fill("Prefer small changes\nKeep contracts stable");
    await page
      .locator('#domain-group-form textarea[name="common_rubric"]')
      .fill("Acceptance criteria pass\nChecks are reported");
    await page
      .locator('#domain-group-form textarea[name="dispatch_hints"]')
      .fill("Use specialized code agents.");
    await page.locator('#domain-group-form button[type="submit"]').click();
    await expect(page.getByRole("heading", { name: "ドメイン", exact: true })).toBeVisible();
    await expect(page.locator("#domain-groups")).toContainText("Software Development");
    await expect(page.locator("#domain-groups")).toContainText("2");
    await page.locator('a[href="/settings/domains/new"]').click();
    await expect(page.locator("#domain-profile-form")).toBeVisible();
    await page.locator('#domain-profile-form input[name="id"]').fill("frontend-ui");
    await page.locator('#domain-profile-form select[name="group_id"]').selectOption("software-development");
    await page.locator('#domain-profile-form input[name="display_name"]').fill("Frontend UI");
    await page
      .locator('#domain-profile-form textarea[name="description"]')
      .fill("User-facing browser UI and interaction flows.");
    await page.locator('#domain-profile-form textarea[name="artifact_types"]').fill("HTML\nscreenshot");
    await page
      .locator('#domain-profile-form textarea[name="rubric"]')
      .fill("Primary workflow is visible\nText does not overlap\nResponsive layout works");
    await page
      .locator('#domain-profile-form textarea[name="dispatch_hints"]')
      .fill("Use this domain for UI layout or interaction requests.");
    await page.locator('#domain-profile-form button[type="submit"]').click();
    await expect(page.getByRole("heading", { name: "ドメイン", exact: true })).toBeVisible();
    await expect(page.locator("#domain-profiles")).toContainText("Frontend UI");
    await expect(page.locator("#domain-profiles")).toContainText("Software Development");
    await expect(page.locator("#domain-profiles")).toContainText("3");
    await page.locator('a[href$="/settings/domains/frontend-ui"]').first().click();
    await expect(page.locator("#domain-profile-form")).toBeVisible();
    await page
      .locator('#domain-profile-form textarea[name="rubric"]')
      .fill("Primary workflow is visible\nText does not overlap\nResponsive layout works\nImportant controls are reachable");
    await page.locator('#domain-profile-form button[type="submit"]').click();
    await expect(page.locator("#domain-profiles")).toContainText("Frontend UI");
    await expect(page.locator("#domain-profiles")).toContainText("4");
    await page.getByRole("tab", { name: /Agents|エージェント/ }).click();
    await expect(page.getByRole("heading", { name: /Agents|エージェント/ })).toBeVisible();
    await expect(page.getByRole("columnheader", { name: "Role" })).toHaveCount(0);
    await expect(page.locator("#agent-profiles")).toContainText("Worker");
    await expect(page.locator("#agent-profiles")).toContainText("Reviewer");
    await expect(page.locator("#agent-profiles")).toContainText("Dispatcher");
    await expect(page.locator("#agent-profiles")).toContainText("Supervisor");
    await page.locator('a[href$="/settings/agents/worker"]').first().click();
    await expect(page.locator("#agent-profile-form")).toBeVisible();
    await page.locator('#agent-profile-form input[name="display_name"]').fill("Default Worker");
    await page.locator('#agent-profile-form button[type="submit"]').click();
    await expect(page.getByRole("heading", { name: /Agents|エージェント/ })).toBeVisible();
    await expect(page.locator("#agent-profiles")).toContainText("Default Worker");
    await page.locator('a[href$="/settings/agents/dispatcher"]').first().click();
    await expect(page.locator("#agent-profile-form")).toBeVisible();
    await page
      .locator('#agent-profile-form textarea[name="description"]')
      .fill("Dispatch work to the most suitable profile.");
    await page.locator('#agent-profile-form button[type="submit"]').click();
    await expect(page.locator("#agent-profiles")).toContainText("Dispatch work to the most suitable profile.");

    await page.locator('a[href="/settings/agents/new"]').click();
    await expect(page.locator("#agent-profile-form")).toBeVisible();
    await page.locator('#agent-profile-form input[name="id"]').fill("ui-agent");
    await page.locator('#agent-profile-form select[name="agent_kind"]').selectOption("codex_cli");
    await page.locator('#agent-profile-form input[name="display_name"]').fill("UI Agent");
    await expect(page.locator('#agent-profile-form select[name="role"]')).toBeVisible();
    await page.locator('#agent-profile-form select[name="role"]').selectOption("worker");
    await page.locator('#agent-profile-form input[name="working_dir"]').fill(".");
    await page.locator('#agent-profile-form select[name="domain_group_ids"]').selectOption("software-development");
    await page.locator('#agent-profile-form select[name="domain_ids"]').selectOption("frontend-ui");
    await page.locator('#agent-profile-form textarea[name="description"]').fill("Use SOUL.md when present.");
    await page.locator('#agent-profile-form textarea[name="specialties"]').fill("ui,e2e");
    await page.locator('#agent-profile-form button[type="submit"]').click();

    await expect(page.locator("#agent-profiles")).toContainText("UI Agent");
    const agentListOutput = runNagare(["agent", "list", "--root", root]);
    expect(agentListOutput).toContain("ui-agent");
    expect(agentListOutput).toContain("domain_groups=software-development");
    expect(agentListOutput).toContain("domains=frontend-ui");
    await page.locator('a[href$="/settings/agents/ui-agent"]').first().click();
    page.once("dialog", async (dialog) => {
      expect(dialog.message()).toContain("UI Agent");
      await dialog.accept();
    });
    await page.locator("#delete-agent-button").click();
    await expect(page.locator("#agent-profiles")).not.toContainText("UI Agent");
    await page.getByRole("link", { name: /Work Queue|作業キュー/ }).click();
    await expect(page.getByRole("link", { name: "Work Itemを作成" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Selected Work" })).toHaveCount(0);
    await expect(page.getByText(/Needs attention|確認が必要/)).toBeVisible();
    await expect(page.locator("#create-work-form")).toHaveCount(0);
    writeFileSync(
      path.join(root, "agent-success.md"),
      [
        "## Nagare Result",
        "status: completed",
        "summary:",
        "- workflow run completed",
        "completed:",
        "- implemented from browser workflow",
        "questions:",
        "next_notes:",
        "- ready for review",
        "next_action: review",
        "",
      ].join("\n"),
      "utf8",
    );
    writeFileSync(
      path.join(root, "review-pass.md"),
      [
        "## Nagare Review",
        "verdict: pass",
        "summary:",
        "- browser workflow review passed",
        "completed:",
        "- reviewed workflow run",
        "criteria:",
        "- 一覧に表示される: pass",
        "- confirm_firstで保存される: pass",
        "findings:",
        "referenced_artifacts:",
        "requested_changes:",
        "questions:",
        "next_notes:",
        "- ready for approval",
        "next_action: approve",
        "",
      ].join("\n"),
      "utf8",
    );
    await page.locator('a[href="/new"]').click();
    await expect(page.locator("#create-work-form")).toBeVisible();
    await page.locator('#create-work-form textarea[name="description"]').fill("ブラウザフォームから作る");
    await page.locator('#create-work-form select[name="domain_group_id"]').selectOption("software-development");
    await page.locator('#create-work-form select[name="domain_id"]').selectOption("frontend-ui");
    await page
      .locator('#create-work-form textarea[name="acceptance"]')
      .fill("一覧に表示される\nconfirm_firstで保存される");
    await page.locator('#create-work-form input[name="command"]').evaluate((node, value) => {
      (node as HTMLInputElement).value = value as string;
    }, typeCommand(path.join(root, "agent-success.md")));
    await page.locator('#create-work-form input[name="review_command"]').evaluate((node, value) => {
      (node as HTMLInputElement).value = value as string;
    }, typeCommand(path.join(root, "review-pass.md")));
    await page.locator('#create-work-form button[type="submit"]').click();

    await expect(page.getByRole("heading", { level: 1, name: "ブラウザフォームから作る" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Dispatch", exact: true })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Run Dispatch" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Run Workflow" })).toHaveCount(0);
    await expect(page.getByRole("heading", { name: "Run Agent" })).toHaveCount(0);
    await expect(page.locator("#detail")).toContainText("承認待ち");
    await expect(page.locator(".answer-panel")).toContainText("最終結果");
    await expect(page.locator(".answer-panel")).toContainText("workflow run completed");
    await expect(page.locator(".answer-panel")).toContainText("検証と実行情報");
    const technicalDetails = page.locator(".technical-details");
    await technicalDetails.locator("> summary").click();
    const history = technicalDetails.locator(".workflow-panel").filter({ hasText: "詳細ログ" });
    await expect(history).toContainText("Workflow Decision");
    await expect(history).toContainText("Reason");
    await expect(history).toContainText("Agent Output");
    await expect(history).toContainText("completed");
    await expect(history).toContainText("next_notes");
    await expect(history).toContainText("output record");
    await expect(history).toContainText("Raw output");
    await expect(history).toContainText("Review");
    await expect(history).toContainText("Criteria");
    await expect(history.locator('[data-event-type="artifact"]')).toHaveCount(0);
    await expect(history.locator('[data-event-type="agent_output"]')).toHaveCount(0);
    await expect(history.locator('[data-event-type="evidence"]')).toHaveCount(0);
    await expect(history.locator('[data-event-type="workflow_decision"]')).toHaveCount(0);
    await expect(history.locator('[data-event-type="dispatch"]')).not.toHaveCount(0);
    await expect(history.locator('[data-event-type="work"]')).not.toHaveCount(0);
    await expect(history.locator('[data-event-type="review"]')).not.toHaveCount(0);
    await expect(history.locator('[data-event-type="run"]')).toHaveCount(0);
    const firstDetails = history.locator(".history-details").first();
    await firstDetails.locator("summary").click();
    await expect(firstDetails).toHaveAttribute("open", "");
    await expect(firstDetails.locator(".detail-section")).not.toHaveCount(0);
    await page.waitForTimeout(1300);
    await expect(firstDetails).toHaveAttribute("open", "");
    await expect(page.getByRole("link", { name: "Action" })).toHaveCount(0);
    await expect(page.getByRole("link", { name: "Feedback" })).toHaveCount(0);
    await expect(page.getByRole("heading", { name: "Human Feedback" })).toHaveCount(0);
    const listOutput = runNagare(["item", "list", "--root", root]);
    expect(listOutput).toContain("ブラウザフォームから作る");
    await page.getByRole("link", { name: "Work Queue" }).first().click();
    await expect(page.locator("#work-items")).toContainText("最終結果");
    await expect(page.locator("#work-items")).toContainText("workflow run completed");
    await expect(page.locator("#work-items")).toContainText("承認待ち");
    await expect(page.locator("#work-items")).toContainText("approve");
    await page.getByRole("button", { name: /Approval|承認/ }).click();
    const createdRow = page.locator("tr", { hasText: "ブラウザフォームから作る" });
    await expect(createdRow).toBeVisible();
    await page.getByRole("button", { name: /Failed|失敗/ }).click();
    await expect(createdRow).toBeHidden();
    await page.getByRole("button", { name: /All|すべて/ }).click();
    await expect(createdRow).toBeVisible();
    page.once("dialog", async (dialog) => {
      expect(dialog.message()).toContain("ブラウザフォームから作る");
      await dialog.accept();
    });
    await page
      .locator("tr", { hasText: "ブラウザフォームから作る" })
      .getByRole("button", { name: /Delete|削除/ })
      .click();
    await expect(page.locator("#work-items")).not.toContainText("ブラウザフォームから作る");
  } finally {
    await stopProcess(server);
  }
});

test("local UI server answers a needs_input work item from the browser", async ({ page }) => {
  const root = mkdtempSync(path.join(tmpdir(), "nagare-ui-answer-e2e-"));
  runNagare(["init", "--root", root]);
  writeFileSync(
    path.join(root, "question.md"),
    [
      "## Nagare Result",
      "status: blocked",
      "completed:",
      "- inspected request",
      "questions:",
      "- 進め方を確認してください",
      "next_notes:",
      "- waiting for browser answer",
      "next_action: answer_question",
      "",
    ].join("\n"),
    "utf8",
  );
  writeFileSync(
    path.join(root, "agent-success.md"),
    [
      "## Nagare Result",
      "status: completed",
      "summary:",
      "- browser server run completed",
      "completed:",
      "- reflected browser answer",
      "questions:",
      "next_notes:",
      "- ready for review",
      "next_action: review",
      "",
    ].join("\n"),
    "utf8",
  );
  writeFileSync(
    path.join(root, "review-pass.md"),
    [
      "## Nagare Review",
      "verdict: pass",
      "summary:",
      "- looks good",
      "completed:",
      "- reviewed browser run",
      "findings:",
      "referenced_artifacts:",
      "requested_changes:",
      "questions:",
      "next_notes:",
      "- ready for approval",
      "next_action: approve",
      "",
    ].join("\n"),
    "utf8",
  );
  const createOutput = runNagare(["item", "create", "--root", root, "--title", "UIで回答する"]);
  const workItemId = createOutput.match(/work_\d+/)?.[0];
  expect(workItemId).toBeTruthy();
  runNagare([
    "item",
    "run",
    workItemId!,
    "--root",
    root,
    "--command",
    "type question.md",
  ]);

  const port = await getFreePort();
  const server = startUiServer(root, port);
  try {
    await waitForServer(`http://127.0.0.1:${port}/`);
    await page.goto(`http://127.0.0.1:${port}/items/${workItemId}`);
    await expect(page.getByRole("heading", { name: "UIで回答する" })).toBeVisible();
    await expect(page.locator("#detail")).toContainText("人の入力待ち");
    await expect(page.locator("#answer-form")).toContainText("進め方を確認してください");
    await page.locator('#answer-form input[name="command"]').evaluate((node, value) => {
      (node as HTMLInputElement).value = value as string;
    }, typeCommand(path.join(root, "agent-success.md")));
    await page.locator('#answer-form input[name="review_command"]').evaluate((node, value) => {
      (node as HTMLInputElement).value = value as string;
    }, typeCommand(path.join(root, "review-pass.md")));
    await page.locator('textarea[name="answer"]').fill("ブラウザから回答しました。続行してください。");
    await page.getByRole("button", { name: "回答を送信" }).click();

    await expect(page.locator("#detail")).toContainText("承認待ち");
    await expect(page.locator("#detail")).toContainText("最終結果を承認");
    await expect(page.getByRole("heading", { name: "Human Feedback" })).toHaveCount(0);
    await page.getByRole("link", { name: /Work Queue|作業キュー/ }).first().click();
    await expect(page.locator("#work-items")).toContainText("最終結果");
    await expect(page.locator("#work-items")).toContainText("browser server run completed");
    await expect(page.locator("#work-items")).toContainText("承認待ち");
    await expect(page.locator("#work-items")).toContainText("approve");
    const showOutput = runNagare(["item", "show", workItemId!, "--root", root]);
    expect(showOutput).toContain("human_feedback");
    expect(showOutput).toContain("ブラウザから回答しました");
    const reviewShowOutput = runNagare(["item", "show", workItemId!, "--root", root]);
    expect(reviewShowOutput).toContain("reflected browser answer");
    expect(reviewShowOutput).toContain("reviewed browser run");
    expect(reviewShowOutput).toContain("approval_gate: state=ready");
    await page.goto(`http://127.0.0.1:${port}/items/${workItemId}`);
    await expect(page.getByRole("heading", { name: "承認" })).toBeVisible();
    await page.getByRole("button", { name: "承認して完了" }).click();

    await expect(page.locator("#detail")).toContainText("完了");
    const approveShowOutput = runNagare(["item", "show", workItemId!, "--root", root]);
    expect(approveShowOutput).toContain(`${workItemId}\tdone`);
    expect(approveShowOutput).toContain("completion: state=done");
    expect(approveShowOutput).toContain("decision");
    expect(approveShowOutput).toContain("approve");
  } finally {
    await stopProcess(server);
  }
});

test("local UI server recovers review requested changes from the browser", async ({ page }) => {
  const root = mkdtempSync(path.join(tmpdir(), "nagare-ui-recover-e2e-"));
  runNagare(["init", "--root", root]);
  writeFileSync(
    path.join(root, "initial-result.md"),
    [
      "## Nagare Result",
      "status: completed",
      "summary:",
      "- initial work completed",
      "completed:",
      "- created initial output",
      "questions:",
      "next_notes:",
      "- ready for review",
      "next_action: review",
      "",
    ].join("\n"),
    "utf8",
  );
  writeFileSync(
    path.join(root, "fixed-result.md"),
    [
      "## Nagare Result",
      "status: completed",
      "summary:",
      "- fixed after review changes",
      "completed:",
      "- recovered after review changes",
      "questions:",
      "next_notes:",
      "- ready for review again",
      "next_action: review",
      "",
    ].join("\n"),
    "utf8",
  );
  writeFileSync(
    path.join(root, "review-changes.md"),
    [
      "## Nagare Review",
      "verdict: request_changes",
      "summary:",
      "- review requested changes",
      "requested_changes:",
      "- Improve the output before approval.",
      "questions:",
      "next_notes:",
      "- rerun work with review findings",
      "next_action: run_agent",
      "",
    ].join("\n"),
    "utf8",
  );

  const createOutput = runNagare(["item", "create", "--root", root, "--title", "UIで復旧する"]);
  const workItemId = createOutput.match(/work_\d+/)?.[0];
  expect(workItemId).toBeTruthy();
  runNagare([
    "item",
    "run",
    workItemId!,
    "--root",
    root,
    "--command",
    typeCommand(path.join(root, "initial-result.md")),
  ]);
  runNagare([
    "item",
    "review",
    workItemId!,
    "--root",
    root,
    "--command",
    typeCommand(path.join(root, "review-changes.md")),
  ]);

  const port = await getFreePort();
  const server = startUiServer(root, port);
  try {
    await waitForServer(`http://127.0.0.1:${port}/`);
    await page.goto(`http://127.0.0.1:${port}/items/${workItemId}`);
    await expect(page.locator("#detail")).toContainText("修正対応待ち");
    await expect(page.locator("#detail")).toContainText("作業エージェントを実行");
    await expect(page.getByRole("heading", { name: "復旧" })).toBeVisible();
    runNagare([
      "item",
      "run",
      workItemId!,
      "--root",
      root,
      "--command",
      typeCommand(path.join(root, "fixed-result.md")),
    ]);
    await page.reload();

    await expect(page.locator("body")).toContainText("レビュー待ち");
    await expect(page.locator("body")).toContainText("レビューを実行");
    await expect(page.locator(".detail-layout")).not.toContainText("single step");
    await expect(page.locator(".detail-layout")).not.toContainText("manual");
    const showOutput = runNagare(["item", "show", workItemId!, "--root", root]);
    expect(showOutput).toContain("recovered after review changes");
  } finally {
    await stopProcess(server);
  }
});

test("local UI server separates process success from invalid contract output", async ({ page }) => {
  const root = mkdtempSync(path.join(tmpdir(), "nagare-ui-invalid-contract-e2e-"));
  runNagare(["init", "--root", root]);
  writeFileSync(
    path.join(root, "work.md"),
    [
      "## Nagare Result",
      "status: completed",
      "summary:",
      "- work completed",
      "completed:",
      "- answered request",
      "next_notes:",
      "- review the result",
      "next_action: review",
      "",
    ].join("\n"),
    "utf8",
  );
  writeFileSync(path.join(root, "bad-review.txt"), "Review passed but no contract block.\n", "utf8");
  const itemOutput = runNagare(["item", "create", "--root", root, "--title", "Invalid review contract"]);
  const workItemId = itemOutput.match(/created (work_\d+)/)?.[1];
  runNagare([
    "item",
    "run",
    workItemId!,
    "--root",
    root,
    "--agent",
    "worker",
    "--command",
    typeCommand(path.join(root, "work.md")),
  ]);
  runNagare([
    "item",
    "review",
    workItemId!,
    "--root",
    root,
    "--command",
    typeCommand(path.join(root, "bad-review.txt")),
  ]);

  const port = await getFreePort();
  const server = startUiServer(root, port);
  try {
    await waitForServer(`http://127.0.0.1:${port}/`);
    await page.goto(`http://127.0.0.1:${port}/items/${workItemId}`);
    await page.locator(".technical-details > summary").click();
    const reviewRun = page.locator('.history-event[data-event-type="review"]', { hasText: "reviewer" }).last();
    await expect(reviewRun).toContainText("contract invalid");
    await expect(reviewRun).toContainText("Status");
    await expect(reviewRun).toContainText("Process status");
    await expect(reviewRun).toContainText("succeeded");
    await expect(reviewRun).toContainText("Process exit");
    await expect(reviewRun).toContainText("0");
    await expect(reviewRun).toContainText("Parse status");
    await expect(reviewRun).toContainText("unparsed");
    const details = reviewRun.locator(".history-details");
    await details.locator("summary").click();
    await expect(details).toContainText("Source records");
    await expect(details).toContainText("Agent Output");
    await expect(page.locator("#detail")).toContainText("復旧を作成または適用");
  } finally {
    await stopProcess(server);
  }
});
