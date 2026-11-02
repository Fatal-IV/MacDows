import { test, before } from "node:test";
import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

before(() => {
  execFileSync("node", [join(root, "scripts", "make-icons.mjs")], { cwd: root, stdio: "pipe" });
});

function icoSizes(file) {
  const buf = readFileSync(file);
  assert.equal(buf.readUInt16LE(2), 1, "not an ICO file");
  const count = buf.readUInt16LE(4);
  const sizes = [];
  for (let i = 0; i < count; i++) sizes.push(buf[6 + i * 16] || 256);
  return sizes.sort((a, b) => a - b);
}

function pngSize(file) {
  const buf = readFileSync(file);
  return [buf.readUInt32BE(16), buf.readUInt32BE(20)];
}

test("app icon.ico holds every required size", () => {
  assert.deepEqual(icoSizes(join(root, "src-tauri", "icons", "icon.ico")), [16, 24, 32, 48, 64, 128, 256]);
});

test("installer gets the same icon set", () => {
  assert.deepEqual(icoSizes(join(root, "installer", "src-tauri", "icons", "icon.ico")), [16, 24, 32, 48, 64, 128, 256]);
  assert.ok(existsSync(join(root, "installer", "src", "assets", "mark.svg")));
});

test("32x32.png is really 32x32", () => {
  assert.deepEqual(pngSize(join(root, "src-tauri", "icons", "32x32.png")), [32, 32]);
});

test("main mark svg uses the approved colors", () => {
  const svg = readFileSync(join(root, "branding", "macdows-mark.svg"), "utf8");
  for (const c of ["#2b1b5a", "#7a4dff", "#05060a", "#7ee0ff", "#ff7a6b"]) assert.ok(svg.includes(c), `missing ${c}`);
});

test("small variants exist and drop the dock dots", () => {
  for (const n of ["24", "16"]) {
    const svg = readFileSync(join(root, "branding", `macdows-mark-${n}.svg`), "utf8");
    assert.ok(!svg.includes("<circle"), `${n}px variant must not contain dock dots`);
  }
});
