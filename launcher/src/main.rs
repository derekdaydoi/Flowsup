// Flowsup launcher (Rust, chỉ thư viện chuẩn + API Windows).
// 1. Mở flowsup.html (cạnh exe) trong cửa sổ app riêng của Edge/Chrome, hồ sơ riêng %LOCALAPPDATA%\Flowsup\window.
// 2. Tự lưu: máy chủ nhỏ chỉ nghe 127.0.0.1:47613, nhận bản JSON từ trang và ghi vào Documents\Flowsup
//    (flowsup-latest.json + backups\flowsup-YYYY-MM-DD.json, giữ 60 bản). Mọi yêu cầu phải kèm mã khoá k
//    do exe sinh mỗi lần chạy và chỉ truyền cho trang qua #k=…, nên trang web khác không đọc/ghi được.
// 3. Lần đầu chạy: chép bản tự lưu cũ của Rootcash (Documents\Rootcash\rootcash-latest.json) sang Flowsup.
// 4. Exe chạy ngầm (≈1 MB RAM) tới khi cửa sổ Flowsup đóng.
#![windows_subsystem = "windows"]

use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const PORT: u16 = 47613;
const APP: &str = "Flowsup";

#[repr(C)]
struct Guid { a: u32, b: u16, c: u16, d: [u8; 8] }
#[repr(C)]
#[derive(Default)]
struct SysTime { y: u16, m: u16, dow: u16, d: u16, h: u16, mi: u16, s: u16, ms: u16 }

#[link(name = "user32")]
extern "system" { fn MessageBoxW(h: *mut u8, text: *const u16, cap: *const u16, flags: u32) -> i32; }
#[link(name = "bcrypt")]
extern "system" { fn BCryptGenRandom(alg: *mut u8, buf: *mut u8, len: u32, flags: u32) -> i32; }
#[link(name = "shell32")]
extern "system" { fn SHGetKnownFolderPath(id: *const Guid, flags: u32, token: *mut u8, out: *mut *mut u16) -> i32; }
#[link(name = "ole32")]
extern "system" { fn CoTaskMemFree(p: *mut u16); }
#[link(name = "kernel32")]
extern "system" { fn GetLocalTime(t: *mut SysTime); }

#[link(name = "user32")]
extern "system" {
    fn EnumWindows(cb: extern "system" fn(isize, isize) -> i32, l: isize) -> i32;
    fn GetWindowTextW(h: isize, s: *mut u16, n: i32) -> i32;
    fn GetClassNameW(h: isize, s: *mut u16, n: i32) -> i32;
    fn IsWindowVisible(h: isize) -> i32;
    fn SendMessageW(h: isize, m: u32, w: usize, l: isize) -> isize;
    fn LoadImageW(inst: isize, name: *const u16, kind: u32, cx: i32, cy: i32, flags: u32) -> isize;
    fn GetDpiForWindow(h: isize) -> u32;
}
#[link(name = "kernel32")]
extern "system" { fn GetModuleHandleW(name: *const u16) -> isize; }

const DOCUMENTS: Guid = Guid { a: 0xFDD39AD0, b: 0x238F, c: 0x46AF, d: [0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03, 0x69, 0xC7] };

fn wide(s: &str) -> Vec<u16> { std::ffi::OsStr::new(s).encode_wide().chain(Some(0)).collect() }
fn message(text: &str) { unsafe { MessageBoxW(std::ptr::null_mut(), wide(text).as_ptr(), wide(APP).as_ptr(), 0x40); } }

fn new_token() -> String {
    let mut b = [0u8; 16];
    let ok = unsafe { BCryptGenRandom(std::ptr::null_mut(), b.as_mut_ptr(), 16, 2) } == 0;   // BCRYPT_USE_SYSTEM_PREFERRED_RNG
    if !ok { return String::new(); }
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

fn documents() -> PathBuf {
    unsafe {
        let mut p: *mut u16 = std::ptr::null_mut();
        if SHGetKnownFolderPath(&DOCUMENTS, 0, std::ptr::null_mut(), &mut p) == 0 && !p.is_null() {
            let mut n = 0; while *p.add(n) != 0 { n += 1; }
            let s = OsString::from_wide(std::slice::from_raw_parts(p, n));
            CoTaskMemFree(p);
            return PathBuf::from(s);
        }
    }
    PathBuf::from(std::env::var_os("USERPROFILE").unwrap_or_default()).join("Documents")
}

fn today() -> String {
    let mut t = SysTime::default();
    unsafe { GetLocalTime(&mut t); }
    format!("{:04}-{:02}-{:02}", t.y, t.m, t.d)
}

fn find_browser() -> Option<PathBuf> {
    let roots = ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"].iter().filter_map(|v| std::env::var_os(v)).map(PathBuf::from).collect::<Vec<_>>();
    for app in [r"Microsoft\Edge\Application\msedge.exe", r"Google\Chrome\Application\chrome.exe"] {
        for r in &roots { let p = r.join(app); if p.is_file() { return Some(p); } }
    }
    None
}

/// file:///C:/... — mã hoá mọi ký tự ngoài bộ an toàn (dấu cách, tiếng Việt trong đường dẫn).
fn file_url(p: &Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    let mut out = String::from("file:///");
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~/:".contains(&b) { out.push(b as char); } else { out.push_str(&format!("%{:02X}", b)); }
    }
    out
}

