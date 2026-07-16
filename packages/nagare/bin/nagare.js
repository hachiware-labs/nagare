#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const { spawn, spawnSync } = require("node:child_process");

const args = process.argv.slice(2);
const extension = process.platform === "win32" ? ".exe" : "";
const isCliCommand = args.length > 0;
const binary = resolveBinary(isCliCommand ? "cli" : "desktop");

if (!binary) {
  const target = isCliCommand ? "CLI" : "desktop";
  console.error(
    `Nagare ${target} binary not found. Reinstall @hachiware-labs/nagare, or set NAGARE_${isCliCommand ? "CLI" : "DESKTOP"}_BINARY.`
  );
  process.exit(1);
}

if (!isCliCommand) {
  const child = spawn(binary, [], {
    stdio: "ignore",
    windowsHide: false,
    env: process.env,
  });
  child.on("error", (error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
} else {
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
  const override = process.env[`NAGARE_${kind.toUpperCase()}_BINARY`];
  const packaged = path.join(
    __dirname,
    `nagare-${kind}-${process.platform}-${process.arch}${extension}`
  );
  return [override, packaged].filter(Boolean).find((candidate) => fs.existsSync(candidate));
}
