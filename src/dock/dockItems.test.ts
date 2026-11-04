import { test } from "node:test";
import assert from "node:assert/strict";
import { mergeDock } from "./dockItems.ts";

const pin = (name: string, exe: string, aumid = "") => ({ id: `C:/pins/${name}.lnk`, name, exe, aumid, icon_png_base64: null });
const win = (id: number, exe: string, aumid = "", title = `w${id}`) => ({ id, title, exe, aumid, icon_png_base64: null });

test("pinned apps come first in their order, then the other windows", () => {
  const out = mergeDock([pin("Edge", "C:/e/msedge.exe"), pin("Files", "C:/Windows/explorer.exe")], [win(1, "C:/x/code.exe"), win(2, "C:/Windows/explorer.exe")]);
  assert.deepEqual(out.map((e) => e.key), ["pin:C:/pins/Edge.lnk", "pin:C:/pins/Files.lnk", "win:1"]);
});

test("a running pinned app keeps one icon and lists its windows (paths compare ignoring case and slash style)", () => {
  const out = mergeDock([pin("Edge", "C:/e/msedge.exe")], [win(5, "c:\\E\\MSEDGE.EXE"), win(6, "C:/e/msedge.exe")]);
  assert.equal(out.length, 1);
  assert.deepEqual(out[0].windows.map((w) => w.id), [5, 6]);
  assert.equal(out[0].running, true);
});

test("a pinned app that is not running has no windows", () => {
  const out = mergeDock([pin("Edge", "C:/e/msedge.exe")], []);
  assert.equal(out[0].running, false);
  assert.equal(out[0].windows.length, 0);
});

test("without pinned apps only windows show", () => {
  const out = mergeDock([], [win(1, "a.exe"), win(2, "b.exe")]);
  assert.deepEqual(out.map((e) => e.key), ["win:1", "win:2"]);
  assert.equal(out[0].running, false);
});

test("a window with an unknown exe is never merged into a pin", () => {
  const out = mergeDock([pin("Edge", "C:/e/msedge.exe")], [win(1, "")]);
  assert.equal(out.length, 2);
});

test("a pinned Store app finds its windows by app id, whatever program hosts them", () => {
  const store = pin("Store", "", "Microsoft.WindowsStore_8wekyb3d8bbwe!App");
  const out = mergeDock([store], [win(1, "C:/Windows/System32/ApplicationFrameHost.exe", "microsoft.windowsstore_8wekyb3d8bbwe!app")]);
  assert.equal(out.length, 1);
  assert.equal(out[0].running, true);
});

test("an empty app id never matches another empty app id", () => {
  const out = mergeDock([pin("Files", "C:/Windows/explorer.exe")], [win(1, "C:/other.exe")]);
  assert.equal(out[0].running, false);
  assert.equal(out.length, 2);
});

test("entries carry the program, app id and window handles the right-click menu needs", () => {
  const out = mergeDock([pin("Edge", "C:/e/msedge.exe", "MSEdge")], [win(5, "C:/e/msedge.exe"), win(6, "C:/o/other.exe", "Other_8wekyb3d8bbwe!App")]);
  assert.equal(out[0].exe, "C:/e/msedge.exe");
  assert.equal(out[0].aumid, "MSEdge");
  assert.deepEqual(out[0].windows.map((w) => w.id), [5]);
  assert.equal(out[1].exe, "C:/o/other.exe");
  assert.equal(out[1].aumid, "Other_8wekyb3d8bbwe!App");
});
