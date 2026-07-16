const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const {
  defaultDesktopRoot,
  desktopEnvironment,
} = require("../bin/nagare.js");

test("desktop launcher isolates package data from the current directory", () => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "nagare-launcher-"));
  const env = { LOCALAPPDATA: path.join(temp, "AppData", "Local") };
  try {
    const resolved = desktopEnvironment(env);

    assert.equal(
      resolved.NAGARE_ROOT,
      path.join(temp, "AppData", "Local", "Nagare", "workspace")
    );
    assert.equal(resolved.NAGARE_ROOT, defaultDesktopRoot(env));
    assert.ok(fs.statSync(resolved.NAGARE_ROOT).isDirectory());
    assert.equal(env.NAGARE_ROOT, undefined);
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
});

test("desktop launcher preserves an explicitly selected project root", () => {
  const resolved = desktopEnvironment({
    LOCALAPPDATA: "C:/Users/example/AppData/Local",
    NAGARE_ROOT: "D:/projects/customer-a",
  });

  assert.equal(resolved.NAGARE_ROOT, "D:/projects/customer-a");
});
