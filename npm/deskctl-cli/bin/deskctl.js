#!/usr/bin/env node

const fs = require("node:fs");
const { spawn } = require("node:child_process");

const { readPackageJson, releaseTag, supportedTarget, vendorBinaryPath } = require("../scripts/support");

function main() {
  const pkg = readPackageJson();
  const target = supportedTarget();
  const binaryPath = vendorBinaryPath(target);

  if (!fs.existsSync(binaryPath)) {
    console.error(
      [
        "deskctl binary is missing from the npm package install.",
        `Expected: ${binaryPath}`,
        `Package version: ${pkg.version}`,
        `Release tag: ${releaseTag(pkg)}`,
        "Try reinstalling deskctl-cli or check that your target is supported."
      ].join("\n")
    );
    process.exit(1);
  }

  const child = spawn(binaryPath, process.argv.slice(2), { stdio: "inherit" });
  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    process.exit(code ?? 1);
  });
}

main();
