#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");
const { spawn } = require("node:child_process");

const binaries = {
  "darwin-arm64": ["aarch64-apple-darwin", "cv-linter"],
  "darwin-x64": ["x86_64-apple-darwin", "cv-linter"],
  "win32-x64": ["x86_64-pc-windows-msvc", "cv-linter.exe"],
};

const platform = `${process.platform}-${process.arch}`;
const selected = binaries[platform];

if (!selected) {
  console.error(`CV Linter does not support ${platform}.`);
  process.exit(1);
}

const binary = path.join(__dirname, "bin", ...selected);

if (!fs.existsSync(binary)) {
  console.error(`CV Linter executable is missing from the extension: ${binary}`);
  process.exit(1);
}

if (process.platform !== "win32") {
  fs.chmodSync(binary, 0o755);
}

const child = spawn(binary, ["mcp", "--stdio"], {
  stdio: "inherit",
  windowsHide: true,
});

child.on("error", (error) => {
  console.error(`Could not start CV Linter: ${error.message}`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => child.kill(signal));
}
