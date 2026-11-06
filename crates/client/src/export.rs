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
        if p.area_m2 > 0.0 {
            v.push((format!("Tasinmaz m2 fiyati: {:.0} {}/m2", p.price / p.area_m2, p.currency), 0));
        }
    }
    if !p.ada_parsel.is_empty() {
        v.push((format!("Ada/Parsel: {}", p.ada_parsel), 0));
    }
    if p.area_m2 > 0.0 {
        let net = if p.area_net > 0.0 {
            format!(" (Net: {} m2)", p.area_net)
        } else {
            String::new()
        };
        v.push((format!("Alan: {} m2 brut{}", p.area_m2, net), 0));
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

/// Portföy resimlerini sayfaya ızgara (grid) düzeninde küçük resimler olarak ekler.
const PDF_THUMB_W: f32 = 52.0; // mm
const PDF_GAP: f32 = 6.0; // mm
const PDF_COLS: usize = 3;

fn add_image_grid_to_pdf(
    doc: &printpdf::PdfDocumentReference,
    page: printpdf::PdfPageIndex,
    layer_idx: printpdf::PdfLayerIndex,
    images: &[Vec<u8>],
    mut y: f32,
) -> Result<(), String> {
    let mut col = 0usize;
    let mut row_h = 0.0f32;
    let mut cur_page = page;
    let mut cur_layer_i = layer_idx;

    for bytes in images {
        let img = match printpdf::image_crate::load_from_memory(bytes) {
            Ok(i) => i,
            Err(_) => continue,
        };
        let (iw, ih) = (img.width() as f32, img.height() as f32);
        if iw == 0.0 || ih == 0.0 {
            continue;
        }
        let h_mm = PDF_THUMB_W * ih / iw;
        row_h = row_h.max(h_mm);

        // Sayfa dolduysa yeni sayfa aç
        if y - row_h < 20.0 {
            let (p, l) = doc.add_page(printpdf::Mm(210.0), printpdf::Mm(297.0), "Katman");
            cur_page = p;
            cur_layer_i = l;
            y = 280.0;
        }

        let x = 18.0 + (col % PDF_COLS) as f32 * (PDF_THUMB_W + PDF_GAP);
        let image = printpdf::Image::from_dynamic_image(&img);
        // dpi=72 => 1px = 1pt; ölçek = hedef_pt / px
        let target_pt = PDF_THUMB_W * 72.0 / 25.4;
        let scale = target_pt / iw;
        image.add_to_layer(
            doc.get_page(cur_page).get_layer(cur_layer_i),
            printpdf::ImageTransform {
                translate_x: Some(printpdf::Mm(x)),
                translate_y: Some(printpdf::Mm(y - h_mm)),
                rotate: None,
                scale_x: Some(scale),
                scale_y: Some(scale),
                dpi: Some(72.0),
            },
        );

        col += 1;
        if col % PDF_COLS == 0 {
            y -= row_h + PDF_GAP;
            row_h = 0.0;
        }
    }
    Ok(())
}

pub fn export_pdf(
    out_path: &Path,
    p: &Portfolio,
    owner: Option<&CrmRecord>,
    agent: &User,
    s: &OutputSettings,
    images: &[Vec<u8>],
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

    let show_images = s.show_cover_image;
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

    // Tüm portföy resimleri küçük resim ızgarası olarak eklenir
    if show_images && !images.is_empty() {
        y -= 8.0;
        if y > 70.0 {
            add_image_grid_to_pdf(&doc, page, layer, images, y)?;
        } else {
            add_image_grid_to_pdf(&doc, page, layer, images, 280.0)?;
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
    images: &[Vec<u8>],
) -> Result<PathBuf, String> {
    let lines = build_lines(p, owner, agent, s);

    // Tüm resimleri normalize JPEG olarak hazırla (ilk resim kapak)
    let mut image_parts: Vec<Vec<u8>> = Vec::new();
    let mut image_sizes: Vec<(u32, u32)> = Vec::new();
    if s.show_cover_image {
        for bytes in images {
            if let Ok(img) = image::load_from_memory(bytes) {
                let (w, h) = (img.width(), img.height());
                let rgb = img.to_rgb8();
                let mut buf: Vec<u8> = Vec::new();
                let mut cursor = Cursor::new(&mut buf);
                if rgb.write_to(&mut cursor, image::ImageFormat::Jpeg).is_ok() {
                    image_parts.push(buf);
                    image_sizes.push((w, h));
                }
            }
        }
    }

    let has_image = !image_parts.is_empty();

    // document.xml gövdesi
    let mut body = String::new();

    // Kapak resmi (tam genişlik)
    if let (Some(&(w, h)), Some(_)) = (image_sizes.first(), image_parts.first()) {
        body.push_str(&drawing_paragraph(w, h, "rIdImg1"));
    }

    // Diğer resimler: 2 sütunlu katalog ızgarası (tablo)
    if image_parts.len() > 1 {
        let mut cells = String::new();
        for (i, &(w, h)) in image_sizes.iter().enumerate().skip(1) {
            let rid = format!("rIdImg{}", i + 1);
            cells.push_str(&drawing_cell(w, h, &rid));
            // 2 sütun: her 2 hücreden sonra satır kapat
            if (i - 1 + 1) % 2 == 0 {
                body.push_str(&format!("<w:tbl><w:tblPr><w:tblW w:w=\"9638\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"4819\"/><w:gridCol w:w=\"4819\"/></w:tblGrid>{cells}</w:tbl>"));
                cells.clear();
                // tablolar arası boşluk
                body.push_str(&paragraph("", 0));
            }
        }
        // Tek kalan hücre varsa yine tablo yap (boş hücreyle)
        if !cells.is_empty() {
            cells.push_str("<w:tc><w:tcPr><w:tcW w:w=\"4819\" w:type=\"dxa\"/></w:tcPr><w:p/></w:tc>");
            body.push_str(&format!("<w:tbl><w:tblPr><w:tblW w:w=\"9638\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"4819\"/><w:gridCol w:w=\"4819\"/></w:tblGrid>{cells}</w:tbl>"));
        }
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
        let mut image_rels = String::new();
        for i in 0..image_parts.len() {
            image_rels.push_str(&format!(
                r#"<Relationship Id="rIdImg{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image{}.jpeg"/>"#,
                i + 1,
                i + 1
            ));
        }
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{image_rels}</Relationships>"#
        )
    };

    // ZIP paketle
    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let opts: zip::write::SimpleFileOptions =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut add = |zip: &mut zip::ZipWriter<std::fs::File>, name: &str, data: &[u8]| -> Result<(), String> {
        zip.start_file(name, opts).map_err(|e| e.to_string())?;
        zip.write_all(data).map_err(|e| e.to_string())?;
        Ok(())
    };

    add(&mut zip, "[Content_Types].xml", content_types.as_bytes())?;
    add(&mut zip, "_rels/.rels", rels.as_bytes())?;
    add(&mut zip, "word/document.xml", document_xml.as_bytes())?;
    add(&mut zip, "word/_rels/document.xml.rels", doc_rels.as_bytes())?;
    for (i, img) in image_parts.iter().enumerate() {
        add(&mut zip, &format!("word/media/image{}.jpeg", i + 1), img)?;
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

fn drawing_paragraph(img_w: u32, img_h: u32, rid: &str) -> String {
    // Sayfa genişliği ~15 cm olacak şekilde ölçekle (EMU: 914400 = 1 inç)
    const MAX_W_EMU: u64 = 5_400_000; // ~15 cm
    let ratio = if img_w == 0 { 1.0 } else { img_h as f64 / img_w as f64 };
    let cx = MAX_W_EMU;
    let cy = (MAX_W_EMU as f64 * ratio) as u64;
    format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="Resim"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="1" name="Resim"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
    )
}

/// Katalog ızgarası hücresi: tablo hücresi içinde ortalanmış küçük resim.
fn drawing_cell(img_w: u32, img_h: u32, rid: &str) -> String {
    const CELL_W_EMU: u64 = 2_700_000; // ~7.5 cm (hücre içi)
    let ratio = if img_w == 0 { 1.0 } else { img_h as f64 / img_w as f64 };
    let cx = CELL_W_EMU;
    let cy = ((CELL_W_EMU as f64 * ratio) as u64).min(CELL_W_EMU); // aşırı uzun resimleri sınırla
    format!(
        r#"<w:tc><w:tcPr><w:tcW w:w="4819" w:type="dxa"/><w:vAlign w:val="center"/></w:tcPr><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="Resim"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="1" name="Resim"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p></w:tc>"#
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

// =====================================================================
// VARDİYA / MESAİ RAPORU
// =====================================================================

/// Raporda kişi başına satır.
#[derive(Debug, Clone, Default)]
pub struct ReportRow {
    pub name: String,
    pub planned_days: i64,
    pub planned_hours: f64,
    pub actual_days: i64,
    pub actual_hours: f64,
    pub approved_days: i64,
}

impl ReportRow {
    pub fn rate(&self) -> f64 {
        if self.planned_days > 0 {
            (self.actual_days as f64 / self.planned_days as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Vardiya/mesai raporunu PDF olarak üretir.
pub fn export_shift_report_pdf(out_path: &Path, month: &str, rows: &[ReportRow]) -> Result<PathBuf, String> {
    use printpdf::*;

    let (doc, page, layer) = PdfDocument::new("Vardiya Raporu", Mm(297.0), Mm(210.0), "Katman 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;
    let lr = doc.get_page(page).get_layer(layer);

    let mut y = 195.0_f32;
    lr.use_text(tr_ascii(&format!("Vardiya / Mesai Raporu  -  {}", month)), 18.0, Mm(15.0), Mm(y), &bold);
    y -= 12.0;
    let header = format!(
        "{:<22} {:>10} {:>10} {:>10} {:>10} {:>8} {:>8}",
        "Personel", "Plan Gun", "Plan Saat", "Gercek Gun", "Toplam Saat", "Onayli", "Devam %"
    );
    lr.use_text(tr_ascii(&header), 10.0, Mm(15.0), Mm(y), &bold);
    y -= 7.0;
    for r in rows {
        let line = format!(
            "{:<22} {:>10} {:>10.1} {:>10} {:>10.1} {:>8} {:>7.0}%",
            tr_ascii(&r.name),
            r.planned_days,
            r.planned_hours,
            r.actual_days,
            r.actual_hours,
            r.approved_days,
            r.rate()
        );
        lr.use_text(tr_ascii(&line), 10.0, Mm(15.0), Mm(y), &font);
        y -= 7.0;
        if y < 15.0 {
            break;
        }
    }

    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut w = std::io::BufWriter::new(file);
    doc.save(&mut w).map_err(|e| e.to_string())?;
    w.flush().ok();
    Ok(out_path.to_path_buf())
}

/// Vardiya/mesai raporunu CSV (Excel uyumlu, UTF-8 BOM + ; ayraç) olarak üretir.
pub fn export_shift_report_csv(out_path: &Path, month: &str, rows: &[ReportRow]) -> Result<PathBuf, String> {
    let mut out = String::from("\u{feff}");
    out.push_str(&format!("Vardiya/Mesai Raporu;{}\n", month));
    out.push_str("Personel;Planlanan Gun;Planlanan Saat;Gerceklesen Gun;Toplam Saat;Onayli Gun;Devam (%)\n");
    for r in rows {
        out.push_str(&format!(
            "{};{};{:.1};{};{:.1};{};{:.0}\n",
            r.name.replace(';', ","),
            r.planned_days,
            r.planned_hours,
            r.actual_days,
            r.actual_hours,
            r.approved_days,
            r.rate()
        ));
    }
    std::fs::write(out_path, out.as_bytes()).map_err(|e| e.to_string())?;
    Ok(out_path.to_path_buf())
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
            area_net: 200.0,
            ada_parsel: "Ada 412, Parsel 7".into(),
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
            is_deleted: false,
            deleted_by_user_id: None,
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
        export_pdf(&out, &sample_portfolio(), None, &sample_agent(), &OutputSettings::default(), &[]).unwrap();
        let data = std::fs::read(&out).unwrap();
        assert!(data.starts_with(b"%PDF"), "geçerli PDF başlığı yok");
        assert!(data.len() > 500);
    }

    #[test]
    fn pdf_with_image_grid() {
        let img = image::RgbImage::from_fn(8, 6, |x, y| image::Rgb([(x * 20) as u8, (y * 30) as u8, 90]));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Jpeg).unwrap();

        let dir = std::env::temp_dir().join("g360_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("grid.pdf");
        export_pdf(&out, &sample_portfolio(), None, &sample_agent(), &OutputSettings::default(), &[buf.into_inner()]).unwrap();
        let data = std::fs::read(&out).unwrap();
        assert!(data.starts_with(b"%PDF"));
        // Resim XObject olarak gömülmeli ve dosya belirgin şekilde büyümeli
        let baseline = {
            let out2 = dir.join("baseline.pdf");
            export_pdf(&out2, &sample_portfolio(), None, &sample_agent(), &OutputSettings::default(), &[]).unwrap();
            std::fs::read(&out2).unwrap().len()
        };
        assert!(data.windows(5).any(|w| w == b"Image"), "resim XObject yok");
        assert!(data.len() > baseline + 100, "resim verisi gömülmedi ({} vs {})", data.len(), baseline);
    }

    #[test]
    fn docx_export_works() {
        let dir = std::env::temp_dir().join("g360_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("test.docx");
        export_docx(&out, &sample_portfolio(), None, &sample_agent(), &OutputSettings::default(), &[]).unwrap();
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
        export_docx(&out, &sample_portfolio(), None, &sample_agent(), &OutputSettings::default(), &[cover]).unwrap();

        let file = std::fs::File::open(&out).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        assert!(zip.by_name("word/media/image1.jpeg").is_ok(), "kapak resmi gömülmedi");
        assert!(zip.by_name("word/document.xml").is_ok());
    }

    #[test]
    fn docx_gallery_grid() {
        // 5 resim: 1 kapak + 4 galeri (2 satır tablo)
        let mut imgs = Vec::new();
        for k in 0..5u8 {
            let img = image::RgbImage::from_fn(6, 6, |x, y| image::Rgb([k * 40, (x * 30) as u8, (y * 30) as u8]));
            let mut buf = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buf, image::ImageFormat::Jpeg).unwrap();
            imgs.push(buf.into_inner());
        }
        let dir = std::env::temp_dir().join("g360_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("gallery.docx");
        export_docx(&out, &sample_portfolio(), None, &sample_agent(), &OutputSettings::default(), &imgs).unwrap();

        let file = std::fs::File::open(&out).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        for i in 1..=5u32 {
            assert!(zip.by_name(&format!("word/media/image{i}.jpeg")).is_ok(), "image{i} yok");
        }
        use std::io::Read;
        let mut doc_bytes = Vec::new();
        zip.by_name("word/document.xml").unwrap().read_to_end(&mut doc_bytes).unwrap();
        let doc = String::from_utf8_lossy(&doc_bytes);
        assert!(doc.contains("<w:tbl>"), "galeri tablosu yok");
        assert!(doc.contains("rIdImg5"), "5. resim referansı yok");
    }

    #[test]
    fn shift_report_pdf_and_csv() {
        let rows = vec![
            ReportRow { name: "Ayşe Kaya".into(), planned_days: 20, planned_hours: 80.0, actual_days: 19, actual_hours: 76.0, approved_days: 18 },
            ReportRow { name: "Mehmet Demir".into(), planned_days: 10, planned_hours: 40.0, actual_days: 10, actual_hours: 40.0, approved_days: 10 },
        ];
        let dir = std::env::temp_dir().join("g360_export_test");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = export_shift_report_pdf(&dir.join("rapor.pdf"), "2026-11", &rows).unwrap();
        assert!(std::fs::read(&pdf).unwrap().starts_with(b"%PDF"));
        let csv = export_shift_report_csv(&dir.join("rapor.csv"), "2026-11", &rows).unwrap();
        let text = std::fs::read_to_string(&csv).unwrap();
        assert!(text.contains("Ayşe Kaya;20;80.0;19;76.0;18;95"), "csv satırı hatalı");
        assert!(text.starts_with('\u{feff}'), "BOM yok");
    }

    #[test]
    fn turkish_transliteration() {
        assert_eq!(tr_ascii("Çağrı Şölen Ünlü İğne"), "Cagri Solen Unlu Igne");
    }
}
