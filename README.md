# Gayrimenkul360

Ofis portföy, CRM, görev, hedef ve mesai yönetim sistemi. Rust ile yazılmıştır; Windows ve Linux üzerinde çalışır.

Ayrıntılı kurulum adımları için: [`install/KURULUM.docx`](install/KURULUM.docx) (Word) veya [`install/KURULUM.md`](install/KURULUM.md).

## Mimari

Sistem istemci-sunucu mimarisi kullanır:

- **Sunucu** (`gayrimenkul-server`): Ofisteki bir bilgisayarda çalışır. Veritabanı (SQLite) ve resimleri saklar.
- **İstemci** (`gayrimenkul-client`): Her çalışanın bilgisayarında çalışan masaüstü arayüz (egui/eframe).

Aynı yerel ağdaki (LAN) tüm bilgisayarlar aynı sunucuya bağlanır. Bir çalışanın eklediği portföy/kayıt anında diğerlerine düşer. Ticari abonelik gerekmez; tamamen ofis içi çalışır.

## Gereksinimler

- Rust (kaynak koddan derlemek için): https://rustup.rs
- Windows'ta MSVC C++ Build Tools
- Linux'ta grafik kütüphaneleri: `libx11-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev`

## Derleme

```bash
cargo build --release
# Windows: build-release.bat
# Linux/macOS: ./build-release.sh
```

Oluşan dosyalar:
- `target/release/gayrimenkul-server`
- `target/release/gayrimenkul-client`

### Windows ikililerini üretme (adım adım)

Rust, ikili dosyaları derlendiği işletim sistemi için üretir; Windows `.exe` dosyaları Windows'ta derlenmelidir.

1. https://rustup.rs adresinden `rustup-init.exe` indirip çalıştırın (varsayılan "Quick install").
2. https://visualstudio.microsoft.com/visual-cpp-build-tools/ adresinden **Build Tools for Visual Studio** kurun; "Desktop development with C++" iş yükünü seçin.
3. Yeni bir komut isteminde `cargo --version` ile kurulumu doğrulayın.
4. Proje klasöründe `build-release.bat` dosyasına çift tıklayın (veya `cargo build --release`).
5. Çıktılar: `target\release\gayrimenkul-server.exe` ve `target\release\gayrimenkul-client.exe`.

İlk derleme paketleri indirdiği için 5-15 dakika sürebilir.

## Ağ (Wi-Fi / LAN) ve Senkronizasyon

- Sunucu ve istemciler TCP/IP kullanır; kablolu ve **Wi-Fi** ağda çalışır.
- Sunucu, UDP 8085 üzerinden kendini duyurur; istemci giriş ekranındaki **"Ağdaki Sunucuları Bul"** ile otomatik bulunur.
- Güvenlik duvarında **TCP 8080** ve (otomatik bulma için) **UDP 8085** açılmalıdır.
- Karşılama ekranı 15 saniyede bir otomatik yenilenir; diğer çalışanların eklediği portföyler görünür.

## Çalıştırma

**Sunucu (ofis ana bilgisayarı):**
```bash
./start-server.sh        # Linux/macOS
start-server.bat         # Windows
```
Ekranda `http://<AĞ-IP>:8080` adresi görünür. Bu adresi çalışanlara verin.

**İstemci (çalışan bilgisayarı):**
```bash
./start-client.sh        # Linux/macOS
start-client.bat         # Windows
```
Giriş ekranındaki "Sunucu Adresi" alanına sunucunun ağ adresini yazın (ör. `http://192.168.1.25:8080`).

## Varsayılan Giriş

- Kullanıcı adı: `admin`
- Parola: `admin123`

> İlk girişten sonra Profilim bölümünden parolayı değiştirin.

## Özellikler

- Kullanıcı girişi ve rol yönetimi (yönetici / danışman)
- Portföy formu: konum, ilçe, mahalle (Muğla'nın tüm ilçeleri ve mahalleleri ön tanımlı), resim ekleme
- Karşılama ekranında portföyler en yeniden en eskiye; thumbnail veya liste görünümü (liste başında küçük resim)
- İlçe/mahalle kategorizasyonu ve filtreler
- Taşınmaz sahibi bilgileri otomatik CRM kaydı olarak; CRM listesi **yalnızca yönetici** görür
- Görev/not atama, tarih-saat hatırlatma ve bildirimler
- Kişiye/ekibe özel hedefler ve otomatik ilerleme takibi
- Mesai giriş/çıkış kaydı ve süre hesabı
- Profil: fotoğraf, ad soyad, telefon, e-posta
- PDF ve DOCX tanıtım belgesi üretimi (DOCX kapak resimli), WhatsApp ile paylaşım
- Yönetici çıktı ayarları: belgede hangi bilgiler görünsün
- **Wi-Fi ve kablolu ağ desteği + sunucu otomatik bulma** (UDP broadcast)
- Bağlantı durumu göstergesi ve karşılama ekranında otomatik senkronizasyon
- Tek tıkla **sunucu yedeği** (veritabanı + resimler, ZIP)
- **Ayarlar Paneli (sekmeli):** Görünüm, Çıktı, Doğrulama, Kullanıcılar
- **Tema desteği:** Karanlık / Aydınlık / Sistem (yerel olarak kalıcı)
- **Dinamik ölçeklendirme:** Ctrl +/−/0 kısayolları ve ayarlardan %60–%250 UI zoom
- **Form doğrulama kuralları:** yönetici tanımlar, sunucuda saklanır, istemci+sunucu uygular

## Proje Yapısı

```
gayrimenkul360/
├── Cargo.toml              # workspace
├── crates/
│   ├── common/             # paylaşılan modeller + Muğla verisi
│   ├── server/             # axum + SQLite sunucu
│   └── client/             # egui masaüstü arayüzü
├── install/                # kurulum kılavuzu (docx/md)
├── start-server.*  start-client.*  build-release.*
```

## Veri ve Yedekleme

Veriler sunucunun çalıştığı klasördeki `data/` altında tutulur:
- `data/gayrimenkul.db` — veritabanı
- `data/uploads/` — resimler

Yedek için sunucuyu kapatıp `data/` klasörünü kopyalayın.

## Ortam Değişkenleri

| Değişken | Açıklama |
|---|---|
| `GAYRIMENKUL_PORT` | Sunucu portu (varsayılan 8080) |
| `GAYRIMENKUL_DATA` | Veri klasörü yolu |
