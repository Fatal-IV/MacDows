import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { findPayload, sumsLine } from "./build-installer.mjs";

test("findPayload names the missing file and the version", () => {
  const dir = mkdtempSync(join(tmpdir(), "macdows-bundle-"));
  try {
    assert.throws(() => findPayload(dir, "0.1.0"), (err) => {
      assert.match(err.message, /MacDows_0\.1\.0_x64-setup\.exe/);
      assert.match(err.message, /not found/);
      return true;
    });
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("findPayload returns the path when the package exists", () => {
  const dir = mkdtempSync(join(tmpdir(), "macdows-bundle-"));
  try {
    const file = join(dir, "MacDows_0.1.0_x64-setup.exe");
    writeFileSync(file, "x");
    assert.equal(findPayload(dir, "0.1.0"), file);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("sumsLine uses the sha256sum format", () => {
  // sha256("abc")
  assert.equal(
    sumsLine("a.exe", Buffer.from("abc")),
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  a.exe",
  );
});
