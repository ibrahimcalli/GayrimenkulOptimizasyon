//! Muğla ili ilçe ve mahalle ön tanımlı verisi.
//! Yeni mahalleler admin panelinden / veritabanından eklenebilir; bu liste
//! uygulamanın hızlı seçim (combobox) kaynağıdır.

pub const IL: &str = "Muğla";

pub const DISTRICTS: &[&str] = &[
    "Bodrum",
    "Dalaman",
    "Datça",
    "Fethiye",
    "Kavaklıdere",
    "Köyceğiz",
    "Marmaris",
    "Menteşe",
    "Milas",
    "Ortaca",
    "Seydikemer",
    "Ula",
    "Yatağan",
];

/// İlçeye ait mahalle listesini döndürür. Bilinmeyen ilçe için boş liste döner.
pub fn neighborhoods(district: &str) -> &'static [&'static str] {
    match district {
        "Bodrum" => BODRUM,
        "Dalaman" => DALAMAN,
        "Datça" => DATCA,
        "Fethiye" => FETHIYE,
        "Kavaklıdere" => KAVAKLIDERE,
        "Köyceğiz" => KOYCEGIZ,
        "Marmaris" => MARMARIS,
        "Menteşe" => MENTESE,
        "Milas" => MILAS,
        "Ortaca" => ORTACA,
        "Seydikemer" => SEYDIKEMER,
        "Ula" => ULA,
        "Yatağan" => YATAGAN,
        _ => &[],
    }
}

pub const BODRUM: &[&str] = &[
    "Ağaçlı", "Akçalan", "Akyarlar", "Bahçelievler", "Bağla", "Bitez", "Cevat Şakir",
    "Cumhuriyet", "Çamlık", "Çarşı", "Çırkan", "Çömlekçi", "Dağbelen", "Dereköy",
    "Dirmil (Gümüşlük)", "Dörttepe", "Eskiçeşme", "Farilya (Gündoğan)", "Geriş",
    "Gökçebel", "Gölbaşı", "Göltürkbükü", "Gümüşlük", "Gündoğan", "Gürece",
    "Güvercinlik", "İslamhaneleri", "Karabağlar", "Karaova", "Kefaluka", "Kızılağaç",
    "Konacık", "Koyunbaba", "Kumbahçe", "Küçükbük", "Mazı", "Mumcular", "Ortakent",
    "Peksimet", "Pınarlıbelen", "Sarıca", "Tepecik", "Turgutreis", "Türkbükü",
    "Umurça", "Yahşi", "Yalıkavak", "Yeniköy", "Yokuşbaşı",
];

pub const DALAMAN: &[&str] = &[
    "Altıntaş", "Atakent", "Bezkese", "Cumhuriyet", "Çöğmen", "Dalaman",
    "Ege", "Elcik", "Gürköy", "Gürlek", "Hürriyet", "Kapukargın", "Karaburun",
    "Karacaağaç", "Karaçalı", "Kargınkürü", "Katrancı", "Kayadibi", "Kılıçlar",
    "Mergenli", "Merkez", "Nif", "Söğütlüyurt", "Şerefler", "Taşbaşı", "Yıldırım",
];

pub const DATCA: &[&str] = &[
    "Cumalı", "Datça", "Eski Datça", "Gebekum", "Hızırşah", "İskele", "Karaincir",
    "Kargı", "Kızlan", "Kumluk", "Mesudiye", "Palamutbükü", "Reşadiye", "Sındı",
    "Yaka", "Yazı", "Yeniköy",
];

pub const FETHIYE: &[&str] = &[
    "Akarca", "Arpacık", "Babataşı", "Bağlarla", "Cami", "Cumhuriyet", "Çaltılar",
    "Çamköy", "Çenger", "Çıtlık", "Çobanlar", "Dodurga", "Eldirek", "Esenköy",
    "Foça", "Gökben", "Gökçeovacık", "Göcek", "Hacıosmanlar", "İncirköy",
    "İnlice", "Kadıköy", "Karaağaç", "Karagözler", "Karakeçililer", "Karamustafa",
    "Karapınar", "Kargı", "Kayadibi", "Kayaköy", "Keçiler", "Kesikkapı",
    "Kılıçarslan", "Kınık", "Korucuk", "Kumluova", "Menteşeoğlu", "Ovacık",
    "Ölüdeniz", "Paşalı", "Pazaryeri", "Sahil", "Seydiler", "Taşyaka", "Tuzla",
    "Üzümlü", "Yaka", "Yanıklar", "Yeni", "Yeşilüzümlü", "Zorlar",
];

