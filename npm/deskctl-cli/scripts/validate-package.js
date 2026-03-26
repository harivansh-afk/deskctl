const fs = require("node:fs");
const path = require("node:path");

const { readPackageJson, supportedTarget, vendorBinaryPath } = require("./support");

function readCargoVersion() {
  const cargoToml = fs.readFileSync(
    path.resolve(__dirname, "..", "..", "..", "Cargo.toml"),
    "utf8"
  );
  const match = cargoToml.match(/^version = "([^"]+)"/m);
  if (!match) {
    throw new Error("Could not determine Cargo.toml version.");
  }
  return match[1];
}

function main() {
  const pkg = readPackageJson();
  const cargoVersion = readCargoVersion();

  if (pkg.version !== cargoVersion) {
    throw new Error(
      `Version mismatch: npm package is ${pkg.version}, Cargo.toml is ${cargoVersion}.`
    );
  }

  if (pkg.bin?.deskctl !== "bin/deskctl.js") {
    throw new Error("deskctl-cli must expose the deskctl bin entrypoint.");
  }

  const target = supportedTarget("linux", "x64");
  const targetPath = vendorBinaryPath(target);
  const vendorDir = path.dirname(targetPath);
  if (!vendorDir.endsWith(path.join("deskctl-cli", "vendor"))) {
    throw new Error("Vendor binary directory resolved unexpectedly.");
  }
}

main();
