use std::path::{Path, PathBuf};
use std::fs;
use std::borrow::Cow;
use pdf::object::*;
use pdf::font::{Font as PdfFont};
use pdf::error::{Result, PdfError};

use font::{self};
use std::sync::Arc;
use super::FontEntry;
use cachelib::sync::SyncCache;

pub static STANDARD_FONTS: &[(&'static str, &'static str)] = &[
    ("Courier", "CourierStd.otf"),
    ("Courier-Bold", "CourierStd-Bold.otf"),
    ("Courier-Oblique", "CourierStd-Oblique.otf"),
    ("Courier-BoldOblique", "CourierStd-BoldOblique.otf"),
    
    ("Times-Roman", "MinionPro-Regular.otf"),
    ("Times-Bold", "MinionPro-Bold.otf"),
    ("Times-Italic", "MinionPro-It.otf"),
    ("Times-BoldItalic", "MinionPro-BoldIt.otf"),
    
    ("Helvetica", "MyriadPro-Regular.otf"),
    ("Helvetica-Bold", "MyriadPro-Bold.otf"),
    ("Helvetica-Oblique", "MyriadPro-It.otf"),
    ("Helvetica-BoldOblique", "MyriadPro-BoldIt.otf"),
    
    ("Symbol", "SY______.PFB"),
    ("ZapfDingbats", "AdobePiStd.otf"),
    
    ("Arial-BoldMT", "Arial-BoldMT.otf"),
    ("ArialMT", "ArialMT.ttf"),
    ("Arial-ItalicMT", "Arial-ItalicMT.otf"),
];

type FontRc = Arc<dyn font::Font + Send + Sync>;
pub struct StandardCache {
    inner: SyncCache<usize, Option<FontRc>>
}
impl StandardCache {
    pub fn new() -> Self {
        StandardCache { inner: SyncCache::new() }
    }
}

pub fn load_font(font_ref: Ref<PdfFont>, resolve: &impl Resolve, standard_fonts: &Path, cache: &StandardCache) -> Result<Option<FontEntry>> {
    let pdf_font = resolve.get(font_ref)?;
    debug!("loading {:?}", pdf_font);
    
    let font: FontRc = match pdf_font.embedded_data(resolve) {
        Some(Ok(data)) => {
            let font = font::parse(&data).map_err(|e| {
                let name = format!("font_{}", pdf_font.name.as_ref().map(|s| s.as_str()).unwrap_or("unnamed"));
                std::fs::write(&name, &data).unwrap();
                println!("font dumped in {}", name);
                PdfError::Other { msg: format!("Font Error: {:?}", e) }
            })?;
            FontRc::from(font)
        }
        Some(Err(e)) => return Err(e),
        None => {
            match STANDARD_FONTS.iter().enumerate().find(|(_, &(name, _))| pdf_font.name.as_ref().map(|s| s == name).unwrap_or(false)) {
                Some((i, &(_, file_name))) => {
                    let val = cache.inner.get(i, || {
                        let data = match std::fs::read(standard_fonts.join(file_name)) {
                            Ok(data) => data,
                            Err(e) => {
                                warn!("can't open {} for {:?} {:?}", file_name, pdf_font.name, e);
                                return None;
                            }
                        };
                        match font::parse(&data) {
                            Ok(f) => Some(f.into()),
                            Err(e) => {
                                warn!("Font Error: {:?}", e);
                                return None;
                            }
                        }
                    });
                    match val {
                        Some(f) => f,
                        None => {
                            return Ok(None);
                        }
                    }
                }
                None => {
                    warn!("no font for {:?}", pdf_font.name);
                    return Ok(None);
                }
            }
        }
    };

    Ok(Some(FontEntry::build(font, pdf_font, resolve)?))
}