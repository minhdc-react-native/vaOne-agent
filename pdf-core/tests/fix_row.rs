use pdf_core::{
    pagination::paginator::{PageItem, Paginator},
    table::{models::TableElement, table_layout::TableLayoutEngine, table_row::TableRow},
    template::models::FormatterContext,
    utils::load_fonts,
};
use printpdf::PdfDocument;
use serde_json::{json, Value};

fn table_json() -> Value {
    let columns = json!([
        {"id":"qty", "header":"SL", "fieldName":"quantity", "width":50, "formatString":"SLG"},
        {"id":"price", "header":"Giá", "fieldName":"price", "width":100, "formatString":"GIA"},
        {"id":"discount", "header":"CK", "fieldName":"discount", "width":40, "formatString":"TIEN"},
        {"id":"amount", "header":"T.Tiền", "fieldName":"amount", "width":"auto", "formatString":"TIEN"}
    ]);
    json!({
        "x":13, "y":20, "width":261, "height":71, "fieldName":"orderDetails",
        "style":{"fontSize":11, "borderStyle":"none"},
        "columns":columns,
        "fixRow":{"row":2,"data":[
            {"columns":[{"id":"qty", "header":"SL", "fieldName":"productName", "width":"auto", "colSpan":4, "formatString":""}]},
            {"columns":columns}
        ]}
    })
}

fn data() -> Value {
    json!({"orderDetails":[
        {"productName":"Ấm siêu tốc", "quantity":1, "price":980000, "discount":0, "amount":980000},
        {"productName":"Bình giữ nhiệt", "quantity":2, "price":125000, "discount":0, "amount":250000}
    ]})
}

fn layout(table: &TableElement, data: &Value) -> pdf_core::table::models::TableLayoutResult {
    let mut pdf = PdfDocument::new("fixRow test");
    let fonts = load_fonts(&mut pdf).unwrap();
    TableLayoutEngine::build(&fonts, 600.0, table, data, FormatterContext::default())
}

#[test]
fn binds_each_item_as_one_two_line_group_with_a_full_width_name() {
    let table: TableElement = serde_json::from_value(table_json()).unwrap();
    let result = layout(&table, &data());
    assert_eq!(result.rows.len(), 2);
    for (row, name) in result.rows.iter().zip(["Ấm siêu tốc", "Bình giữ nhiệt"]) {
        assert_eq!(row.cells.len(), 5);
        let title = &row.cells[0];
        assert_eq!(title.content, name);
        assert_eq!(title.col_span, 4);
        assert_eq!(title.width, table.width);
        assert_eq!(title.x, table.x);
        for cell in &row.cells[1..] {
            assert!((cell.y - title.y - title.height).abs() < 0.01);
            assert!(cell.y + cell.height <= row.y + row.height + 0.01);
        }
    }
    assert_eq!(result.rows[0].cells[1].content, "1,00");
    assert_eq!(result.rows[0].cells[2].content, "980.000");
    assert_eq!(result.rows[1].cells[1].content, "2,00");
    assert!((result.rows[1].y - result.rows[0].y - result.rows[0].height).abs() < 0.01);
    assert!((result.height - result.header_height() - result.rows_height()).abs() < 0.01);
}

#[test]
fn long_name_wraps_and_moves_both_its_details_and_the_next_product() {
    let table: TableElement = serde_json::from_value(table_json()).unwrap();
    let short = layout(&table, &data());
    let mut long_data = data();
    long_data["orderDetails"][0]["productName"] =
        json!("Ấm siêu tốc Silver Crest 1.7L - 3000W - màu xanh ".repeat(4));
    let long = layout(&table, &long_data);
    assert!(long.rows[0].cells[0].height > short.rows[0].cells[0].height);
    assert!(long.rows[0].cells[1].y > short.rows[0].cells[1].y);
    assert!(long.rows[1].y > short.rows[1].y);
    assert!((long.rows[0].y + long.rows[0].height - long.rows[1].y).abs() < 0.01);
}

