// seticon <exe> <ico>: gắn icon nhiều cỡ vào exe (RT_ICON + RT_GROUP_ICON) bằng API Windows,
// thay cho windres/rc.exe (bộ Rust GNU không kèm). Explorer, lối tắt và taskbar lấy icon từ đây.
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[link(name = "kernel32")]
extern "system" {
    fn BeginUpdateResourceW(file: *const u16, delete_existing: i32) -> *mut u8;
    fn UpdateResourceW(h: *mut u8, kind: *const u16, name: *const u16, lang: u16, data: *const u8, size: u32) -> i32;
    fn EndUpdateResourceW(h: *mut u8, discard: i32) -> i32;
}
const RT_ICON: usize = 3;
const RT_GROUP_ICON: usize = 14;

fn u16le(b: &[u8], i: usize) -> u16 { u16::from_le_bytes([b[i], b[i + 1]]) }
fn u32le(b: &[u8], i: usize) -> u32 { u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) }

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 3 { eprintln!("dùng: seticon <exe> <ico>"); std::process::exit(2); }
    let ico = std::fs::read(&a[2]).expect("không đọc được file ico");
    let n = u16le(&ico, 4) as usize;
    assert!(u16le(&ico, 2) == 1 && n > 0, "file ico không hợp lệ");
    let exe: Vec<u16> = OsStr::new(&a[1]).encode_wide().chain(Some(0)).collect();
    unsafe {
        let h = BeginUpdateResourceW(exe.as_ptr(), 0);
        assert!(!h.is_null(), "không mở được exe để ghi icon");
        let mut group = vec![0u8, 0, 1, 0]; group.extend_from_slice(&(n as u16).to_le_bytes());
        for i in 0..n {
            let e = 6 + 16 * i;
            let (size, off) = (u32le(&ico, e + 8), u32le(&ico, e + 12) as usize);
            let data = &ico[off..off + size as usize];
            let ok = UpdateResourceW(h, RT_ICON as *const u16, (i + 1) as *const u16, 0, data.as_ptr(), size);
            assert!(ok != 0, "ghi icon {} lỗi", i + 1);
            group.extend_from_slice(&ico[e..e + 12]);                                 // rộng, cao, màu, 0, planes, bpp, kích thước
            group.extend_from_slice(&((i + 1) as u16).to_le_bytes());                  // số hiệu RT_ICON
        }
        let ok = UpdateResourceW(h, RT_GROUP_ICON as *const u16, 1 as *const u16, 0, group.as_ptr(), group.len() as u32);
        assert!(ok != 0, "ghi nhóm icon lỗi");
        assert!(EndUpdateResourceW(h, 0) != 0, "lưu exe lỗi");
    }
    println!("Đã gắn {n} cỡ icon vào {}", a[1]);
}
