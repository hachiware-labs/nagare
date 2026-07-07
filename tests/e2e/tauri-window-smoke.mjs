import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync } from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../..");
const appPath = path.join(
  repoRoot,
  "apps",
  "nagare-desktop",
  "src-tauri",
  "target",
  "release",
  process.platform === "win32" ? "nagare-desktop.exe" : "nagare-desktop",
);
const port = Number(process.env.NAGARE_TAURI_DRIVER_PORT || "4459");
const strict = process.env.NAGARE_DESKTOP_E2E_STRICT === "1";
const testRoot = path.join(os.tmpdir(), `nagare-tauri-window-${process.pid}`);

class SkipError extends Error {}

function commandExists(command) {
  const checker = process.platform === "win32" ? "where.exe" : "which";
  return spawnSync(checker, [command], { stdio: "ignore" }).status === 0;
}

function skip(message) {
  if (strict) {
    throw new Error(message);
  }
  throw new SkipError(message);
}

function requestJson(method, urlPath, body) {
  return new Promise((resolve, reject) => {
    const payload = body ? JSON.stringify(body) : "";
    const request = http.request(
      {
        hostname: "127.0.0.1",
        port,
        path: urlPath,
        method,
        headers: payload
          ? {
              "content-type": "application/json",
              "content-length": Buffer.byteLength(payload),
            }
          : undefined,
      },
      (response) => {
        let raw = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          raw += chunk;
        });
        response.on("end", () => {
          const parsed = raw ? JSON.parse(raw) : {};
          if (response.statusCode >= 400) {
            const message = parsed?.value?.message || raw || `HTTP ${response.statusCode}`;
            reject(new Error(message));
            return;
          }
          resolve(parsed);
        });
      },
    );
    request.on("error", reject);
    if (payload) request.write(payload);
    request.end();
  });
}

async function waitForDriver() {
  const deadline = Date.now() + 10_000;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const status = await requestJson("GET", "/status");
      if (status?.value?.ready) return status;
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw lastError || new Error("tauri-driver did not become ready");
}

