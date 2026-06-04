#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{
    self, Align, Color32, ComboBox, FontFamily, FontId, Key, Layout, RichText, ScrollArea, Stroke,
    TextEdit, TextStyle, Ui, Vec2,
};
use std::cmp::{max, min};
use std::collections::HashMap;

const ROWS: usize = 80;
const COLS: usize = 26;
const CELL_WIDTH: f32 = 116.0;
const CELL_HEIGHT: f32 = 30.0;
const ROW_HEADER_WIDTH: f32 = 46.0;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Modern Ops Spreadsheet")
            .with_inner_size([1380.0, 860.0])
            .with_min_inner_size([1060.0, 680.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Modern Ops Spreadsheet",
        options,
        Box::new(|cc| Ok(Box::new(SpreadsheetApp::new(cc)))),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CellAddress {
    row: usize,
    col: usize,
}

impl CellAddress {
    const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CellRange {
    start: CellAddress,
    end: CellAddress,
}

impl CellRange {
    const fn single(address: CellAddress) -> Self {
        Self {
            start: address,
            end: address,
        }
    }

    fn normalized(self) -> Self {
        Self {
            start: CellAddress::new(
                min(self.start.row, self.end.row),
                min(self.start.col, self.end.col),
            ),
            end: CellAddress::new(
                max(self.start.row, self.end.row),
                max(self.start.col, self.end.col),
            ),
        }
    }

    fn contains(self, address: CellAddress) -> bool {
        let normalized = self.normalized();
        address.row >= normalized.start.row
            && address.row <= normalized.end.row
            && address.col >= normalized.start.col
            && address.col <= normalized.end.col
    }

    fn width(self) -> usize {
        let normalized = self.normalized();
        normalized.end.col - normalized.start.col + 1
    }

    fn height(self) -> usize {
        let normalized = self.normalized();
        normalized.end.row - normalized.start.row + 1
    }
}

#[derive(Clone)]
struct SpreadsheetCell {
    raw: String,
    style: CellStyle,
}

impl Default for SpreadsheetCell {
    fn default() -> Self {
        Self {
            raw: String::new(),
            style: CellStyle::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CellStyle {
    bold: bool,
    italic: bool,
    fill: FillColor,
    format: NumberFormat,
    align: CellAlign,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            fill: FillColor::None,
            format: NumberFormat::Auto,
            align: CellAlign::Left,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FillColor {
    None,
    Header,
    Blue,
    Green,
    Amber,
    Rose,
}

impl FillColor {
    fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Header => "Header",
            Self::Blue => "Blue",
            Self::Green => "Green",
            Self::Amber => "Amber",
            Self::Rose => "Rose",
        }
    }

    fn color(self) -> Color32 {
        match self {
            Self::None => palette::CELL,
            Self::Header => Color32::from_rgb(36, 48, 63),
            Self::Blue => Color32::from_rgb(23, 52, 78),
            Self::Green => Color32::from_rgb(21, 62, 47),
            Self::Amber => Color32::from_rgb(78, 54, 22),
            Self::Rose => Color32::from_rgb(77, 35, 45),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NumberFormat {
    Auto,
    Number,
    Currency,
    Percent,
}

impl NumberFormat {
    fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Number => "Number",
            Self::Currency => "Currency",
            Self::Percent => "Percent",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellAlign {
    Left,
    Center,
    Right,
}

impl CellAlign {
    fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Center => "Center",
            Self::Right => "Right",
        }
    }
}

#[derive(Clone, Debug)]
enum CalcValue {
    Empty,
    Number(f64),
    Text(String),
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn label(self) -> &'static str {
        match self {
            Self::Ascending => "Ascending",
            Self::Descending => "Descending",
        }
    }
}

struct SpreadsheetApp {
    cells: Vec<Vec<SpreadsheetCell>>,
    selected: CellAddress,
    range: CellRange,
    range_text: String,
    formula_input: String,
    sheet_name: String,
    workbook_title: String,
    status: String,
    clipboard: String,
    csv_buffer: String,
    find_query: String,
    replace_query: String,
    range_name_input: String,
    sort_direction: SortDirection,
    named_ranges: HashMap<String, CellRange>,
    show_formulas: bool,
    show_headers: bool,
    zoom: f32,
}

impl SpreadsheetApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);

        let mut app = Self {
            cells: vec![vec![SpreadsheetCell::default(); COLS]; ROWS],
            selected: CellAddress::new(0, 0),
            range: CellRange::single(CellAddress::new(0, 0)),
            range_text: "A1".to_owned(),
            formula_input: String::new(),
            sheet_name: "Operations Plan".to_owned(),
            workbook_title: "Modern Ops Spreadsheet".to_owned(),
            status: "Ready".to_owned(),
            clipboard: String::new(),
            csv_buffer: String::new(),
            find_query: String::new(),
            replace_query: String::new(),
            range_name_input: "Selection".to_owned(),
            sort_direction: SortDirection::Ascending,
            named_ranges: HashMap::new(),
            show_formulas: false,
            show_headers: true,
            zoom: 1.0,
        };

        app.load_sample_workbook();
        app
    }

    fn load_sample_workbook(&mut self) {
        self.clear_workbook();
        self.sheet_name = "Operations Plan".to_owned();
        self.workbook_title = "Modern Ops Spreadsheet".to_owned();

        let headers = [
            "Project",
            "Owner",
            "Budget",
            "Spent",
            "Remaining",
            "Progress",
            "Risk",
            "Forecast",
            "Notes",
        ];
        for (col, value) in headers.iter().enumerate() {
            self.set_raw(0, col, value);
            self.cells[0][col].style.bold = true;
            self.cells[0][col].style.fill = FillColor::Header;
            self.cells[0][col].style.align = CellAlign::Center;
        }

        let rows = [
            [
                "Retail rollout",
                "Nora",
                "45000",
                "31800",
                "=C2-D2",
                "0.71",
                "Medium",
                "=C2*(1+F2*0.12)",
                "POS migration in flight",
            ],
            [
                "Warehouse scanners",
                "Omar",
                "18000",
                "12250",
                "=C3-D3",
                "0.58",
                "Low",
                "=C3*(1+F3*0.08)",
                "Waiting for devices",
            ],
            [
                "Finance close",
                "Maya",
                "26000",
                "24100",
                "=C4-D4",
                "0.89",
                "High",
                "=C4*(1+F4*0.16)",
                "Critical reporting month",
            ],
            [
                "Analytics portal",
                "Samir",
                "32000",
                "19800",
                "=C5-D5",
                "0.62",
                "Medium",
                "=C5*(1+F5*0.10)",
                "Dashboard rebuild",
            ],
            [
                "Backup refresh",
                "Lina",
                "14000",
                "7600",
                "=C6-D6",
                "0.47",
                "Low",
                "=C6*(1+F6*0.06)",
                "Hardware quote approved",
            ],
        ];

        for (row_index, row) in rows.iter().enumerate() {
            for (col, value) in row.iter().enumerate() {
                self.set_raw(row_index + 1, col, value);
            }
        }

        self.set_raw(8, 0, "Totals");
        self.set_raw(8, 2, "=SUM(C2:C6)");
        self.set_raw(8, 3, "=SUM(D2:D6)");
        self.set_raw(8, 4, "=SUM(E2:E6)");
        self.set_raw(8, 5, "=AVG(F2:F6)");
        self.set_raw(8, 7, "=SUM(H2:H6)");
        for col in 0..9 {
            self.cells[8][col].style.bold = true;
            self.cells[8][col].style.fill = FillColor::Blue;
        }

        for row in 1..=8 {
            for col in [2, 3, 4, 7] {
                self.cells[row][col].style.format = NumberFormat::Currency;
                self.cells[row][col].style.align = CellAlign::Right;
            }
            self.cells[row][5].style.format = NumberFormat::Percent;
            self.cells[row][5].style.align = CellAlign::Right;
        }

        self.cells[1][6].style.fill = FillColor::Amber;
        self.cells[3][6].style.fill = FillColor::Rose;
        self.cells[2][6].style.fill = FillColor::Green;
        self.cells[5][6].style.fill = FillColor::Green;

        self.named_ranges
            .insert("Budget".to_owned(), parse_range("C2:C6").unwrap());
        self.named_ranges
            .insert("Spent".to_owned(), parse_range("D2:D6").unwrap());
        self.named_ranges
            .insert("Progress".to_owned(), parse_range("F2:F6").unwrap());

        self.select_cell(CellAddress::new(1, 2));
        self.status =
            "Loaded a realistic workbook with formulas, formats, and sample data".to_owned();
    }

    fn clear_workbook(&mut self) {
        self.cells = vec![vec![SpreadsheetCell::default(); COLS]; ROWS];
        self.named_ranges.clear();
        self.clipboard.clear();
        self.csv_buffer.clear();
        self.status = "Workbook cleared".to_owned();
    }

    fn set_raw(&mut self, row: usize, col: usize, value: &str) {
        if row < ROWS && col < COLS {
            self.cells[row][col].raw = value.to_owned();
        }
    }

    fn select_cell(&mut self, address: CellAddress) {
        self.selected = address;
        self.range = CellRange::single(address);
        self.range_text = format_address(address);
        self.formula_input = self.cells[address.row][address.col].raw.clone();
    }

    fn select_range(&mut self, range: CellRange) {
        let normalized = clamp_range(range).normalized();
        self.range = normalized;
        self.selected = normalized.start;
        self.range_text = format_range(normalized);
        self.formula_input = self.cells[self.selected.row][self.selected.col].raw.clone();
    }

    fn update_range_from_text(&mut self) {
        if let Some(range) = parse_range(&self.range_text) {
            self.select_range(range);
            self.status = format!("Selected {}", format_range(self.range));
        } else {
            self.status = "Range must look like A1 or A1:D12".to_owned();
        }
    }

    fn evaluated_cells(&self) -> Vec<Vec<CalcValue>> {
        let mut cache = vec![vec![None; COLS]; ROWS];
        for row in 0..ROWS {
            for col in 0..COLS {
                let mut visiting = Vec::new();
                let _ = evaluate_cell(
                    &self.cells,
                    &self.named_ranges,
                    &mut cache,
                    &mut visiting,
                    row,
                    col,
                );
            }
        }

        cache
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|value| value.unwrap_or(CalcValue::Empty))
                    .collect()
            })
            .collect()
    }

    fn selected_stats(&self, evaluated: &[Vec<CalcValue>]) -> SelectionStats {
        let mut stats = SelectionStats::default();
        let range = self.range.normalized();
        for row in range.start.row..=range.end.row {
            for col in range.start.col..=range.end.col {
                match &evaluated[row][col] {
                    CalcValue::Number(value) => {
                        stats.count += 1;
                        stats.sum += value;
                        stats.min = stats
                            .min
                            .map_or(Some(*value), |current| Some(current.min(*value)));
                        stats.max = stats
                            .max
                            .map_or(Some(*value), |current| Some(current.max(*value)));
                    }
                    CalcValue::Text(text) if !text.is_empty() => stats.text_count += 1,
                    CalcValue::Error(_) => stats.errors += 1,
                    _ => {}
                }
            }
        }
        stats
    }

    fn apply_style_to_range(&mut self, style: CellStyle) {
        let range = self.range.normalized();
        for row in range.start.row..=range.end.row {
            for col in range.start.col..=range.end.col {
                self.cells[row][col].style = style;
            }
        }
        self.status = format!("Applied style to {}", format_range(range));
    }

    fn copy_range(&mut self, evaluated: &[Vec<CalcValue>]) {
        self.clipboard = self.range_as_tsv(evaluated);
        self.status = format!(
            "Copied {} cells into the internal clipboard",
            self.range.width() * self.range.height()
        );
    }

    fn paste_clipboard(&mut self) {
        if self.clipboard.trim().is_empty() {
            self.status = "Internal clipboard is empty".to_owned();
            return;
        }

        let start = self.selected;
        for (row_offset, line) in self.clipboard.lines().enumerate() {
            for (col_offset, value) in line.split('\t').enumerate() {
                let row = start.row + row_offset;
                let col = start.col + col_offset;
                if row < ROWS && col < COLS {
                    self.cells[row][col].raw = value.to_owned();
                }
            }
        }
        self.status = format!("Pasted clipboard at {}", format_address(start));
    }

    fn clear_range(&mut self) {
        let range = self.range.normalized();
        for row in range.start.row..=range.end.row {
            for col in range.start.col..=range.end.col {
                self.cells[row][col] = SpreadsheetCell::default();
            }
        }
        self.formula_input.clear();
        self.status = format!("Cleared {}", format_range(range));
    }

    fn fill_down(&mut self) {
        let range = self.range.normalized();
        if range.height() < 2 {
            self.status = "Select at least two rows to fill down".to_owned();
            return;
        }

        for col in range.start.col..=range.end.col {
            let template = self.cells[range.start.row][col].clone();
            for row in range.start.row + 1..=range.end.row {
                self.cells[row][col] = template.clone();
            }
        }
        self.status = format!("Filled down {}", format_range(range));
    }

    fn insert_formula(&mut self, function_name: &str) {
        let target = self.selected;
        let range = format_range(self.range);
        let formula = format!("={}({})", function_name, range);
        self.cells[target.row][target.col].raw = formula.clone();
        self.formula_input = formula;
        self.status = format!(
            "Inserted {} formula into {}",
            function_name,
            format_address(target)
        );
    }

    fn export_csv(&mut self, evaluated: &[Vec<CalcValue>]) {
        let used = self.used_range();
        let mut lines = Vec::new();
        for row in 0..=used.end.row {
            let mut fields = Vec::new();
            for col in 0..=used.end.col {
                fields.push(csv_escape(&display_value(
                    &evaluated[row][col],
                    self.cells[row][col].style.format,
                    false,
                    &self.cells[row][col].raw,
                )));
            }
            lines.push(fields.join(","));
        }
        self.csv_buffer = lines.join("\n");
        self.status = format!("Exported {} as CSV text", format_range(used));
    }

    fn import_csv(&mut self) {
        if self.csv_buffer.trim().is_empty() {
            self.status = "CSV buffer is empty".to_owned();
            return;
        }

        self.clear_workbook();
        let rows = parse_csv(&self.csv_buffer);
        for (row, fields) in rows.iter().enumerate().take(ROWS) {
            for (col, value) in fields.iter().enumerate().take(COLS) {
                self.cells[row][col].raw = value.clone();
            }
        }
        self.select_cell(CellAddress::new(0, 0));
        self.status = "Imported CSV buffer into the workbook".to_owned();
    }

    fn range_as_tsv(&self, evaluated: &[Vec<CalcValue>]) -> String {
        let range = self.range.normalized();
        let mut lines = Vec::new();
        for row in range.start.row..=range.end.row {
            let mut fields = Vec::new();
            for col in range.start.col..=range.end.col {
                fields.push(display_value(
                    &evaluated[row][col],
                    self.cells[row][col].style.format,
                    self.show_formulas,
                    &self.cells[row][col].raw,
                ));
            }
            lines.push(fields.join("\t"));
        }
        lines.join("\n")
    }

    fn used_range(&self) -> CellRange {
        let mut end = CellAddress::new(0, 0);
        for row in 0..ROWS {
            for col in 0..COLS {
                if !self.cells[row][col].raw.is_empty() {
                    end.row = end.row.max(row);
                    end.col = end.col.max(col);
                }
            }
        }
        CellRange::single(CellAddress::new(0, 0)).with_end(end)
    }

    fn find_next(&mut self) {
        let query = self.find_query.trim().to_lowercase();
        if query.is_empty() {
            self.status = "Enter a search term first".to_owned();
            return;
        }

        let mut row = self.selected.row;
        let mut col = self.selected.col + 1;
        for _ in 0..ROWS * COLS {
            if col >= COLS {
                col = 0;
                row = (row + 1) % ROWS;
            }

            let raw = self.cells[row][col].raw.to_lowercase();
            if raw.contains(&query) {
                self.select_cell(CellAddress::new(row, col));
                self.status = format!("Found match at {}", format_address(self.selected));
                return;
            }
            col += 1;
        }
        self.status = "No match found".to_owned();
    }

    fn replace_in_range(&mut self) {
        if self.find_query.is_empty() {
            self.status = "Enter text to replace first".to_owned();
            return;
        }

        let range = self.range.normalized();
        let mut replacements = 0;
        for row in range.start.row..=range.end.row {
            for col in range.start.col..=range.end.col {
                if self.cells[row][col].raw.contains(&self.find_query) {
                    self.cells[row][col].raw = self.cells[row][col]
                        .raw
                        .replace(&self.find_query, &self.replace_query);
                    replacements += 1;
                }
            }
        }
        self.status = format!("Replaced text in {} cells", replacements);
    }

    fn sort_range(&mut self, evaluated: &[Vec<CalcValue>]) {
        let range = self.range.normalized();
        if range.height() < 2 {
            self.status = "Select multiple rows to sort".to_owned();
            return;
        }

        let key_col = range.start.col;
        let mut rows: Vec<(SortKey, Vec<SpreadsheetCell>)> = (range.start.row..=range.end.row)
            .map(|row| {
                (
                    SortKey::from_value(&evaluated[row][key_col]),
                    self.cells[row][range.start.col..=range.end.col].to_vec(),
                )
            })
            .collect();

        rows.sort_by(|left, right| match self.sort_direction {
            SortDirection::Ascending => left.0.cmp(&right.0),
            SortDirection::Descending => right.0.cmp(&left.0),
        });

        for (row_offset, (_, row_data)) in rows.into_iter().enumerate() {
            for (col_offset, cell) in row_data.into_iter().enumerate() {
                self.cells[range.start.row + row_offset][range.start.col + col_offset] = cell;
            }
        }
        self.status = format!("Sorted {} by {}", format_range(range), column_name(key_col));
    }

    fn define_named_range(&mut self) {
        let name = sanitize_range_name(&self.range_name_input);
        if name.is_empty() {
            self.status = "Range name is empty, so a named range was not created".to_owned();
            return;
        }

        self.named_ranges
            .insert(name.clone(), self.range.normalized());
        self.status = format!("Named range '{}' = {}", name, format_range(self.range));
    }
}

