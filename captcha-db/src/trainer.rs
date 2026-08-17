use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, bail};

use crate::{database::Database, glyph::extract_feature, svg::parse_svg};

pub struct Trainer {
    db: Database,
}

impl Trainer {
    /// Database rỗng
    pub fn new() -> Self {
        Self {
            db: Database::new(),
        }
    }

    /// Load database
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            db: Database::load(path)?,
        })
    }

    /// Train một captcha
    pub fn train(&mut self, svg: &str, text: &str) -> Result<()> {
        let mut glyphs = parse_svg(svg)?;

        let chars: Vec<char> = text.chars().collect();

        if glyphs.len() != chars.len() {
            bail!(
                "Glyph count ({}) != text count ({})",
                glyphs.len(),
                chars.len()
            );
        }

        for (glyph, ch) in glyphs.iter_mut().zip(chars.iter()) {
            let feature = extract_feature(glyph, Some(*ch));

            self.db.add(*ch, feature);
        }

        Ok(())
    }

    /// Train nhiều captcha
    pub fn train_many<I>(&mut self, items: I) -> Result<()>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        for (svg, text) in items {
            self.train(&svg, &text)?;
        }

        Ok(())
    }

    /// Lưu database
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.db.optimize();
        self.db.save(path)
    }

    /// Xóa toàn bộ database
    pub fn clear(&mut self) {
        self.db.clear();
    }

    /// Tổng số ký tự đã học
    pub fn glyph_count(&self) -> usize {
        self.db.len()
    }

    /// Thống kê
    pub fn stats(&self) -> HashMap<char, usize> {
        self.db.stats()
    }

    /// Lấy Database
    pub fn database(&self) -> &Database {
        &self.db
    }

    pub fn database_mut(&mut self) -> &mut Database {
        &mut self.db
    }

    pub fn into_database(self) -> Database {
        self.db
    }
}
