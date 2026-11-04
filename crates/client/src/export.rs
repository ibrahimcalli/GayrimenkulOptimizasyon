use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use common::{CrmRecord, OutputSettings, Portfolio, User};

/// Türkçe karakterleri PDF yerleşik fontlarıyla uyumlu olacak şekilde ASCII'ye çevirir.
/// (DOCX çıktısı Unicode'u tam destekler; bu dönüşüm yalnızca PDF içindir.)
pub fn tr_ascii(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ç' => 'c', 'Ç' => 'C',
            'ğ' => 'g', 'Ğ' => 'G',
            'ı' => 'i', 'İ' => 'I',
            'ö' => 'o', 'Ö' => 'O',
            'ş' => 's', 'Ş' => 'S',
            'ü' => 'u', 'Ü' => 'U',
            'â' => 'a', 'î' => 'i', 'û' => 'u',
            '₺' => ' ', '“' | '”' => '"', '‘' | '’' => '\'',
            _ => c,
        })
        .collect()
}

fn money(p: &Portfolio) -> String {
    let sym = match p.currency.as_str() {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        _ => "TL",
    };
    let formatted = format_thousands(p.price);
    format!("{formatted} {sym}")
}

fn format_thousands(v: f64) -> String {
    let n = v.round() as i64;
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('.');
        }
        out.push(ch);
    }
    let mut r: String = out.chars().rev().collect();
    if n < 0 {
        r.insert(0, '-');
    }
    r
}

/// Portföy + iletişim bilgilerinden yazdırılabilir satır listesi üretir.
fn build_lines(p: &Portfolio, owner: Option<&CrmRecord>, agent: &User, s: &OutputSettings) -> Vec<(String, u8)> {
    // (metin, tür)  tür: 0 normal, 1 alt başlık, 2 başlık
    let mut v: Vec<(String, u8)> = Vec::new();
    v.push((s.header_text.clone(), 2));
    v.push((format!("{} - {}", p.listing_type_label(), p.category_label()), 1));
    v.push((p.title.clone(), 0));
    v.push(("".into(), 0));
    if s.show_price {
        v.push((format!("Fiyat: {}", money(p)), 0));
    }
    if p.area_m2 > 0.0 {
        v.push((format!("Alan: {} m2", p.area_m2), 0));
    }
    if !p.room_count.is_empty() {
        v.push((format!("Oda Sayısı: {}", p.room_count), 0));
    }
    if !p.building_age.is_empty() {
        v.push((format!("Bina Yaşı: {}", p.building_age), 0));
    }
    if !p.floor.is_empty() || !p.total_floors.is_empty() {
        v.push((format!("Kat: {} / {}", p.floor, p.total_floors), 0));
    }
    if !p.heating.is_empty() {
        v.push((format!("Isıtma: {}", p.heating), 0));
    }
    v.push((format!("Eşyalı: {}", if p.furnished { "Evet" } else { "Hayır" }), 0));
    v.push((format!("Durum: {}", p.status_label()), 0));
    let loc = format!("Konum: {} / {} / {}", p.il, p.ilce, p.mahalle);
    let loc = if p.address.is_empty() { loc } else { format!("{loc} - {}", p.address) };
    v.push((loc, 0));
    if p.latitude != 0.0 || p.longitude != 0.0 {
        v.push((format!("Koordinat: {:.5}, {:.5}", p.latitude, p.longitude), 0));
    }
    if s.show_description && !p.description.is_empty() {
        v.push(("".into(), 0));
        v.push(("Açıklama".into(), 1));
        for line in wrap(&p.description, 90) {
            v.push((line, 0));
        }
    }

    // İletişim bilgileri
    v.push(("".into(), 0));
    v.push(("İletişim".into(), 1));
    if s.show_name {
        if !agent.full_name.is_empty() {
            v.push((format!("Yetkili: {}", agent.full_name), 0));
        }
        if let Some(o) = owner {
            v.push((format!("Taşınmaz Sahibi: {}", o.full_name), 0));
        }
    }
    if s.show_phone {
        if !agent.phone.is_empty() {
            v.push((format!("Telefon: {}", agent.phone), 0));
        }
    }
    if s.show_email && !agent.email.is_empty() {
        v.push((format!("E-posta: {}", agent.email), 0));
    }
    if let Some(o) = owner {
        if s.show_phone && !o.phone.is_empty() {
            v.push((format!("Sahip Telefonu: {}", o.phone), 0));
        }
    }
    v.push(("".into(), 0));
    v.push((s.footer_text.clone(), 1));
    v
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let mut line = String::new();
        for word in raw.split_whitespace() {
            if line.len() + word.len() + 1 > width && !line.is_empty() {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        out.push(line);
    }
    out
}

// =====================================================================
// PDF
// =====================================================================

pub fn export_pdf(
    out_path: &Path,
    p: &Portfolio,
    owner: Option<&CrmRecord>,
    agent: &User,
    s: &OutputSettings,
) -> Result<PathBuf, String> {
    use printpdf::*;

    let lines = build_lines(p, owner, agent, s);
    let (doc, page, layer) = PdfDocument::new(
        tr_ascii(&p.title),
        Mm(210.0),
        Mm(297.0),
        "Katman 1",
    );
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| e.to_string())?;
    let bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| e.to_string())?;
    let layer_ref = doc.get_page(page).get_layer(layer);

    let mut y = 280.0_f32;
    for (text, kind) in lines {
        let (size, use_bold) = match kind {
            2 => (22.0, true),
            1 => (14.0, true),
            _ => (11.0, false),
        };
        if !text.is_empty() {
            let f = if use_bold { &bold } else { &font };
            layer_ref.use_text(tr_ascii(&text), size, Mm(18.0), Mm(y), f);
        }
        y -= size * 0.55 + 3.0;
        if y < 20.0 {
            break;
        }
    }

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut writer = std::io::BufWriter::new(file);
    doc.save(&mut writer).map_err(|e| e.to_string())?;
    writer.flush().ok();
    Ok(out_path.to_path_buf())
}

