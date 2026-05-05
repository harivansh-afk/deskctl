const crypto = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");

const PACKAGE_ROOT = path.resolve(__dirname, "..");
const VENDOR_DIR = path.join(PACKAGE_ROOT, "vendor");
const PACKAGE_JSON = path.join(PACKAGE_ROOT, "package.json");

function readPackageJson() {
  return JSON.parse(fs.readFileSync(PACKAGE_JSON, "utf8"));
}

function releaseTag(pkg) {
  return process.env.DESKCTL_RELEASE_TAG || `v${pkg.version}`;
}

function supportedTarget(platform = process.platform, arch = process.arch) {
  if (platform === "linux" && arch === "x64") {
    return {
      platform,
      arch,
      assetName: "deskctl-linux-x86_64",
      binaryName: "deskctl-linux-x86_64"
    };
  }

  throw new Error(
    `deskctl currently supports linux-x64 only. Received ${platform}-${arch}.`
  );
}

function vendorBinaryPath(target) {
  return path.join(VENDOR_DIR, target.binaryName);
}

function releaseBaseUrl(tag) {
  return (
    process.env.DESKCTL_RELEASE_BASE_URL ||
    `https://git.harivan.sh/harivansh-afk/deskctl/releases/download/${tag}`
  );
}

function releaseAssetUrl(tag, assetName) {
  return process.env.DESKCTL_DOWNLOAD_URL || `${releaseBaseUrl(tag)}/${assetName}`;
}

function checksumsUrl(tag) {
  return `${releaseBaseUrl(tag)}/checksums.txt`;
}

function ensureVendorDir() {
  fs.mkdirSync(VENDOR_DIR, { recursive: true });
}

function checksumForAsset(contents, assetName) {
  const line = contents
    .split("\n")
    .map((value) => value.trim())
    .find((value) => value.endsWith(`  ${assetName}`) || value.endsWith(` *${assetName}`));

  if (!line) {
    throw new Error(`Could not find checksum entry for ${assetName}.`);
  }

  return line.split(/\s+/)[0];
}

function sha256(buffer) {
  return crypto.createHash("sha256").update(buffer).digest("hex");
}

function download(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if (
          response.statusCode &&
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          response.resume();
          resolve(download(response.headers.location));
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Download failed for ${url}: HTTP ${response.statusCode}`));
          return;
        }

        const chunks = [];
        response.on("data", (chunk) => chunks.push(chunk));
        response.on("end", () => resolve(Buffer.concat(chunks)));
      })
      .on("error", reject);
  });
}

function installLocalBinary(sourcePath, targetPath) {
  fs.copyFileSync(sourcePath, targetPath);
  fs.chmodSync(targetPath, 0o755);
}

module.exports = {
  PACKAGE_ROOT,
  VENDOR_DIR,
  checksumsUrl,
  checksumForAsset,
  download,
  ensureVendorDir,
  installLocalBinary,
  readPackageJson,
  releaseAssetUrl,
  releaseTag,
  sha256,
  supportedTarget,
  vendorBinaryPath
};
