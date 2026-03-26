const fs = require("node:fs");

const {
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
} = require("./support");

async function main() {
  const pkg = readPackageJson();
  const target = supportedTarget();
  const targetPath = vendorBinaryPath(target);

  ensureVendorDir();

  if (process.env.DESKCTL_BINARY_PATH) {
    installLocalBinary(process.env.DESKCTL_BINARY_PATH, targetPath);
    return;
  }

  const tag = releaseTag(pkg);
  const assetUrl = releaseAssetUrl(tag, target.assetName);
  const checksumText = (await download(checksumsUrl(tag))).toString("utf8");
  const expectedSha = checksumForAsset(checksumText, target.assetName);
  const asset = await download(assetUrl);
  const actualSha = sha256(asset);

  if (actualSha !== expectedSha) {
    throw new Error(
      `Checksum mismatch for ${target.assetName}. Expected ${expectedSha}, got ${actualSha}.`
    );
  }

  fs.writeFileSync(targetPath, asset);
  fs.chmodSync(targetPath, 0o755);
}

main().catch((error) => {
  console.error(`deskctl-cli install failed: ${error.message}`);
  process.exit(1);
});
