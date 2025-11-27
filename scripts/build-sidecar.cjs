/**
 * Build Ignite Bridge Sidecar
 *
 * Cross-platform script untuk compile Node.js SEA (Single Executable Application)
 * Supports: Windows, Linux, macOS
 */

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const os = require("os");

const ROOT_DIR = path.join(__dirname, "..");
const SRC_BRIDGE_DIR = path.join(ROOT_DIR, "src-bridge");
const BINARIES_DIR = path.join(ROOT_DIR, "src-tauri", "binaries");

// Detect platform and architecture
const platform = os.platform();
const arch = os.arch();

// Get sidecar name based on Tauri target triple
function getSidecarName() {
  if (platform === "win32") {
    return "ignite-x86_64-pc-windows-msvc.exe";
  } else if (platform === "linux") {
    return arch === "arm64"
      ? "ignite-aarch64-unknown-linux-gnu"
      : "ignite-x86_64-unknown-linux-gnu";
  } else if (platform === "darwin") {
    return arch === "arm64"
      ? "ignite-aarch64-apple-darwin"
      : "ignite-x86_64-apple-darwin";
  }
  throw new Error(`Unsupported platform: ${platform}`);
}

function run(cmd, options = {}) {
  console.log(`> ${cmd}`);
  try {
    execSync(cmd, { stdio: "inherit", ...options });
  } catch (e) {
    if (!options.ignoreError) {
      throw e;
    }
  }
}

async function main() {
  console.log(`\n🔨 Building Ignite Bridge Sidecar for ${platform}/${arch}\n`);

  // 1. Bundle with esbuild
  console.log("📦 Bundling with esbuild...");
  run(
    "npx esbuild src-bridge/ignite.cjs --bundle --platform=node --outfile=src-bridge/.cache/ignite-bundle.cjs",
    { cwd: ROOT_DIR }
  );

  // 2. Generate SEA blob
  console.log("\n🔧 Generating SEA blob...");
  run("node --experimental-sea-config sea-config.json", {
    cwd: path.join(SRC_BRIDGE_DIR, ".cache"),
  });

  // 3. Copy node executable
  console.log("\n📋 Copying Node.js executable...");
  const nodeExe = process.execPath;
  const sidecarName = getSidecarName();
  const tempExe = path.join(
    SRC_BRIDGE_DIR,
    ".cache",
    platform === "win32" ? "ignite.exe" : "ignite"
  );

  fs.copyFileSync(nodeExe, tempExe);

  // On macOS/Linux, need to make it executable and potentially remove signature
  if (platform === "darwin") {
    // Remove code signature on macOS
    run(`codesign --remove-signature "${tempExe}"`, { ignoreError: true });
  }
  if (platform !== "win32") {
    fs.chmodSync(tempExe, 0o755);
  }

  // 4. Inject SEA blob with postject
  console.log("\n💉 Injecting SEA blob...");
  const blobPath = path.join(SRC_BRIDGE_DIR, "ignite-sea.blob");

  if (platform === "darwin") {
    run(
      `npx postject "${tempExe}" NODE_SEA_BLOB "${blobPath}" --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2 --macho-segment-name NODE_SEA`,
      { cwd: ROOT_DIR }
    );
  } else {
    run(
      `npx postject "${tempExe}" NODE_SEA_BLOB "${blobPath}" --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2`,
      { cwd: ROOT_DIR }
    );
  }

  // 5. Create binaries directory
  if (!fs.existsSync(BINARIES_DIR)) {
    fs.mkdirSync(BINARIES_DIR, { recursive: true });
  }

  // 6. Move to binaries folder
  const finalPath = path.join(BINARIES_DIR, sidecarName);
  console.log(`\n📁 Moving to ${finalPath}...`);

  if (fs.existsSync(finalPath)) {
    fs.unlinkSync(finalPath);
  }
  fs.renameSync(tempExe, finalPath);

  // 7. Compress with UPX (optional, skip on macOS as UPX doesn't work well there)
  if (platform !== "darwin") {
    console.log("\n🗜️ Compressing with UPX...");
    try {
      run(`upx --best "${finalPath}"`, { cwd: ROOT_DIR });
    } catch (e) {
      console.log("⚠️ UPX compression skipped (UPX not installed or failed)");
    }
  }

  // 8. Sign on macOS (ad-hoc)
  if (platform === "darwin") {
    console.log("\n🔏 Signing executable...");
    run(`codesign --sign - "${finalPath}"`, { ignoreError: true });
  }

  // Get final size
  const stats = fs.statSync(finalPath);
  const sizeMB = (stats.size / 1024 / 1024).toFixed(1);

  console.log(`\n✅ Build complete!`);
  console.log(`   Output: ${finalPath}`);
  console.log(`   Size: ${sizeMB} MB\n`);
}

main().catch((err) => {
  console.error("❌ Build failed:", err.message);
  process.exit(1);
});