pub const KAVAKLIDERE: &[&str] = &[
    "Bakır", "Baraj", "Cumhuriyet", "Çamlıyurt", "Çatak", "Çavdar", "Derebağ",
    "Gökçukur", "Kabaca", "Kavaklıdere", "Kurucuova", "Menteşe", "Nebeliler",
    "Ortaköy", "Salkım", "Yeni", "Yeşilköy",
];

pub const KOYCEGIZ: &[&str] = &[
    "Arıkaya", "Beyobası", "Cumhuriyet", "Çandır", "Çayhisar", "Döğüşbelen",
    "Ekincik", "Gelişim", "Gülpınar", "Hamitköy", "Karaçam", "Kavakarası",
    "Köyceğiz", "Merkez", "Pınar", "Sazak", "Sultaniye", "Toparlar", "Ulucami",
    "Yayla", "Yeni", "Zeytinalanı",
];

pub const MARMARIS: &[&str] = &[
    "Armutalan", "Belceğiz", "Beldibi", "Camiavlu", "Çamlı", "Çetibeli",
    "Çıldır", "Çubucak", "Ekincik", "Gölenye", "Hisarönü", "İçmeler",
    "Karaca", "Kayabaşı", "Kızılkaya", "Kızılyaka", "Orhaniye", "Osmaniye",
    "Selimiye", "Siteler", "Söğüt", "Taşlı", "Tepe", "Turunç", "Yalancıboğaz",
    "Yeşilbelde", "Zeytinliburun",
];

pub const MENTESE: &[&str] = &[
    "Akçaova", "Akçakaya", "Akyer", "Akyol", "Alemler", "Avcılar", "Balıbey",
    "Bayır", "Bozyer", "Camikebir", "Cumhuriyet", "Çakır", "Çamoluk", "Çatakbağ",
    "Çaybükü", "Çınar", "Çiftlik", "Dağdibi", "Dağpınar", "Denizova",
    "Dere", "Doğan", "Dokuzçam", "Düğerek", "Emirbeyazıt", "Esençay", "Gökpınar",
    "Göktepe", "Gölcük", "Gülağzı", "Günlüce", "Hacırüştü", "İkizce", "İsmetpaşa",
    "Karacaören", "Karakuyu", "Karşıyaka", "Kıran", "Kızılağaç", "Kızılyaka",
    "Konakaltı", "Kozağaç", "Kötekli", "Kuyucak", "Kuzluk", "Menteşe",
    "Muslihittin", "Müştakbey", "Orhaniye", "Ortaköy", "Otmanlar", "Özlüce",
    "Paşapınarı", "Salihpaşalar", "Sarnıç", "Sıtkıkoçman", "Sungur", "Şenyurt",
    "Şükrüpaşa", "Taşlı", "Turgut", "Turkevleri", "Yemişendere", "Yenibağyaka",
    "Yenice", "Yeniköy(Menteşe)", "Yeşilbağcılar", "Yeşilköy", "Yılanlı",
    "Yörükoğlu", "Zeytin", "Zeytinköy",
];

pub const MILAS: &[&str] = &[
    "Ağaçlıhüyük", "Ahmetçavuşhayıtlı", "Akçakaya", "Akçalı", "Akkovanlık",
    "Akyol", "Alaçam", "Alatepe", "Aslanyaka", "Avşar", "Aydınlık", "Baharlı",
    "Bahçeburun", "Balcılar", "Bayır", "Beçin", "Beyciler", "Boğaziçi",
    "Burgaz", "Cumhuriyet", "Çakırca", "Çalı", "Çamköy", "Çamlıbelen",
    "Çamlıca", "Çamlıyurt", "Çandır", "Çapak", "Çınarlı", "Çökertme",
    "Çomakdağ", "Damlıboğaz", "Danişment", "Demirciler", "Dere", "Derince",
    "Dibekdere", "Dörttepe", "Eğridere", "Ekinanbarı", "Ekizköy", "Ekiztaş",
    "Emek", "Epçe", "Etrenli", "Fesleğen", "Gelinören", "Gökçeler", "Göldere",
    "Gölyaka", "Gümüşlük", "Güneş", "Günlük", "Gürçamlar", "Hacıahmetler",
    "Hacıilyas", "Hasanlar", "Hisarbaşı", "Hisarcık", "Hüsamlar", "İçme",
    "İkizköy", "İkiztaş", "İsmetpaşa", "Kafaca", "Kalemköy", "Kalınağıl",
    "Kandak", "Kapıkırı", "Karacaağaç", "Karacahisar", "Karakuyu", "Karapınar",
    "Kargıcak", "Kayabaşı", "Kayadere", "Kemikler", "Ketendere", "Kısırlar",
    "Kıyıkışlacık", "Kızılağaç", "Kızılcakuyu", "Kızılcayıkık", "Konak",
    "Korucuk", "Köşk", "Köyceğiz", "Kulak", "Kurban", "Kurudere", "Kuzyaka",
    "Küçükdibekdere", "Menteş", "Meşelik", "Narlı", "Ortaköy", "Ovakışlacık",
    "Ören", "Pınararası", "Pınarcık", "Pınarköy", "Sarıkaya", "Savran",
    "Sek", "Selimiye", "Şenköy", "Şevketiye", "Tuzabat", "Türkevleri",
    "Uzunalan", "Yaka", "Yakaköy", "Yaşyer", "Yayla", "Yeniköy(Milas)",
    "Yeşilova", "Yusufca", "Zeytin",
];