/// Icon taskbar rõ nét: Edge tự phóng favicon ra cỡ lẻ (40 px ở 125%) nên taskbar bị mờ.
/// Ở đây gắn thẳng icon của exe, đúng cỡ taskbar theo tỉ lệ màn hình (24 px × dpi/96), vẽ sẵn trong flowsup.ico.
extern "system" fn collect(h: isize, l: isize) -> i32 {
    let out = unsafe { &mut *(l as *mut Vec<isize>) };
    let mut t = [0u16; 64]; let mut c = [0u16; 64];
    unsafe {
        let n = GetWindowTextW(h, t.as_mut_ptr(), 64); let m = GetClassNameW(h, c.as_mut_ptr(), 64);
        if IsWindowVisible(h) != 0 && String::from_utf16_lossy(&t[..n.max(0) as usize]) == APP
            && String::from_utf16_lossy(&c[..m.max(0) as usize]) == "Chrome_WidgetWin_1" { out.push(h); }
    }
    1
}

struct Icons(std::collections::HashMap<u32, (isize, isize)>);
impl Icons {
    fn apply(&mut self) {
        let mut wins: Vec<isize> = Vec::new();
        unsafe { EnumWindows(collect, &mut wins as *mut Vec<isize> as isize); }
        for h in wins {
            let dpi = unsafe { GetDpiForWindow(h) }.max(96);
            let (big, small) = *self.0.entry(dpi).or_insert_with(|| unsafe {
                let inst = GetModuleHandleW(std::ptr::null());
                let px = |base: u32| (base * dpi / 96) as i32;
                (LoadImageW(inst, 1 as *const u16, 1, px(24), px(24), 0), LoadImageW(inst, 1 as *const u16, 1, px(16), px(16), 0))
            });
            if big == 0 { continue; }
            unsafe {
                if SendMessageW(h, 0x7F, 1, 0) != big {                                  // WM_GETICON ICON_BIG
                    SendMessageW(h, 0x80, 1, big);                                       // WM_SETICON ICON_BIG
                    if small != 0 { SendMessageW(h, 0x80, 0, small); }                   // WM_SETICON ICON_SMALL
                }
            }
        }
    }
}

/// Edge/Chrome giữ khoá độc quyền trên <hồ sơ>\lockfile khi đang chạy; tắt hẳn thì file biến mất.
fn browser_alive(profile: &Path) -> bool {
    match fs::OpenOptions::new().write(true).share_mode(0).open(profile.join("lockfile")) {
        Ok(_) => false,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(_) => true,
    }
}

fn write_atomic(file: &Path, text: &[u8]) -> std::io::Result<()> {
    let tmp = file.with_extension("json.tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, file)                                                            // thay file cũ trong một bước
}

fn reply(s: &mut TcpStream, code: u16, body: &[u8]) {
    let status = match code { 200 => "OK", 204 => "No Content", _ => "Error" };
    let head = format!("HTTP/1.1 {code} {status}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\n\
Access-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: *\r\n\
Access-Control-Allow-Private-Network: true\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", body.len());
    let _ = s.write_all(head.as_bytes());
    let _ = s.write_all(body);
    let _ = s.flush();
}

struct Ctx { token: String, data: PathBuf, lock: Mutex<()> }

fn handle(mut s: TcpStream, ctx: &Ctx) {
    let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = s.set_write_timeout(Some(Duration::from_secs(5)));
    // Đọc phần đầu (tối đa 16 KB) tới dòng trống.
    let mut head = Vec::new(); let mut byte = [0u8; 1];
    while head.len() < 16384 {
        match s.read(&mut byte) { Ok(1) => head.push(byte[0]), _ => break }
        if head.ends_with(b"\r\n\r\n") { break; }
    }
    let head = String::from_utf8_lossy(&head).to_string();
    let mut lines = head.split("\r\n");
    let first: Vec<&str> = lines.next().unwrap_or("").split(' ').collect();
    if first.len() < 2 { return; }
    let (method, target) = (first[0], first[1]);
    let path = target.split('?').next().unwrap_or("");
    let key = target.find("k=").map(|i| target[i + 2..].split('&').next().unwrap_or("")).unwrap_or("");
    let mut len: usize = 0;
    for l in lines {
        if l.len() > 15 && l[..15].eq_ignore_ascii_case("content-length:") { len = l[15..].trim().parse().unwrap_or(0); }
    }
    if len > 10 * 1024 * 1024 { return reply(&mut s, 413, b"{\"ok\":false}"); }
    let mut body = vec![0u8; len]; let mut got = 0;
    while got < len { match s.read(&mut body[got..]) { Ok(n) if n > 0 => got += n, _ => break } }
    body.truncate(got);

    if method == "OPTIONS" { return reply(&mut s, 204, b""); }
    if ctx.token.len() != 32 || key != ctx.token { return reply(&mut s, 403, b"{\"ok\":false}"); }
    let latest = ctx.data.join("flowsup-latest.json");
    if method == "POST" && path == "/save" {
        let json = String::from_utf8_lossy(&body);
        if !json.trim_start().starts_with('{') || !json.contains("\"rootcash-desktop\"") { return reply(&mut s, 400, b"{\"ok\":false}"); }
        let ok = {
            let _g = ctx.lock.lock().unwrap_or_else(|e| e.into_inner());
            let bdir = ctx.data.join("backups");
            fs::create_dir_all(&bdir).is_ok()
                && write_atomic(&latest, &body).is_ok()
                && write_atomic(&bdir.join(format!("flowsup-{}.json", today())), &body).is_ok()
                && { prune(&bdir); true }
        };
        if !ok { return reply(&mut s, 500, b"{\"ok\":false}"); }
        let dir = ctx.data.to_string_lossy().replace('\\', "\\\\").replace('"', "");
        reply(&mut s, 200, format!("{{\"ok\":true,\"dir\":\"{dir}\"}}").as_bytes());
    } else if method == "GET" && path == "/latest" {
        match fs::read(&latest) { Ok(b) => reply(&mut s, 200, &b), Err(_) => reply(&mut s, 404, b"{\"ok\":false}") }
    } else {
        reply(&mut s, 404, b"{\"ok\":false}");
    }
}

/// Giữ 60 bản sao lưu theo ngày mới nhất.
fn prune(bdir: &Path) {
    let mut v: Vec<PathBuf> = fs::read_dir(bdir).into_iter().flatten().flatten().map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.starts_with("flowsup-") && n.ends_with(".json")))
        .collect();
    v.sort(); v.reverse();
    for old in v.into_iter().skip(60) { let _ = fs::remove_file(old); }
}