trait WithEnd {
    fn with_end(self, end: CellAddress) -> Self;
}

impl WithEnd for CellRange {
    fn with_end(self, end: CellAddress) -> Self {
        Self {
            start: self.start,
            end,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SortKey {
    number: Option<OrderedFloat>,
    text: String,
}

impl SortKey {
    fn from_value(value: &CalcValue) -> Self {
        match value {
            CalcValue::Number(number) => Self {
                number: Some(OrderedFloat(*number)),
                text: String::new(),
            },
            CalcValue::Text(text) => Self {
                number: None,
                text: text.to_lowercase(),
            },
            CalcValue::Error(error) => Self {
                number: None,
                text: error.to_lowercase(),
            },
            CalcValue::Empty => Self::default(),
        }
    }
}

impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.number, other.number) {
            (Some(left), Some(right)) => left.cmp(&right),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => self.text.cmp(&other.text),
        }
    }
}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct OrderedFloat(f64);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.total_cmp(&other.0).is_eq()
    }
}

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

#[derive(Default)]
struct SelectionStats {
    count: usize,
    text_count: usize,
    errors: usize,
    sum: f64,
    min: Option<f64>,
    max: Option<f64>,
}

impl SelectionStats {
    fn average(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }
}

impl eframe::App for SpreadsheetApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let evaluated = self.evaluated_cells();

        egui::Panel::top("top_bar")
            .frame(app_frame().inner_margin(egui::Margin::symmetric(16, 12)))
            .show_inside(ui, |ui| self.top_bar(ui, &evaluated));

        egui::Panel::left("side_panel")
            .resizable(true)
            .default_size(280.0)
            .size_range(245.0..=360.0)
            .frame(app_frame().inner_margin(egui::Margin::same(14)))
            .show_inside(ui, |ui| self.side_panel(ui, &evaluated));

        egui::CentralPanel::default()
            .frame(app_frame().inner_margin(egui::Margin::same(12)))
            .show_inside(ui, |ui| self.sheet_area(ui, &evaluated));

        egui::Panel::bottom("status_bar")
            .frame(app_frame().inner_margin(egui::Margin::symmetric(14, 8)))
            .show_inside(ui, |ui| self.status_bar(ui, &evaluated));
    }
}

