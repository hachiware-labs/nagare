#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");
const extension = process.platform === "win32" ? ".exe" : "";
const binary = path.join(repoRoot, "target", "release", `nagare${extension}`);
const wrapper = path.join(repoRoot, "packages", "nagare", "bin", "nagare.js");

const args = process.argv.slice(2);

if (process.env.NAGARE_ROOT && !args.includes("--root")) {
  args.push("--root", process.env.NAGARE_ROOT);
}

if (process.env.NAGARE_SKIP_BUILD !== "1") {
  run("cargo", ["build", "--release", "-p", "nagare-cli"], { cwd: repoRoot });
}

const env = {
  ...process.env,
  NAGARE_BINARY: binary,
};

const result = spawnSync(process.execPath, [wrapper, ...args], {
  cwd: repoRoot,
  env,
  stdio: "inherit",
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function run(command, commandArgs, options) {
  const result = spawnSync(command, commandArgs, {
    ...options,
    stdio: "inherit",
    shell: process.platform === "win32",
  });

  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
