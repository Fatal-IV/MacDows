// Builds MacDows-Setup.exe: the branded installer with the MacDows NSIS package embedded.
//
//   node scripts/build-installer.mjs              build the app too (needs TAURI_SIGNING_PRIVATE_KEY)
//   node scripts/build-installer.mjs --skip-app   reuse the NSIS package from an earlier `tauri build`
//
// Output: installer/dist/MacDows-Setup.exe and installer/dist/SHA256SUMS.txt
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

/** Path of the NSIS package for `version` inside the bundle folder, or a clear error. */
export function findPayload(bundleDir, version) {
  const name = `MacDows_${version}_x64-setup.exe`;
  const file = join(bundleDir, name);
  if (!existsSync(file)) {
    throw new Error(`NSIS package ${name} not found in ${bundleDir}. Run \`bun run tauri build\` first, or drop --skip-app.`);
  }
  return file;
}

/** One line in `sha256sum` format. */
export function sumsLine(name, buffer) {
  return `${createHash("sha256").update(buffer).digest("hex")}  ${name}`;
}

function run(cmd, args, cwd, env = {}) {
  const result = spawnSync(cmd, args, { cwd, stdio: "inherit", shell: true, env: { ...process.env, ...env } });
  if (result.status !== 0) throw new Error(`${cmd} ${args.join(" ")} failed with code ${result.status}`);
}

function main() {
  const skipApp = process.argv.includes("--skip-app");
  const version = JSON.parse(readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8")).version;
  const installer = join(root, "installer");
  const bundleDir = join(root, "src-tauri", "target", "release", "bundle", "nsis");

  if (!skipApp) run("bun", ["run", "tauri", "build"], root);
  const payload = findPayload(bundleDir, version);

  const payloadDir = join(installer, "src-tauri", "payload");
  mkdirSync(payloadDir, { recursive: true });
  copyFileSync(payload, join(payloadDir, "MacDows-nsis-setup.exe"));

  run("bun", ["install"], installer);
  run("bun", ["run", "tauri", "build", "--no-bundle"], installer, { MACDOWS_APP_VERSION: version });

  const built = join(installer, "src-tauri", "target", "release", "macdows-setup.exe");
  if (!existsSync(built)) throw new Error(`installer exe not found at ${built}`);
  const dist = join(installer, "dist");
  mkdirSync(dist, { recursive: true });
  const out = join(dist, "MacDows-Setup.exe");
  copyFileSync(built, out);
  writeFileSync(join(dist, "SHA256SUMS.txt"), [sumsLine("MacDows-Setup.exe", readFileSync(out)), sumsLine(`MacDows_${version}_x64-setup.exe`, readFileSync(payload))].join("\n") + "\n");
  console.log(`MacDows-Setup.exe: ${(readFileSync(out).length / 1048576).toFixed(2)} MB`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (err) {
    console.error(String(err.message ?? err));
    process.exit(1);
  }
}