impl SpreadsheetApp {
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            if input.key_pressed(Key::ArrowRight) {
                self.move_selection(0, 1);
            }
            if input.key_pressed(Key::ArrowLeft) {
                self.move_selection(0, -1);
            }
            if input.key_pressed(Key::ArrowDown) {
                self.move_selection(1, 0);
            }
            if input.key_pressed(Key::ArrowUp) {
                self.move_selection(-1, 0);
            }
            if input.key_pressed(Key::Enter) {
                self.move_selection(1, 0);
            }
        });
    }

    fn move_selection(&mut self, row_delta: isize, col_delta: isize) {
        let row = (self.selected.row as isize + row_delta).clamp(0, ROWS as isize - 1) as usize;
        let col = (self.selected.col as isize + col_delta).clamp(0, COLS as isize - 1) as usize;
        if row != self.selected.row || col != self.selected.col {
            self.select_cell(CellAddress::new(row, col));
        }
    }

    fn top_bar(&mut self, ui: &mut Ui, evaluated: &[Vec<CalcValue>]) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(&self.workbook_title)
                        .size(24.0)
                        .strong()
                        .color(palette::TEXT),
                );
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Sheet").color(palette::MUTED));
                    ui.add_sized(
                        [180.0, 24.0],
                        TextEdit::singleline(&mut self.sheet_name).hint_text("Sheet name"),
                    );
                });
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Sample").clicked() {
                    self.load_sample_workbook();
                }
                if ui.button("Clear").clicked() {
                    self.clear_workbook();
                    self.select_cell(CellAddress::new(0, 0));
                }
                if ui.button("Export CSV").clicked() {
                    self.export_csv(evaluated);
                }
                if ui.button("Copy").clicked() {
                    self.copy_range(evaluated);
                }
                if ui.button("Paste").clicked() {
                    self.paste_clipboard();
                }
            });
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format_address(self.selected))
                    .strong()
                    .color(palette::BLUE),
            );
            let response = ui.add_sized(
                [ui.available_width(), 30.0],
                TextEdit::singleline(&mut self.formula_input)
                    .hint_text("Formula bar: type text, numbers, or formulas like =SUM(C2:C6)")
                    .desired_width(f32::INFINITY),
            );

            if response.changed() {
                self.cells[self.selected.row][self.selected.col].raw = self.formula_input.clone();
            }
        });
    }

    fn side_panel(&mut self, ui: &mut Ui, evaluated: &[Vec<CalcValue>]) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                card(ui, |ui| {
                    ui.label(RichText::new("Selection").strong().color(palette::TEXT));
                    ui.horizontal(|ui| {
                        ui.add_sized([130.0, 26.0], TextEdit::singleline(&mut self.range_text));
                        if ui.button("Apply").clicked() {
                            self.update_range_from_text();
                        }
                    });
                    ui.label(
                        RichText::new(format!(
                            "{} cells",
                            self.range.width() * self.range.height()
                        ))
                        .color(palette::MUTED),
                    );
                });

                ui.add_space(10.0);
                let stats = self.selected_stats(evaluated);
                card(ui, |ui| {
                    ui.label(RichText::new("Analysis").strong().color(palette::TEXT));
                    metric_line(ui, "Count", stats.count.to_string());
                    metric_line(ui, "Text", stats.text_count.to_string());
                    metric_line(ui, "Errors", stats.errors.to_string());
                    metric_line(ui, "Sum", format_number(stats.sum));
                    metric_line(
                        ui,
                        "Average",
                        stats
                            .average()
                            .map(format_number)
                            .unwrap_or_else(|| "-".to_owned()),
                    );
                    metric_line(
                        ui,
                        "Min / Max",
                        format!(
                            "{} / {}",
                            stats
                                .min
                                .map(format_number)
                                .unwrap_or_else(|| "-".to_owned()),
                            stats
                                .max
                                .map(format_number)
                                .unwrap_or_else(|| "-".to_owned())
                        ),
                    );
                });

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(RichText::new("Formulas").strong().color(palette::TEXT));
                    ui.horizontal_wrapped(|ui| {
                        for formula in ["SUM", "AVG", "MIN", "MAX", "COUNT"] {
                            if ui.button(formula).clicked() {
                                self.insert_formula(formula);
                            }
                        }
                    });
                    ui.checkbox(&mut self.show_formulas, "Show formulas");
                });

                ui.add_space(10.0);
                self.format_panel(ui);

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(RichText::new("Data").strong().color(palette::TEXT));
                    ui.horizontal(|ui| {
                        ComboBox::from_id_salt("sort_direction")
                            .selected_text(self.sort_direction.label())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.sort_direction,
                                    SortDirection::Ascending,
                                    SortDirection::Ascending.label(),
                                );
                                ui.selectable_value(
                                    &mut self.sort_direction,
                                    SortDirection::Descending,
                                    SortDirection::Descending.label(),
                                );
                            });
                        if ui.button("Sort").clicked() {
                            self.sort_range(evaluated);
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Fill down").clicked() {
                            self.fill_down();
                        }
                        if ui.button("Clear range").clicked() {
                            self.clear_range();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [128.0, 26.0],
                            TextEdit::singleline(&mut self.range_name_input)
                                .hint_text("Range name"),
                        );
                        if ui.button("Name").clicked() {
                            self.define_named_range();
                        }
                    });
                });

                if !self.named_ranges.is_empty() {
                    ui.add_space(10.0);
                    card(ui, |ui| {
                        ui.label(RichText::new("Named Ranges").strong().color(palette::TEXT));
                        let mut ranges: Vec<(String, CellRange)> = self
                            .named_ranges
                            .iter()
                            .map(|(name, range)| (name.clone(), *range))
                            .collect();
                        ranges.sort_by(|left, right| left.0.cmp(&right.0));
                        for (name, range) in ranges {
                            if ui
                                .button(format!("{}  {}", name, format_range(range)))
                                .clicked()
                            {
                                self.select_range(range);
                            }
                        }
                    });
                }

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(
                        RichText::new("Find / Replace")
                            .strong()
                            .color(palette::TEXT),
                    );
                    ui.add(TextEdit::singleline(&mut self.find_query).hint_text("Find"));
                    ui.add(TextEdit::singleline(&mut self.replace_query).hint_text("Replace with"));
                    ui.horizontal(|ui| {
                        if ui.button("Find next").clicked() {
                            self.find_next();
                        }
                        if ui.button("Replace range").clicked() {
                            self.replace_in_range();
                        }
                    });
                });

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(RichText::new("CSV Buffer").strong().color(palette::TEXT));
                    ui.add(
                        TextEdit::multiline(&mut self.csv_buffer)
                            .desired_rows(8)
                            .desired_width(f32::INFINITY)
                            .hint_text("CSV export/import text"),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Export").clicked() {
                            self.export_csv(evaluated);
                        }
                        if ui.button("Import").clicked() {
                            self.import_csv();
                        }
                    });
                });

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(RichText::new("View").strong().color(palette::TEXT));
                    ui.checkbox(&mut self.show_headers, "Show headers");
                    ui.add(egui::Slider::new(&mut self.zoom, 0.75..=1.35).text("Zoom"));
                });
            });
    }

    fn format_panel(&mut self, ui: &mut Ui) {
        let selected_style = self.cells[self.selected.row][self.selected.col].style;
        let mut style = selected_style;

        card(ui, |ui| {
            ui.label(RichText::new("Format").strong().color(palette::TEXT));
            ui.horizontal(|ui| {
                ui.checkbox(&mut style.bold, "Bold");
                ui.checkbox(&mut style.italic, "Italic");
            });

            ComboBox::from_id_salt("fill_color")
                .selected_text(style.fill.label())
                .show_ui(ui, |ui| {
                    for fill in [
                        FillColor::None,
                        FillColor::Header,
                        FillColor::Blue,
                        FillColor::Green,
                        FillColor::Amber,
                        FillColor::Rose,
                    ] {
                        ui.selectable_value(&mut style.fill, fill, fill.label());
                    }
                });

            ComboBox::from_id_salt("number_format")
                .selected_text(style.format.label())
                .show_ui(ui, |ui| {
                    for format in [
                        NumberFormat::Auto,
                        NumberFormat::Number,
                        NumberFormat::Currency,
                        NumberFormat::Percent,
                    ] {
                        ui.selectable_value(&mut style.format, format, format.label());
                    }
                });

            ComboBox::from_id_salt("cell_align")
                .selected_text(style.align.label())
                .show_ui(ui, |ui| {
                    for align in [CellAlign::Left, CellAlign::Center, CellAlign::Right] {
                        ui.selectable_value(&mut style.align, align, align.label());
                    }
                });

            if style != selected_style {
                self.apply_style_to_range(style);
            }
        });
    }

    fn sheet_area(&mut self, ui: &mut Ui, evaluated: &[Vec<CalcValue>]) {
        let scaled_width = CELL_WIDTH * self.zoom;
        let scaled_height = CELL_HEIGHT * self.zoom;

        ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Grid::new("spreadsheet_grid")
                    .spacing(Vec2::new(0.0, 0.0))
                    .striped(false)
                    .show(ui, |ui| {
                        if self.show_headers {
                            ui.add_sized([ROW_HEADER_WIDTH, scaled_height], egui::Label::new(""));
                            for col in 0..COLS {
                                header_cell(ui, &column_name(col), scaled_width, scaled_height);
                            }
                            ui.end_row();
                        }

                        for row in 0..ROWS {
                            if self.show_headers {
                                header_cell(
                                    ui,
                                    &(row + 1).to_string(),
                                    ROW_HEADER_WIDTH,
                                    scaled_height,
                                );
                            }
                            for col in 0..COLS {
                                self.cell_widget(
                                    ui,
                                    row,
                                    col,
                                    &evaluated[row][col],
                                    scaled_width,
                                    scaled_height,
                                );
                            }
                            ui.end_row();
                        }
                    });
            });
    }

    fn cell_widget(
        &mut self,
        ui: &mut Ui,
        row: usize,
        col: usize,
        value: &CalcValue,
        width: f32,
        height: f32,
    ) {
        let address = CellAddress::new(row, col);
        let is_selected = self.selected == address;
        let is_in_range = self.range.contains(address);
        let style = self.cells[row][col].style;
        let mut fill = style.fill.color();
        if is_in_range {
            fill = blend(fill, palette::RANGE, 0.18);
        }
        if is_selected {
            fill = blend(fill, palette::BLUE, 0.24);
        }

        let stroke = if is_selected {
            Stroke::new(2.0, palette::BLUE)
        } else if is_in_range {
            Stroke::new(1.0, palette::GREEN)
        } else {
            Stroke::new(1.0, palette::GRID)
        };

        let text_color = match value {
            CalcValue::Error(_) => palette::ROSE,
            _ => palette::TEXT,
        };
        let label_text = display_value(
            value,
            style.format,
            self.show_formulas,
            &self.cells[row][col].raw,
        );

        egui::Frame::default()
            .fill(fill)
            .stroke(stroke)
            .inner_margin(egui::Margin::symmetric(4, 2))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(width, height));
                if is_selected {
                    let response = ui.add_sized(
                        [width - 8.0, height - 4.0],
                        TextEdit::singleline(&mut self.cells[row][col].raw)
                            .desired_width(width - 8.0),
                    );
                    if response.changed() {
                        self.formula_input = self.cells[row][col].raw.clone();
                    }
                } else {
                    let text = styled_text(label_text, style, text_color);
                    let button = egui::Button::new(text)
                        .frame(false)
                        .fill(Color32::TRANSPARENT);
                    let response = ui
                        .with_layout(layout_for_align(style.align), |ui| {
                            ui.add_sized([width - 8.0, height - 4.0], button)
                        })
                        .inner;
                    if response.clicked() {
                        self.select_cell(address);
                    }
                    if response.double_clicked() {
                        self.select_range(CellRange::single(address));
                    }
                }
            });
    }

    fn status_bar(&mut self, ui: &mut Ui, evaluated: &[Vec<CalcValue>]) {
        let stats = self.selected_stats(evaluated);
        ui.horizontal(|ui| {
            ui.label(RichText::new(&self.status).color(palette::MUTED));
            ui.separator();
            ui.label(format!("Range {}", format_range(self.range)));
            ui.separator();
            ui.label(format!("Sum {}", format_number(stats.sum)));
            ui.separator();
            ui.label(format!(
                "Average {}",
                stats
                    .average()
                    .map(format_number)
                    .unwrap_or_else(|| "-".to_owned())
            ));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(format!("{} x {}", ROWS, COLS));
            });
        });
    }
}

