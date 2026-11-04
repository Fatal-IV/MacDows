# Dock özellikleri: tasarım

Tarih: 2026-11-04 · Durum: kullanıcı onayı bekliyor (yazılı belge)

## 1. Amaç

Dock, Nectar'ın Dock'undaki temel davranışları kazanır: sağ tık menüsü, sabitleme, pencere önizlemesi, sürükleyerek sıralama, sürücüler, Win+sayı kısayolları, özel simge, "Uygulama ekle". Dock'un sabit simgeleri Windows görev çubuğundan zaten canlı okunuyor (tamamlandı, `pinned_apps.rs`); bu belge onun üstüne ekleneni anlatır.

Kullanıcı kararları (2026-11-04):
- Nectar'dan alınacaklar: pencere önizlemesi, sabitleri sürükleyip sıralama, sürücüler, Win+sayı, özel simge, aynı uygulamanın birden çok penceresi, sabit olmayanların açıkken görünüp kapanınca silinmesi, **Uygulama ekle**.
- **Takvim simgesi alınmaz** (Ada'daki Takvim ve Zamanlayıcı yeterli).
- Sürücüler: yerel ve çıkarılabilir hepsi, ayardan kapatılabilir.
- Win+sayı: önce `RegisterHotKey`; olmazsa Nectar'daki gibi düşük seviyeli klavye kancası.
- Önizleme: çalışan simgenin üzerine gelince, her pencere için küçük resim ve kapatma düğmesi; tek pencerede de gösterilir; ayardan kapatılabilir.
- Windows'tan sabit kaldırma çalışıyor (HP Support Assistant ile denendi: `taskbarunpin` komutu kabul edildi).

## 2. Nectar'dan fark (bilerek)

Nectar Windows sabitlerini bir kez içe aktarır, sonra kendi listesini tutar. MacDows Windows'u kaynak olarak tutmaya devam eder: Windows'ta sabitlenen/kaldırılan Dock'a yansır. MacDows yalnızca **sırayı** ve **özel simgeleri** üstüne ekler; Windows'un kendi sırasına ve listesine sürükleme dokunmaz.

## 3. Model

`settings.json`'a (Rust sahibi):

| Anahtar | Anlam | Varsayılan |
|---|---|---|
| `dock.pinned_order` | Sabit kimliklerinin MacDows sırası. Listede olmayan sabitler sona eklenir, kalkanlar sessizce atılır. | `[]` |
| `dock.local_pins` | Windows'un sabitlemeyi kabul etmediği, yalnızca MacDows'ta sabit uygulamalar (`{ id, name, exe, aumid }`). | `[]` |
| `dock.custom_icons` | Kimlik → MacDows'a kopyalanmış simge dosyası adı. | `{}` |
| `dock.show_drives` | Sürücüleri göster. | açık |
| `dock.previews` | Üzerine gelince pencere önizlemesi. | açık |
| `dock.win_number` | Win+1..9 kısayolları. | açık |

Simge dosyaları `%APPDATA%\MacDows\icons\` altında tutulur; seçilen resim 128×128 PNG'ye küçültülüp kopyalanır (özgün dosya silinse de simge kalır).

## 4. Özellikler

### 4.1 Sağ tık menüsü
Yerel (Tauri) menü, dilli. Sabit simge: **Yeni pencere · Görev çubuğundan kaldır · Simgeyi değiştir · Simgeyi sıfırla (özel simge varsa) · Uygulamadan çık (çalışıyorsa)**. Sabit olmayan çalışan simge: **Yeni pencere · Görev çubuğuna sabitle · Uygulamadan çık**. Boş Dock alanı: **Uygulama ekle · MacDows Ayarları · Görev Yöneticisi**. Tarayıcının kendi menüsü hiçbir yerde çıkmaz (düzeltildi).

- *Yeni pencere:* uygulamayı yeni örnek olarak başlatır (sabit için kısayol, değilse exe yolu).
- *Uygulamadan çık:* uygulamanın tüm pencerelerine `WM_CLOSE` gönderir (zorla öldürmez).
- *Sabitle:* önce Windows'un `taskbarpin` komutu; kabul edilmezse `dock.local_pins`'e eklenir ve kullanıcıya bildirilir. *Kaldır:* önce `taskbarunpin`; olmazsa mevcut yedek yol (yerel gizleme / yerel sabiti silme).

### 4.2 Pencere önizlemesi ve birden çok pencere
Çalışan bir simgenin üzerine gelince (~300 ms gecikmeyle) simgenin üstünde ayrı, saydam bir önizleme penceresi açılır: uygulamanın her penceresi için başlık, küçük resim ve kapatma düğmesi. Resme tıklayınca o pencere öne gelir (küçültülmüşse geri yüklenir). Pencere sayısı 2 veya daha fazlaysa simgenin köşesinde sayı rozeti görünür. İmleç hem simgeden hem önizlemeden ayrılınca kapanır. Küçük resimler `PrintWindow` (`PW_RENDERFULLCONTENT`) ile alınır, önizleme açıkken saniyede bir yenilenir, kapalıyken hiç iş yapılmaz. Küçük resim alınamazsa uygulama simgesi gösterilir. Simgeye tıklama davranışı değişmez (çalışıyorsa ilk pencereyi öne getirir, değilse başlatır).

### 4.3 Sürükleyerek sıralama
Yalnızca sabit simgeler sürüklenir; yeni yer `dock.pinned_order`'a yazılır. Sabit olmayanlar açılış sırasıyla kalır. Sürükleme sırasında diğer simgeler yer açar (animasyonlu); Esc veya Dock dışına bırakma iptal eder.

### 4.4 Sürücüler
Sabitlerden sonra, sabit olmayanlardan önce, ayraçla ayrılmış sürücü simgeleri (harf + yerel/çıkarılabilir simgesi). Tıklayınca Gezgin'de açılır. Çıkarılabilirler takılınca eklenir, çıkarılınca kalkar (`WM_DEVICECHANGE`); kapasite/ad ipucunda. `dock.show_drives` kapalıysa hiçbiri görünmez.

### 4.5 Win+1..9
Dock'taki sıraya göre: birinci sabit Win+1. Çalışıyorsa öne getirir (birden çok pencerede sırayla geçer), değilse başlatır. Win+Shift+sayı: yeni pencere. Kayıt `RegisterHotKey` ile denenir; Windows `Win+sayı`yı kendine ayırdığı için başarısız olursa Nectar'daki gibi `WH_KEYBOARD_LL` kancasına geçilir ve bu durum bir kez kullanıcıya bildirilir. Kanca yalnızca bu tuşları yakalar ve yutar, başka hiçbir tuşa dokunmaz. Sabit sayısı 9'dan azsa kullanılmayan tuşlar Windows'a bırakılır. `dock.win_number` kapalıysa kayıt/kanca hiç kurulmaz.

### 4.6 Özel simge
*Simgeyi değiştir:* dosya seçici (.png/.jpg/.ico/.exe); resim 128×128 PNG'ye çevrilir, `dock.custom_icons`'a yazılır, Dock anında yenilenir. *Sıfırla:* kayıt ve dosya silinir. Özel simge hem sabit hem sabit olmayan simgede geçerlidir (kimlik: sabit için kimliği, değilse exe yolu).

### 4.7 Uygulama ekle
Boş alan menüsünden açılan, Dock'un üstünde ayrı panel: üstte arama kutusu; "Çalışanlar" ve "Tüm uygulamalar" bölümleri (Başlat menüsü kısayolları ve `shell:AppsFolder` ile Mağaza uygulamaları). Bir uygulamaya tıklamak onu sabitler (4.1'deki Sabitle yolu). Zaten sabit olanlar listede gri ve işaretli görünür.

### 4.8 Ayarlar
Çubuklar sayfasına: Sürücüleri göster, Pencere önizlemesi, Win+sayı kısayolları anahtarları. Bunlar sonra "Dock" kategorisine (Dock ayarları işi) taşınır.

## 5. Davranış kuralları

1. Sıralama listesi bayatlayabilir: olmayan kimlikler atlanır, yeniler sona eklenir (örn. Windows'ta yeni sabitleme).
2. Özel simge dosyası yoksa/bozuksa özgün simge gösterilir.
3. Önizleme penceresi Dock'un görünürlük mantığını (`visibility.rs`) bozmaz: Dock gizlenirken önizleme de kapanır.
4. Tam ekran uygulama varken Win+sayı çalışmaya devam eder, Dock ise yalnızca kenarda belirir (mevcut mantık).
5. Win+sayı kancası hiçbir tuş olayını beklemez/bekletmez (kancada iş yapılmaz, yalnızca olay kuyruğa atılır).
6. Yeni özelliklerin hiçbiri Dock'un kapalıyken (`replace_taskbar` kapalı, Dock gizli) arka planda iş yapmaz.

## 6. Aşamalar

1. **Menü ve sabitleme:** menü genişletme, Yeni pencere, Uygulamadan çık, Sabitle/Kaldır (yedek yollarla), `dock.local_pins`.
2. **Önizleme:** önizleme penceresi, küçük resim alma, rozet, kapatma, ayar anahtarı.
3. **Sıralama ve özel simge:** `dock.pinned_order`, sürükleme; simge seçme/sıfırlama.
4. **Sürücüler.**
5. **Win+sayı:** `RegisterHotKey`, yedek kanca, bildirim.
6. **Uygulama ekle:** panel, liste, arama.

Her aşama ayrı commit; her aşamadan sonra gerçek Dock'ta (ekran görüntüsü + tıklama) deneme.

## 7. Kapsam dışı

Takvim simgesi, ikinci monitör, uygulama gruplama kuralları (tarayıcı konakçısı ayrımı), Dock'un adaptif modu, pencere önizlemesinde sürükleyerek yeniden dizme.

## 8. Riskler

- **`taskbarpin`/`taskbarunpin`:** Windows sürümleri arasında davranış değişebilir. Kaldırma bu PC'de çalıştı; sabitleme denenmedi. Yedek yollar (yerel sabit/gizleme) bu yüzden var.
- **Win+sayı:** `RegisterHotKey` büyük olasılıkla Windows tuşu birleşimleri için reddedilir; yedek kanca antivirüs uyarısı çıkarabilir (bu yüzden ayarlı ve yalnızca gerekirse).
- **Küçük resim:** `PrintWindow` GPU ile çizen bazı pencerelerde boş/siyah dönebilir; o zaman simge gösterilir.
- **Ayrı önizleme/panel pencereleri:** `tray_panel.rs`'deki desenle (merkez X, Dock üstü) yazılır; odak çalmamalı ve Dock gizliyken kapanmalı.
- **Boyut:** `Dock.tsx` büyüyor; sürükleme, sürücü ve önizleme mantığı ayrı dosyalara (`dockDrag.ts`, `DockPreview.tsx`, `drives.rs`, `hotkeys.rs`) konur.
