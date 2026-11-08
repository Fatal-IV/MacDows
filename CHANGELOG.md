# Changelog

All notable changes to MacDows are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/), versioning: [SemVer](https://semver.org/).

## [Unreleased]

## [0.3.0] - 2026-11-08

### Added
- Battery notices on the island, shown like the Wi-Fi and Bluetooth ones: the charger was plugged in, the charger was pulled out, the battery is full and the battery is low (the limit is the low battery setting of the status widgets). A battery held at its charge limit by battery protection (80%, for example) counts as full. A notice waits a moment so that a screen that blanks when the charger is pulled out does not hide it.
- When the update server does not answer (for example an error status from GitHub) the check says so ("the update server is not answering right now") instead of blaming your connection.

### Changed
- Settings has one Appearance page with a switch at the top for the Dynamic Island and the Dock. The theme they share (color mode and style) sits above the settings of the one you chose. The Dynamic Island page is no longer called Appearance, and "Size" is now "Island size".
- "Start with Windows" now uses a task that runs when you sign in. Windows holds back programs started from the usual startup entry for a few seconds, which made the dock and the island appear late; the task is not held back. The old startup entry is removed.
- The surface styles are Solid and Material. The glass styles (Frosted and Liquid glass) and the opacity setting are gone; each style has its own fixed opacity.
- The color mode is Follow Windows, Light or Dark. The "Windows accent color" mode and the accent setting are gone (highlights are monochrome), and the Start menu is always painted in the theme color.
- The light theme contrast setting is shown only when the theme is light (the light mode, or following Windows while Windows is light).
- An update check that gets no answer ends after 20 seconds instead of waiting; a download has no such limit.

### Fixed
- Joining a Wi-Fi network with a typed password always failed with "wrong password" when the network name has a letter outside plain English (for example the Turkish "ı" in a phone hotspot named "... adlı kişiye ait ..."). The profile now carries the exact bytes of the network name, so Windows looks for the right network.

### Türkçe
### Eklendi
- Adada, Wi-Fi ve Bluetooth bildirimleri gibi gösterilen pil bildirimleri: şarj takıldı, şarj çıkarıldı, pil doldu ve pil azaldı (sınır, durum widget'larındaki düşük pil ayarıdır). Pil korumasının şarjı tuttuğu sınırda (örneğin %80) bekleyen pil dolu sayılır. Bildirim, şarj çıkarılınca kararan bir ekran yüzünden kaçmasın diye kısa bir süre bekler.
- Güncelleme sunucusu yanıt vermediğinde (örneğin GitHub'dan bir hata durumu geldiğinde) denetim, bağlantını suçlamak yerine "güncelleme sunucusu şu an yanıt vermiyor" der.

### Değişti
- Ayarlar'da tek bir Görünüm sayfası var; üstündeki anahtarla Dinamik Ada ya da Dock seçilir. İkisinin ortak teması (renk modu ve stil) seçtiğin ayarların üstünde durur. Dinamik Ada sayfasının adı artık Görünüm değil, "Boyut" ayarı da "Ada boyutu".
- "Windows ile başlat" artık oturum açınca çalışan bir görev kullanır. Windows, olağan başlangıç girdisinden başlayan programları birkaç saniye bekletir; dock ve ada bu yüzden geç görünüyordu. Görev bekletilmez. Eski başlangıç girdisi silinir.
- Yüzey stilleri Solid ve Material'dir. Cam stilleri (Buzlu ve Sıvı cam) ile saydamlık ayarı kalktı; her stilin kendi sabit saydamlığı var.
- Renk modu Windows'u izle, Açık ya da Koyu'dur. "Windows vurgu rengi" modu ve vurgu rengi ayarı kalktı (vurgular tek renklidir), Başlat menüsü her zaman tema renginde boyanır.
- Açık tema kontrastı ayarı yalnızca tema açıkken gösterilir (açık mod ya da Windows açıkken Windows'u izleme).
- Yanıt alamayan bir güncelleme denetimi beklemek yerine 20 saniye sonra biter; indirmede böyle bir sınır yoktur.

### Düzeltildi
- Yazılan parolayla Wi-Fi ağına katılmak, ağ adında düz İngilizce dışında bir harf varsa (örneğin "... adlı kişiye ait ..." adlı bir telefon hotspot'undaki Türkçe "ı") her zaman "yanlış parola" ile başarısız oluyordu. Profil artık ağ adının tam baytlarını taşıyor, böylece Windows doğru ağı arıyor.

## [0.2.8] - 2026-11-08

### Added
- Two island types: **Notch** (stuck to the top edge, as before) and **Dynamic Island**, a pill that floats a little below the top edge with round corners all over. Chosen in Settings > Appearance on two tiles that show a small monitor.
- Multi-monitor support: every monitor you switch on in Settings > Appearance > Monitors gets its own island and its own dock (only the primary monitor has them by default). Each dock opens its previews, the hidden-icons panel, the app catalog and the right-click menu on its own monitor; Win+number goes to the dock on the monitor under the pointer; a fullscreen app only affects the bars of its own monitor; the Windows taskbar of a monitor with a dock is hidden too and comes back when MacDows exits or crashes.
- The gradient behind the media widget has eight effects (four directions, two diagonals, an aurora and a breathing light), chosen on animated tiles in Settings (or switched off). It shows on the small island too, only on the media widget, and stands still when Windows is set to reduce motion.
- While a video plays (Netflix, YouTube, a film) the robot sits in an armchair and watches: eating popcorn, or a screen with a play sign and a line that follows the video (chosen in Settings). Music keeps the head shake.
- Sound bars next to the clock on the small island while music plays.
- The island visibility (Fixed, Smart, Peek) is chosen on three tiles with a small animated monitor each.
- A "Light theme contrast" setting (none, outline and shadow, gray fill) for the island on white windows.
- The Dynamic Island and Appearance pages are one page called Appearance; every card of it (Layout, Monitors, Theme, Screens, Navigation) starts folded and opens on a click.
- The dock visibility is chosen on three tiles with a small animated monitor, and the Dock settings page is split into cards that open and close.
- The gradient behind the media widget has an on/off switch that remembers the last effect.

### Changed
- With paused media, hovering the island widens it and shows its widgets instead of opening the player (a click still opens it).
- The dock shows its Start and Search buttons from the first moment of the takeover, and the Windows taskbar is hidden before the slow auto-hide change.

### Fixed
- The Peek dock no longer swallows clicks meant for the program underneath (Spotify's player buttons), and it is hidden again when it should be.
- The hidden-icons panel opens again on Windows builds that hide the tray from UI Automation, opens faster (the icons are read when the pointer reaches the button), and keeps the docks in view while it is open.
- The dock shows the real icon of Windows Settings and other Store apps; a second click on the icon of the window in front minimizes it.
- The random greeting waves with the raised arm again; the head shake belongs to the music robot only.
- The island and the dock are placed at once while MacDows starts instead of wandering through default places for seconds.
- The island is round at the top too while the player and the other panels are open in the Dynamic Island type.
- The Start and Search buttons of the dock show up right after MacDows starts instead of up to 20 seconds later.
- While music plays, the visualizer sits closer to the clock and the island is narrower; the eyes and the cover keep their room.
- A connect or disconnect notice for something you just did in the Control Center brings you back to the same page afterwards, and clicking a notice opens that page.
- Disconnecting a Bluetooth device no longer says "could not connect" when Windows is slow to report the new state.

### Türkçe
### Eklendi
- İki ada tipi: **Notch** (eskisi gibi üst kenara yapışık) ve **Dinamik Ada**, üst kenarın biraz altında yüzen, her yanı yuvarlak bir hap. Ayarlar > Görünüm'de küçük bir monitör gösteren iki kutucukla seçilir.
- Çoklu monitör desteği: Ayarlar > Görünüm > Monitörler'de açtığınız her monitörün kendi adası ve kendi dock'u olur (varsayılan olarak yalnızca ana monitörde vardır). Her dock; önizlemeleri, gizli simgeler panelini, uygulama kataloğunu ve sağ tık menüsünü kendi monitöründe açar; Win+sayı imlecin olduğu monitördeki dock'a gider; tam ekran bir uygulama yalnızca kendi monitörünün ada ve dock'unu etkiler; dock'u olan monitörün Windows görev çubuğu da gizlenir ve MacDows kapanınca ya da çökünce geri gelir.
- Medya widget'ının arkasındaki gradyanın sekiz efekti var (dört yön, iki çapraz, aurora ve nefes alan ışık); Ayarlar'da hareketli kutucuklarla seçilir (ya da kapatılır). Küçük adada da görünür, yalnızca medya widget'ında çıkar ve Windows'ta hareket azaltma açıksa durur.
- Bir video oynarken (Netflix, YouTube, film) robot koltuğa oturup izler: mısır yer ya da ekranında oynat işareti ve videoyu izleyen bir çizgi olur (Ayarlar'dan seçilir). Müzikte kafa sallama aynen kalır.
- Müzik çalarken küçük adada saatin yanında ses çubukları görünür.
- Ada görünürlüğü (Sabit, Akıllı, Peek) her biri küçük, hareketli bir monitör gösteren üç kutucukla seçilir.
- Beyaz pencerelerin önünde adanın belli olması için "Açık tema kontrastı" ayarı (yok, kenarlık ve gölge, gri dolgu).
- Dinamik Ada ve Görünüm sayfaları "Görünüm" adıyla tek sayfa oldu; sayfadaki tüm kutular (Yerleşim, Monitörler, Tema, Ekranlar, Gezinme) kapalı gelir ve tıklayınca açılır.
- Dock görünürlüğü küçük, hareketli bir monitör gösteren üç kutucukla seçilir; Dock ayarlar sayfası açılıp kapanan kutulara bölündü.
- Medya widget'ının arkasındaki gradyanın, son seçilen efekti hatırlayan bir aç/kapat anahtarı var.

### Değişti
- Medya duraklatılmışken adanın üstüne gelince oynatıcı açılmaz, ada yana genişleyip widget'ları gösterir (tıklayınca yine açılır).
- Dock, Başlat ve Arama düğmelerini devralmanın ilk anından gösterir; Windows görev çubuğu yavaş otomatik gizle değişikliğinden önce gizlenir.

### Düzeltildi
- Peek dock artık altındaki programın tıklamalarını yutmuyor (Spotify'ın oynatıcı düğmeleri) ve gizlenmesi gerektiğinde gizleniyor.
- Gizli simgeler paneli, tepsiyi UI Automation'dan saklayan Windows sürümlerinde yeniden açılıyor, daha hızlı açılıyor (simgeler imleç düğmeye gelince okunuyor) ve açıkken dock'ları görünür tutuyor.
- Dock, Windows Ayarlar ve diğer Store uygulamalarının gerçek simgesini gösteriyor; öndeki pencerenin simgesine ikinci tıklama pencereyi küçültüyor.
- Rastgele selamlamada robot yine kaldırdığı koluyla el sallıyor; kafa sallama yalnızca müzik robotuna ait.
- Ada ve dock, MacDows açılırken saniyelerce varsayılan yerlerde dolaşmak yerine hemen yerine oturuyor.
- Dinamik Ada tipinde oynatıcı ve diğer paneller açıkken ada üstten de yuvarlak.
- Dock'taki Başlat ve Arama düğmeleri MacDows açıldıktan sonra 20 saniyeye kadar beklemeden hemen görünüyor.
- Müzik çalarken ses çubuğu saate daha yakın duruyor ve ada daha dar; göz ile kapak yerlerini koruyor.
- Kontrol Merkezi'nde yaptığınız bir bağlanma veya kesme için gelen bildirimden sonra aynı sayfaya dönülüyor; bildirime tıklamak da o sayfayı açıyor.
- Windows yeni durumu geç bildirdiğinde bir Bluetooth cihazının bağlantısını kesmek artık "bağlanamadı" demiyor.

## [0.2.7] - 2026-11-06

### Added
- When you take a photo with the Windows Camera app, the island shows an animated camera icon for a moment (the shutter closes and a flash goes off). MacDows only notices that a new picture was saved in your Camera Roll folder; it never looks at the picture.

### Türkçe
### Eklendi
- Windows Kamera uygulamasıyla fotoğraf çektiğinizde ada kısa süreliğine animasyonlu bir kamera simgesi gösterir (deklanşör kapanır ve flaş patlar). MacDows yalnızca Camera Roll klasörünüze yeni bir resim kaydedildiğini fark eder; resme hiç bakmaz.

## [0.2.6] - 2026-11-06

### Fixed
- The island keeps its normal width while it greets you.

### Türkçe
### Düzeltildi
- Ada sizi selamlarken normal genişliğinde kalıyor.

## [0.2.5] - 2026-11-06

### Changed
- MacDows now greets you. Don't worry, it isn't watching you.

### Türkçe
### Değişti
- MacDows artık sizi selamlıyor, endişelenmeyin sizi izlemiyor.

## [0.2.4] - 2026-11-06

### Fixed
- A white box no longer shows up around and under the island (and around the dock). It came from cutting their windows down to their shape. Now the island and the dock ignore the mouse while the pointer is outside what is drawn, so clicks still reach the programs next to them, and nothing is painted around them.

### Türkçe
### Düzeltildi
- Adanın çevresinde, altında (ve dock'un çevresinde) beyaz bir kutu görünmesi giderildi. Pencerelerin çizilen şekle kırpılmasından geliyordu. Artık imleç çizilenin dışındayken ada ve dock fareyi yok sayıyor; yanlarındaki programlar tıklamaları almaya devam ediyor ve çevrelerine hiçbir şey çizilmiyor.

## [0.2.3] - 2026-11-06

### Added
- After an update, MacDows opens Settings by itself and shows a "What's new" window with the notes of the new version, in the app's language (English or Turkish). It appears once per update, never after a fresh install.
- MacDows says hi when it starts: look closely at the island, it has a little character now. Nothing to worry about.

### Changed
- Updates no longer show the plain Windows installer window. MacDows downloads its own installer (its signature is checked first) and the same branded window used for installing shows "Updating…", then opens MacDows again by itself. (Versions up to 0.2.2 still need one click on "Update" in that window.)

### Türkçe
### Eklendi
- Güncellemeden sonra MacDows kendiliğinden Ayarlar'ı açar ve yeni sürümün notlarını uygulamanın dilinde (İngilizce ya da Türkçe) bir "Yenilikler" penceresinde gösterir. Her güncellemede bir kez çıkar, yeni kurulumdan sonra çıkmaz.
- MacDows açılırken selam veriyor: adaya dikkatli bakın, artık küçük bir karakteri var. Endişelenecek bir şey yok.

### Değişti
- Güncellemeler artık sade Windows kurucu penceresini göstermez. MacDows kendi kurucusunu indirir (önce imzası doğrulanır) ve kurulumdaki markalı pencere "Güncelleniyor…" yazar, sonra MacDows'u kendiliğinden yeniden açar. (0.2.2 ve öncesi sürümlerde bu pencerede bir kez "Güncelle"ye basmak gerekir.)

## [0.2.2] - 2026-11-06

### Added
- Update notes follow the app's language: release notes can carry a Turkish part under a "### Türkçe" heading, and Settings > Updates shows the English or the Turkish part to match the language setting (notes without that heading are shown whole).
- A friendlier island: on the very first start after an install it drops in with a bounce and the dock rises from the bottom, and the island greets you every time MacDows starts, also after the computer has booted.

### Changed
- The installer says "Updating…" (and "Update", "MacDows is updated.") instead of "Installing…" when it replaces an older version.
- The island and the dock no longer block clicks around them: their transparent windows (580x340 and 900x120) are cut down to the shape that is actually drawn, so programs underneath (browser tabs, title bars) keep working right next to a small island. In Peek and Smart mode the pointer only counts as near the island inside that shape.

### Türkçe
### Eklendi
- Güncelleme notları uygulamanın diline göre gösterilir: sürüm notlarında "### Türkçe" başlığı altında Türkçe bir bölüm olabilir, Ayarlar > Güncellemeler dil ayarına göre İngilizce ya da Türkçe bölümü gösterir (bu başlığı olmayan notlar olduğu gibi gösterilir).
- Daha cana yakın bir ada: kurulumdan sonraki ilk açılışta ada zıplayarak iner, dock alttan yükselir; ayrıca MacDows her açıldığında, bilgisayar yeniden başladıktan sonra da, ada sizi selamlar.

### Değişti
- Kurucu, eski bir sürümün yerine geçerken "Kuruluyor…" yerine "Güncelleniyor…" (ve "Güncelleyin", "MacDows güncellendi.") yazar.
- Ada ve dock artık çevrelerindeki tıklamaları engellemez: şeffaf pencereleri (580x340 ve 900x120) gerçekten çizilen şekle kırpılır, böylece küçük adanın hemen yanındaki programlar (tarayıcı sekmeleri, başlık çubukları) çalışmaya devam eder. Peek ve Akıllı modda imleç, adaya yalnızca bu şeklin içindeyken yakın sayılır.

## [0.2.1] - 2026-11-06

### Changed
- Visibility "Fixed" now also stays on screen over fullscreen apps (for the island unless "In fullscreen apps: Never show" is chosen, and for the dock). Smart and Peek still wait for the pointer at the screen edge there.

### Added
- A small new detail on the island's right side makes it feel more alive; it gives way to the status widgets while the pointer is over the island.

### Türkçe
### Değişti
- "Sabit" görünürlük artık tam ekran uygulamaların üstünde de ekranda kalır (ada için, "Tam ekran uygulamalarda: Hiç göster" seçili değilse; dock için de). Akıllı ve Peek modları orada yine imleci ekran kenarında bekler.

### Eklendi
- Adanın sağ tarafındaki küçük yeni bir detay onu daha canlı gösteriyor; imleç adanın üzerine gelince durum widget'larına yer açar.

## [0.2.0] - 2026-11-05


### Added
- Control Center, Wi-Fi: the arrow on the Wi-Fi tile opens a list of networks by signal strength. Connect to saved and open networks, join new ones with a password typed right in the island (also hidden networks), disconnect, and forget a network. A wrong password says so and brings the previous network back. Company (802.1X) networks open Windows settings. Settings > Dynamic Island > Quick settings > "Wi-Fi page".
- Control Center, VPN: a VPN card shows Windows VPN connections (switch one on or off with a click, or open the list for several) and tells when a VPN app (WireGuard, OpenVPN and the like) has a tunnel up. Only stored sign-ins are used.
- Control Center, Bluetooth: "Add a device" scans for nearby devices and pairs them inside the island (a plain yes, a code to compare, or a PIN to type), and paired devices can be removed. When the setting is off, "Add a device" opens Windows settings as before.
- While a text field in the island has the keyboard, the island stays open even if the pointer leaves it, and closes when typing ends.
- Status widgets on the island: weather, battery, CPU, memory, disk and network speed sit on either side of the clock while the pointer is over the island (not while the player is open). Settings > Dynamic Island > Status has a pill preview where widgets are dragged between the left area, the right area and the "Available" tray (up to 3 per side; Alt+Left/Right moves a focused widget between areas, Alt+Up/Down reorders). The numbers are only sampled while the island is hovered and a CPU, memory, disk or network widget is placed, every 2 seconds. Weather comes from Open-Meteo for the city chosen in settings (temperature in Celsius or Fahrenheit); only the city name and its coordinates are sent, and with no city nothing is sent at all. The battery widget turns red below a chosen percentage (default 20%) unless the charger is connected, and hides itself on computers without a battery. The old "battery percentage" setting becomes the battery widget (switched off there, it stays off).

### Changed
- Settings window: the round button that fanned the categories out is gone; the categories (General, Dynamic Island, Dock, Appearance, Updates, About) are a fixed list on the left.
- Settings > Dynamic Island: each screen's card (Status, Music, Quick settings) starts folded and opens with a click on its header (a chevron shows which ones have more settings); the on/off switch does not open or close it.
- The dock's right-click menu for a running app now shows the window's own Windows menu (Restore, Move, Size, Minimize, Maximize, Close) under the dock's items, and "Quit app" is gone in favor of Windows's Close.
- Pinning and unpinning in the dock no longer touch the Windows taskbar: "Keep in Dock" adds a pin only MacDows keeps (or shows a hidden one again), "Remove from Dock" hides a Windows pin in the dock only. The dock still reads the taskbar's pins to start with.

### Fixed
- Wi-Fi: networks that only accept WPA3 can be joined with a typed password (the list now carries each network's security type). A slow network is reported as "No answer from the network" instead of "Wrong password": a wrong password is now recognized when Windows tries and gives up. A failed attempt puts the network's previously saved profile back instead of leaving the failed one, and the wait no longer rescans the whole network list every half second.
- VPN: certificate (IKEv2) and EAP profiles, which have no stored password, can connect from the island. The adapter list is read from properly aligned memory, and Windows's IP-HTTPS adapter no longer makes the tile read as "VPN on".
- Bluetooth pairing: "rejected" is shown only when the device or you declined (canceled or rejected by handler); timeouts and other failures say "failed". A paired device that gets its name during scanning no longer shows up in the list of devices to pair.
- Dock menu on a running app: Move, Size, Minimize and the other window commands bring the window forward first, so they work on background windows.
- Typing in an island text field no longer dismisses a notification banner that is still showing when the field closes.
- Shutting down Windows while MacDows runs no longer shows two error windows (taskkill failing with 0xc0000142) whose dismissal cancelled the shutdown: MacDows skips restarting Start and Search while Windows is shutting down or logging off.
- Notifications cleared from the island's history stay cleared: they used to come back at the next refresh or restart because Windows still listed them.

### Known limitations
- The new Wi-Fi, VPN and Bluetooth pages were tried on one PC only; WPA3-only networks, certificate and EAP VPN profiles and several Bluetooth pairing kinds have unit tests but were not tried against real hardware.
- Primary monitor only.
- The hidden-icons button of the dock may not open its panel on some systems (it needs a Windows Explorer restart).
- The installer is not signed with a code-signing certificate, so Windows SmartScreen may show a warning.

## [0.1.0] - 2026-11-04

First public release. A Dynamic Island at the top of the screen and a dock at the bottom that take the place of the Windows taskbar (it comes back whenever you quit MacDows or turn the switch off). Windows's own Start, Search and quick settings stay.

### Added
- The dock shows the apps pinned to the Windows taskbar (programs, File Explorer and Store apps) in Windows's order, and follows Windows when something is pinned or unpinned. A running pinned app keeps its place and gets a dot; a click brings it forward or starts it. Settings > Dock > "Show pinned apps".
- Right-click menu in the dock: new window, unpin from the taskbar, pin to the taskbar, change or reset the icon, quit the app, and (on the empty dock) add app, MacDows settings and Task Manager. Windows often refuses to pin for other programs; then MacDows keeps the pin itself.
- Drag the pinned apps to reorder them (only the order in the dock changes, not the Windows taskbar), and give any dock app a custom icon from a picture or a program.
- Window previews: hover a running app to see its windows as small live pictures; click one to bring it forward or close it. A badge shows the window count from two windows. Can be turned off in settings.
- Drives (local disks and USB drives) in the dock; a click opens them in File Explorer. Can be turned off in settings.
- Win+1 to Win+9 start or focus the pinned apps in dock order, Win+Shift+number opens a new window. Windows does not let programs register these keys, so MacDows uses a keyboard hook that only looks at Win and the digits; an empty slot is left to Windows. Can be turned off in settings.
- "Add app" in the dock's right-click menu: a searchable list of installed programs and Store apps; a click pins one.
- The Bluetooth tile of the quick settings shows the name of the connected device ("Name +1" for more).
- A dock button that jumps to the desktop (Win+D). Off by default; Settings > Dock.
- The app follows the light or dark theme of Windows by default, and can be set to light or dark.
- The installer asks whether MacDows should start with Windows (a fresh install only; off by default).
- Windows's own Start and Search are used, as they are on a normal PC (the Start and Search buttons of the dock open them, and so do the Windows key and Win+S). While one of Windows's menus (Start, Search, quick settings) is open, the dock stays in view under it.
- Settings > Appearance > "Paint the Windows Start menu in the theme color". Off by default: MacDows then only makes Windows's system theme light or dark like its own. On, Start also takes the theme color; Windows only allows that through the system accent color, so every app's accent color is the theme color while MacDows runs. Your own colors come back when MacDows closes, whichever way it ends.
- The music bars in the notch follow the real audio the PC is playing (five frequency bands, analysed locally; nothing is recorded or sent anywhere).
- The notch player shows the elapsed and remaining time with a bar you can click to seek, and a system volume slider (the mouse wheel over it works too).
- **Dynamic Island settings**: a page of its own in Settings. Every screen (Status, Music, Calendar and timer, Quick settings, Notifications, Shelf) can be switched off, but at least one stays on. Choose what the wheel, a click and the middle button do, the order of the screens, whether the island returns to the last screen, how it behaves in fullscreen apps (edge or never) and its size. Old settings are carried over.
- **Hover opens the player**: while something is playing (or paused) the island opens the player with its controls as soon as the pointer rests on it. Can be turned off.
- **Media source buttons**: when two or more programs have media (Spotify, a browser tab, a video player) the player shows a button with the icon and name of each; the paused ones fade. Click one to see and control it.
- **Settings are split into pages**: General, Dynamic Island, Dock (including "Replace the Windows taskbar"), Appearance, Updates and About.
- **Dock** at the bottom center, above the taskbar: one icon per open app window, click to bring it to the front, icons grow when you hover them, a dot marks each running app. The dock has a black, edge-attached shape with curved sides.
- **Stable dock order**: icons stay where they first appeared, new apps are added at the end, closed apps disappear.
- **Store app icons**: apps installed from the Microsoft Store, such as WhatsApp, now show their real icon instead of a blank one.
- **Fullscreen detection**: the notch and dock hide automatically while a fullscreen app is in front (a fullscreen video or a game) and come back when you leave fullscreen. Works event-based, without polling.
- **Visibility modes** for the notch and the dock, set separately: *Fixed* (default, always visible except in fullscreen), *Smart* (hides when a window covers it, shows when you point at it) and *Peek* (appears when the cursor reaches the screen edge). Both bars are shown for a few seconds after launch so you can see the app started.
- **Single instance**: starting MacDows again does not open a second copy; it opens the Settings window of the running one.
- **Languages**: English and Turkish. Missing translations fall back to English.
- **Settings file** at `%APPDATA%\MacDows\settings.json`. A broken or empty file never stops the app: it starts with defaults and keeps the broken file as `settings.json.bad`.
- **Signed releases**: every release is built by GitHub Actions and the update packages are signed with the MacDows key, which is what the in-app update verifies.
- **MacDows logo**: a violet tile with a notch (equalizer bars) on top, a gradient "M" in the middle and a dock with three dots at the bottom. It replaces the placeholder icons in the tray, the window and the installer. Small versions (24 and 16 px) are simplified so they stay readable.
- **Branded installer** `MacDows-Setup.exe`: one window with a single Install button, a progress bar and an Open button, in English or Turkish. It installs MacDows for the current user (no administrator prompt) and installs WebView2 first if the PC does not have it (needs internet).
- Every release also carries `MacDows-Setup.exe` and a `SHA256SUMS.txt` to verify the download.
- **Updates inside the app**: MacDows looks for a new release 30 seconds after it starts and then every 12 hours, quietly. When one exists a small blue dot appears on the notch and Settings gets an **Updates** card with the new version, what is new, and an **Update and restart** button. Nothing is installed until you press it; the update is verified with the MacDows signing key and an update that fails the check is refused. You can switch the automatic check off or press **Check now** at any time.

### Changed
- The notch is now called the Dynamic Island in the app, the installer and the settings.
- "Start with Windows" is off by default. Matching the color of Windows's Start and Search to the theme is on by default.
- The Settings window has its own title bar, a left menu with categories and its own controls.
- The notch now follows Nectar's design: one damped spring for width, height and corners, concave corners that join it to the screen edge, a slide-in at startup, a status pill and a music pill you switch with the mouse wheel or a click on the clock, a music pill that opens itself when playback starts, a full player with artwork, a scrolling title and an album-colored glow.
- **License changed from MIT to GPL-3.0**, because the notch is adapted from Nectar (GPL-3.0). See `NOTICE`.
- Turkish interface text now addresses you formally ("siz").

### Fixed
- The battery of a Bluetooth headset could show an old value: the same headset has several device entries in Windows and the stale one could win. The freshest reading is used now.
- The browser's own right-click menu ("Save image as", "Inspect") no longer shows on the dock and the Dynamic Island.
- In a fullscreen app the notch and dock no longer disappear completely; they appear when the pointer reaches the screen edge.
- Settings could open behind other windows and be unreachable while the taskbar was replaced; it now comes to the front.

### Known limitations
- Primary monitor only.
- The Windows notification banner still appears next to the island's own.
- The hidden-icons button of the dock may not open its panel on some systems (it needs a Windows Explorer restart).
- Wi-Fi network switching, Bluetooth pairing and VPN inside the island are planned; for now the quick settings only toggle them, and "Add device" opens Windows settings.
- Not every part of the island has its own switch yet (clock format, music extras, banner time and so on are planned).
- The installer is not signed with a code-signing certificate, so Windows SmartScreen may show a warning.
