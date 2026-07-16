#!/usr/bin/env node

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

function main() {
  const args = process.argv.slice(2);
  const isCliCommand = args.length > 0;
  const binary = resolveBinary(isCliCommand ? "cli" : "desktop");

  if (!binary) {
    const target = isCliCommand ? "CLI" : "desktop";
    console.error(
      `Nagare ${target} binary not found. Reinstall @hachiware-labs/nagare, or set NAGARE_${isCliCommand ? "CLI" : "DESKTOP"}_BINARY.`
    );
    process.exitCode = 1;
    return;
  }

  if (!isCliCommand) {
    const result = spawnSync(binary, [], {
      stdio: "inherit",
      windowsHide: false,
      env: desktopEnvironment(process.env),
    });
    if (result.error) {
      console.error(result.error.message);
      process.exitCode = 1;
    } else if (result.status !== 0) {
      process.exitCode = result.status ?? 1;
    }
    return;
  }

  const result = spawnSync(binary, args, {
    stdio: "inherit",
    env: process.env,
  });
  if (result.error) {
    console.error(result.error.message);
    process.exitCode = 1;
  } else {
    process.exitCode = result.status ?? 1;
  }
}

function resolveBinary(kind) {
  const extension = process.platform === "win32" ? ".exe" : "";
  const override = process.env[`NAGARE_${kind.toUpperCase()}_BINARY`];
  const packaged = path.join(
    __dirname,
    `nagare-${kind}-${process.platform}-${process.arch}${extension}`
  );
  return [override, packaged].filter(Boolean).find((candidate) => fs.existsSync(candidate));
}

function desktopEnvironment(environment) {
  const next = { ...environment };
  if (!String(next.NAGARE_ROOT || "").trim()) {
    const root = defaultDesktopRoot(environment);
    fs.mkdirSync(root, { recursive: true });
    next.NAGARE_ROOT = root;
  }
  return next;
}

function defaultDesktopRoot(environment = process.env) {
  const appData = environment.LOCALAPPDATA || environment.APPDATA || os.homedir();
  return path.join(appData, "Nagare", "workspace");
}

if (require.main === module) {
  main();
}

module.exports = { defaultDesktopRoot, desktopEnvironment };
