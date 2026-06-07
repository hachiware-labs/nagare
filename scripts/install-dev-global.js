#!/usr/bin/env node

const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");

run("node", [path.join("scripts", "stage-package-binary.js")], repoRoot);
run("npm", ["link", "--workspace", "@hachiware-labs/nagare"], repoRoot);

console.log("installed development nagare globally");

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