/// Lần đầu: mang bản tự lưu của Rootcash sang Flowsup để mở lên đã có dữ liệu.
fn migrate(data: &Path) {
    let latest = data.join("flowsup-latest.json");
    if latest.exists() { return; }
    let old = documents().join("Rootcash").join("rootcash-latest.json");
    if old.is_file() && fs::create_dir_all(data).is_ok() { let _ = fs::copy(old, latest); }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let test = args.get(1).map_or(false, |a| a == "--test");                           // --test <thư mục>: chỉ chạy máy chủ 90 giây
    let local = PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join(APP);
    let _ = fs::create_dir_all(&local);
    let profile = local.join("window");
    let data = if test { PathBuf::from(args.get(2).cloned().unwrap_or_else(|| ".".into())) } else { documents().join(APP) };
    let token_file = local.join(if test { "token-test" } else { "token" });
    if !test { migrate(&data); }

    let server = TcpListener::bind((Ipv4Addr::LOCALHOST, PORT)).ok();
    let token = if server.is_some() {
        let t = new_token(); let _ = fs::write(&token_file, &t); t
    } else {
        fs::read_to_string(&token_file).map(|t| t.trim().to_string()).unwrap_or_default()  // đã có Flowsup đang chạy → dùng chung máy chủ
    };

    if !test {
        let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default();
        let page = exe_dir.join("flowsup.html");
        if !page.is_file() { return message("Không thấy flowsup.html cạnh Flowsup.exe."); }
        let Some(browser) = find_browser() else { return message("Cần Microsoft Edge hoặc Google Chrome để hiển thị Flowsup."); };
        let url = file_url(&page) + &(if token.len() == 32 { format!("#k={token}") } else { String::new() });
        let started = Command::new(browser)
            .arg(format!("--app={url}"))
            .arg(format!("--user-data-dir={}", profile.display()))
            .args(["--no-first-run", "--no-default-browser-check", "--window-size=1440,920", "--lang=vi",
                   // nhẹ hơn: tắt các dịch vụ phụ của trình duyệt mà một app cục bộ không cần
                   "--disable-extensions", "--disable-sync", "--disable-background-networking", "--disable-component-update",
                   "--disable-features=msEdgeSidebarV2,msHubApps,EdgeCollections,msShoppingExp,msEdgeTranslate"])
            .spawn();
        if started.is_err() { return message("Không mở được trình duyệt để hiển thị Flowsup."); }
    }
    let Some(server) = server else { return };

    let ctx = Arc::new(Ctx { token, data, lock: Mutex::new(()) });
    {
        let ctx = ctx.clone();
        thread::spawn(move || for c in server.incoming().flatten() { let ctx = ctx.clone(); thread::spawn(move || handle(c, &ctx)); });
    }
    if test { thread::sleep(Duration::from_secs(90)); return; }
    // Vòng chạy ngầm: gắn lại icon nếu Edge đổi (mở cửa sổ mới, tải lại trang), và thoát khi cửa sổ đóng.
    let mut icons = Icons(Default::default());
    for tick in 0u32.. {
        icons.apply();
        if tick >= 10 && tick % 3 == 0 && !browser_alive(&profile) { break; }
        thread::sleep(Duration::from_secs(if tick < 30 { 1 } else { 2 }));
    }
}
