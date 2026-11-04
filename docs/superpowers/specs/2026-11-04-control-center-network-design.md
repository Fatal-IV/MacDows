# Kontrol Merkezi: Wi-Fi, Bluetooth ve VPN sayfaları, tasarım

Tarih: 2026-11-04 · Durum: kullanıcı onayı bekliyor (yazılı belge)

## 1. Amaç

Ada'daki Hızlı Ayarlar ekranına ("Kontrol Merkezi") Windows'un kendi panelindeki ağ özelliklerinden gerekenler taşınır: başka bir Wi-Fi ağına bağlanma, yeni Bluetooth cihazı eşleştirme ve VPN. Dock'taki Hızlı Ayarlar düğmesi Windows'un panelini açmaya devam eder (kullanıcı kararı: o menü kalır).

Kullanıcı kararları (2026-11-04):
- **Wi-Fi:** sinyale göre ağ listesi, kayıtlı ağa bağlan, yeni ağ için şifre (Ada'nın içinde, yazarken Ada klavye odağını alır), açık ağ, kurumsal (802.1X) ağda Windows ayarlarını aç, bağlantıyı kes, **ağı unutma**, **gizli ağa elle bağlanma**.
- **Bluetooth:** "Cihaz ekle" Ada'nın içinde yakındaki yeni cihazları tarar ve eşleştirir (PIN/onay Ada'da); eşleşmiş cihazlarda bağlan/kes kalır, **eşleşmeyi kaldırma** eklenir.
- **VPN:** Windows'un yerleşik VPN bağlantıları listelenir, bağlanılır, kesilir. Üçüncü taraf uygulamaların VPN'leri (WireGuard, Nord...) Windows üzerinden yönetilemez; yalnızca "VPN açık" durumu görünür. Kullanıcı şu an VPN kullanmıyor; ilk denemede yalnızca liste ve boş durum görülebilir.
- Deneme: Wi-Fi'ya kullanıcının telefon hotspot'uyla bağlanılır (şifre deneme sırasında kullanıcıdan alınır); Bluetooth eşleştirme yakındaki bir telefon/kulaklıkla.

## 2. Arayüz

Hızlı Ayarlar ekranındaki kartlar bugünkü gibi kalır; **Wi-Fi** kartına Bluetooth'taki gibi `›` oku gelir, ayrıca bir **VPN** kartı (kapalı/açık/bağlanıyor, `›` oku) eklenir. Sayfalar ekranın içinde açılır, `‹` ile geri dönülür (BluetoothPage deseni).

### 2.1 Wi-Fi sayfası
- Üstte Wi-Fi anahtarı (kartla aynı), altında ağ listesi: ad, sinyal çubukları, kilit simgesi, **Bağlı** / **Kayıtlı** etiketi. Güçlü sinyal önce, bağlı ağ en üstte. Sayfa açıkken 5 saniyede bir taranır; tarama sürerken ince bir "taranıyor" göstergesi.
- Tıklama: **kayıtlı ağ** → bağlan. **Açık ağ** → bağlan. **Şifreli yeni ağ** (WPA2/WPA3-kişisel) → satır açılır, şifre kutusu + Bağlan; Enter bağlar, Esc vazgeçer; yanlış şifrede satırda hata ("Şifre yanlış"). **Kurumsal (802.1X) ağ** → satırda "Windows ayarlarını aç" (`ms-settings:network-wifi`).
- Bağlı ağın satırı: **Bağlantıyı kes**. Kayıtlı ağlarda satırın ayrıntı menüsünde **Ağı unut** (onaylı: "Bu ağ unutulsun mu?" Evet/İptal).
- Listenin sonunda **Gizli ağa bağlan**: SSID, güvenlik türü (Açık / WPA2 / WPA3), şifre alanlarıyla küçük form.
- Wi-Fi kapalıyken liste yerine "Wi-Fi kapalı" ve Aç düğmesi. Wi-Fi bağdaştırıcısı yoksa kart/sayfa "Kullanılamıyor".

