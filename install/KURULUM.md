# Gayrimenkul360 - Kurulum ve Kullanım Kılavuzu

> Bu dosyanın Word (.docx) sürümü aynı klasörde `KURULUM.docx` olarak yer alır.

## 1. Genel Bakış

Gayrimenkul360, bir gayrimenkul ofisinin portföylerini, taşınmaz sahibi/müşteri kayıtlarını (CRM), çalışan görevlerini, hedeflerini ve mesailerini yöneten bir masaüstü uygulamasıdır. Rust ile yazılmıştır, Windows ve Linux'ta çalışır.

Sistem iki bileşenden oluşur:

1. **Sunucu** (`gayrimenkul-server`): Ofisteki bir bilgisayarda çalışır, tüm veriyi saklar.
2. **İstemci** (`gayrimenkul-client`): Her çalışanın bilgisayarında çalışır, sunucuya bağlanır.

Aynı ağdaki tüm bilgisayarlar aynı sunucuya bağlanınca, bir çalışanın girdiği kayıt diğerlerine anında düşer. Ticari abonelik gerekmez.

## 2. Sistem Gereksinimleri

- Sunucu: Windows 10/11 veya modern Linux, tercihen sabit yerel IP
- İstemci: Windows 10/11 veya Linux, sunucu ile aynı ağda

## 3. Kurulum Yöntemleri

### Yöntem A - Hazır dosyalar
Pakette derlenmiş ikili dosyalar varsa Rust gerekmez; `start-server` / `start-client` betiklerini çalıştırın. Windows için Windows'ta, Linux için Linux'ta derleme gerekir.

### Yöntem B - Kaynak koddan derleme
1. Rust kurun: https://rustup.rs
2. Windows'ta ek olarak "Visual Studio C++ Build Tools" (MSVC).
3. Linux'ta: `libx11-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev`
4. `cargo build --release`
5. Çıktılar: `target/release/gayrimenkul-server`, `target/release/gayrimenkul-client`

Windows: `build-release.bat`, Linux/macOS: `./build-release.sh`

### Windows derlemesi (adım adım)
Windows ikilileri yalnızca Windows'ta derlenir (Rust, derlendiği işletim sistemi için çalıştırılabilir dosya üretir). Adımlar:

1. **Rust kurulumu:** https://rustup.rs adresinden `rustup-init.exe` dosyasını indirip çalıştırın. Kurulum sırasında "Quick install" seçeneğini (varsayılan) kullanın. Kurulumdan sonra komut istemini kapatıp yeniden açın.
2. **C++ derleyici (MSVC):** rustup, Visual Studio C++ Build Tools yoksa uyarır. https://visualstudio.microsoft.com/visual-cpp-build-tools/ adresinden "Build Tools for Visual Studio" kurun ve "Desktop development with C++" iş yükünü seçin.
3. **Kontrol:** Komut isteminde `cargo --version` ve `rustc --version` çalıştırın; sürüm yazdırılıyorsa hazırsınız.
4. **Derleme:** Proje klasörüne girin ve `build-release.bat` dosyasına çift tıklayın (veya komut isteminde `cargo build --release`). İlk derleme internetten paketleri indirdiği için uzun sürebilir (5-15 dk).
5. **Çıktılar:** `target\release\gayrimenkul-server.exe` ve `target\release\gayrimenkul-client.exe`.

Not: Windows'ta derlenen ikililer Windows'ta, Linux'ta derlenenler Linux'ta çalışır. Her iki işletim sistemi için ayrı derleme alın.

## 4. Sunucunun Kurulması

1. Paketi sunucu bilgisayara kopyalayın.
2. `start-server.bat` (Windows) veya `./start-server.sh` (Linux) çalıştırın.
3. Ekranda görünen **Ağ adresini** not edin (ör. `http://192.168.1.25:8080`).
4. Sunucu penceresini açık tutun.

**Güvenlik duvarı:** 8080 portuna izin verin.
- Windows: Gelen kuralı, TCP 8080.
- Linux: `sudo ufw allow 8080/tcp`

## 5. İstemcenin Kurulması

1. Paketi her çalışan bilgisayara kopyalayın.
2. `start-client.bat` / `./start-client.sh` çalıştırın.
3. Giriş ekranında **Sunucu Adresi** alanına sunucunun ağ adresini yazın.
4. Kullanıcı adı ve parola ile giriş yapın.

## 5.1 Wi-Fi / Ağ Senkronizasyonu ve Otomatik Sunucu Bulma