// =====================================================================
// DOCX (bağımsız paket; kapak resmi gömülür)
// =====================================================================

pub fn export_docx(
    out_path: &Path,
    p: &Portfolio,
    owner: Option<&CrmRecord>,
    agent: &User,
    s: &OutputSettings,
    cover: Option<&[u8]>,
) -> Result<PathBuf, String> {
    let lines = build_lines(p, owner, agent, s);

    // Kapak resmini normalize JPEG olarak hazırla
    let mut image_part: Option<(Vec<u8>, u32, u32)> = None;
    if s.show_cover_image {
        if let Some(bytes) = cover {
            if let Ok(img) = image::load_from_memory(bytes) {
                let (w, h) = (img.width(), img.height());
                let rgb = img.to_rgb8();
                let mut buf: Vec<u8> = Vec::new();
                let mut cursor = Cursor::new(&mut buf);
                if rgb.write_to(&mut cursor, image::ImageFormat::Jpeg).is_ok() {
                    image_part = Some((buf, w, h));
                }
            }
        }
    }

    let has_image = image_part.is_some();

    // document.xml gövdesi
    let mut body = String::new();
    if let Some((_, w, h)) = &image_part {
        body.push_str(&drawing_paragraph(*w, *h));
    }
    for (text, kind) in &lines {
        body.push_str(&paragraph(text, *kind));
    }

    let document_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body>{body}<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1134" w:right="1134" w:bottom="1134" w:left="1134"/></w:sectPr></w:body>
</w:document>"#
    );

    let content_types = {
        let mut extra = String::new();
        if has_image {
            extra.push_str(r#"<Default Extension="jpeg" ContentType="image/jpeg"/>"#);
        }
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>{extra}
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#
        )
    };

    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

    let doc_rels = {
        let image_rel = if has_image {
            r#"<Relationship Id="rIdImg" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/cover.jpeg"/>"#
        } else {
            ""
        };
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{image_rel}</Relationships>"#
        )
    };

    // ZIP paketle
    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let opts: zip::write::SimpleFileOptions =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let add = |zip: &mut zip::ZipWriter<std::fs::File>, name: &str, data: &[u8]| -> Result<(), String> {
        zip.start_file(name, opts).map_err(|e| e.to_string())?;
        zip.write_all(data).map_err(|e| e.to_string())?;
        Ok(())
    };

    add(&mut zip, "[Content_Types].xml", content_types.as_bytes())?;
    add(&mut zip, "_rels/.rels", rels.as_bytes())?;
    add(&mut zip, "word/document.xml", document_xml.as_bytes())?;
    add(&mut zip, "word/_rels/document.xml.rels", doc_rels.as_bytes())?;
    if let Some((img, _, _)) = &image_part {
        add(&mut zip, "word/media/cover.jpeg", img)?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(out_path.to_path_buf())
}

fn esc_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn paragraph(text: &str, kind: u8) -> String {
    let (size, bold, color) = match kind {
        2 => (36, true, "1F3864"),
        1 => (26, true, "2E74B5"),
        _ => (22, false, "000000"),
    };
    let b = if bold { "<w:b/>" } else { "" };
    let empty_run = if text.is_empty() { "<w:r><w:t xml:space=\"preserve\"> </w:t></w:r>" } else { "" };
    format!(
        r#"<w:p><w:pPr><w:spacing w:after="120"/><w:rPr>{b}<w:sz w:val="{size}"/><w:color w:val="{color}"/></w:rPr></w:pPr><w:r><w:rPr>{b}<w:sz w:val="{size}"/><w:color w:val="{color}"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>{empty_run}</w:p>"#,
        esc_xml(text)
    )
}