### 2.2 Şifre ve klavye odağı
Ada pencereleri odak almaz. Şifre (ya da SSID) kutusuna tıklayınca Ada penceresi odak alır (`set_focus`), yazma süresince imleç Ada'dan ayrılsa da Ada kapanmaz. Odak başka pencereye geçince ya da Esc ile yazma biter ve Ada olağan davranışına döner. Şifre bellekte yalnızca bağlanma çağrısı boyunca tutulur, hiçbir dosyaya yazılmaz, günlüğe düşmez.

### 2.3 Bluetooth sayfası
- Mevcut liste (eşleşmiş cihazlar, bağlı olanlar önce, pil %) aynen kalır; her satıra **Eşleşmeyi kaldır** (onaylı).
- **Cihaz ekle** artık Windows ayarlarını açmaz; sayfa içinde "Yakındaki cihazlar" tarama listesine geçer: adı olan, henüz eşleşmemiş cihazlar, tarama sürerken canlı eklenir/kalkar, üstte tarama göstergesi.
- Cihaza tıklama → eşleştirme. Windows'un istediği doğrulama türüne göre: **onay** ("Eşleşsin mi?" Evet/Hayır), **PIN eşleştirme** (Ada kodu gösterir, "Cihazdaki kodla aynı mı?" Evet/Hayır), **PIN girişi** (kutu + Eşleştir, klavye odağı 2.2'deki gibi), **yalnızca eşleş** (sormadan). Başarıda cihaz eşleşmiş listeye geçer; başarısızlıkta satırda neden yazar ("Eşleşme reddedildi", "Zaman aşımı").

### 2.4 VPN sayfası
- Windows'un VPN bağlantıları (Ayarlar > Ağ > VPN): ad, durum (Bağlı / Bağlı değil / Bağlanıyor), **Bağlan** / **Bağlantıyı kes**. Kimlik bilgisi gerekirse ya da bağlanma başarısızsa satırda Windows'un hata metni; "Windows ayarlarını aç" bağlantısı (`ms-settings:network-vpn`).
- Hiç Windows VPN'i yoksa: "VPN bağlantısı yok" + "Windows ayarlarını aç". Bir üçüncü taraf VPN bağdaştırıcısı etkinse (ağ bağdaştırıcı türü tünel/VPN) kart "VPN açık (uygulama)" der, düğme yoktur.
- VPN kartı Hızlı Ayarlar'da yalnızca VPN desteği ayarı açıksa görünür (3. bölüm).

## 3. Ayarlar

`island.screens.quick` altına: `wifi_page`, `vpn` (kart ve sayfa), `bluetooth_pairing` (cihaz ekle tarama/eşleştirme). Hepsi varsayılan açık; kapalıyken eski davranış (kartlar açma/kapama, "Cihaz ekle" Windows ayarlarını açar). Ayarlar sayfasında Hızlı Ayarlar bölümüne üç anahtar.

## 4. Teknik tasarım

- **Wi-Fi (`wifi.rs`):** Win32 `wlanapi` (`WlanOpenHandle`, `WlanEnumInterfaces`, `WlanScan`, `WlanGetAvailableNetworkList`, `WlanConnect`, `WlanDisconnect`, `WlanSetProfile`, `WlanDeleteProfile`, `WlanQueryInterface`). Kayıtlı profil bilgisi ve sinyal kalitesi listeden gelir. Yeni ağ için WPA2-Personal / WPA3-SAE / açık ağ profil XML'i üretilir (saf fonksiyon, test edilir), `WlanSetProfile` ile kaydedilip `WlanConnect` çağrılır; bağlantı sonucu `WLAN_NOTIFICATION` ile (ya da kısa yoklamayla) izlenir. 802.1X ve bilinmeyen kimlik doğrulama türlerinde yalnızca "kurumsal" işareti konur.
- **Komutlar:** `wifi_state`, `wifi_scan`, `wifi_networks`, `wifi_connect(ssid, password?)`, `wifi_connect_hidden(ssid, security, password?)`, `wifi_disconnect`, `wifi_forget(ssid)`.
- **Bluetooth (`bluetooth.rs`):** `DeviceWatcher` (`BluetoothDevice.GetDeviceSelectorFromPairingState(false)`) ile tarama; `DeviceInformation.Pairing.Custom.PairAsync` + `PairingRequested` olayı (`ConfirmOnly`, `ConfirmPinMatch`, `ProvidePin`) ile eşleştirme; `UnpairAsync` ile kaldırma. Komutlar: `bt_scan_start`, `bt_scan_stop`, `bt_pair(id)`, `bt_pairing_answer(accept, pin?)`, `bt_unpair(id)`; olaylar `bt-found`, `bt-lost`, `bt-pairing-request`, `bt-paired`.
- **VPN (`vpn.rs`):** RAS API (`RasEnumEntriesW`, `RasEnumConnectionsW`, `RasDialW`, `RasHangUpW`, `RasGetConnectStatusW`) Windows VPN profilleri için; üçüncü taraf tünel bağdaştırıcıları ağ bağdaştırıcı listesinden (`GetAdaptersAddresses`, tünel/VPN türü ve bağlı durum) "VPN açık" olarak bulunur. Komutlar: `vpn_list`, `vpn_connect(name)`, `vpn_disconnect(name)`.
- **Odak:** `island_focus(on: bool)` komutu notch penceresinde `set_focus`/odak bırakmayı yapar; ön yüz yazma sürerken `onHoverEnd`'i bastırır.
- **Ön yüz:** `QuickSettings.tsx`'e sayfa deseninin üçüncü ve dördüncü örneği (`WifiPage`, `VpnPage`; `BluetoothPage` genişler); ağ listeleri `bridge.ts` tipleriyle. Saf yardımcılar (sinyal çubuğu, sıralama, hata metni eşleme, profil XML'i) ayrı dosyalarda test edilir.