Sistem, sunucu ile istemciler arasında TCP/IP kullanır. Bu nedenle **kablolu (Ethernet) ağda olduğu gibi Wi-Fi ağında da** çalışır. Aynı Wi-Fi ağına/ağ grubuna bağlı tüm bilgisayarlar aynı sunucuya bağlanır ve kayıtlar anında senkronize olur.

**Otomatik sunucu bulma:**
- Sunucu açıldığında UDP 8085 portu üzerinden kendini ağa duyurur.
- İstemci giriş ekranındaki **"🔍 Ağdaki Sunucuları Bul"** butonuna basınca ağdaki sunucular listelenir; listeden seçince adres otomatik dolar. IP adresi elle de girilebilir.
- Otomatik bulma çalışmıyorsa (bazı modemlerde yayın engellenir) sunucunun **Ağ adresini** elle yazın.

**Güvenlik duvarı:** Sunucuda iki porta izin verin:
- TCP 8080 (istemci bağlantısı)
- UDP 8085 (otomatik bulma) — yalnızca otomatik bulma kullanılacaksa gereklidir.

**Wi-Fi ipuçları:**
- Bilgisayarların aynı SSID/ağa bağlı olduğundan emin olun.
- "Misafir ağı" (Guest Network) istemcileri genelde birbirinden yalıtır; bu ağda sunucuya erişilemez. Ana ağı kullanın.
- Sunucu bilgisayara modem/router üzerinden **sabit (statik) IP** vermeniz önerilir; böylece adres değişmez.

**İnternet üzerinden (farklı ofis/uzaktan):** İstemci bir alan adı/URL de kabul eder. Router üzerinde 8080 portunu sunucuya yönlendirirseniz (port forwarding) veya bir tünel/reverse-proxy kullanırsanız uzaktaki bilgisayarlar da bağlanabilir. Bu durumda güvenlik için VPN veya HTTPS önerilir.

## 6. İlk Giriş

- Varsayılan yönetici: `admin` / `admin123`
- **İlk girişten sonra parolayı değiştirin.**
- Yönetici, Kullanıcılar bölümünden her çalışana hesap açar.
- Her çalışan Profilim'den ad-soyad, telefon, e-posta ve fotoğraf girer.
- Yönetici, Çıktı Ayarları'ndan belgede görünecek alanları seçer.

## 7. Özellikler

- **Portföyler:** Form ile kayıt; ilçe/mahalle ön tanımlı (Muğla ve ilçeleri); resim ekleme; ilk resim kapak.
- **Karşılama:** En yeniden eskiye; thumbnail/liste görünümü; filtre ve arama.
- **CRM (yalnızca yönetici):** Taşınmaz sahibi/müşteri kayıtları; danışmanlar göremez.
- **Görevler:** Kişiye/ekibe atama, tarih-saat, otomatik bildirim.
- **Hedefler:** Kişiye/ekibe hedef, otomatik ilerleme.
- **Mesai:** Giriş/çıkış ve süre hesabı.
- **Çıktı:** PDF/DOCX belge, WhatsApp paylaşımı, haritada açma.

## 8. Veri ve Yedekleme

Sunucunun yanındaki `data/` klasörü:
- `data/gayrimenkul.db` — veritabanı
- `data/uploads/` — resimler

Yedek için sunucuyu kapatıp `data/` klasörünü kopyalayın.

**Tek tıkla yedek:** Yönetici, **Çıktı Ayarları > Yedekleme > "Sunucu Yedeğini İndir (ZIP)"** butonuyla veritabanı ve tüm resimleri içeren bir ZIP'i indirip otomatik açabilir. Dosya kullanıcının ev klasöründeki `Gayrimenkul360-Cikti` altına kaydedilir.

## 9. Gelişmiş Ayarlar

| Değişken | Açıklama |
|---|---|
| `GAYRIMENKUL_PORT` | Sunucu portu (varsayılan 8080) |
| `GAYRIMENKUL_DATA` | Veri klasörü yolu |

## 10. Sorun Giderme

- **"Sunucuya bağlanılamadı"**: Sunucu açık mı, adres doğru mu, güvenlik duvarı 8080'e izin veriyor mu?
- **Diğer bilgisayardan bağlanamıyorum**: `127.0.0.1` yerine sunucunun `192.168.x.x` adresini kullanın.
- **Linux'ta pencere açılmıyor**: Grafik kütüphanelerini kurun.
- **Parola unutuldu**: Yönetici, Kullanıcılar bölümünden günceller.

## 11. Dosya Yapısı

- `crates/common` — modeller ve Muğla ilçe/mahalle verisi
- `crates/server` — sunucu (axum, SQLite)
- `crates/client` — masaüstü arayüz (egui/eframe)
- `install` — bu kılavuz
