import pngToIco from "png-to-ico";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const inputPath = path.join(__dirname, "src-tauri", "icons", "icon.png");
const outputPath = path.join(__dirname, "src-tauri", "icons", "icon.ico");

console.log("Converting PNG to ICO...");
console.log("Input:", inputPath);
console.log("Output:", outputPath);

// Backup old icon if exists
if (fs.existsSync(outputPath)) {
  fs.copyFileSync(outputPath, outputPath + ".backup");
  console.log("Backed up old icon.ico");
}

pngToIco(inputPath)
  .then((buf) => {
    fs.writeFileSync(outputPath, buf);
    console.log("✅ Successfully converted PNG to ICO!");
    console.log("Icon created at:", outputPath);
  })
  .catch((err) => {
    console.error("❌ Error converting icon:", err);
    process.exit(1);
  });
