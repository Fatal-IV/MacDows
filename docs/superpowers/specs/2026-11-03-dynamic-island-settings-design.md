# Dinamik Ada ayarları: tasarım

Tarih: 2026-11-03 · Durum: kullanıcı onayı bekliyor (yazılı belge)

## 1. Amaç

Ayarlar'da "Dinamik Ada" adlı ayrı bir kategori. Ada'nın tüm ekranları ve her ekranın özellikleri tek yerden açılıp kapatılabilir. Ada'yla ilgili dağınık ayarlar (görünürlük) bu kategoriye taşınır.

Kullanıcı kararları (2026-11-03):
- Tüm menüler (ekranlar) yer alır; hepsi ve alt özellikleri açılıp kapatılabilir.
- **İkinci saat eklenmeyecek.** **Takvim senkronizasyonu eklenmeyecek.** Hava durumu da kapsam dışı.
- **Görünürlük (Sabit / Akıllı / Göz Atma) Genel'den bu kategoriye taşınır.** Dock'un görünürlüğü Çubuklar'da kalır.
- **Monitör seçimi şu an yok** (Ada birincil ekranda kalır).
- Varsayım (itiraz gelmedi): en az bir ekran açık kalır.

## 2. Ayar modeli

`settings.json` içinde yeni `island` nesnesi. Rust tarafı sahibidir; `set_setting` yolu gibi tek bir komutla değişir ve ayar değişince Ada'ya `settings-changed` olayı gider, böylece Ada yeniden başlamadan güncellenir. Eksik anahtarlar varsayılanla tamamlanır (eski dosyalar bozulmaz). Mevcut `notch_visibility` değeri `island.visibility` altına taşınır; eski anahtar bir kez okunup aktarılır.

### 2.1 Ekranlar

| Anahtar | Ekran | Alt ayarlar (hepsi aç/kapa, aksi belirtilmedikçe varsayılan açık) |
|---|---|---|
| `screens.status` | Durum | saat biçimi (24 saat/12 saat, varsayılan 24), pil yüzdesi |
| `screens.music` | Müzik (pill ve oynatıcı) | görselleştirici, albüm ışığı, ilerleme çubuğu, yeni parçada kendiliğinden açılma, albüm kapağı |
| `screens.tools` | Takvim ve Zamanlayıcı | takvim, hafta başlangıcı (Pzt/Paz, varsayılan Pzt), zamanlayıcı (ses, ilerleme çubuğu), kronometre |
| `screens.quick` | Hızlı Ayarlar | kutular: Wi-Fi, Bluetooth, Rahatsız Etmeyin; anahtarlar: Pil tasarrufu, Uçak modu, Mobil etkin nokta, Erişilebilirlik; çubuklar: Parlaklık, Ses; düğmeler: Kilit, MacDows Ayarları, Ayarlar |
| `screens.notifications` | Bildirimler | banner (süre 3–10 sn, varsayılan 5), geçmişi kaydet, saklanacak sayı (25/50/100, varsayılan 100), uygulama bazlı sessize alma listesi |
| `screens.shelf` | Raf | saklama (oturum boyunca / kalıcı, varsayılan oturum), en fazla öğe (varsayılan 40), bırakınca otomatik aç |

### 2.2 Gezinme

- `nav.wheel`: tekerlekle ekran geçişi (açık).
- `nav.order`: ekran sırası (liste, "Sıfırla" ile varsayılana döner).
- `nav.click`: tıklama davranışı: `smart` (medya çalıyorsa oynatıcı, yoksa Hızlı Ayarlar; varsayılan) | `quick` | `player`.
- `nav.middle`: orta tuş Takvim ve Zamanlayıcı'yı açar (açık).
- `nav.remember`: imleç çıkıp dönünce son ekrana dön (kapalı).

### 2.3 Görünürlük ve yerleşim

- `visibility`: `fixed` | `smart` | `peek` (şu anki değer korunur).
- `fullscreen`: tam ekranda davranış (mevcut mantık, ayar olarak görünür).
- `scale`: boyut ölçeği (mevcut `nectar-scale` ayarı buraya bağlanır).

## 3. Davranış kuralları

1. **En az bir ekran açık.** Son açık ekranın anahtarı kapatılamaz; satır soluk görünür ve nedenini yazar.
2. **Kapalı ekran yok sayılır:** tekerlek geçişinde, tıklama hedefinde ve diğer ekranlardaki bağlantılarda (ör. banner'a tıklayınca geçmiş) görünmez. Banner'a tıklama bildirim ekranı kapalıysa banner'ı yalnızca kapatır.
3. **Raf kapalıyken** Ada dosya bırakmaya tepki vermez.
4. **Bildirim banner'ı kapalıyken** geçmiş açıksa bildirimler yine kaydedilir. **Geçmiş kapatılırsa** kayıtlı geçmiş silinsin mi diye sorulur; evet denirse dosya silinir.
5. **Hızlı Ayarlar'daki öğeler** kapalıysa satır/kutu yerleşimden çıkar; boş kalan alan diğer öğelerle doldurulur. Hepsi kapalı bir bölüm (ör. tüm kutular) gizlenir.
6. **Tıklama:** `nav.click` hedefi kapalı bir ekransa sıradaki açık ekran açılır; hiç açık ekran yoksa tıklama bir şey yapmaz (kural 1 buna izin vermez).
7. **Ekran sırası:** Durum ve Müzik pill durumlarıdır; sıra yalnızca açık ekranlar arasında uygulanır.
8. Her bölümün altında "Varsayılana sıfırla".

## 4. Arayüz

Ayarlar radyal menüsüne "Dinamik Ada" kategorisi eklenir. Sayfa bölümleri: Görünürlük ve yerleşim · Ekranlar (altı açılır bölüm, her birinde anahtar ve alt ayarlar) · Gezinme. Mevcut `Row` ve anahtar bileşenleri kullanılır; sıralama listesi yeni küçük bir bileşendir.

## 5. Aşamalar

1. **Altyapı:** `island` ayar modeli, aktarım, `settings-changed` olayı, kategori sayfası iskeleti, görünürlüğün taşınması. (Test: eksik anahtar tamamlama, eski ayardan aktarım, tek-ekran kuralı.)
2. **Ekran anahtarları ve gezinme:** altı ekranın aç/kapa, tekerlek, tıklama, orta tuş, sıra, hatırlama.
3. **Alt özellik anahtarları:** Müzik, Takvim ve Zamanlayıcı, Hızlı Ayarlar, Bildirimler, Raf.
4. **Cila:** sıfırlama düğmeleri, kapatma onayları, boş durumlar, metinler (TR/EN).

Her aşama ayrı commit'lerle ilerler.

## 6. Kapsam dışı

İkinci saat, takvim senkronizasyonu, hava durumu, monitör seçimi (çoklu monitör), Nectar'daki Denetim Merkezi düzeni özelleştirmesi.

## 7. Riskler

- Ada'nın tüm ekranları şu an `DynamicIsland.tsx` içinde tek dosyada durumla yönetiliyor. Anahtarları eklemek dosyayı büyütür; ekran listesi tek bir kaynaktan (ekran tanımları) türetilerek durum bayrakları sadeleştirilecek.
- Ayarlar ayrı bir pencerede, Ada ise başka bir pencerede çalışıyor. Canlı yenileme için Tauri olayı kullanılacak; olay kaçarsa Ada bir sonraki açılışta ayarı okur.
