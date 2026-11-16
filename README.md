# Rootcash Desktop

Khung thu – chi – trả nợ – cho vay cho máy tính: nhập khung một lần, xem **free float** từng tháng, biết ngày nào dòng tiền xuống thấp và khi nào lấp được khoảng hổng nợ.

```
FREE FLOAT = Thu + Thu hồi cho vay − Chi phí − Trả nợ
```

Tiền giải ngân (nhận vay, chuyển cho vay) chỉ làm đổi số dư, không bao giờ tính là free float.

## Tải về

**[⬇ Tải Rootcash-Desktop.zip](https://github.com/derekdaydoi/Rootcash-Desktop/releases/latest/download/Rootcash-Desktop.zip)** (Windows)

1. Giải nén, giữ nguyên các file cạnh nhau.
2. Nhấp đúp `Rootcash.exe`.
3. Nếu máy chặn file exe (phần mềm diệt virus thường chặn exe chưa ký), nhấp đúp `Rootcash.cmd` — app giống hệt, chỉ không có tự lưu ra file.

Máy cần có Microsoft Edge hoặc Google Chrome. Không cần cài đặt gì thêm. Tự build exe: chạy `build.cmd`.

## Local only

- **Không có máy chủ ngoài, không tài khoản.** App không gửi số liệu của bạn đi đâu cả, không tải gì từ mạng (dùng font của Windows).
- **Dữ liệu nằm trên máy bạn**, trong hồ sơ riêng của Rootcash (`%LOCALAPPDATA%\Rootcash\window`), tách khỏi trình duyệt thường.
- **Tự lưu ra file:** khi mở bằng `Rootcash.exe`, mỗi lần sửa app ghi một bản JSON vào `Documents\Rootcash` (`rootcash-latest.json` + `backups\` một bản mỗi ngày, giữ 60 ngày). Nếu app mở lên mà trống, nó tự khôi phục từ bản gần nhất.
  - Exe chạy một máy chủ nhỏ **chỉ nghe 127.0.0.1:47613**; mỗi lần mở sinh một mã khoá mới chỉ truyền cho cửa sổ app, nên trang web khác không đọc/ghi được.
- **Xuất file / Nhập file** vẫn có để chuyển sang máy khác.

## App làm gì

- **Số dư hiện có + ngày nhập:** số dư tiền mặt chạy tiếp từ ngày đó; Khung (thu, chi phí, trả nợ, free float) luôn tính đủ cả tháng.
- **Thu** và **Chi phí:** mỗi khoản có nhãn **Hàng tháng / Chỉ một tháng / Khoảng tháng**; danh sách đổi theo tháng đang xem. Thu có ngày tiền về; chi phí trải đều theo ngày.
- **Đi vay** và **Cho vay** dùng chung một bộ tính, tự sinh lịch theo đúng ngày:
  - Ngày vay, ngày kỳ đầu, **ngày đáo hạn** (kỳ cuối có thể lẻ ngày).
  - Trả gốc: đều mỗi kỳ, một lần lúc đáo hạn, hoặc góp đều.
  - Tính lãi: không lãi, phẳng, dư nợ giảm dần, cố định mỗi kỳ, hoặc **tổng tiền lãi** (chia theo số ngày).
  - **Lãi theo ngày thực tế** của từng kỳ: lãi ngày = lãi năm ÷ 365 (lãi tháng × 12 ÷ 365).
  - Trả lãi: mỗi kỳ, trả trước (vào ngày kỳ đầu), hoặc một lần lúc đáo hạn.
  - **Chưa hẹn ngày trả gốc:** gốc vẫn tính vào dư nợ; lãi (nếu có) vẫn trả hằng tháng từ ngày kỳ đầu — hợp cho đáo thẻ, thấu chi.
- **M/X + i (%) = Rate:** tiền phải trả mỗi kỳ trên mỗi đồng gốc — M/X là phần gốc, i là phần lãi.
- **Thẻ số liệu:** Free float · Trả nợ / Thu · M/X + i · Đáy số dư · Số dư cuối tháng (thác nước tiền mặt).
- **6 / 12 / 24 tháng tới:** cột Thu chứa cột chi; phần lộ ra là free float, phần đỏ là chi vượt thu, chấm đỏ là tháng có ngày số dư âm.
- **Trong tháng:** số dư cuối mỗi ngày; danh sách gom theo ngày, bấm **+** để xem tiền vào / tiền ra chi tiết.
- **Vị thế nợ:** đang vay, cho vay, nợ ròng (gap) và tháng số dư đủ lấp gap.

## Mã nguồn

| File | Vai trò |
| --- | --- |
| `rootcash.html` | Toàn bộ app: giao diện và phần tính toán (khối `<script id="domain">`, hàm thuần). HTML/CSS/JS thuần, không framework, không bước build. |
| `Rootcash.cs` | Trình mở: mở `rootcash.html` trong cửa sổ riêng của Edge/Chrome và chạy máy chủ tự lưu chỉ trong máy. |
| `Rootcash.cmd` | Trình mở dự phòng (không tự lưu). `Rootcash.cmd shortcut` tạo lối tắt trên Desktop. |
| `build.cmd` | Tạo `Rootcash.exe` bằng trình biên dịch có sẵn trong Windows. |
| `rootcash.ico` | Icon app. |

Bản cho iPhone: [Rootcash webapp](https://github.com/derekdaydoi/Rootcash). Rootcash Desktop nhập được file sao lưu của bản đó.

---

© Copyright from derekdaydoi
