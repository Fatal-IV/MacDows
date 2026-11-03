using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Windows;
using System.Windows.Input;

namespace MacDows.Setup;

public partial class SetupWindow : Window
{
    private bool _busy;

    public SetupWindow() => InitializeComponent();

    private void OnDrag(object s, MouseButtonEventArgs e)
    {
        if (e.ButtonState == MouseButtonState.Pressed) DragMove();
    }

    private void OnCloseClick(object s, RoutedEventArgs e)
    {
        if (!_busy) Close();
    }

    private static string InstalledExe => Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "MacDows", "macdows.exe");

    private async void OnInstallClick(object s, RoutedEventArgs e)
    {
        if (_busy) return;
        _busy = true;
        Show(WorkingPanel);
        Bar.Value = 5;
        StatusText.Text = "Hazırlanıyor…";
        var temp = Path.Combine(Path.GetTempPath(), "MacDows-Setup-" + Guid.NewGuid().ToString("N"));
        try
        {
            using var res = Assembly.GetExecutingAssembly().GetManifestResourceStream("payload.exe")
                ?? throw new InvalidOperationException("Kurulum paketi bulunamadı.");
            Directory.CreateDirectory(temp);
            var payload = Path.Combine(temp, "MacDows-install.exe");
            await using (var f = File.Create(payload)) await res.CopyToAsync(f);

            Bar.Value = 20;
            StatusText.Text = "MacDows kuruluyor…";
            var ticker = new System.Windows.Threading.DispatcherTimer { Interval = TimeSpan.FromMilliseconds(300) };
            ticker.Tick += (_, _) => { if (Bar.Value < 90) Bar.Value += (90 - Bar.Value) * 0.06; };
            ticker.Start();
            int code;
            try
            {
                using var p = Process.Start(new ProcessStartInfo(payload, "/S") { UseShellExecute = false })!;
                await p.WaitForExitAsync();
                code = p.ExitCode;
            }
            finally { ticker.Stop(); }

            if (code != 0 || !File.Exists(InstalledExe))
                throw new InvalidOperationException($"Kurucu {code} koduyla çıktı.");
            Bar.Value = 100;
            Show(DonePanel);
        }
        catch (Exception ex)
        {
            ErrorText.Text = ex.Message;
            Show(ErrorPanel);
        }
        finally
        {
            _busy = false;
            try { Directory.Delete(temp, true); } catch { }
        }
    }

    private void OnLaunchClick(object s, RoutedEventArgs e)
    {
        Process.Start(new ProcessStartInfo(InstalledExe) { UseShellExecute = true, WorkingDirectory = Path.GetDirectoryName(InstalledExe) });
        Close();
    }

    private void Show(FrameworkElement panel)
    {
        foreach (var p in new FrameworkElement[] { ReadyPanel, WorkingPanel, DonePanel, ErrorPanel })
            p.Visibility = ReferenceEquals(p, panel) ? Visibility.Visible : Visibility.Collapsed;
    }
}
