# Durum widget'ları (Nectar'daki gibi), tasarım

Tarih: 2026-11-04 · Durum: kullanıcı onayı bekliyor (yazılı belge)

## 1. Amaç

Ada'nın durum pill'ine (saat) Nectar'daki gibi küçük **durum widget'ları** eklenir: hava, pil, CPU, RAM, disk, ağ. Ada'nın üzerine gelince pill genişler, soldaki ve sağdaki widget'lar saatın iki yanında görünür. Hangi widget'ın hangi tarafta olduğu ayarlarda **sürükle-bırak** ile seçilir (Nectar'daki `nectar-status-widgets` düzeni: `{"left":[...],"right":[...]}`).

Kullanıcı kararları (2026-11-04):
- Altı widget'ın hepsi: hava, pil, CPU, RAM, disk, ağ.
- Üzerine gelince, saatın iki yanında (kapalıyken yalnızca saat; şimdiki pil yüzdesi davranışı gibi).
- Yerleşim sürükle-bırak ile (Nectar gibi). Seçici satırları değil.

Nectar incelemesi (kaynak: `kua8/Nectar` SETTINGS.md ve sürüm notları, GPL-3.0; bu proje de GPL-3.0): varsayılan `{"left":["weather"],"right":["battery"]}`; geçerli değerler `weather`, `battery`, `cpu`, `ram`, `disk`, `net`; pil, pili olmayan masaüstünde gizlenir; hava durumu şehir boşsa IP'den konum bulur (bunu yapmıyoruz, bkz. §4); sıcaklık birimi `celsius`/`fahrenheit`. Nectar'ın CPU/RAM/disk/ağ gösterim biçimi belgelenmemiş; biçimi biz seçiyoruz (§3). Nectar'ın kodu kopyalanmaz, davranış yeniden yazılır; ikonlar MacDows'un kendi çizimleri (SF Symbols kopyalanmaz, bkz. `docs/HANDOFF.md`).

