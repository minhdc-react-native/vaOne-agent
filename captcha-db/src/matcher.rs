use anyhow::{Result, bail};

use crate::{
    database::Database,
    glyph::extract_feature,
    models::{Glyph, GlyphFeature},
};

pub struct Matcher {
    db: Database,
}

impl Matcher {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Train thêm một captcha
    pub fn train(&mut self, glyphs: &[Glyph], text: &str) -> Result<()> {
        let chars: Vec<char> = text.chars().collect();

        if chars.len() != glyphs.len() {
            bail!(
                "Glyph count ({}) != text count ({})",
                glyphs.len(),
                chars.len()
            );
        }

        for (glyph, ch) in glyphs.iter().zip(chars.iter()) {
            let feature = extract_feature(glyph, Some(*ch));

            self.db.add(*ch, feature);
        }

        Ok(())
    }

    /// Nhận diện captcha
    pub fn recognize(&self, glyphs: &[Glyph]) -> String {
        let mut text = String::new();

        for glyph in glyphs {
            let feature = extract_feature(glyph, None);

            match self.match_one(&feature) {
                Some(ch) => text.push(ch),
                None => text.push('?'),
            }
        }

        text
    }

    /// Match một glyph
    fn match_one(&self, input: &GlyphFeature) -> Option<char> {
        //
        // 1. Hash giống hệt
        //
        for class in self.db.data.classes.values() {
            if class.samples.iter().any(|s| s.hash == input.hash) {
                return Some(class.ch);
            }
        }

        //
        // 2. Cosine similarity
        //
        let mut best_char = None;
        let mut best_score = -1.0f32;

        for class in self.db.data.classes.values() {
            for sample in &class.samples {
                let score = cosine_similarity(&sample.vector, &input.vector);

                if score > best_score {
                    best_score = score;
                    best_char = Some(class.ch);
                }
            }
        }

        if best_score >= 0.97 { best_char } else { None }
    }

    pub fn optimize(&mut self) {
        self.db.optimize();
    }

    pub fn glyph_count(&self) -> usize {
        self.db.len()
    }

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

/// Cosine similarity
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());

    if n == 0 {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;

    for i in 0..n {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }

    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }

    dot / (na.sqrt() * nb.sqrt())
}
