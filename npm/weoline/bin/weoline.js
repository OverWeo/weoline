#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import resolve from "#resolve";

const exe = resolve();

// Node 22.15+: replace this process with the native binary (zero overhead)
if (process.platform !== "win32" && typeof process.execve === "function") {
  try {
    process.execve(exe, [exe, ...process.argv.slice(2)]);
  } catch (e) {
    // Only fall through for errors where execve itself is broken,
    // not for errors that execFileSync would also hit
    if (e.code !== "ENOSYS" && e.code !== "EINVAL") {
      process.exitCode = 1;
      console.error(e.message);
      process.exit(1);
    }
  }
}

try {
  execFileSync(exe, process.argv.slice(2), { stdio: "inherit" });
} catch (e) {
  if (typeof e.status === "number") {
    process.exitCode = e.status;
  } else if (e.signal) {
    // Child killed by signal (e.g., SIGINT from Ctrl+C) — re-raise
    process.kill(process.pid, e.signal);
  } else {
    throw e;
  }
}
