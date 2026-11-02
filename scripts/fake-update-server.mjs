// Local stand-in for a GitHub release, used to test the in-app update without publishing anything.
//
//   node scripts/fake-update-server.mjs <dir> <version> <port> [--tamper]
//
// <dir> must hold MacDows_<version>_x64-setup.exe and its .sig (from `tauri build` with the real key).
// Serves GET /latest.json and GET /package.exe. With --tamper the first byte of the served package is changed,
// so the signature check must refuse it.
import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const [dir, version, portArg] = process.argv.slice(2);
const tamper = process.argv.includes("--tamper");
if (!dir || !version || !portArg) {
  console.error("usage: node scripts/fake-update-server.mjs <dir> <version> <port> [--tamper]");
  process.exit(2);
}
const port = Number(portArg);
const name = `MacDows_${version}_x64-setup.exe`;
const pkg = readFileSync(join(dir, name));
const signature = readFileSync(join(dir, `${name}.sig`), "utf8").trim();
if (tamper) pkg[0] = pkg[0] ^ 0xff;

const manifest = {
  version,
  notes: `Fake release ${version} for the local update test.\n- Updated through the fake server`,
  pub_date: new Date().toISOString(),
  platforms: { "windows-x86_64": { signature, url: `http://localhost:${port}/package.exe` } },
};

createServer((req, res) => {
  if (req.url === "/latest.json") {
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify(manifest));
  } else if (req.url === "/package.exe") {
    res.writeHead(200, { "Content-Type": "application/octet-stream", "Content-Length": pkg.length });
    res.end(pkg);
  } else {
    res.writeHead(404).end();
  }
  console.log(`${new Date().toISOString()} ${req.method} ${req.url}`);
}).listen(port, "127.0.0.1", () => console.log(`fake release ${version} on http://localhost:${port}/latest.json${tamper ? " (tampered package)" : ""}`));
