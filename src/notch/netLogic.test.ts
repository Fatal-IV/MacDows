import { test } from "node:test";
import assert from "node:assert/strict";
import { signalBars, securityLabel, errorText, canSubmit, vpnLabel, vpnIsOn } from "./netLogic.ts";

test("signal maps to 0-4 bars", () => {
  const cases: Array<[number, number]> = [[0, 0], [1, 1], [25, 1], [26, 2], [50, 2], [51, 3], [75, 3], [76, 4], [100, 4]];
  for (const [signal, bars] of cases) assert.equal(signalBars(signal), bars, `signal ${signal}`);
});

test("security label is open, secured or enterprise", () => {
  const t = (k: string) => k;
  const base = { ssid: "TestNet", signal: 50, saved: false, connected: false, auth: "wpa2" as const };
  assert.equal(securityLabel({ ...base, secured: false, enterprise: false }, t), "net.open");
  assert.equal(securityLabel({ ...base, secured: true, enterprise: false }, t), "net.secured");
  assert.equal(securityLabel({ ...base, secured: true, enterprise: true }, t), "net.enterprise");
});

test("every backend error key has a text and unknown ones fall back to failed", () => {
  const t = (k: string) => k;
  for (const key of ["not_found", "failed", "timeout", "needs_password", "enterprise", "wrong_password"]) assert.equal(errorText(key, t), `net.err.${key}`);
  assert.equal(errorText("surprise", t), "net.err.failed");
});

test("canSubmit: SSID required, WPA2 8-63 characters, WPA3 at least 1, open needs none", () => {
  assert.equal(canSubmit("", "open", ""), false);
  assert.equal(canSubmit("TestNet", "open", ""), true);
  assert.equal(canSubmit("TestNet", "wpa2", "1234567"), false, "7 characters");
  assert.equal(canSubmit("TestNet", "wpa2", "12345678"), true, "8 characters");
  assert.equal(canSubmit("TestNet", "wpa2", "x".repeat(63)), true, "63 characters");
  assert.equal(canSubmit("TestNet", "wpa2", "x".repeat(64)), false, "64 characters");
  assert.equal(canSubmit("TestNet", "wpa2", "with some spaces"), true, "spaces are allowed");
  assert.equal(canSubmit("TestNet", "wpa3", ""), false);
  assert.equal(canSubmit("TestNet", "wpa3", "a"), true);
  assert.equal(canSubmit("   ", "open", ""), false, "blank SSID");
});

test("the VPN card reads connecting, on, on (app) or off", () => {
  const t = (k: string) => k;
  const e = (state: string) => ({ name: "Work", state });
  assert.equal(vpnLabel({ entries: [], app_vpn_active: false }, t), "vpn.off");
  assert.equal(vpnLabel({ entries: [e("disconnected")], app_vpn_active: false }, t), "vpn.off");
  assert.equal(vpnLabel({ entries: [e("connected")], app_vpn_active: false }, t), "vpn.on");
  assert.equal(vpnLabel({ entries: [e("connecting")], app_vpn_active: true }, t), "vpn.connecting");
  assert.equal(vpnLabel({ entries: [e("disconnected")], app_vpn_active: true }, t), "vpn.onApp");
  assert.equal(vpnLabel(null, t), "vpn.off");
});

test("the VPN tile is on for a connected VPN or an app tunnel, not while connecting", () => {
  const e = (state: string) => ({ name: "Work", state });
  assert.equal(vpnIsOn(null), false);
  assert.equal(vpnIsOn({ entries: [], app_vpn_active: false }), false);
  assert.equal(vpnIsOn({ entries: [e("connected")], app_vpn_active: false }), true);
  assert.equal(vpnIsOn({ entries: [e("disconnected")], app_vpn_active: true }), true);
  assert.equal(vpnIsOn({ entries: [e("connecting")], app_vpn_active: true }), false);
});
