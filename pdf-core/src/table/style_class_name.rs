use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::models::ElementStyle;

static TAILWIND_COLORS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("black", "#000000"),
        ("white", "#FFFFFF"),
        ("gray-400", "#9CA3AF"),
        ("gray-500", "#6B7280"),
        ("gray-600", "#4B5563"),
        ("gray-700", "#374151"),
        ("gray-800", "#1F2937"),
        ("gray-900", "#111827"),
        ("red-500", "#EF4444"),
        ("red-600", "#DC2626"),
        ("green-500", "#22C55E"),
        ("green-600", "#16A34A"),
        ("blue-500", "#3B82F6"),
        ("blue-600", "#2563EB"),
        ("yellow-500", "#EAB308"),
        ("orange-500", "#F97316"),
        ("purple-500", "#A855F7"),
        ("pink-500", "#EC4899"),
        ("cyan-500", "#06B6D4"),
        ("emerald-500", "#10B981"),
        ("indigo-500", "#6366F1"),
        ("violet-500", "#8B5CF6"),
    ])
});

pub fn class_name_to_style(class_name: &str, style: &mut ElementStyle) -> ElementStyle {
    let classes: Vec<&str> = class_name
        .split_whitespace()
        .map(|c| {
            c.trim_start_matches("hover:")
                .trim_start_matches("md:")
                .trim_start_matches("lg:")
                .trim_start_matches('!')
        })
        .collect();

    // ==========================
    // font weight
    // ==========================
    if classes.iter().any(|c| {
        matches!(
            *c,
            "font-semibold" | "font-bold" | "font-extrabold" | "font-black"
        )
    }) {
        style.font_weight = Some("bold".to_string());
    } else {
        style.font_weight = Some("normal".to_string());
    }

    // ==========================
    // italic
    // ==========================
    style.font_style = Some(
        if classes.contains(&"italic") {
            "italic"
        } else {
            "normal"
        }
        .to_string(),
    );

    // ==========================
    // text decoration
    // ==========================
    style.text_decoration = Some(
        if classes.contains(&"underline") {
            "underline"
        } else if classes.contains(&"line-through") {
            "line-through"
        } else {
            "none"
        }
        .to_string(),
    );

    // ==========================
    // font size
    // ==========================
    let size = classes.iter().find_map(|c| match *c {
        "text-xs" => Some(9.0),
        "text-sm" => Some(10.0),
        "text-base" => Some(11.0),
        "text-lg" => Some(12.0),
        "text-xl" => Some(14.0),
        "text-2xl" => Some(16.0),
        "text-3xl" => Some(18.0),
        "text-4xl" => Some(22.0),
        _ => None,
    });

    style.font_size = size;

    // ==========================
    // text color
    // ==========================
    if let Some(cls) = classes.iter().find(|c| c.starts_with("text-")) {
        let color = cls.trim_start_matches("text-");

        if color.starts_with("[#") && color.ends_with(']') {
            style.color = Some(color[1..color.len() - 1].to_string());
        } else if let Some(v) = TAILWIND_COLORS.get(color) {
            style.color = Some((*v).to_string());
        }
    }

    // ==========================
    // background
    // ==========================
    if let Some(cls) = classes.iter().find(|c| c.starts_with("bg-")) {
        let color = cls.trim_start_matches("bg-");

        if color.starts_with("[#") && color.ends_with(']') {
            style.background_color = Some(color[1..color.len() - 1].to_string());
        } else if color == "transparent" {
            style.background_color = Some("transparent".to_string());
        } else if let Some(v) = TAILWIND_COLORS.get(color) {
            style.background_color = Some((*v).to_string());
        }
    }

    style.clone()
}
