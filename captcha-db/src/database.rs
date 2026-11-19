use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::Result;

use crate::models::{CaptchaDatabase, GlyphClass, GlyphFeature};

const MAX_SAMPLE: usize = 5;
const DEFAULT_DB: &str = include_str!("../assets/default-db.json");

pub struct Database {
    pub data: CaptchaDatabase,
}

impl Database {
    pub fn new() -> Self {
        Self {
            data: CaptchaDatabase::default(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if path.exists() {
            let text = fs::read_to_string(path)?;
            let data: CaptchaDatabase = serde_json::from_str(&text)?;
            return Ok(Self { data });
        }

        // Chưa có database -> tạo từ dữ liệu mặc định
        let data: CaptchaDatabase = serde_json::from_str(DEFAULT_DB)?;

        // tạo thư mục cha nếu chưa có
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, serde_json::to_string_pretty(&data)?)?;

        Ok(Self { data })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        let json = serde_json::to_string_pretty(&self.data)?;

        match fs::write(path, json) {
            Ok(_) => {
                println!("write ok");
            }
            Err(e) => {
                println!("write err: {:?}", e);
            }
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.data.classes.clear();
    }

    /// Tổng số sample
    pub fn len(&self) -> usize {
        self.data.classes.values().map(|c| c.samples.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.data.classes.is_empty()
    }

    /// Toàn bộ class
    pub fn classes(&self) -> &HashMap<char, GlyphClass> {
        &self.data.classes
    }

    /// Thêm một sample
    pub fn add(&mut self, ch: char, feature: GlyphFeature) {
        let class = self.data.classes.entry(ch).or_insert_with(|| GlyphClass {
            ch,
            samples: Vec::new(),
        });

        //
        // Hash giống hệt -> bỏ
        //
        if class.samples.iter().any(|g| g.hash == feature.hash) {
            return;
        }

        //
        // Vector gần như giống hệt -> bỏ
        //
        if class
            .samples
            .iter()
            .any(|g| cosine_similarity(&g.vector, &feature.vector) > 0.995)
        {
            return;
        }

        //
        // Chưa đủ 5 mẫu
        //
        if class.samples.len() < MAX_SAMPLE {
            class.samples.push(feature);
            return;
        }

        //
        // Đã đủ 5 mẫu
        // Thay sample gần nhất nếu cần
        //
        let mut replace = None;
        let mut score = f32::MAX;

        for (i, sample) in class.samples.iter().enumerate() {
            let s = cosine_similarity(&sample.vector, &feature.vector);

            if s < score {
                score = s;
                replace = Some(i);
            }
        }

        if score < 0.98 {
            if let Some(i) = replace {
                class.samples[i] = feature;
            }
        }
    }

    pub fn stats(&self) -> HashMap<char, usize> {
        self.data
            .classes
            .iter()
            .map(|(c, cls)| (*c, cls.samples.len()))
            .collect()
    }

    /// Không cần optimize nữa vì add() đã chống trùng
    pub fn optimize(&mut self) {
        for class in self.data.classes.values_mut() {
            let mut seen = HashSet::new();

            class.samples.retain(|g| seen.insert(g.hash.clone()));
        }
    }

    /// Trả về toàn bộ sample của một ký tự
    pub fn samples_of(&self, ch: char) -> Option<&Vec<GlyphFeature>> {
        self.data.classes.get(&ch).map(|c| &c.samples)
    }

    /// Lấy toàn bộ sample
    pub fn all_samples(&self) -> Vec<&GlyphFeature> {
        self.data
            .classes
            .values()
            .flat_map(|c| c.samples.iter())
            .collect()
    }

    /// Candidate có hash giống trước
    pub fn find_candidates<'a>(&'a self, input: &GlyphFeature) -> Vec<&'a GlyphFeature> {
        let mut result = Vec::new();

        for class in self.data.classes.values() {
            for sample in &class.samples {
                if sample.hash == input.hash {
                    result.push(sample);
                }
            }
        }

        if !result.is_empty() {
            return result;
        }

        self.data
            .classes
            .values()
            .flat_map(|c| c.samples.iter())
            .collect()
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