Kapsam dışı: widget'a tıklayınca ayrıntı açmak, widget'lar için kendi renkleri, Denetim Merkezi düzenini sürükleyerek özelleştirmek (Nectar 1.7.0'daki), IP ile konum bulma.

## 2. Yerleşim ve görünüm

- Pill'in solunda `left`, sağında `right` listesi sırasıyla görünür; saat ortada kalır. Bir tarafta **en fazla 3** widget (Nectar belgelemiyor; pill'in ekrana sığması için bizim sınırımız).
- Widget = küçük ikon + değer (örn. bulut ikonu + `13°`, işlemci ikonu + `24%`). Metin 12 px, ada'nın mevcut yazı stiliyle.
- Pill genişliği `DynamicIsland.tsx` içindeki mevcut genişlik zincirine (hovered durumu) eklenir: `taban + 54 px × widget sayısı` (hız gösteren ağ widget'ı 84 px). Yay animasyonu aynı kalır.
- Hiç widget yoksa (ya da hepsi gizliyse, örn. pilsiz PC'de yalnızca pil seçili) pill bugünkü gibi davranır.
- Pil widget'ı pilsiz makinede gizlenir ve yerleşimden düşer. Eski "üzerine gelince pil yüzdesi" satırı bu widget'a taşınır; `status.battery_percent` anahtarı ile birleştirilir (§5).
- Düşük pil: eşik altındaki pil widget'ı kırmızı tonda görünür (eşik varsayılan %20, 5-50 arası 5'lik adım; Nectar'daki gibi). Uyarı titreşimi yok.

## 3. Veri

Hepsi Rust'ta tek komutla, ön yüz yalnızca ada üzerine gelindiğinde ve en az bir istatistik widget'ı yerleşikken her **2 saniyede** sorar. Kapalıyken hiç örnekleme yok.

`get_system_stats() -> SystemStats { cpu: u8, ram: u8, disk: u8, net_down: u64, net_up: u64 }`
- **CPU:** `GetSystemTimes` iki örnek arası fark, 0-100 yüzde. İlk çağrıda önceki örnek yoksa 0.
- **RAM:** `GlobalMemoryStatusEx` `dwMemoryLoad`.
- **Disk:** sistem sürücüsünün (genelde C:) doluluk yüzdesi, `GetDiskFreeSpaceExW`.
- **Ağ:** tüm yukarıdaki (up) ve yazılımsal olmayan arayüzlerin bayt sayaçlarının farkı / geçen süre; `GetIfTable2` (`Win32_NetworkManagement_IpHelper` zaten ekli). Gösterim: `↓ 1,2 MB/s`; 1 KB/s altı `0`.
- Önceki örnek `static Mutex` içinde tutulur.
- **Pil:** mevcut `get_system_status` (yüzde, şarjda mı).
- **Hava:** Open-Meteo (anahtar gerektirmez): önce şehir adı → koordinat (`geocoding-api.open-meteo.com/v1/search`), sonra `api.open-meteo.com/v1/forecast?current=temperature_2m,weather_code`. İstek **ön yüzden `fetch` ile** yapılır (Rust'ta HTTP kütüphanesi yok, `csp` kapalı; yeni bağımlılık eklenmez). Sonuç 30 dakika önbellekte (`localStorage`); ağ yoksa son değer, o da yoksa widget gizli. Hava kodu → ikon eşlemesi saf fonksiyon (`weatherIcon(code)`: açık, parçalı bulutlu, bulutlu, sis, yağmur, kar, fırtına). Sıcaklık °C veya °F.

## 4. Gizlilik

- Hava widget'ı seçili olduğunda **şehir adı ve koordinatı** Open-Meteo'ya gider. Başka hiçbir veri gönderilmez. Şehir boşsa istek yapılmaz ve widget görünmez (IP ile konum bulma yok).
- Varsayılan düzende `weather` sol tarafta olsa da şehir yazılana kadar hiçbir şey internete gitmez.
- Bu davranış ayarlarda tek cümleyle açıklanır.

## 5. Ayarlar modeli

`island.screens.status` genişler (Rust `StatusCfg`, TS `StatusCfg`, `islandDefaults.ts`):
- `widgets: { left: string[], right: string[] }`, varsayılan `{ left: ["weather"], right: ["battery"] }`.
- `weather_city: string` (boş), `weather_lat: f64`, `weather_lon: f64` (şehir seçilince yazılır), `temp_unit: "celsius" | "fahrenheit"`, `low_battery: u8` (20).
- Normalleştirme (`island_settings::normalize`): bilinmeyen kimlikler atılır, aynı widget iki yerde olamaz (ilk görülen kalır), taraf başına en fazla 3, eski ayardan gelenler korunur.
- Geçiş: eski `battery_percent: true` ise ve `widgets` yoksa pil sağda yerleşik gelir; `false` ise pil yerleşimden çıkarılır. `battery_percent` anahtarı kaldırılır (okuma geriye uyumlu).

## 6. Ayarlar arayüzü

Ayarlar > Dynamic Island > Durum bölümü:
- Pill önizlemesi: `[sol alan] 12:34 [sağ alan]`. Alanlar bırakma bölgeleri; altında **Kullanılabilir** tepsisi, henüz yerleşmemiş widget'ları tutar.
- Her widget bir "chip" (ikon + ad). Chip'ler alanlar arasında ve alan içinde sürüklenir; bırakılan yer sırayı belirler. Taraf doluysa (3) bırakma reddedilir ve alan kısa süre kırmızı yanar.
- **Sürükleme işaretçi olaylarıyla yapılır** (`pointerdown/move/up` + `setPointerCapture`), HTML5 sürükle-bırak ile değil: Tauri pencere dosya sürükleme yakalaması (`dragDropEnabled`) tarayıcı içi sürüklemeyi bozabilir. Chip'ler klavyeyle de taşınır: odaktayken `Alt+←/→` alan değiştirir, `Alt+↑/↓` sırayı değiştirir.
- Pilsiz PC'de pil chip'i tepside soluk görünür ve açıklama alır ("Bu bilgisayarda pil yok").
- Altında: **Şehir** kutusu (yazınca Open-Meteo aramasından açılır liste, seçince koordinat kaydedilir), **Birim** seçici (°C/°F), **Düşük pil eşiği** kaydırıcısı, **Sıfırla** düğmesi (varsayılan düzen).

## 7. Modüller

- `src-tauri/src/system_stats.rs` (yeni): `get_system_stats`, saf yardımcılar (`cpu_percent(prev, now)`, `format_rate(bytes_per_s)` yalnızca TS tarafında; Rust'ta fark hesapları).
- `src-tauri/src/island_settings.rs`, `src/bridge.ts`, `src/notch/islandDefaults.ts`: `StatusCfg` genişlemesi ve normalleştirme.
- `src/notch/StatusWidgets.tsx` (yeni): widget satırı, ikonlar, veri çekme (`useStats`, `useWeather`).
- `src/notch/widgetLogic.ts` (+ test, yeni): yerleşimi normalleştirme, genişlik hesabı, değer biçimleme (`formatRate`, `formatTemp`), `weatherIcon`, düşük pil rengi.
- `src/notch/DynamicIsland.tsx`: pill'e `StatusWidgets`, genişlik zinciri.
- `src/settings/StatusWidgetsEditor.tsx` (yeni): sürükle-bırak düzenleyici; `src/settings/IslandSettings.tsx` içine eklenir.
- Dil dosyaları (`widget.*`), `CHANGELOG.md`.

## 8. Test

- Rust: `normalize` (bilinmeyen/çift/aşırı widget, eski ayar geçişi), CPU yüzde farkı, ağ sayaç farkı (taşma ve sıfır süre dahil).
- TS (`node --test`): `formatRate`, `formatTemp` (°C/°F, yuvarlama), `weatherIcon` tablosu, yerleşim kuralları (taraf sınırı, çift, pilsiz makine), genişlik, düşük pil.
- Gerçek PC: her widget'ın Windows Görev Yöneticisi ile uyumlu değer gösterdiği ve hava için bir şehirle (kullanıcının şehri: Ankara) doğru sıcaklığın geldiği ekran görüntüsüyle; sürükle-bırak ayarlar penceresinde elle denenir (pointer olayları uygulama içinde yalnızca gerçek pencerede doğrulanabilir).

## 9. Riskler

- Pill'in genişlemesi çok sayıda widget'ta ekranın üstünde yer kaplar: taraf başına 3 sınırı.
- Open-Meteo erişilemezse hava widget'ı gizlenir; hata ayar sayfasında şehir kutusunun altında görünür.
- Disk ve ağ sayaçları sürücü/arayüz değişince sıçrayabilir: negatif fark 0 sayılır.