async function waitForElement(sessionId, selector) {
  const deadline = Date.now() + 10_000;
  let lastError;
  while (Date.now() < deadline) {
    try {
      return await findElement(sessionId, selector);
    } catch (error) {
      lastError = error;
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
  }
  throw lastError || new Error(`element did not appear: ${selector}`);
}

async function executeScript(sessionId, script, args = []) {
  return requestJson("POST", `/session/${sessionId}/execute/sync`, {
    script,
    args,
  });
}

function elementId(value) {
  return value?.["element-6066-11e4-a52e-4f735466cecf"] || value?.ELEMENT;
}

async function findElement(sessionId, selector) {
  const response = await requestJson("POST", `/session/${sessionId}/element`, {
    using: "css selector",
    value: selector,
  });
  const id = elementId(response.value);
  if (!id) throw new Error(`element not found: ${selector}`);
  return id;
}

async function elementText(sessionId, id) {
  const response = await requestJson("GET", `/session/${sessionId}/element/${id}/text`);
  return response.value;
}

function mismatchVersion(message) {
  return /Current browser version is ([0-9.]+)/i.exec(message)?.[1] || "";
}

function downloadMatchingMsedgedriver(version) {
  if (process.platform !== "win32") {
    throw new Error("automatic msedgedriver download is only implemented for Windows");
  }
  if (!/^\d+\.\d+\.\d+\.\d+$/.test(version)) {
    throw new Error(`invalid WebView2 version for msedgedriver download: ${version}`);
  }
  const targetDir = path.join(os.tmpdir(), `msedgedriver-${version}-win64`);
  const target = path.join(targetDir, "msedgedriver.exe");
  if (existsSync(target)) return target;

  mkdirSync(targetDir, { recursive: true });
  const zip = path.join(os.tmpdir(), `msedgedriver-${version}-win64.zip`);
  const command = [
    "$ErrorActionPreference = 'Stop'",
    `$zip = ${JSON.stringify(zip)}`,
    `$dir = ${JSON.stringify(targetDir)}`,
    `$uri = ${JSON.stringify(`https://msedgedriver.microsoft.com/${version}/edgedriver_win64.zip`)}`,
    "if (Test-Path $zip) { Remove-Item -LiteralPath $zip -Force }",
    "Invoke-WebRequest -Uri $uri -OutFile $zip -UseBasicParsing",
    "Expand-Archive -LiteralPath $zip -DestinationPath $dir -Force",
  ].join("; ");
  const result = spawnSync("powershell", ["-NoProfile", "-Command", command], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.status !== 0 || !existsSync(target)) {
    throw new Error(
      `failed to download matching msedgedriver ${version}: ${result.stderr || result.stdout}`,
    );
  }
  return target;
}

async function runSmoke(nativeDriver) {
  mkdirSync(testRoot, { recursive: true });

  const args = ["--port", String(port)];
  if (nativeDriver) {
    args.push("--native-driver", nativeDriver);
  }

  const driver = spawn("tauri-driver", args, {
    stdio: ["ignore", "pipe", "pipe"],
    env: {
      ...process.env,
      NAGARE_ROOT: testRoot,
    },
    windowsHide: true,
  });
  let stdout = "";
  let stderr = "";
  let storageSnapshot;
  driver.stdout.on("data", (chunk) => {
    stdout += chunk.toString();
  });
  driver.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });

  let sessionId = "";
  try {
    await waitForDriver();
    const sessionResponse = await requestJson("POST", "/session", {
      capabilities: {
        alwaysMatch: {
          browserName: "wry",
          "tauri:options": {
            application: appPath,
          },
        },
      },
    });
    sessionId = sessionResponse?.value?.sessionId;
    if (!sessionId) throw new Error("WebDriver session id was not returned");

    const titleResponse = await requestJson("GET", `/session/${sessionId}/title`);
    if (titleResponse.value !== "Nagare") {
      throw new Error(`unexpected window title: ${titleResponse.value}`);
    }
    const snapshotResponse = await executeScript(
      sessionId,
      "return Object.fromEntries(Object.keys(localStorage).map((key) => [key, localStorage.getItem(key)]));",
    );
    storageSnapshot = snapshotResponse.value || {};
    await executeScript(
      sessionId,
      "localStorage.clear(); localStorage.setItem('nagare.root', arguments[0]); location.reload(); return true;",
      [testRoot],
    );
    const pageTitle = await elementText(sessionId, await waitForElement(sessionId, "#page-title"));
    const setupButton = await elementText(
      sessionId,
      await waitForElement(sessionId, "#setup-open-button"),
    );
    if (pageTitle !== "ワーク") {
      throw new Error(`unexpected page title: ${pageTitle}`);
    }
    if (setupButton !== "セットアップを始める") {
      throw new Error(`unexpected setup button text: ${setupButton}`);
    }
    console.log("tauri-window-smoke passed");
  } finally {
    if (sessionId) {
      if (storageSnapshot) {
        await executeScript(
          sessionId,
          "localStorage.clear(); for (const [key, value] of Object.entries(arguments[0])) { if (value != null) localStorage.setItem(key, value); } return true;",
          [storageSnapshot],
        ).catch(() => {});
      }
      await requestJson("DELETE", `/session/${sessionId}`).catch(() => {});
    }
    driver.kill();
    if (process.env.NAGARE_TAURI_DRIVER_DEBUG === "1") {
      console.log(stdout);
      console.error(stderr);
    }
  }
}

async function main() {
  if (!commandExists("tauri-driver")) {
    skip("tauri-driver is not installed");
  }
  if (!existsSync(appPath)) {
    skip(`desktop executable is missing; run npm run build --workspace @hachiware-labs/nagare-desktop -- --debug first: ${appPath}`);
  }
  const nativeDriver = process.env.NAGARE_MSEDGEDRIVER || "";
  try {
    try {
      await runSmoke(nativeDriver);
    } catch (error) {
      const version = nativeDriver ? "" : mismatchVersion(error.message);
      if (!version) throw error;
      try {
        const downloadedDriver = downloadMatchingMsedgedriver(version);
        await runSmoke(downloadedDriver);
      } catch (downloadError) {
        skip(
          `native WebDriver version does not match WebView2 runtime and automatic matching driver setup failed: ${downloadError.message}`,
        );
      }
    }
  } finally {
    rmSync(testRoot, { recursive: true, force: true });
  }
}

main().catch((error) => {
  if (error instanceof SkipError) {
    console.log(`SKIP tauri-window-smoke: ${error.message}`);
    process.exit(0);
  }
  console.error(error);
  process.exit(1);
});