#[test]
fn absent_or_empty_fixrow_preserves_the_regular_table() {
    let mut config = table_json();
    config.as_object_mut().unwrap().remove("fixRow");
    let regular: TableElement = serde_json::from_value(config.clone()).unwrap();
    assert_eq!(regular.columns[0].col_span, 1);
    let result = layout(&regular, &data());
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].cells.len(), 4);
    assert_eq!(result.rows[0].cells[0].content, "1,00");
    config["fixRow"] = json!({"row":2,"data":[]});
    let empty: TableElement = serde_json::from_value(config).unwrap();
    let empty_result = layout(&empty, &data());
    assert_eq!(empty_result.height, result.height);
    assert_eq!(empty_result.rows[0].cells.len(), 4);
}

#[test]
fn blank_format_keeps_text_while_currency_still_formats() {
    for format in [None, Some(String::new()), Some("  ".into())] {
        assert_eq!(
            TableRow::apply_format(FormatterContext::default(), "Ấm siêu tốc".into(), &format),
            "Ấm siêu tốc"
        );
    }
    assert_eq!(
        TableRow::apply_format(
            FormatterContext::default(),
            "980000".into(),
            &Some("TIEN".into())
        ),
        "980.000"
    );
}

#[test]
fn spans_are_bounded_and_empty_data_emits_no_product_groups() {
    let mut config = table_json();
    config["fixRow"]["data"][0]["columns"][0]["colSpan"] = json!(999);
    let table: TableElement = serde_json::from_value(config).unwrap();
    assert_eq!(layout(&table, &data()).rows[0].cells[0].width, table.width);
    assert!(layout(&table, &json!({"orderDetails":[]})).rows.is_empty());
}

#[test]
fn pagination_repeats_headers_and_never_splits_a_product_group() {
    let table: TableElement = serde_json::from_value(table_json()).unwrap();
    let mut many = data();
    many["orderDetails"] = Value::Array(
        (0..20)
            .map(|i| {
                let mut item = data()["orderDetails"][0].clone();
                item["productName"] = json!(format!("Product {i}"));
                item
            })
            .collect(),
    );
    let built = layout(&table, &many);
    let (pages, _) = Paginator::paginate(
        vec![PageItem::Table {
            element: table,
            layout: built,
        }],
        287.0,
        320.0,
        false,
    )
    .unwrap();
    assert!(pages.len() > 1);
    let mut index = 0;
    for page in pages {
        for item in page.items {
            if let PageItem::Table { layout, .. } = item {
                assert!(!layout.headers.is_empty());
                assert!(layout.bottom() <= 320.0 - 72.0 + 0.01);
                for row in layout.rows {
                    assert_eq!(row.cells.len(), 5);
                    assert_eq!(row.cells[0].content, format!("Product {index}"));
                    assert!(row
                        .cells
                        .iter()
                        .all(|cell| cell.y >= row.y
                            && cell.y + cell.height <= row.y + row.height + 0.01));
                    index += 1;
                }
            }
        }
    }
    assert_eq!(index, 20);
}

#[test]
fn first_group_near_page_bottom_moves_intact_to_available_space() {
    let mut table: TableElement = serde_json::from_value(table_json()).unwrap();
    table.y = 240.0;
    let built = layout(&table, &data());
    let (pages, _) = Paginator::paginate(
        vec![PageItem::Table {
            element: table,
            layout: built,
        }],
        287.0,
        320.0,
        false,
    )
    .unwrap();
    for page in pages {
        for item in page.items {
            if let PageItem::Table { layout, .. } = item {
                assert!(layout.bottom() <= 248.01);
                assert!(!layout.rows.is_empty());
                assert_eq!(layout.rows[0].cells.len(), 5);
            }
        }
    }
}

#[test]
fn static_fixrow_still_binds_against_root_data() {
    let mut config = table_json();
    config["fieldName"] = json!("");
    config["fixRow"] = json!({"row":1,"data":[{"columns":[
        {"id":"qty", "header":"", "fieldName":"caption", "formatString":""},
        {"id":"price", "header":"", "fieldName":"totalAmount", "formatString":"TIEN"}
    ]}]});
    let table: TableElement = serde_json::from_value(config).unwrap();
    let built = layout(
        &table,
        &json!({"caption":"Tổng cộng", "totalAmount":2130000}),
    );
    assert!(built.headers.is_empty());
    assert_eq!(built.rows.len(), 1);
    assert_eq!(built.rows[0].cells[0].content, "Tổng cộng");
    assert_eq!(built.rows[0].cells[1].content, "2.130.000");
}
