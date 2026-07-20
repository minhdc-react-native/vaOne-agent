use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const DATABASE_NAME: &str = "captcha-db.json";

/// =========================
/// Geometry
/// =========================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min_x: f64::MAX,
            min_y: f64::MAX,
            max_x: f64::MIN,
            max_y: f64::MIN,
        }
    }
}

impl BoundingBox {
    pub fn update(&mut self, p: Point) {
        self.min_x = self.min_x.min(p.x);
        self.min_y = self.min_y.min(p.y);
        self.max_x = self.max_x.max(p.x);
        self.max_y = self.max_y.max(p.y);
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }
}

/// =========================
/// SVG Commands
/// =========================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathCommand {
    MoveTo(Point),

    LineTo(Point),

    Horizontal(f64),

    Vertical(f64),

    QuadTo {
        control: Point,
        end: Point,
    },

    CubicTo {
        control1: Point,
        control2: Point,
        end: Point,
    },

    Close,
}

/// =========================
/// Glyph sau khi parse
/// =========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    /// path gốc trong SVG
    pub raw_path: String,

    /// command sau parse
    pub commands: Vec<PathCommand>,

    /// bounding box
    pub bbox: BoundingBox,

    /// SHA256 của normalized path
    pub fingerprint: Option<String>,
}

/// SVG sau khi parse
#[derive(Debug, Clone)]
pub struct ParsedSvg {
    pub glyphs: Vec<Glyph>,
}

/// =========================
/// Feature dùng Matcher V2
/// =========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphFeature {
    pub hash: String,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphClass {
    pub ch: char,

    /// tối đa 5 mẫu
    pub samples: Vec<GlyphFeature>,
}

/// Database học được
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaptchaDatabase {
    pub classes: HashMap<char, GlyphClass>,
}

/// =========================
/// Kết quả Match
/// =========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// Ví dụ: 8XFQFE
    pub text: String,

    /// fingerprint từng glyph
    pub fingerprints: Vec<String>,

    /// index những glyph chưa học
    pub unknown: Vec<usize>,
}

/// =========================
/// Train
/// =========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainRequest {
    /// SVG gốc
    pub svg: String,

    /// captcha đúng
    pub captcha: String,
}
