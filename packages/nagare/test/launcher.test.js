const assert = require("node:assert/strict");
const path = require("node:path");
const test = require("node:test");

const { defaultDesktopRoot, desktopEnvironment } = require("../bin/nagare.js");

test("desktop launcher isolates package data from the current directory", () => {
  const env = { LOCALAPPDATA: "C:/Users/example/AppData/Local" };
  const resolved = desktopEnvironment(env);

  assert.equal(
    resolved.NAGARE_ROOT,
    path.join("C:/Users/example/AppData/Local", "Nagare", "workspace")
  );
  assert.equal(resolved.NAGARE_ROOT, defaultDesktopRoot(env));
  assert.equal(env.NAGARE_ROOT, undefined);
});

test("desktop launcher preserves an explicitly selected project root", () => {
  const resolved = desktopEnvironment({
    LOCALAPPDATA: "C:/Users/example/AppData/Local",
    NAGARE_ROOT: "D:/projects/customer-a",
  });

  assert.equal(resolved.NAGARE_ROOT, "D:/projects/customer-a");
});
