use std::collections::HashMap;

use printpdf::*;

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
    fn collect_edges(layout: &TableLayoutResult) -> HashMap<Edge, usize> {
        let mut edges = HashMap::<Edge, usize>::new();

        for row in &layout.rows {
            for cell in &row.cells {
                //--------------------------------------------------
                // Top
                //--------------------------------------------------

                Self::insert_edge(
                    &mut edges,
                    Edge {
                        kind: EdgeKind::Top,

                        x1: px(cell.x),
                        y1: px(cell.y),

                        x2: px(cell.x + cell.width),
                        y2: px(cell.y),
                    },
                );

                //--------------------------------------------------
                // Bottom
                //--------------------------------------------------

                Self::insert_edge(
                    &mut edges,
                    Edge {
                        kind: EdgeKind::Bottom,

                        x1: px(cell.x),
                        y1: px(cell.y + cell.height),

                        x2: px(cell.x + cell.width),
                        y2: px(cell.y + cell.height),
                    },
                );

                //--------------------------------------------------
                // Left
                //--------------------------------------------------

                Self::insert_edge(
                    &mut edges,
                    Edge {
                        kind: EdgeKind::Left,

                        x1: px(cell.x),
                        y1: px(cell.y),

                        x2: px(cell.x),
                        y2: px(cell.y + cell.height),
                    },
                );

                //--------------------------------------------------
                // Right
                //--------------------------------------------------

                Self::insert_edge(
                    &mut edges,
                    Edge {
                        kind: EdgeKind::Right,

                        x1: px(cell.x + cell.width),
                        y1: px(cell.y),

                        x2: px(cell.x + cell.width),
                        y2: px(cell.y + cell.height),
                    },
                );
            }
        }

        edges
    }

    fn insert_edge(edges: &mut HashMap<Edge, usize>, edge: Edge) {
        *edges.entry(edge).or_insert(0) += 1;
    }

    fn unique_edges(edges: HashMap<Edge, usize>) -> Vec<Edge> {
        edges.into_iter().map(|(e, _)| e).collect()
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

        lines.sort_by(|a, b| a.y.cmp(&b.y).then(a.x1.cmp(&b.x1)).then(a.x2.cmp(&b.x2)));

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

        lines.sort_by(|a, b| a.x.cmp(&b.x).then(a.y1.cmp(&b.y1)).then(a.y2.cmp(&b.y2)));

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
            let y = page_height - line.y as f32 / 100.0;
            ops.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point::new(
                                Unit::px_to_mm(line.x1 as f32 / 100.0),
                                Unit::px_to_mm(y),
                            ),

                            bezier: false,
                        },
                        LinePoint {
                            p: Point::new(
                                Unit::px_to_mm(line.x2 as f32 / 100.0),
                                Unit::px_to_mm(y),
                            ),

                            bezier: false,
                        },
                    ],

                    is_closed: false,
                },
            });
        }
    }

    fn draw_vertical(ops: &mut Vec<Op>, lines: &[VLine], page_height: f32) {
        for line in lines {
            let y1 = page_height - line.y1 as f32 / 100.0;
            let y2 = page_height - line.y2 as f32 / 100.0;
            ops.push(Op::DrawLine {
                line: Line {
                    points: vec![
                        LinePoint {
                            p: Point::new(
                                Unit::px_to_mm(line.x as f32 / 100.0),
                                Unit::px_to_mm(y1),
                            ),

                            bezier: false,
                        },
                        LinePoint {
                            p: Point::new(
                                Unit::px_to_mm(line.x as f32 / 100.0),
                                Unit::px_to_mm(y2),
                            ),

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
            pt: Pt(width / 2.0),
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
