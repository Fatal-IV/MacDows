// Renders the MacDows logo (branding/*.svg) into every icon file the app and the installer need.
// Usage: node scripts/make-icons.mjs
import { Resvg } from "@resvg/resvg-js";
import pngToIco from "png-to-ico";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const read = (name) => readFileSync(join(root, "branding", name), "utf8");
const main = read("macdows-mark.svg");
const small = { 16: read("macdows-mark-16.svg"), 24: read("macdows-mark-24.svg") };

const render = (svg, size) => new Resvg(svg, { fitTo: { mode: "width", value: size } }).render().asPng();
// 16 and 24 px use the simplified drawings so the mark stays readable.
const pngFor = (size) => render(small[size] ?? main, size);

const work = mkdtempSync(join(tmpdir(), "macdows-icons-"));
const source = join(work, "app-icon-1024.png");
writeFileSync(source, render(main, 1024));

// Tauri produces every format it knows (png set, icns, Windows tile images, ...).
execFileSync("bun", ["run", "tauri", "icon", source], { cwd: root, stdio: "pipe", shell: true });

// MacDows is Windows only: drop the mobile icon folders Tauri also generates.
for (const dir of ["android", "ios"]) rmSync(join(root, "src-tauri", "icons", dir), { recursive: true, force: true });

// icon.ico is rebuilt so it carries the size-specific drawings.
const icoSizes = [16, 24, 32, 48, 64, 128, 256];
const ico = await pngToIco(icoSizes.map(pngFor));
const appIcons = join(root, "src-tauri", "icons");
writeFileSync(join(appIcons, "icon.ico"), ico);

// The installer reuses the same files.
const installerIcons = join(root, "installer", "src-tauri", "icons");
mkdirSync(installerIcons, { recursive: true });
for (const name of ["icon.ico", "icon.icns", "32x32.png", "128x128.png", "128x128@2x.png", "icon.png"]) {
  try {
    copyFileSync(join(appIcons, name), join(installerIcons, name));
  } catch {
    // icon.png may not exist in every Tauri version
  }
}
const assets = join(root, "installer", "src", "assets");
mkdirSync(assets, { recursive: true });
copyFileSync(join(root, "branding", "macdows-mark.svg"), join(assets, "mark.svg"));

console.log("icons written to src-tauri/icons and installer/src-tauri/icons");