fn drawing_paragraph(img_w: u32, img_h: u32) -> String {
    // Sayfa genişliği ~15 cm olacak şekilde ölçekle (EMU: 914400 = 1 inç)
    const MAX_W_EMU: u64 = 5_400_000; // ~15 cm
    let ratio = if img_w == 0 { 1.0 } else { img_h as f64 / img_w as f64 };
    let cx = MAX_W_EMU;
    let cy = (MAX_W_EMU as f64 * ratio) as u64;
    format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="Kapak"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="1" name="Kapak"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rIdImg"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
    )
}

// Portföy model yardımcıları (istemci tarafı etiketleme)
// Portfolio tipi `common` crate'inde tanımlı olduğundan inherent impl yerine
// bir uzantı (extension) trait'i kullanılır.
pub trait PortfolioLabels {
    fn listing_type_label(&self) -> String;
    fn category_label(&self) -> String;
    fn status_label(&self) -> String;
}

impl PortfolioLabels for Portfolio {
    fn listing_type_label(&self) -> String {
        common::ListingType::from_str(&self.listing_type).label().to_string()
    }
    fn category_label(&self) -> String {
        common::PropertyCategory::from_str(&self.category).label().to_string()
    }
    fn status_label(&self) -> String {
        common::PortfolioStatus::from_str(&self.status).label().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{Portfolio, Role, User};

    fn sample_portfolio() -> Portfolio {
        Portfolio {
            id: 1,
            title: "Deniz Manzaralı Villa".into(),
            listing_type: "satilik".into(),
            category: "konut".into(),
            status: "yayinda".into(),
            price: 12500000.0,
            currency: "TRY".into(),
            area_m2: 240.0,
            room_count: "5+2".into(),
            building_age: "3".into(),
            floor: "2".into(),
            total_floors: "3".into(),
            heating: "Kombi".into(),
            furnished: true,
            il: "Muğla".into(),
            ilce: "Bodrum".into(),
            mahalle: "Bitez".into(),
            address: "Bitez Mah. 123 Sok. No:5".into(),
            latitude: 37.0345,
            longitude: 27.3456,
            description: "Güney cephe, geniş teras, deniz manzarası.".into(),
            owner_crm_id: None,
            created_by: 1,
            created_by_name: "Sistem Yöneticisi".into(),
            created_at: "2026-11-03 10:00:00".into(),
            updated_at: "2026-11-03 10:00:00".into(),
            images: Vec::new(),
        }
    }

    fn sample_agent() -> User {
        User {
            id: 1,
            username: "admin".into(),
            full_name: "Ahmet Öztürk".into(),
            role: Role::Admin,
            phone: "0532 000 00 00".into(),
            email: "ahmet@gayrimenkul360.com".into(),
            photo: None,
            active: true,
            created_at: "2026-11-03 10:00:00".into(),
        }
    }

    #[test]
    fn pdf_export_works() {
        let dir = std::env::temp_dir().join("g360_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("test.pdf");
        export_pdf(&out, &sample_portfolio(), None, &sample_agent(), &OutputSettings::default()).unwrap();
        let data = std::fs::read(&out).unwrap();
        assert!(data.starts_with(b"%PDF"), "geçerli PDF başlığı yok");
        assert!(data.len() > 500);
    }

    #[test]
    fn docx_export_works() {
        let dir = std::env::temp_dir().join("g360_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("test.docx");
        export_docx(&out, &sample_portfolio(), None, &sample_agent(), &OutputSettings::default(), None).unwrap();
        let data = std::fs::read(&out).unwrap();
        // DOCX bir ZIP'tir: PK imzası ile başlar
        assert!(data.starts_with(b"PK"), "geçerli DOCX (zip) başlığı yok");
        assert!(data.len() > 500);
    }

    #[test]
    fn docx_with_cover_embeds_media() {
        // 4x4 küçük JPEG üret
        let img = image::RgbImage::from_fn(4, 4, |x, y| image::Rgb([(x * 40) as u8, (y * 40) as u8, 128]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Jpeg).unwrap();
        let cover = buf.into_inner();

        let dir = std::env::temp_dir().join("g360_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("cover.docx");
        export_docx(&out, &sample_portfolio(), None, &sample_agent(), &OutputSettings::default(), Some(&cover)).unwrap();

        let file = std::fs::File::open(&out).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        assert!(zip.by_name("word/media/cover.jpeg").is_ok(), "kapak resmi gömülmedi");
        assert!(zip.by_name("word/document.xml").is_ok());
    }

    #[test]
    fn turkish_transliteration() {
        assert_eq!(tr_ascii("Çağrı Şölen Ünlü İğne"), "Cagri Solen Unlu Igne");
    }
}
