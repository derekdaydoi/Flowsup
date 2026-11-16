// Rootcash launcher
// 1. Mở rootcash.html (cạnh exe) trong cửa sổ app riêng của Edge/Chrome, hồ sơ riêng %LOCALAPPDATA%\Rootcash\window.
// 2. Tự lưu: chạy máy chủ nhỏ chỉ nghe 127.0.0.1:47613, nhận bản JSON từ trang và ghi vào Documents\Rootcash
//    (rootcash-latest.json + backups\rootcash-YYYY-MM-DD.json, giữ 60 bản). Mọi yêu cầu phải kèm mã khoá k
//    do exe sinh mỗi lần chạy và chỉ truyền cho trang qua #k=…, nên trang web khác không đọc/ghi được.
// 3. Exe chạy ngầm tới khi cửa sổ Rootcash đóng.
using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Management;
using System.Net;
using System.Net.Sockets;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using System.Windows.Forms;

static class Program {
    const int Port = 47613;
    static string Token = "", DataDir, Profile;

    [STAThread]
    static void Main(string[] argv) {
        bool test = argv.Length > 0 && argv[0] == "--test";                         // --test <thư mục>: chỉ chạy máy chủ 90 giây
        string local = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Rootcash");
        Directory.CreateDirectory(local);
        Profile = Path.Combine(local, "window");
        DataDir = test && argv.Length > 1 ? argv[1] : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments), "Rootcash");
        string tokenFile = Path.Combine(local, test ? "token-test" : "token");

        TcpListener server = null;
        try { server = new TcpListener(IPAddress.Loopback, Port); server.Start(); } catch { server = null; }
        if (server != null) { Token = NewToken(); File.WriteAllText(tokenFile, Token); }
        else if (File.Exists(tokenFile)) Token = File.ReadAllText(tokenFile).Trim();   // đã có Rootcash đang chạy → dùng chung máy chủ

        if (!test) {
            string page = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "rootcash.html");
            if (!File.Exists(page)) { MessageBox.Show("Không thấy rootcash.html cạnh Rootcash.exe.", "Rootcash"); return; }
            string browser = FindBrowser();
            if (browser == null) { MessageBox.Show("Cần Microsoft Edge hoặc Google Chrome để hiển thị Rootcash.", "Rootcash"); return; }
            string url = new Uri(page).AbsoluteUri + (Token.Length == 32 ? "#k=" + Token : "");
            string args = "--app=\"" + url + "\" --user-data-dir=\"" + Profile + "\" --no-first-run --no-default-browser-check --window-size=1440,920";
            Process.Start(new ProcessStartInfo(browser, args) { UseShellExecute = false });
        }
        if (server == null) return;

        new Thread(() => { while (true) { try { var c = server.AcceptTcpClient(); ThreadPool.QueueUserWorkItem(_ => Handle(c)); } catch { return; } } }) { IsBackground = true }.Start();
        if (test) { Thread.Sleep(90000); return; }
        Thread.Sleep(10000);
        while (BrowserAlive()) Thread.Sleep(10000);
    }

    static string NewToken() {
        var b = new byte[16]; using (var r = RandomNumberGenerator.Create()) r.GetBytes(b);
        return string.Concat(b.Select(x => x.ToString("x2")));
    }

    static string FindBrowser() {
        string[] roots = { Environment.GetEnvironmentVariable("ProgramFiles(x86)"), Environment.GetEnvironmentVariable("ProgramFiles"),
                           Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData) };
        foreach (string a in new[] { @"Microsoft\Edge\Application\msedge.exe", @"Google\Chrome\Application\chrome.exe" })
            foreach (string r in roots) { if (string.IsNullOrEmpty(r)) continue; string p = Path.Combine(r, a); if (File.Exists(p)) return p; }
        return null;
    }

    static bool BrowserAlive() {
        try {
            using (var q = new ManagementObjectSearcher("SELECT CommandLine FROM Win32_Process WHERE Name='msedge.exe' OR Name='chrome.exe'"))
                foreach (ManagementObject o in q.Get()) { var cl = o["CommandLine"] as string; if (cl != null && cl.IndexOf(Profile, StringComparison.OrdinalIgnoreCase) >= 0) return true; }
        } catch { return true; }
        return false;
    }

    static void Handle(TcpClient c) {
        using (c) try {
            c.ReceiveTimeout = 5000; c.SendTimeout = 5000;
            var s = c.GetStream();
            var head = new StringBuilder(); int b, n = 0;
            while ((b = s.ReadByte()) != -1 && n++ < 16384) { head.Append((char)b); if (head.Length >= 4 && head.ToString(head.Length - 4, 4) == "\r\n\r\n") break; }
            string[] lines = head.ToString().Split(new[] { "\r\n" }, StringSplitOptions.None), first = lines[0].Split(' ');
            if (first.Length < 2) return;
            string method = first[0], target = first[1], path = target.Split('?')[0];
            string key = target.Contains("k=") ? target.Substring(target.IndexOf("k=") + 2).Split('&')[0] : "";
            int len = 0;
            foreach (var l in lines) if (l.StartsWith("Content-Length:", StringComparison.OrdinalIgnoreCase)) int.TryParse(l.Substring(15).Trim(), out len);
            if (len < 0 || len > 10 * 1024 * 1024) { Reply(s, 413, "{\"ok\":false}"); return; }
            var body = new byte[len]; int got = 0;
            while (got < len) { int r = s.Read(body, got, len - got); if (r <= 0) break; got += r; }

            if (method == "OPTIONS") { Reply(s, 204, ""); return; }
            if (Token.Length != 32 || key != Token) { Reply(s, 403, "{\"ok\":false}"); return; }
            string latest = Path.Combine(DataDir, "rootcash-latest.json");
            if (method == "POST" && path == "/save") {
                string json = Encoding.UTF8.GetString(body, 0, got);
                if (!json.TrimStart().StartsWith("{") || !json.Contains("\"rootcash-desktop\"")) { Reply(s, 400, "{\"ok\":false}"); return; }
                lock (typeof(Program)) {
                    string bdir = Path.Combine(DataDir, "backups"); Directory.CreateDirectory(bdir);
                    WriteAtomic(latest, json);
                    WriteAtomic(Path.Combine(bdir, "rootcash-" + DateTime.Now.ToString("yyyy-MM-dd") + ".json"), json);
                    foreach (var old in Directory.GetFiles(bdir, "rootcash-*.json").OrderByDescending(f => f).Skip(60)) File.Delete(old);
                }
                Reply(s, 200, "{\"ok\":true,\"dir\":\"" + DataDir.Replace("\\", "\\\\").Replace("\"", "") + "\"}");
            }
            else if (method == "GET" && path == "/latest") {
                if (File.Exists(latest)) Reply(s, 200, File.ReadAllText(latest, Encoding.UTF8)); else Reply(s, 404, "{\"ok\":false}");
            }
            else Reply(s, 404, "{\"ok\":false}");
        } catch { }
    }

    static void WriteAtomic(string file, string text) {
        string tmp = file + ".tmp";
        File.WriteAllText(tmp, text, new UTF8Encoding(false));
        if (File.Exists(file)) File.Replace(tmp, file, null); else File.Move(tmp, file);
    }

    static void Reply(Stream s, int code, string body) {
        byte[] b = Encoding.UTF8.GetBytes(body);
        string status = code == 200 ? "OK" : code == 204 ? "No Content" : "Error";
        string h = "HTTP/1.1 " + code + " " + status + "\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: " + b.Length +
                   "\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: *\r\n" +
                   "Access-Control-Allow-Private-Network: true\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n";
        byte[] hb = Encoding.ASCII.GetBytes(h);
        s.Write(hb, 0, hb.Length); s.Write(b, 0, b.Length); s.Flush();
    }
}