## 5. Aşamalar

1. **Wi-Fi arka planı** (`wifi.rs`: liste, bağlan/kes/unut, profil XML'i + testler) ve gerçek makinede liste denemesi (bağlanma yok).
2. **Wi-Fi sayfası** (liste, kayıtlı/açık ağa bağlan, bağlantıyı kes, unut onayı, Wi-Fi kapalı hâli).
3. **Şifre girişi ve odak** (`island_focus`, şifreli ağ, yanlış şifre, gizli ağ formu); hotspot ile gerçek deneme.
4. **VPN** (`vpn.rs`, kart ve sayfa, boş durum).
5. **Bluetooth tarama ve eşleştirme** (cihaz ekle listesi, doğrulama türleri).
6. **Eşleşmeyi kaldırma, ayar anahtarları, metinler (TR/EN), CHANGELOG.**

Her aşama ayrı commit; her aşama sonrası gerçek denemeyle doğrulanır. Bağlanma/kesme denemeleri yalnızca kullanıcının belirttiği ağ ve cihazla yapılır; kullanıcının mevcut bağlantısı sorulmadan kesilmez.

## 6. Kapsam dışı

VPN profili oluşturma/düzenleme (Windows ayarlarında yapılır), üçüncü taraf VPN uygulamalarını açıp kapatma, ağ özelliklerini (IP, DNS, ölçülü bağlantı) düzenleme, Wi-Fi Direct, Bluetooth cihaz yeniden adlandırma, Wi-Fi şifresini göster.

## 7. Riskler

- **Bağlantı kopması:** Kayıtlı olmayan ya da yanlış şifreli bir ağa bağlanma denemesi mevcut bağlantıyı kesebilir; arayüz bunu bağlanmadan önce göstermez ama hata durumunda önceki ağa dönmeyi dener (`WlanConnect` başarısızsa kayıtlı önceki profili yeniden bağlar).
- **WPA3 ve profil XML'i:** Bazı bağdaştırıcılar WPA3-SAE'yi desteklemez; liste "WPA3" ağlarını bağdaştırıcı destekliyorsa bağlanılır gösterir, desteklemiyorsa "Bu bağdaştırıcı desteklemiyor".
- **Bluetooth eşleştirme:** Özel eşleştirme olayları UI iş parçacığı beklentisi taşıyabilir; eşleştirme ayrı iş parçacığında yürür, cevap beklerken zaman aşımı (30 sn) vardır.
- **Odak:** Ada odak alırken Dock/görünürlük mantığı (`visibility.rs`) Ada'yı gizlememeli; yazma sürerken Ada "tutulur" (`hold_notch`).
- **VPN:** RAS yalnızca klasik Windows VPN profillerini kapsar; kimlik bilgisi isteyen profillerde Windows'un kendi iletişim kutusuna yönlendirilir.
