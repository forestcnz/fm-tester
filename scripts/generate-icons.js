import sharp from 'sharp';
import pngToIco from 'png-to-ico';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { mkdirSync, existsSync, writeFileSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const iconsDir = join(__dirname, '../src-tauri/icons');
const svgPath = join(iconsDir, 'icon.svg');

// 确保目录存在
if (!existsSync(iconsDir)) {
  mkdirSync(iconsDir, { recursive: true });
}

// 需要生成的图标尺寸
const sizes = [
  { name: '32x32.png', size: 32 },
  { name: '64x64.png', size: 64 },
  { name: '128x128.png', size: 128 },
  { name: '128x128@2x.png', size: 256 },
  { name: '256x256.png', size: 256 },
  { name: '512x512.png', size: 512 },
  { name: 'icon.png', size: 512 },
  // Windows Store icons
  { name: 'Square30x30Logo.png', size: 30 },
  { name: 'Square44x44Logo.png', size: 44 },
  { name: 'Square71x71Logo.png', size: 71 },
  { name: 'Square89x89Logo.png', size: 89 },
  { name: 'Square107x107Logo.png', size: 107 },
  { name: 'Square142x142Logo.png', size: 142 },
  { name: 'Square150x150Logo.png', size: 150 },
  { name: 'Square284x284Logo.png', size: 284 },
  { name: 'Square310x310Logo.png', size: 310 },
  { name: 'StoreLogo.png', size: 50 },
];

async function generateIcons() {
  console.log('开始生成图标...');
  
  // 生成 PNG 文件
  for (const { name, size } of sizes) {
    const outputPath = join(iconsDir, name);
    await sharp(svgPath)
      .resize(size, size)
      .png()
      .toFile(outputPath);
    console.log(`✓ 生成 ${name} (${size}x${size})`);
  }
  
  // 生成 ICO 文件 (Windows)
  console.log('\n生成 ICO 文件...');
  const icoSizes = [16, 32, 48, 64, 128, 256];
  const icoBuffers = await Promise.all(
    icoSizes.map(size => 
      sharp(svgPath).resize(size, size).png().toBuffer()
    )
  );
  
  const icoBuffer = await pngToIco(icoBuffers);
  const icoPath = join(iconsDir, 'icon.ico');
  writeFileSync(icoPath, icoBuffer);
  console.log('✓ 生成 icon.ico');
  
  // ICNS 文件需要在 macOS 上生成，或使用在线工具
  console.log('\n⚠️  ICNS 文件需要在 macOS 上生成，或使用在线工具：');
  console.log('   - 在线工具: https://cloudconvert.com/png-to-icns');
  console.log('   - macOS 命令: iconutil -c icns icon.iconset');
  
  console.log('\n✅ 所有图标已生成完成！');
}

generateIcons().catch(console.error);