pub const ORTACA: &[&str] = &[
    "Akıncı", "Arıkbaşı", "Atatürk", "Bahçelievler", "Beşköprü", "Cumhuriyet",
    "Çaylı", "Dalaklı", "Demirci", "Dereköy", "Ekşiliyurt", "Esenpınar",
    "Fevziye", "Gökbel", "Gölbaşı", "Güzelyurt", "Karaburun", "Karadonlar",
    "Kemaliye", "Kızılbel", "Mergenli", "Okçular", "Ortaca", "Sarigerme",
    "Tepearası", "Terzialiler", "Yeniköy",
];

pub const SEYDIKEMER: &[&str] = &[
    "Alacat", "Arifler", "Arsaköy", "Atlıdere", "Bağlıağaç", "Bayırköy",
    "Bekçiler", "Boğalar", "Cumhuriyet", "Çaltıözü", "Çatak", "Çaykenarı",
    "Çobanlar", "Çökek", "Dodurga", "Doğanlar", "Dont", "Düğer", "Eşen",
    "Gerişburnu", "Girmeler", "Gölbent", "Güneşli", "Hacıosmanlar", "İzzettinköy",
    "Kabaağaç", "Kadıköy", "Karaköy", "Kayabaşı", "Kayadibi", "Kıncılar",
    "Kınık", "Kırkpınar", "Korubükü", "Kumluova", "Minare", "Ortaköy",
    "Ören", "Paşalı", "Sarıyer", "Seydiler", "Söğütlüdere", "Şebnem",
    "Temel", "Uğurlu", "Yakabağ", "Yakaköy", "Yayla", "Yeniköy(Seydikemer)",
    "Zorlar",
];

pub const ULA: &[&str] = &[
    "Akçapınar", "Akyaka", "Alparslan", "Arıcılar", "Ataköy", "Ayazkıyı",
    "Çamlı", "Çiçekli", "Çörüş", "Demirtepe", "Elmalı", "Esentepe", "Gökçe",
    "Gökova", "Gölcük", "Kavakçalı", "Kıyıdere", "Kızılağaç", "Kızılyaka",
    "Köyceğiz", "Örnekköy", "Portakallık", "Sarayyanı", "Şirinköy",
    "Turgut", "Ula", "Yeşilçam", "Yeşilova",
];

pub const YATAGAN: &[&str] = &[
    "Akgedik", "Akyol", "Alaşar", "Atatürk", "Bağyaka", "Bahçeyaka",
    "Bencik", "Bozarmut", "Bozüyük", "Cazkırlar", "Cumhuriyet", "Çakırlar",
    "Çamlıca", "Çukuröz", "Dere", "Deştin", "Doğanköy", "Elmacık", "Esenköy",
    "Gökgedik", "Gökpınar", "Hacıbayramlar", "Hacıveliler", "Hisarardı",
    "Kafaca", "Katrancı", "Kırbıyıklar", "Kızılağaç", "Kızılkaya",
    "Kızılyaka", "Konak", "Kozağaç", "Köklük", "Madenler", "Mesken",
    "Nebiköy", "Sağlık", "Sarnıç", "Şahinler", "Şerefköy", "Turgut",
    "Türkmen", "Yava", "Yayla", "Yeni", "Yeşilbağcılar", "Yeşilköy",
    "Yukarıyayla", "Zafer",
];
