#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const args = process.argv.slice(2);
const repoRoot = path.resolve(__dirname, "..", "..", "..");

if (args.length === 0 && launchDesktopDev(repoRoot)) {
  process.exit(0);
}

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

const result = spawnSync(binary, args.length === 0 ? ["ui", "serve", "--browser"] : args, {
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
  return path.join(repoRoot, "target", "release", `nagare${extension}`);
}

function launchDesktopDev(root) {
  if (process.env.NAGARE_NO_DESKTOP_AUTO_LAUNCH === "1") {
    return false;
  }

  const desktopPackage = path.join(root, "apps", "nagare-desktop", "package.json");
  const rootPackage = path.join(root, "package.json");
  if (!fs.existsSync(desktopPackage) || !fs.existsSync(rootPackage)) {
    return false;
  }

  if (process.env.NAGARE_DESKTOP_DRY_RUN === "1") {
    console.log("nagare desktop dev");
    return true;
  }

  const result = spawnSync(
    "npm",
    ["run", "dev", "--workspace", "@hachiware-labs/nagare-desktop"],
    {
      cwd: root,
      stdio: "inherit",
      env: process.env,
      shell: process.platform === "win32",
    }
  );

  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }

  process.exit(result.status ?? 1);
}
