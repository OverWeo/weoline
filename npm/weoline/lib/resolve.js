import fs from "node:fs";
import module from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const PLATFORMS = {
  "darwin-arm64": "@overweo/weoline-darwin-arm64",
  "darwin-x64": "@overweo/weoline-darwin-x64",
  "linux-arm64": "@overweo/weoline-linux-arm64",
  "linux-x64": "@overweo/weoline-linux-x64",
  "win32-arm64": "@overweo/weoline-win32-arm64",
  "win32-x64": "@overweo/weoline-win32-x64",
};

export default function resolve() {
  const key = `${process.platform}-${process.arch}`;
  const pkg = PLATFORMS[key];
  if (!pkg) {
    throw new Error(`weoline: unsupported platform ${key}`);
  }

  const target = `${pkg}/package.json`;
  let pkgDir;

  // Try import.meta.resolve first (Node 20.6+), fall back to createRequire
  try {
    pkgDir = path.dirname(fileURLToPath(import.meta.resolve(target)));
  } catch {
    try {
      const require = module.createRequire(import.meta.url);
      pkgDir = path.dirname(require.resolve(target));
    } catch {
      throw new Error(
        `Unable to resolve ${pkg}. Your platform may be unsupported, or the package is missing. ` +
        `Try: npm install (or pnpm install / yarn install)`
      );
    }
  }

  const exe = path.join(pkgDir, process.platform === "win32" ? "weoline.exe" : "weoline");
  if (!fs.existsSync(exe)) {
    throw new Error(`Binary not found: ${exe}`);
  }
  return exe;
}