fn header_cell(ui: &mut Ui, text: &str, width: f32, height: f32) {
    egui::Frame::default()
        .fill(palette::HEADER)
        .stroke(Stroke::new(1.0, palette::GRID))
        .inner_margin(egui::Margin::symmetric(4, 2))
        .show(ui, |ui| {
            ui.add_sized(
                [width, height],
                egui::Label::new(RichText::new(text).strong().color(palette::MUTED)),
            );
        });
}

fn card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::group(ui.style())
        .fill(palette::CARD)
        .stroke(Stroke::new(1.0, palette::BORDER))
        .inner_margin(egui::Margin::same(12))
        .show(ui, add_contents);
}

fn metric_line(ui: &mut Ui, label: &str, value: String) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(palette::MUTED));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).strong().color(palette::TEXT));
        });
    });
}

fn styled_text(text: String, style: CellStyle, color: Color32) -> RichText {
    let mut rich = RichText::new(text).color(color);
    if style.bold {
        rich = rich.strong();
    }
    if style.italic {
        rich = rich.italics();
    }
    rich
}

fn layout_for_align(align: CellAlign) -> Layout {
    match align {
        CellAlign::Left => Layout::left_to_right(Align::Center),
        CellAlign::Center => Layout::centered_and_justified(egui::Direction::LeftToRight),
        CellAlign::Right => Layout::right_to_left(Align::Center),
    }
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(10.0, 7.0);
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(23.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = palette::BG;
    visuals.window_fill = palette::CARD;
    visuals.faint_bg_color = palette::CELL;
    visuals.extreme_bg_color = palette::FIELD;
    visuals.selection.bg_fill = palette::BLUE;
    visuals.widgets.inactive.bg_fill = palette::FIELD;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 58, 74);
    visuals.widgets.active.bg_fill = Color32::from_rgb(55, 87, 112);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette::TEXT);
    style.visuals = visuals;
    ctx.set_global_style(style);
}

