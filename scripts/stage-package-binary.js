#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");
const extension = process.platform === "win32" ? ".exe" : "";
const source = path.join(repoRoot, "target", "release", `nagare${extension}`);
const target = path.join(
  repoRoot,
  "packages",
  "nagare",
  "bin",
  `nagare-${process.platform}-${process.arch}${extension}`
);

run("cargo", ["build", "--release", "-p", "nagare-cli"], repoRoot);

fs.copyFileSync(source, target);
if (process.platform !== "win32") {
  fs.chmodSync(target, 0o755);
}

console.log(`staged ${path.relative(repoRoot, target)}`);

function run(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
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
