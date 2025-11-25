import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Create a simple blue PNG icon (1x1 pixel as base)
const createSimpleIcon = (outputPath) => {
  // Simple 1x1 blue PNG in base64
  const simplePNG = Buffer.from(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
    "base64"
  );

  fs.writeFileSync(outputPath, simplePNG);
  console.log(`Created: ${outputPath}`);
};

const iconsDir = path.join(__dirname, "src-tauri", "icons");

// Create basic placeholder PNGs
createSimpleIcon(path.join(iconsDir, "32x32.png"));
createSimpleIcon(path.join(iconsDir, "128x128.png"));
createSimpleIcon(path.join(iconsDir, "128x128@2x.png"));
createSimpleIcon(path.join(iconsDir, "icon.png"));

console.log("Basic icon files created.");