fn app_frame() -> egui::Frame {
    egui::Frame::default().fill(palette::BG)
}

fn blend(base: Color32, overlay: Color32, amount: f32) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    let inverse = 1.0 - amount;
    Color32::from_rgb(
        (base.r() as f32 * inverse + overlay.r() as f32 * amount) as u8,
        (base.g() as f32 * inverse + overlay.g() as f32 * amount) as u8,
        (base.b() as f32 * inverse + overlay.b() as f32 * amount) as u8,
    )
}

fn evaluate_cell(
    cells: &[Vec<SpreadsheetCell>],
    named_ranges: &HashMap<String, CellRange>,
    cache: &mut [Vec<Option<CalcValue>>],
    visiting: &mut Vec<CellAddress>,
    row: usize,
    col: usize,
) -> CalcValue {
    if row >= ROWS || col >= COLS {
        return CalcValue::Error("#REF".to_owned());
    }

    if let Some(value) = &cache[row][col] {
        return value.clone();
    }

    let address = CellAddress::new(row, col);
    if visiting.contains(&address) {
        return CalcValue::Error("#CYCLE".to_owned());
    }

    visiting.push(address);
    let raw = cells[row][col].raw.trim();
    let value = if raw.is_empty() {
        CalcValue::Empty
    } else if let Some(formula) = raw.strip_prefix('=') {
        let mut parser = FormulaParser::new(formula, cells, named_ranges, cache, visiting);
        match parser.parse() {
            Ok(number) if number.is_finite() => CalcValue::Number(number),
            Ok(_) => CalcValue::Error("#NUM".to_owned()),
            Err(error) => CalcValue::Error(error),
        }
    } else if let Ok(number) = raw.parse::<f64>() {
        CalcValue::Number(number)
    } else {
        CalcValue::Text(raw.to_owned())
    };
    visiting.pop();
    cache[row][col] = Some(value.clone());
    value
}

