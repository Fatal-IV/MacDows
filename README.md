<div align="center">

# MacDows

**A Dynamic Island and a dock for Windows.**

[![Latest release](https://img.shields.io/github/v/release/Fatal-IV/MacDows?label=latest&color=0a84ff)](https://github.com/Fatal-IV/MacDows/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/Fatal-IV/MacDows/total?color=30d158)](https://github.com/Fatal-IV/MacDows/releases)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)

[**Download**](https://github.com/Fatal-IV/MacDows/releases/latest) · [Changelog](CHANGELOG.md) · [Türkçe](#türkçe)

</div>

---

## What it is

MacDows puts a Dynamic Island at the top of your screen and a dock at the bottom. Both are drawn by MacDows itself and follow your theme, with smooth motion throughout.

### The island

- **Now playing:** cover art, title, progress bar, and play, pause and skip controls for the player you use.
- **Notifications:** banners and a history you can look back at.
- **Control Center:** Wi-Fi, Bluetooth and VPN pages, volume and brightness, and a power menu (sleep, hibernate, restart, shut down, sign out).
- **Extras:** clock, weather, timer, and a shelf where you can drop files.
- **Layouts:** Compact or Wide, floating or attached to the screen edge.

### The dock

- Pinned and running apps with previews.
- Recycle Bin that accepts dropped files.
- Folder stacks: your Downloads and other folders fan out, with the newest files one click away.

### Everywhere

- Works on several monitors; each one can have its own island and dock.
- Light and dark themes, glass and material looks.
- English and Turkish.
- Small and quick: built with Tauri, Rust and React.

## Install

1. Download **`MacDows-Setup.exe`** from the [latest release](https://github.com/Fatal-IV/MacDows/releases/latest).
2. Run it and press **Install**. No administrator permission is needed; MacDows is installed for your user.
3. Press **Open MacDows**. The tray icon (in the hidden icons) opens Settings.

Good to know:

- The installer is not code-signed yet, so Windows SmartScreen may show an "unknown publisher" warning. Choose **More info → Run anyway**.
- MacDows needs the WebView2 runtime. Windows 11 has it. If it is missing, the installer downloads and installs it first (internet required).
- Uninstall from **Windows Settings → Apps**.

## Updates

MacDows checks for a new release a little after it starts and then every 12 hours. When there is one, a small blue dot appears on the island and **Settings → Updates** shows what is new.

- Nothing is installed until you press **Update and restart**.
- Every update is verified with the MacDows signing key before it is installed.
- You can turn the automatic check off in the same card.

## About this repository

This repository holds the releases and the update feed (`latest.json`). The source code is kept in a private repository, so there is nothing to build here. For what changed in each version, see the [changelog](CHANGELOG.md).

## License

MacDows is free to use, for personal and commercial purposes. It may not be modified or redistributed. See [LICENSE](LICENSE) for the full terms and [NOTICE](NOTICE) for third-party credits.

---

## Türkçe

**MacDows**, Windows için bir Dynamic Island ve dock'tur.

- **Ada:** çalan müzik (kapak, ilerleme çubuğu, kontroller), bildirimler ve geçmişi, Wi-Fi / Bluetooth / VPN sayfalı Denetim Merkezi, güç menüsü, saat, hava durumu, zamanlayıcı ve dosya rafı.
- **Dock:** sabit ve çalışan uygulamalar, dosya bırakılabilen Geri Dönüşüm Kutusu, klasör yığınları.
- Birden fazla monitörde çalışır, açık ve koyu tema, Türkçe ve İngilizce arayüz.

**Kurulum:** [son sürümden](https://github.com/Fatal-IV/MacDows/releases/latest) `MacDows-Setup.exe` dosyasını indirin, çalıştırıp **Install**'a basın (yönetici izni gerekmez). Kurulum imzalı olmadığı için SmartScreen uyarı verebilir: **Ek bilgi → Yine de çalıştır**.

**Güncelleme:** MacDows yeni sürümü kendisi bulur, adada mavi bir nokta çıkar. Siz **Güncelle ve yeniden başlat**'a basmadan hiçbir şey kurulmaz; güncellemeler imza anahtarıyla doğrulanır.

**Lisans:** Kişisel ve ticari olarak ücretsiz kullanılabilir; değiştirilemez ve yeniden dağıtılamaz. Ayrıntılar için [LICENSE](LICENSE) dosyasına bakın.
