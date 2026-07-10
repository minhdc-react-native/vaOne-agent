use crate::pdf::table::models::TableRowLayout;
use printpdf::*;
use std::collections::HashSet;

use crate::pdf::{
    fonts::PdfFonts, models::ElementStyle, table::models::TableLayoutResult, utils::Unit,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EdgeKind {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Edge {
    kind: EdgeKind,

    x1: i32,
    y1: i32,

    x2: i32,
    y2: i32,
}

fn px(v: f32) -> i32 {
    (v * 100.0).round() as i32
}

#[derive(Clone, Copy, Debug)]
pub struct HLine {
    pub y: i32,
    pub x1: i32,
    pub x2: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct VLine {
    pub x: i32,
    pub y1: i32,
    pub y2: i32,
}

pub struct TableBorder;

impl TableBorder {
    fn collect_edges(layout: &TableLayoutResult) -> HashSet<Edge> {
        let mut edges = HashSet::<Edge>::new();

        for row in &layout.headers {
            Self::collect_row_edges(&row, &mut edges);
        }

        for row in &layout.rows {
            Self::collect_row_edges(&row, &mut edges);
        }

        edges
    }

    fn collect_row_edges(row: &TableRowLayout, edges: &mut HashSet<Edge>) {
        for cell in &row.cells {
            let x_start = px(cell.x);
            let y_start = px(cell.y);
            let x_end = px(cell.x + cell.width);
            let y_end = px(cell.y + cell.height);

            Self::insert_edge(
                edges,
                Edge {
                    kind: EdgeKind::Top,
                    x1: x_start,
                    y1: y_start,
                    x2: x_end,
                    y2: y_start,
                },
            );

            Self::insert_edge(
                edges,
                Edge {
                    kind: EdgeKind::Bottom,
                    x1: x_start,
                    y1: y_end,
                    x2: x_end,
                    y2: y_end,
                },
            );

            Self::insert_edge(
                edges,
                Edge {
                    kind: EdgeKind::Left,
                    x1: x_start,
                    y1: y_start,
                    x2: x_start,
                    y2: y_end,
                },
            );

            Self::insert_edge(
                edges,
                Edge {
                    kind: EdgeKind::Right,
                    x1: x_end,
                    y1: y_start,
                    x2: x_end,
                    y2: y_end,
                },
            );
        }
    }

    fn insert_edge(edges: &mut HashSet<Edge>, edge: Edge) {
        edges.insert(edge);
    }

    fn unique_edges(edges: HashSet<Edge>) -> Vec<Edge> {
        edges.into_iter().collect()
    }

    fn merge_horizontal(edges: &[Edge]) -> Vec<HLine> {
        //--------------------------------------------------
        // Convert
        //--------------------------------------------------

        let mut lines = Vec::<HLine>::new();

        for e in edges {
            if e.y1 == e.y2 {
                let (x1, x2) = if e.x1 <= e.x2 {
                    (e.x1, e.x2)
                } else {
                    (e.x2, e.x1)
                };

                lines.push(HLine { y: e.y1, x1, x2 });
            }
        }

        //--------------------------------------------------
        // Sort
        //--------------------------------------------------

        lines.sort_unstable_by(|a, b| a.y.cmp(&b.y).then(a.x1.cmp(&b.x1)).then(a.x2.cmp(&b.x2)));

        //--------------------------------------------------
        // Merge
        //--------------------------------------------------

        let mut result = Vec::<HLine>::new();

        for line in lines {
            if let Some(last) = result.last_mut() {
                if last.y == line.y && line.x1 <= last.x2 {
                    last.x2 = last.x2.max(line.x2);
                    continue;
                }
            }

            result.push(line);
        }

        result
    }

    fn merge_vertical(edges: &[Edge]) -> Vec<VLine> {
        //--------------------------------------------------
        // Convert
        //--------------------------------------------------

        let mut lines = Vec::<VLine>::new();

        for e in edges {
            if e.x1 == e.x2 {
                let (y1, y2) = if e.y1 <= e.y2 {
                    (e.y1, e.y2)
                } else {
                    (e.y2, e.y1)
                };

                lines.push(VLine { x: e.x1, y1, y2 });
            }
        }

        //--------------------------------------------------
        // Sort
        //--------------------------------------------------
        lines.sort_unstable_by(|a, b| a.x.cmp(&b.x).then(a.y1.cmp(&b.y1)).then(a.y2.cmp(&b.y2)));

        //--------------------------------------------------
        // Merge
        //--------------------------------------------------

        let mut result = Vec::<VLine>::new();

        for line in lines {
            if let Some(last) = result.last_mut() {
                if last.x == line.x && line.y1 <= last.y2 {
                    last.y2 = last.y2.max(line.y2);
                    continue;
                }
            }

            result.push(line);
        }

        result
    }

    fn draw_horizontal(ops: &mut Vec<Op>, lines: &[HLine], page_height: f32) {
        for line in lines {
            let y = Unit::px100_to_mm((page_height * 100.0).round() as i32 - line.y);

            ops.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point::new(Unit::px100_to_mm(line.x1), y),
                            bezier: false,
                        },
                        LinePoint {
                            p: Point::new(Unit::px100_to_mm(line.x2), y),
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                },
            });
        }
    }

    fn draw_vertical(ops: &mut Vec<Op>, lines: &[VLine], page_height: f32) {
        let page = (page_height * 100.0).round() as i32;

        for line in lines {
            let x = Unit::px100_to_mm(line.x);

            let y1 = Unit::px100_to_mm(page - line.y1);

            let y2 = Unit::px100_to_mm(page - line.y2);

            ops.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point::new(x, y1),
                            bezier: false,
                        },
                        LinePoint {
                            p: Point::new(x, y2),
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                },
            });
        }
    }

    pub fn draw(
        ops: &mut Vec<Op>,
        fonts: &PdfFonts,
        layout: &TableLayoutResult,
        style: &ElementStyle,
        page_height: f32,
    ) {
        //----------------------------------------------------------
        // Không có border
        //----------------------------------------------------------

        let width = style.border_width.unwrap_or(0.0);

        if width <= 0.0 {
            return;
        }

        //----------------------------------------------------------
        // Color
        //----------------------------------------------------------

        if let Some(color) = &style.border_color {
            ops.push(Op::SetOutlineColor {
                col: fonts.parse_color(color),
            });
        }

        //----------------------------------------------------------
        // Width
        //----------------------------------------------------------

        ops.push(Op::SetOutlineThickness {
            pt: Unit::px_to_pt(width),
        });

        //----------------------------------------------------------
        // Dash
        //----------------------------------------------------------

        match style.border_style.as_deref() {
            Some("dashed") => {
                ops.push(Op::SetLineDashPattern {
                    dash: LineDashPattern {
                        offset: 0,

                        dash_1: Some(2),

                        gap_1: Some(2),

                        dash_2: None,
                        gap_2: None,

                        dash_3: None,
                        gap_3: None,
                    },
                });
            }

            Some("dotted") => {
                ops.push(Op::SetLineDashPattern {
                    dash: LineDashPattern {
                        offset: 0,

                        dash_1: Some(1),

                        gap_1: Some(2),

                        dash_2: None,
                        gap_2: None,

                        dash_3: None,
                        gap_3: None,
                    },
                });
            }

            _ => {
                ops.push(Op::SetLineDashPattern {
                    dash: LineDashPattern::default(),
                });
            }
        }

        //----------------------------------------------------------
        // Collect
        //----------------------------------------------------------

        let edges = Self::collect_edges(layout);

        //----------------------------------------------------------
        // Unique
        //----------------------------------------------------------

        let edges = Self::unique_edges(edges);

        //----------------------------------------------------------
        // Merge
        //----------------------------------------------------------

        let hlines = Self::merge_horizontal(&edges);

        let vlines = Self::merge_vertical(&edges);

        //----------------------------------------------------------
        // Draw
        //----------------------------------------------------------
        Self::draw_horizontal(ops, &hlines, page_height);
        Self::draw_vertical(ops, &vlines, page_height);
    }
}
