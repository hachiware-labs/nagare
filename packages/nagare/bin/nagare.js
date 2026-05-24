#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const candidates = [
  process.env.NAGARE_BINARY,
  packagedBinary(),
  devBinary(),
].filter(Boolean);

const binary = candidates.find((candidate) => fs.existsSync(candidate));

if (!binary) {
  console.error(
    "nagare binary not found. Build with `cargo build --release` and set NAGARE_BINARY, or install a package with a bundled platform binary."
  );
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function packagedBinary() {
  const extension = process.platform === "win32" ? ".exe" : "";
  return path.join(__dirname, `nagare-${process.platform}-${process.arch}${extension}`);
}

function devBinary() {
  const extension = process.platform === "win32" ? ".exe" : "";
  return path.resolve(__dirname, "..", "..", "..", "target", "release", `nagare${extension}`);
}