struct FormulaParser<'a> {
    chars: Vec<char>,
    pos: usize,
    cells: &'a [Vec<SpreadsheetCell>],
    named_ranges: &'a HashMap<String, CellRange>,
    cache: &'a mut [Vec<Option<CalcValue>>],
    visiting: &'a mut Vec<CellAddress>,
}

impl<'a> FormulaParser<'a> {
    fn new(
        input: &str,
        cells: &'a [Vec<SpreadsheetCell>],
        named_ranges: &'a HashMap<String, CellRange>,
        cache: &'a mut [Vec<Option<CalcValue>>],
        visiting: &'a mut Vec<CellAddress>,
    ) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            cells,
            named_ranges,
            cache,
            visiting,
        }
    }

    fn parse(&mut self) -> Result<f64, String> {
        let value = self.expression()?;
        self.skip_ws();
        if self.pos < self.chars.len() {
            Err("#PARSE".to_owned())
        } else {
            Ok(value)
        }
    }

    fn expression(&mut self) -> Result<f64, String> {
        let mut value = self.term()?;
        loop {
            self.skip_ws();
            if self.consume('+') {
                value += self.term()?;
            } else if self.consume('-') {
                value -= self.term()?;
            } else {
                break;
            }
        }
        Ok(value)
    }

    fn term(&mut self) -> Result<f64, String> {
        let mut value = self.factor()?;
        loop {
            self.skip_ws();
            if self.consume('*') {
                value *= self.factor()?;
            } else if self.consume('/') {
                let divisor = self.factor()?;
                if divisor == 0.0 {
                    return Err("#DIV/0".to_owned());
                }
                value /= divisor;
            } else {
                break;
            }
        }
        Ok(value)
    }

    fn factor(&mut self) -> Result<f64, String> {
        self.skip_ws();
        if self.consume('-') {
            return Ok(-self.factor()?);
        }
        if self.consume('+') {
            return self.factor();
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<f64, String> {
        self.skip_ws();
        if self.consume('(') {
            let value = self.expression()?;
            self.expect(')')?;
            return Ok(value);
        }

        if self
            .peek()
            .map_or(false, |ch| ch.is_ascii_digit() || ch == '.')
        {
            return self.number();
        }

        if self.peek().map_or(false, |ch| ch.is_ascii_alphabetic()) {
            let start = self.pos;
            let ident = self.identifier();
            self.skip_ws();
            if self.consume('(') {
                return self.function(&ident);
            }

            self.pos = start;
            if let Some(address) = self.cell_ref() {
                if self.consume(':') {
                    if let Some(end) = self.cell_ref() {
                        return Ok(self
                            .range_numbers(CellRange {
                                start: address,
                                end,
                            })?
                            .iter()
                            .sum());
                    }
                    return Err("#REF".to_owned());
                }
                return self.cell_number(address);
            }

            self.pos = start;
            let named = self.identifier();
            if let Some(range) = lookup_named_range(self.named_ranges, &named) {
                return Ok(self.range_numbers(range)?.iter().sum());
            }
        }

        Err("#VALUE".to_owned())
    }

    fn function(&mut self, name: &str) -> Result<f64, String> {
        let mut values = Vec::new();
        self.skip_ws();
        if self.consume(')') {
            return Err("#ARGS".to_owned());
        }

        loop {
            self.skip_ws();
            if let Some(range) = self.try_range_argument() {
                values.extend(self.range_numbers(range)?);
            } else {
                values.push(self.expression()?);
            }

            self.skip_ws();
            if self.consume(',') || self.consume(';') {
                continue;
            }
            self.expect(')')?;
            break;
        }

        let upper = name.to_ascii_uppercase();
        match upper.as_str() {
            "SUM" => Ok(values.iter().sum()),
            "AVG" | "AVERAGE" => {
                if values.is_empty() {
                    Err("#DIV/0".to_owned())
                } else {
                    Ok(values.iter().sum::<f64>() / values.len() as f64)
                }
            }
            "MIN" => values
                .into_iter()
                .reduce(f64::min)
                .ok_or_else(|| "#VALUE".to_owned()),
            "MAX" => values
                .into_iter()
                .reduce(f64::max)
                .ok_or_else(|| "#VALUE".to_owned()),
            "COUNT" => Ok(values.len() as f64),
            "ABS" => one_arg(values, f64::abs),
            "ROUND" => {
                if values.is_empty() {
                    Err("#ARGS".to_owned())
                } else {
                    let places = values
                        .get(1)
                        .copied()
                        .unwrap_or(0.0)
                        .round()
                        .clamp(0.0, 8.0);
                    let factor = 10_f64.powf(places);
                    Ok((values[0] * factor).round() / factor)
                }
            }
            _ => Err("#NAME".to_owned()),
        }
    }

    fn try_range_argument(&mut self) -> Option<CellRange> {
        let saved = self.pos;
        if let Some(start) = self.cell_ref() {
            if self.consume(':') {
                if let Some(end) = self.cell_ref() {
                    return Some(CellRange { start, end });
                }
            }
        }

        self.pos = saved;
        let name = self.identifier();
        if name.is_empty() {
            self.pos = saved;
            return None;
        }
        if let Some(range) = lookup_named_range(self.named_ranges, &name) {
            Some(range)
        } else {
            self.pos = saved;
            None
        }
    }

    fn range_numbers(&mut self, range: CellRange) -> Result<Vec<f64>, String> {
        let range = clamp_range(range).normalized();
        let mut numbers = Vec::new();
        for row in range.start.row..=range.end.row {
            for col in range.start.col..=range.end.col {
                match evaluate_cell(
                    self.cells,
                    self.named_ranges,
                    self.cache,
                    self.visiting,
                    row,
                    col,
                ) {
                    CalcValue::Number(number) => numbers.push(number),
                    CalcValue::Text(text) => {
                        if let Ok(number) = text.trim().parse::<f64>() {
                            numbers.push(number);
                        }
                    }
                    CalcValue::Error(error) => return Err(error),
                    CalcValue::Empty => {}
                }
            }
        }
        Ok(numbers)
    }

    fn cell_number(&mut self, address: CellAddress) -> Result<f64, String> {
        let address = clamp_address(address);
        match evaluate_cell(
            self.cells,
            self.named_ranges,
            self.cache,
            self.visiting,
            address.row,
            address.col,
        ) {
            CalcValue::Number(number) => Ok(number),
            CalcValue::Text(text) => text.trim().parse::<f64>().map_err(|_| "#VALUE".to_owned()),
            CalcValue::Error(error) => Err(error),
            CalcValue::Empty => Err("#VALUE".to_owned()),
        }
    }

    fn number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        while self
            .peek()
            .map_or(false, |ch| ch.is_ascii_digit() || ch == '.')
        {
            self.pos += 1;
        }
        self.chars[start..self.pos]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .map_err(|_| "#VALUE".to_owned())
    }

    fn identifier(&mut self) -> String {
        let start = self.pos;
        while self
            .peek()
            .map_or(false, |ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn cell_ref(&mut self) -> Option<CellAddress> {
        self.skip_ws();
        let start = self.pos;
        let mut col_name = String::new();
        while self.peek().map_or(false, |ch| ch.is_ascii_alphabetic()) {
            col_name.push(self.chars[self.pos].to_ascii_uppercase());
            self.pos += 1;
        }

        let digit_start = self.pos;
        while self.peek().map_or(false, |ch| ch.is_ascii_digit()) {
            self.pos += 1;
        }

        if col_name.is_empty() || digit_start == self.pos {
            self.pos = start;
            return None;
        }

        let row_number = self.chars[digit_start..self.pos]
            .iter()
            .collect::<String>()
            .parse::<usize>()
            .ok()?;

        let col = column_index(&col_name)?;
        if row_number == 0 {
            return None;
        }
        Some(CellAddress::new(row_number - 1, col))
    }

    fn skip_ws(&mut self) {
        while self.peek().map_or(false, |ch| ch.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        self.skip_ws();
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err("#PARSE".to_owned())
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
}

fn one_arg(values: Vec<f64>, function: impl FnOnce(f64) -> f64) -> Result<f64, String> {
    values
        .first()
        .copied()
        .map(function)
        .ok_or_else(|| "#ARGS".to_owned())
}

fn lookup_named_range(named_ranges: &HashMap<String, CellRange>, name: &str) -> Option<CellRange> {
    named_ranges
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, range)| *range)
}

fn sanitize_range_name(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch)
            } else if ch == ' ' || ch == '-' {
                Some('_')
            } else {
                None
            }
        })
        .collect()
}

