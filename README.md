# Flowsup

Khung thu, chi, trả nợ và cho vay cho máy tính Windows. Nhập khung một lần, rồi xem từng tháng: ngày nào số dư xuống thấp nhất, tháng này dư ra bao nhiêu, khi nào hết nợ.

```
Free float = Thu + Thu hồi cho vay − Chi phí − Trả nợ
```

Tiền giải ngân (nhận vay, chuyển cho vay) chỉ làm đổi số dư, không tính vào dư trong tháng.

## Cài và mở

1. Giữ các file cạnh nhau: `Flowsup.exe`, `flowsup.html`, `flowsup.ico`, `Flowsup.cmd`.
2. Nhấp đúp `Flowsup.exe`. Muốn có lối tắt trên Desktop: chạy `Flowsup.cmd shortcut`.
3. Nếu máy chặn exe (phần mềm diệt virus hay chặn exe chưa ký), nhấp đúp `Flowsup.cmd`: app giống hệt, chỉ không tự lưu ra file.

Máy cần Microsoft Edge hoặc Google Chrome. Không cần cài gì thêm.

## Local only

- **Không máy chủ ngoài, không tài khoản.** App không gửi số liệu đi đâu và không tải gì từ mạng (dùng font của Windows).
- **Dữ liệu nằm trên máy bạn**, trong hồ sơ riêng `%LOCALAPPDATA%\Flowsup\window`, tách khỏi trình duyệt thường.
- **Tự lưu ra file:** khi mở bằng `Flowsup.exe`, mỗi lần sửa app ghi một bản JSON vào `Documents\Flowsup` (`flowsup-latest.json` và `backups\` một bản mỗi ngày, giữ 60 ngày). App mở lên mà trống thì tự khôi phục từ bản gần nhất.
  - `Flowsup.exe` viết bằng Rust, chỉ dùng thư viện chuẩn và API Windows, chạy ngầm với khoảng 1,4 MB bộ nhớ riêng. Máy chủ tự lưu **chỉ nghe 127.0.0.1:47613**; mỗi lần mở sinh một mã khoá mới chỉ truyền cho cửa sổ app, nên trang web khác không đọc hay ghi được.
- **Xuất file** để chuyển sang máy khác; **Nhập file** nằm trong menu ⋯.

## Ba trang

- **Tháng này:** con số lớn nhất là **số dư thấp nhất trong tháng**, kèm ngày, số ngày âm, số tiền cần thêm, câu ngắn "âm từ ngày nào, dương lại nhờ khoản nào" và mức thấp nhất của 6 tháng tới. Bên cạnh là phép tính số dư + vào − ra = cuối tháng và Free float. Biểu đồ số dư từng ngày có vạch hôm nay. Danh sách **Đến hạn** theo ngày, gộp gốc + lãi của từng khoản; khoản đã qua ngày hiện mờ.
- **Phân tích:** khung tháng (tiền vào = 100%, các nhóm chi, phần còn lại là Free float), chia theo Nhóm hoặc Nội dung; biểu đồ dòng chảy "Tiền đi đâu"; 6 / 12 / 24 tháng tới; vị thế nợ (nợ xoay vòng tách riêng, vốn hợp tác kinh doanh không trừ vào nợ ròng, cảnh báo khi một nguồn thu chiếm quá nửa tiền vào); lịch hết nợ theo năm.
- **Dữ liệu thô:** số dư, thu, chi phí và bảng khoản vay, lọc theo Nhóm và Nội dung; bấm một khoản để sửa, Enter để lưu.

## Cách tính

- **Chốt tháng:** mỗi tháng giữ một số dư chốt riêng (số tiền + ngày). Chốt tháng sau không ghi đè tháng trước. Tiền mặt chạy tiếp từ ngày chốt; app so với khung dự tính và báo **lệch**.
- **Thu** và **Chi phí:** Hàng tháng / Chỉ một tháng / Khoảng tháng. Thu có ngày tiền về; chi phí trải đều theo ngày.
- **Đi vay** và **Cho vay** dùng chung một bộ tính, tự sinh lịch theo đúng ngày:
  - Ngày vay, ngày kỳ đầu, **ngày đáo hạn** (kỳ cuối có thể lẻ ngày).
  - Trả gốc: đều mỗi kỳ, một lần lúc đáo hạn, hoặc góp đều.
  - Tính lãi: không lãi, phẳng, dư nợ giảm dần, cố định mỗi kỳ, hoặc **tổng tiền lãi** (chia theo số ngày).
  - **Lãi theo ngày thực tế:** lãi ngày = lãi năm ÷ 365 (lãi tháng × 12 ÷ 365).
  - Trả lãi: mỗi kỳ, trả trước (vào ngày kỳ đầu), hoặc một lần lúc đáo hạn.
  - **Chưa hẹn ngày trả gốc:** gốc vẫn tính vào dư nợ; lãi (nếu có) trả hằng tháng, hợp cho đáo thẻ, thấu chi.

## Mã nguồn

| File | Vai trò |
| --- | --- |
| `flowsup.html` | Toàn bộ app: giao diện và phần tính toán (khối `<script id="domain">`, hàm thuần). HTML/CSS/JS thuần, không framework. |
| `launcher/` | Mã Rust của `Flowsup.exe` (mở cửa sổ app, máy chủ tự lưu) và `seticon` (gắn icon vào exe). |
| `build.cmd` | Tạo `Flowsup.exe`: cần rustup với toolchain `stable-x86_64-pc-windows-gnu`. |
| `tools/make-icons.mjs` | Vẽ logo flow / sup ở mọi cỡ (16 → 256 px) bằng Edge, ghép `flowsup.ico` và favicon trong trang. |
| `tests/` | Kiểm tra phần tính toán (chèn vào bản sao của trang, mở bằng Edge). |
| `Flowsup.cmd` | Trình mở dự phòng (không tự lưu). `Flowsup.cmd shortcut` tạo lối tắt trên Desktop. |

---

© Copyright from derekdaydoi
