// Tạo icon Flowsup: vẽ bằng Edge (headless) từ tools/icongen.html, rồi
//  - ghép flowsup.ico (16 → 256 px, mỗi cỡ một ảnh PNG vẽ riêng)
//  - chèn favicon các cỡ vào flowsup.html (giữa <!--ICONS--> và <!--/ICONS-->)
//  - lưu ảnh xem thử vào build/icons/
// Chạy: node tools/make-icons.mjs
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, existsSync, mkdtempSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const edge = ['C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe', 'C:/Program Files/Microsoft/Edge/Application/msedge.exe'].find(existsSync);
if (!edge) throw new Error('Không thấy Microsoft Edge');
const prof = mkdtempSync(join(tmpdir(), 'fs-icon-'));
// Edge ghi --dump-dom vào console của tiến trình cha, nên gọi qua PowerShell để bắt được đầu ra.
const cmd = `& '${edge}' --headless=new --disable-gpu '--user-data-dir=${prof}' --virtual-time-budget=3000 --dump-dom '${pathToFileURL(join(root, 'tools', 'icongen.html')).href}' | Out-String -Width 100000000`;
const dom = execFileSync('powershell', ['-NoProfile', '-Command', cmd], { encoding: 'utf8', maxBuffer: 64 << 20 });
const body = dom.replace(/\s+/g, ' ').match(/<pre id="out">BEGIN (.*?) END<\/pre>/)?.[1];
if (!body) throw new Error('Edge không trả về icon');
const imgs = body.trim().split(' ').map(l => { const [px, b64] = l.split(':'); return { px: +px, png: Buffer.from(b64, 'base64') }; });

const out = join(root, 'build', 'icons'); mkdirSync(out, { recursive: true });
imgs.forEach(i => writeFileSync(join(out, `flowsup-${i.px}.png`), i.png));

// ICO: ICONDIR + ICONDIRENTRY[] + dữ liệu PNG
const head = Buffer.alloc(6 + 16 * imgs.length); head.writeUInt16LE(0, 0); head.writeUInt16LE(1, 2); head.writeUInt16LE(imgs.length, 4);
let off = head.length;
imgs.forEach((i, n) => {
  const e = 6 + 16 * n;
  head.writeUInt8(i.px >= 256 ? 0 : i.px, e); head.writeUInt8(i.px >= 256 ? 0 : i.px, e + 1);
  head.writeUInt8(0, e + 2); head.writeUInt8(0, e + 3); head.writeUInt16LE(1, e + 4); head.writeUInt16LE(32, e + 6);
  head.writeUInt32LE(i.png.length, e + 8); head.writeUInt32LE(off, e + 12); off += i.png.length;
});
writeFileSync(join(root, 'flowsup.ico'), Buffer.concat([head, ...imgs.map(i => i.png)]));

// Favicon trong trang: Edge chọn cỡ hợp nhất cho cửa sổ app và taskbar
const page = join(root, 'flowsup.html'), html = readFileSync(page, 'utf8');
const links = imgs.filter(i => [16, 20, 24, 30, 32, 36, 40, 48, 64, 256].includes(i.px))
  .map(i => `<link rel="icon" type="image/png" sizes="${i.px}x${i.px}" href="data:image/png;base64,${i.png.toString('base64')}">`).join('\n');
const next = html.replace(/<!--ICONS-->[\s\S]*?<!--\/ICONS-->/, `<!--ICONS-->\n${links}\n<!--/ICONS-->`);
if (next === html && !html.includes(links)) throw new Error('Không thấy chỗ <!--ICONS--> trong flowsup.html');
writeFileSync(page, next);
console.log('Icon:', imgs.map(i => i.px).join(', '), '→ flowsup.ico + flowsup.html');