fn display_value(value: &CalcValue, format: NumberFormat, show_formula: bool, raw: &str) -> String {
    if show_formula && raw.starts_with('=') {
        return raw.to_owned();
    }

    match value {
        CalcValue::Empty => String::new(),
        CalcValue::Number(number) => match format {
            NumberFormat::Auto => format_number(*number),
            NumberFormat::Number => format!("{:.2}", number),
            NumberFormat::Currency => format!("${:.2}", number),
            NumberFormat::Percent => format!("{:.1}%", number * 100.0),
        },
        CalcValue::Text(text) => text.clone(),
        CalcValue::Error(error) => error.clone(),
    }
}

fn format_number(number: f64) -> String {
    if (number.fract()).abs() < f64::EPSILON {
        format!("{:.0}", number)
    } else {
        format!("{:.2}", number)
    }
}

fn format_address(address: CellAddress) -> String {
    format!("{}{}", column_name(address.col), address.row + 1)
}

fn format_range(range: CellRange) -> String {
    let normalized = range.normalized();
    if normalized.start == normalized.end {
        format_address(normalized.start)
    } else {
        format!(
            "{}:{}",
            format_address(normalized.start),
            format_address(normalized.end)
        )
    }
}

fn parse_range(input: &str) -> Option<CellRange> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parts: Vec<&str> = trimmed.split(':').collect();
    match parts.as_slice() {
        [single] => parse_address(single).map(CellRange::single),
        [start, end] => Some(CellRange {
            start: parse_address(start)?,
            end: parse_address(end)?,
        }),
        _ => None,
    }
}

fn parse_address(input: &str) -> Option<CellAddress> {
    let trimmed = input.trim();
    let mut letters = String::new();
    let mut digits = String::new();

    for ch in trimmed.chars() {
        if ch.is_ascii_alphabetic() && digits.is_empty() {
            letters.push(ch.to_ascii_uppercase());
        } else if ch.is_ascii_digit() {
            digits.push(ch);
        } else {
            return None;
        }
    }

    let row = digits.parse::<usize>().ok()?.checked_sub(1)?;
    let col = column_index(&letters)?;
    Some(clamp_address(CellAddress::new(row, col)))
}

fn clamp_range(range: CellRange) -> CellRange {
    CellRange {
        start: clamp_address(range.start),
        end: clamp_address(range.end),
    }
}

fn clamp_address(address: CellAddress) -> CellAddress {
    CellAddress::new(address.row.min(ROWS - 1), address.col.min(COLS - 1))
}

fn column_name(mut index: usize) -> String {
    let mut name = String::new();
    loop {
        let remainder = index % 26;
        name.insert(0, (b'A' + remainder as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    name
}

fn column_index(name: &str) -> Option<usize> {
    if name.is_empty() {
        return None;
    }

    let mut index = 0usize;
    for ch in name.chars() {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        index = index * 26 + (ch.to_ascii_uppercase() as usize - 'A' as usize + 1);
    }
    index.checked_sub(1)
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn parse_csv(input: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut chars = input.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                row.push(field.clone());
                field.clear();
            }
            '\n' if !in_quotes => {
                row.push(field.clone());
                field.clear();
                rows.push(row.clone());
                row.clear();
            }
            '\r' => {}
            _ => field.push(ch),
        }
    }

    row.push(field);
    rows.push(row);
    rows
}

mod palette {
    use eframe::egui::Color32;

    pub const BG: Color32 = Color32::from_rgb(15, 20, 28);
    pub const CARD: Color32 = Color32::from_rgb(24, 31, 41);
    pub const FIELD: Color32 = Color32::from_rgb(20, 26, 35);
    pub const CELL: Color32 = Color32::from_rgb(18, 24, 32);
    pub const HEADER: Color32 = Color32::from_rgb(29, 38, 50);
    pub const GRID: Color32 = Color32::from_rgb(48, 61, 78);
    pub const BORDER: Color32 = Color32::from_rgb(55, 70, 88);
    pub const TEXT: Color32 = Color32::from_rgb(233, 240, 247);
    pub const MUTED: Color32 = Color32::from_rgb(145, 158, 174);
    pub const BLUE: Color32 = Color32::from_rgb(88, 166, 255);
    pub const GREEN: Color32 = Color32::from_rgb(78, 203, 137);
    pub const ROSE: Color32 = Color32::from_rgb(239, 99, 119);
    pub const RANGE: Color32 = Color32::from_rgb(83, 210, 158);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_cells() -> Vec<Vec<SpreadsheetCell>> {
        vec![vec![SpreadsheetCell::default(); COLS]; ROWS]
    }

    fn evaluate_for_test(
        cells: &[Vec<SpreadsheetCell>],
        named_ranges: &HashMap<String, CellRange>,
        row: usize,
        col: usize,
    ) -> CalcValue {
        let mut cache = vec![vec![None; COLS]; ROWS];
        let mut visiting = Vec::new();
        evaluate_cell(cells, named_ranges, &mut cache, &mut visiting, row, col)
    }

    #[test]
    fn evaluates_arithmetic_ranges_and_cell_refs() {
        let mut cells = blank_cells();
        let named_ranges = HashMap::new();
        cells[0][0].raw = "10".to_owned();
        cells[1][0].raw = "15".to_owned();
        cells[0][1].raw = "=SUM(A1:A2) * 2 + 5".to_owned();

        match evaluate_for_test(&cells, &named_ranges, 0, 1) {
            CalcValue::Number(value) => assert_eq!(value, 55.0),
            other => panic!("expected number, got {other:?}"),
        }
    }

    #[test]
    fn evaluates_named_ranges_inside_functions() {
        let mut cells = blank_cells();
        let mut named_ranges = HashMap::new();
        cells[0][2].raw = "100".to_owned();
        cells[1][2].raw = "250".to_owned();
        cells[2][2].raw = "=SUM(Budget)".to_owned();
        named_ranges.insert(
            "Budget".to_owned(),
            CellRange {
                start: CellAddress::new(0, 2),
                end: CellAddress::new(1, 2),
            },
        );

        match evaluate_for_test(&cells, &named_ranges, 2, 2) {
            CalcValue::Number(value) => assert_eq!(value, 350.0),
            other => panic!("expected number, got {other:?}"),
        }
    }

    #[test]
    fn reports_circular_references() {
        let mut cells = blank_cells();
        let named_ranges = HashMap::new();
        cells[0][0].raw = "=B1".to_owned();
        cells[0][1].raw = "=A1".to_owned();

        match evaluate_for_test(&cells, &named_ranges, 0, 0) {
            CalcValue::Error(error) => assert_eq!(error, "#CYCLE"),
            other => panic!("expected cycle error, got {other:?}"),
        }
    }

    #[test]
    fn parses_csv_quotes_and_commas() {
        let parsed = parse_csv("Name,Notes\nAmina,\"ready, waiting\"\nOmar,\"said \"\"yes\"\"\"");

        assert_eq!(parsed[0], vec!["Name", "Notes"]);
        assert_eq!(parsed[1], vec!["Amina", "ready, waiting"]);
        assert_eq!(parsed[2], vec!["Omar", "said \"yes\""]);
    }
}
