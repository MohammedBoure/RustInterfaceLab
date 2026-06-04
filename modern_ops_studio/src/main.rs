#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{
    self, Align, Color32, ComboBox, FontFamily, FontId, Key, Layout, RichText, ScrollArea, Stroke,
    TextEdit, TextStyle, Ui, Vec2,
};
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet, VecDeque};

const ROWS: usize = 80;
const COLS: usize = 26;
const CELL_WIDTH: f32 = 116.0;
const CELL_HEIGHT: f32 = 30.0;
const ROW_HEADER_WIDTH: f32 = 46.0;
const ERR_REF: &str = "#REF!";
const ERR_VALUE: &str = "#VALUE!";
const ERR_DIV_ZERO: &str = "#DIV/0!";
const ERR_NAME: &str = "#NAME?";
const ERR_NUM: &str = "#NUM!";
const ERR_NA: &str = "#N/A";
const ERR_CYCLE: &str = "#CYCLE!";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    Currency(f64),
    Percent(f64),
    Boolean(bool),
    Date(i64),
    Text(String),
    Error(String),
}

impl CalcValue {
    fn numeric_value(&self) -> Option<f64> {
        match self {
            Self::Number(value) | Self::Currency(value) | Self::Percent(value) => Some(*value),
            Self::Boolean(value) => Some(if *value { 1.0 } else { 0.0 }),
            Self::Date(days) => Some(*days as f64),
            Self::Text(text) => text_to_number(text),
            Self::Empty | Self::Error(_) => None,
        }
    }

    fn text_value(&self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Number(value) => format_number(*value),
            Self::Currency(value) => format!("{:.2}", value),
            Self::Percent(value) => format!("{:.6}", value),
            Self::Boolean(value) => {
                if *value {
                    "TRUE".to_owned()
                } else {
                    "FALSE".to_owned()
                }
            }
            Self::Date(days) => format_date(*days),
            Self::Text(text) => text.clone(),
            Self::Error(error) => error.clone(),
        }
    }

    fn truthy(&self) -> Result<bool, String> {
        match self {
            Self::Boolean(value) => Ok(*value),
            Self::Number(value) | Self::Currency(value) | Self::Percent(value) => Ok(*value != 0.0),
            Self::Date(_) => Ok(true),
            Self::Text(text) => match text.trim().to_ascii_uppercase().as_str() {
                "TRUE" => Ok(true),
                "FALSE" => Ok(false),
                "" => Ok(false),
                _ => Err(ERR_VALUE.to_owned()),
            },
            Self::Empty => Ok(false),
            Self::Error(error) => Err(error.clone()),
        }
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChartKind {
    Bar,
    Line,
    Pie,
    Scatter,
}

impl ChartKind {
    fn label(self) -> &'static str {
        match self {
            Self::Bar => "Bar",
            Self::Line => "Line",
            Self::Pie => "Pie",
            Self::Scatter => "Scatter",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompareOperator {
    GreaterThan,
    LessThan,
    Equal,
    Between,
}

impl CompareOperator {
    fn label(self) -> &'static str {
        match self {
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::Equal => "=",
            Self::Between => "Between",
        }
    }
}

#[derive(Clone, Debug)]
struct ConditionalRule {
    range: CellRange,
    operator: CompareOperator,
    threshold: f64,
    second_threshold: f64,
    fill: FillColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValidationKind {
    NumberBetween,
    DateBetween,
    List,
    NonEmpty,
}

impl ValidationKind {
    fn label(self) -> &'static str {
        match self {
            Self::NumberBetween => "Number range",
            Self::DateBetween => "Date range",
            Self::List => "List",
            Self::NonEmpty => "Required",
        }
    }
}

#[derive(Clone, Debug)]
struct ValidationRule {
    range: CellRange,
    kind: ValidationKind,
    min: f64,
    max: f64,
    list: Vec<String>,
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
    filters: HashMap<usize, String>,
    filter_column: usize,
    filter_query: String,
    chart_kind: ChartKind,
    conditional_rules: Vec<ConditionalRule>,
    conditional_operator: CompareOperator,
    conditional_threshold: f64,
    conditional_second_threshold: f64,
    conditional_fill: FillColor,
    validation_rules: Vec<ValidationRule>,
    validation_kind: ValidationKind,
    validation_min: f64,
    validation_max: f64,
    validation_list: String,
    calculated: Vec<Vec<CalcValue>>,
    dirty: Vec<Vec<bool>>,
    dependencies: HashMap<CellAddress, Vec<CellAddress>>,
    dependents: HashMap<CellAddress, Vec<CellAddress>>,
    graph_dirty: bool,
    last_recalc_count: usize,
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
            filters: HashMap::new(),
            filter_column: 0,
            filter_query: String::new(),
            chart_kind: ChartKind::Bar,
            conditional_rules: Vec::new(),
            conditional_operator: CompareOperator::GreaterThan,
            conditional_threshold: 0.0,
            conditional_second_threshold: 1.0,
            conditional_fill: FillColor::Green,
            validation_rules: Vec::new(),
            validation_kind: ValidationKind::NumberBetween,
            validation_min: 0.0,
            validation_max: 1.0,
            validation_list: "Low,Medium,High".to_owned(),
            calculated: vec![vec![CalcValue::Empty; COLS]; ROWS],
            dirty: vec![vec![true; COLS]; ROWS],
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            graph_dirty: true,
            last_recalc_count: 0,
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
        self.conditional_rules.push(ConditionalRule {
            range: parse_range("F2:F6").unwrap(),
            operator: CompareOperator::GreaterThan,
            threshold: 0.7,
            second_threshold: 1.0,
            fill: FillColor::Green,
        });
        self.conditional_rules.push(ConditionalRule {
            range: parse_range("D2:D6").unwrap(),
            operator: CompareOperator::GreaterThan,
            threshold: 20000.0,
            second_threshold: 0.0,
            fill: FillColor::Amber,
        });
        self.validation_rules.push(ValidationRule {
            range: parse_range("F2:F6").unwrap(),
            kind: ValidationKind::NumberBetween,
            min: 0.0,
            max: 1.0,
            list: Vec::new(),
        });
        self.validation_rules.push(ValidationRule {
            range: parse_range("G2:G6").unwrap(),
            kind: ValidationKind::List,
            min: 0.0,
            max: 0.0,
            list: vec!["Low".to_owned(), "Medium".to_owned(), "High".to_owned()],
        });

        self.select_cell(CellAddress::new(1, 2));
        self.status =
            "Loaded a realistic workbook with formulas, formats, and sample data".to_owned();
    }

    fn clear_workbook(&mut self) {
        self.cells = vec![vec![SpreadsheetCell::default(); COLS]; ROWS];
        self.named_ranges.clear();
        self.filters.clear();
        self.conditional_rules.clear();
        self.validation_rules.clear();
        self.clipboard.clear();
        self.csv_buffer.clear();
        self.reset_calculation_state();
        self.status = "Workbook cleared".to_owned();
    }

    fn set_raw(&mut self, row: usize, col: usize, value: &str) {
        if row < ROWS && col < COLS {
            self.set_cell_raw(CellAddress::new(row, col), value.to_owned());
        }
    }

    fn set_cell_raw(&mut self, address: CellAddress, value: String) {
        let address = clamp_address(address);
        if self.cells[address.row][address.col].raw != value {
            self.cells[address.row][address.col].raw = value;
            self.mark_cell_changed(address);
        }
    }

    fn reset_calculation_state(&mut self) {
        self.calculated = vec![vec![CalcValue::Empty; COLS]; ROWS];
        self.dirty = vec![vec![true; COLS]; ROWS];
        self.dependencies.clear();
        self.dependents.clear();
        self.graph_dirty = true;
        self.last_recalc_count = 0;
    }

    fn mark_all_dirty(&mut self) {
        for row in 0..ROWS {
            for col in 0..COLS {
                self.dirty[row][col] = true;
            }
        }
        self.graph_dirty = true;
    }

    fn mark_cell_changed(&mut self, address: CellAddress) {
        let address = clamp_address(address);
        let mut queue = VecDeque::from([address]);
        let mut seen = HashSet::new();
        while let Some(current) = queue.pop_front() {
            if !seen.insert(current) {
                continue;
            }
            self.dirty[current.row][current.col] = true;
            if let Some(children) = self.dependents.get(&current) {
                for child in children {
                    queue.push_back(*child);
                }
            }
        }
        self.graph_dirty = true;
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

    fn evaluated_cells(&mut self) -> Vec<Vec<CalcValue>> {
        self.recalculate_dirty();
        self.calculated.clone()
    }

    fn rebuild_dependency_graph(&mut self) {
        self.dependencies.clear();
        self.dependents.clear();
        for row in 0..ROWS {
            for col in 0..COLS {
                let address = CellAddress::new(row, col);
                let dependencies =
                    collect_dependencies(&self.cells[row][col].raw, &self.named_ranges);
                for dependency in &dependencies {
                    self.dependents
                        .entry(*dependency)
                        .or_default()
                        .push(address);
                }
                self.dependencies.insert(address, dependencies);
            }
        }
        self.graph_dirty = false;
    }

    fn recalculate_dirty(&mut self) {
        if self.graph_dirty {
            self.rebuild_dependency_graph();
        }

        let mut cache = vec![vec![None; COLS]; ROWS];
        for row in 0..ROWS {
            for col in 0..COLS {
                if !self.dirty[row][col] {
                    cache[row][col] = Some(self.calculated[row][col].clone());
                }
            }
        }

        let mut recalculated = 0usize;
        for row in 0..ROWS {
            for col in 0..COLS {
                if self.dirty[row][col] {
                    let mut visiting = Vec::new();
                    let _ = evaluate_cell(
                        &self.cells,
                        &self.named_ranges,
                        &mut cache,
                        &mut visiting,
                        row,
                        col,
                    );
                    recalculated += 1;
                }
            }
        }

        for row in 0..ROWS {
            for col in 0..COLS {
                self.calculated[row][col] = cache[row][col].clone().unwrap_or(CalcValue::Empty);
                self.dirty[row][col] = false;
            }
        }
        self.last_recalc_count = recalculated;
    }

    fn selected_stats(&self, evaluated: &[Vec<CalcValue>]) -> SelectionStats {
        let mut stats = SelectionStats::default();
        let range = self.range.normalized();
        for row in range.start.row..=range.end.row {
            for col in range.start.col..=range.end.col {
                match &evaluated[row][col] {
                    CalcValue::Number(value)
                    | CalcValue::Currency(value)
                    | CalcValue::Percent(value) => {
                        stats.count += 1;
                        stats.sum += value;
                        stats.values.push(*value);
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
        let clipboard = self.clipboard.clone();
        for (row_offset, line) in clipboard.lines().enumerate() {
            for (col_offset, value) in line.split('\t').enumerate() {
                let row = start.row + row_offset;
                let col = start.col + col_offset;
                if row < ROWS && col < COLS {
                    self.set_cell_raw(CellAddress::new(row, col), value.to_owned());
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
                self.mark_cell_changed(CellAddress::new(row, col));
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
                self.mark_cell_changed(CellAddress::new(row, col));
            }
        }
        self.status = format!("Filled down {}", format_range(range));
    }

    fn insert_formula(&mut self, function_name: &str) {
        let target = self.selected;
        let range = format_range(self.range);
        let formula = format!("={}({})", function_name, range);
        self.set_cell_raw(target, formula.clone());
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

        let csv = self.csv_buffer.clone();
        self.clear_workbook();
        self.csv_buffer = csv.clone();
        let rows = parse_csv(&csv);
        for (row, fields) in rows.iter().enumerate().take(ROWS) {
            for (col, value) in fields.iter().enumerate().take(COLS) {
                self.set_cell_raw(CellAddress::new(row, col), value.clone());
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
                    let replaced = self.cells[row][col]
                        .raw
                        .replace(&self.find_query, &self.replace_query);
                    self.set_cell_raw(CellAddress::new(row, col), replaced);
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
                self.mark_cell_changed(CellAddress::new(
                    range.start.row + row_offset,
                    range.start.col + col_offset,
                ));
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
        self.mark_all_dirty();
        self.status = format!("Named range '{}' = {}", name, format_range(self.range));
    }

    fn apply_filter(&mut self) {
        if self.filter_query.trim().is_empty() {
            self.filters.remove(&self.filter_column);
            self.status = format!("Cleared filter on {}", column_name(self.filter_column));
        } else {
            self.filters
                .insert(self.filter_column, self.filter_query.trim().to_owned());
            self.status = format!(
                "Filtered {} by '{}'",
                column_name(self.filter_column),
                self.filter_query.trim()
            );
        }
    }

    fn clear_filters(&mut self) {
        self.filters.clear();
        self.filter_query.clear();
        self.status = "Cleared all filters".to_owned();
    }

    fn row_matches_filters(&self, evaluated: &[Vec<CalcValue>], row: usize) -> bool {
        if row == 0 || self.filters.is_empty() {
            return true;
        }

        self.filters.iter().all(|(col, query)| {
            if *col >= COLS {
                return true;
            }
            let text = display_value(
                &evaluated[row][*col],
                self.cells[row][*col].style.format,
                false,
                &self.cells[row][*col].raw,
            )
            .to_lowercase();
            text.contains(&query.to_lowercase())
        })
    }

    fn add_conditional_rule(&mut self) {
        self.conditional_rules.push(ConditionalRule {
            range: self.range.normalized(),
            operator: self.conditional_operator,
            threshold: self.conditional_threshold,
            second_threshold: self.conditional_second_threshold,
            fill: self.conditional_fill,
        });
        self.status = format!("Added conditional format on {}", format_range(self.range));
    }

    fn conditional_fill_for(&self, address: CellAddress, value: &CalcValue) -> Option<FillColor> {
        let number = value.numeric_value()?;
        self.conditional_rules
            .iter()
            .rev()
            .find(|rule| rule.range.contains(address) && compare_number_rule(number, rule))
            .map(|rule| rule.fill)
    }

    fn add_validation_rule(&mut self) {
        let list = self
            .validation_list
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        self.validation_rules.push(ValidationRule {
            range: self.range.normalized(),
            kind: self.validation_kind,
            min: self.validation_min,
            max: self.validation_max,
            list,
        });
        self.status = format!("Added data validation on {}", format_range(self.range));
    }

    fn is_cell_invalid(&self, address: CellAddress, value: &CalcValue) -> bool {
        self.validation_rules
            .iter()
            .any(|rule| rule.range.contains(address) && !validation_passes(rule, value))
    }

    fn invalid_cell_count(&self, evaluated: &[Vec<CalcValue>]) -> usize {
        let mut count = 0;
        for row in 0..ROWS {
            for col in 0..COLS {
                if self.is_cell_invalid(CellAddress::new(row, col), &evaluated[row][col]) {
                    count += 1;
                }
            }
        }
        count
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
            CalcValue::Number(number)
            | CalcValue::Currency(number)
            | CalcValue::Percent(number) => Self {
                number: Some(OrderedFloat(*number)),
                text: String::new(),
            },
            CalcValue::Date(days) => Self {
                number: Some(OrderedFloat(*days as f64)),
                text: String::new(),
            },
            CalcValue::Boolean(value) => Self {
                number: Some(OrderedFloat(if *value { 1.0 } else { 0.0 })),
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
    values: Vec<f64>,
}

impl SelectionStats {
    fn average(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    fn std_dev(&self) -> Option<f64> {
        std_dev(self.values.clone(), true).ok()
    }

    fn percentile(&self, percentile_value: f64) -> Option<f64> {
        percentile(self.values.clone(), percentile_value).ok()
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
                self.set_cell_raw(self.selected, self.formula_input.clone());
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
                    metric_line(
                        ui,
                        "Std dev",
                        stats
                            .std_dev()
                            .map(format_number)
                            .unwrap_or_else(|| "-".to_owned()),
                    );
                    metric_line(
                        ui,
                        "P25 / P50 / P75",
                        format!(
                            "{} / {} / {}",
                            stats
                                .percentile(0.25)
                                .map(format_number)
                                .unwrap_or_else(|| "-".to_owned()),
                            stats
                                .percentile(0.50)
                                .map(format_number)
                                .unwrap_or_else(|| "-".to_owned()),
                            stats
                                .percentile(0.75)
                                .map(format_number)
                                .unwrap_or_else(|| "-".to_owned())
                        ),
                    );
                });

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(RichText::new("Chart").strong().color(palette::TEXT));
                    ComboBox::from_id_salt("chart_kind")
                        .selected_text(self.chart_kind.label())
                        .show_ui(ui, |ui| {
                            for kind in [
                                ChartKind::Bar,
                                ChartKind::Line,
                                ChartKind::Pie,
                                ChartKind::Scatter,
                            ] {
                                ui.selectable_value(&mut self.chart_kind, kind, kind.label());
                            }
                        });
                    ui.add_space(6.0);
                    chart_view(
                        ui,
                        self.chart_kind,
                        &chart_points_for_range(evaluated, self.range),
                    );
                });

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(
                        RichText::new("Grouped Summary")
                            .strong()
                            .color(palette::TEXT),
                    );
                    let groups = grouped_summary(evaluated, self.range);
                    if groups.is_empty() {
                        ui.label(
                            RichText::new("Select at least two columns").color(palette::MUTED),
                        );
                    } else {
                        for group in groups.iter().take(8) {
                            metric_line(
                                ui,
                                &group.key,
                                format!(
                                    "sum {}  avg {}  n {}",
                                    format_number(group.sum),
                                    format_number(group.average()),
                                    group.count
                                ),
                            );
                        }
                    }
                });

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(RichText::new("Formulas").strong().color(palette::TEXT));
                    ui.horizontal_wrapped(|ui| {
                        for formula in [
                            "SUM",
                            "AVG",
                            "MIN",
                            "MAX",
                            "COUNT",
                            "MEDIAN",
                            "STDEV",
                            "VAR",
                            "CORREL",
                            "SUMIF",
                            "COUNTIF",
                            "VLOOKUP",
                            "XLOOKUP",
                            "IF",
                            "ROUNDUP",
                            "PERCENTILE",
                            "QUARTILE",
                            "MODE",
                            "COVARIANCE",
                        ] {
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
                    ui.horizontal(|ui| {
                        ComboBox::from_id_salt("filter_column")
                            .selected_text(column_name(self.filter_column))
                            .show_ui(ui, |ui| {
                                for col in 0..COLS {
                                    ui.selectable_value(
                                        &mut self.filter_column,
                                        col,
                                        column_name(col),
                                    );
                                }
                            });
                        ui.add_sized(
                            [118.0, 26.0],
                            TextEdit::singleline(&mut self.filter_query).hint_text("Filter text"),
                        );
                    });
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Apply filter").clicked() {
                            self.apply_filter();
                        }
                        if ui.button("Clear filters").clicked() {
                            self.clear_filters();
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

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(
                        RichText::new("Conditional Formatting")
                            .strong()
                            .color(palette::TEXT),
                    );
                    ui.horizontal(|ui| {
                        ComboBox::from_id_salt("conditional_operator")
                            .selected_text(self.conditional_operator.label())
                            .show_ui(ui, |ui| {
                                for operator in [
                                    CompareOperator::GreaterThan,
                                    CompareOperator::LessThan,
                                    CompareOperator::Equal,
                                    CompareOperator::Between,
                                ] {
                                    ui.selectable_value(
                                        &mut self.conditional_operator,
                                        operator,
                                        operator.label(),
                                    );
                                }
                            });
                        ComboBox::from_id_salt("conditional_fill")
                            .selected_text(self.conditional_fill.label())
                            .show_ui(ui, |ui| {
                                for fill in [
                                    FillColor::Blue,
                                    FillColor::Green,
                                    FillColor::Amber,
                                    FillColor::Rose,
                                ] {
                                    ui.selectable_value(
                                        &mut self.conditional_fill,
                                        fill,
                                        fill.label(),
                                    );
                                }
                            });
                    });
                    ui.add(
                        egui::Slider::new(&mut self.conditional_threshold, -100000.0..=100000.0)
                            .text("Value"),
                    );
                    if self.conditional_operator == CompareOperator::Between {
                        ui.add(
                            egui::Slider::new(
                                &mut self.conditional_second_threshold,
                                -100000.0..=100000.0,
                            )
                            .text("And"),
                        );
                    }
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Add rule").clicked() {
                            self.add_conditional_rule();
                        }
                        if ui.button("Clear rules").clicked() {
                            self.conditional_rules.clear();
                            self.status = "Cleared conditional formats".to_owned();
                        }
                    });
                });

                ui.add_space(10.0);
                card(ui, |ui| {
                    ui.label(
                        RichText::new("Data Validation")
                            .strong()
                            .color(palette::TEXT),
                    );
                    ComboBox::from_id_salt("validation_kind")
                        .selected_text(self.validation_kind.label())
                        .show_ui(ui, |ui| {
                            for kind in [
                                ValidationKind::NumberBetween,
                                ValidationKind::DateBetween,
                                ValidationKind::List,
                                ValidationKind::NonEmpty,
                            ] {
                                ui.selectable_value(&mut self.validation_kind, kind, kind.label());
                            }
                        });
                    if matches!(
                        self.validation_kind,
                        ValidationKind::NumberBetween | ValidationKind::DateBetween
                    ) {
                        ui.add(
                            egui::Slider::new(&mut self.validation_min, -100000.0..=100000.0)
                                .text("Min"),
                        );
                        ui.add(
                            egui::Slider::new(&mut self.validation_max, -100000.0..=100000.0)
                                .text("Max"),
                        );
                    }
                    if self.validation_kind == ValidationKind::List {
                        ui.add(
                            TextEdit::singleline(&mut self.validation_list)
                                .hint_text("Allowed values, comma separated"),
                        );
                    }
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Add validation").clicked() {
                            self.add_validation_rule();
                        }
                        if ui.button("Clear validation").clicked() {
                            self.validation_rules.clear();
                            self.status = "Cleared data validation".to_owned();
                        }
                    });
                    metric_line(
                        ui,
                        "Invalid cells",
                        self.invalid_cell_count(evaluated).to_string(),
                    );
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
                            if !self.row_matches_filters(evaluated, row) {
                                continue;
                            }
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
        if let Some(rule_fill) = self.conditional_fill_for(address, value) {
            fill = rule_fill.color();
        }
        if is_in_range {
            fill = blend(fill, palette::RANGE, 0.18);
        }
        if is_selected {
            fill = blend(fill, palette::BLUE, 0.24);
        }

        let invalid = self.is_cell_invalid(address, value);
        let stroke = if invalid {
            Stroke::new(2.0, palette::ROSE)
        } else if is_selected {
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
                    let mut raw = self.cells[row][col].raw.clone();
                    let response = ui.add_sized(
                        [width - 8.0, height - 4.0],
                        TextEdit::singleline(&mut raw).desired_width(width - 8.0),
                    );
                    if response.changed() {
                        self.set_cell_raw(address, raw.clone());
                        self.formula_input = raw;
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
            ui.separator();
            ui.label(format!("Recalc {}", self.last_recalc_count));
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

#[derive(Clone)]
struct ChartPoint {
    label: String,
    x: f64,
    y: f64,
}

#[derive(Clone)]
struct GroupSummary {
    key: String,
    count: usize,
    sum: f64,
}

impl GroupSummary {
    fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

fn chart_points_for_range(evaluated: &[Vec<CalcValue>], range: CellRange) -> Vec<ChartPoint> {
    let range = range.normalized();
    let mut points = Vec::new();
    if range.width() >= 2 {
        for row in range.start.row..=range.end.row {
            let label = evaluated[row][range.start.col].text_value();
            if let Some(y) = evaluated[row][range.start.col + 1].numeric_value() {
                points.push(ChartPoint {
                    label: if label.is_empty() {
                        format_address(CellAddress::new(row, range.start.col))
                    } else {
                        label
                    },
                    x: points.len() as f64,
                    y,
                });
            }
        }
    } else {
        for row in range.start.row..=range.end.row {
            if let Some(y) = evaluated[row][range.start.col].numeric_value() {
                points.push(ChartPoint {
                    label: format_address(CellAddress::new(row, range.start.col)),
                    x: points.len() as f64,
                    y,
                });
            }
        }
    }
    points
}

fn chart_view(ui: &mut Ui, kind: ChartKind, points: &[ChartPoint]) {
    let desired = Vec2::new(ui.available_width(), 210.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, egui::CornerRadius::same(8), palette::FIELD);
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(8),
        Stroke::new(1.0, palette::BORDER),
        egui::StrokeKind::Outside,
    );

    if points.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Select numeric data",
            FontId::new(14.0, FontFamily::Proportional),
            palette::MUTED,
        );
        return;
    }

    match kind {
        ChartKind::Bar => draw_bar_chart(&painter, rect, points),
        ChartKind::Line => draw_line_chart(&painter, rect, points),
        ChartKind::Pie => draw_pie_chart(&painter, rect, points),
        ChartKind::Scatter => draw_scatter_chart(&painter, rect, points),
    }
}

fn chart_bounds(points: &[ChartPoint]) -> (f64, f64, f64, f64) {
    let min_x = points
        .iter()
        .map(|point| point.x)
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = points
        .iter()
        .map(|point| point.y)
        .fold(f64::INFINITY, f64::min)
        .min(0.0);
    let max_y = points
        .iter()
        .map(|point| point.y)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(0.0);
    (min_x, max_x, min_y, max_y)
}

fn chart_pos(rect: egui::Rect, x: f64, y: f64, bounds: (f64, f64, f64, f64)) -> egui::Pos2 {
    let plot = rect.shrink2(Vec2::new(18.0, 18.0));
    let (min_x, max_x, min_y, max_y) = bounds;
    let x_span = (max_x - min_x).abs().max(1.0);
    let y_span = (max_y - min_y).abs().max(1.0);
    egui::pos2(
        plot.left() + ((x - min_x) / x_span) as f32 * plot.width(),
        plot.bottom() - ((y - min_y) / y_span) as f32 * plot.height(),
    )
}

fn draw_bar_chart(painter: &egui::Painter, rect: egui::Rect, points: &[ChartPoint]) {
    let plot = rect.shrink2(Vec2::new(18.0, 18.0));
    let max = points
        .iter()
        .map(|point| point.y.abs())
        .fold(0.0, f64::max)
        .max(1.0);
    let slot = plot.width() / points.len().max(1) as f32;
    for (index, point) in points.iter().enumerate() {
        let height = (point.y.abs() / max) as f32 * plot.height();
        let left = plot.left() + index as f32 * slot + slot * 0.16;
        let right = left + slot * 0.68;
        let bar = egui::Rect::from_min_max(
            egui::pos2(left, plot.bottom() - height),
            egui::pos2(right, plot.bottom()),
        );
        painter.rect_filled(bar, egui::CornerRadius::same(3), palette::BLUE);
        if index < 4 {
            painter.text(
                egui::pos2((left + right) / 2.0, plot.bottom() + 2.0),
                egui::Align2::CENTER_TOP,
                &point.label,
                FontId::new(10.0, FontFamily::Proportional),
                palette::MUTED,
            );
        }
    }
}

fn draw_line_chart(painter: &egui::Painter, rect: egui::Rect, points: &[ChartPoint]) {
    let bounds = chart_bounds(points);
    let mut previous = None;
    for point in points {
        let pos = chart_pos(rect, point.x, point.y, bounds);
        painter.circle_filled(pos, 3.5, palette::GREEN);
        if let Some(previous) = previous {
            painter.line_segment([previous, pos], Stroke::new(2.0, palette::BLUE));
        }
        previous = Some(pos);
    }
}

fn draw_scatter_chart(painter: &egui::Painter, rect: egui::Rect, points: &[ChartPoint]) {
    let bounds = chart_bounds(points);
    for point in points {
        let pos = chart_pos(rect, point.x, point.y, bounds);
        painter.circle_filled(pos, 4.0, palette::AMBER);
    }
}

fn draw_pie_chart(painter: &egui::Painter, rect: egui::Rect, points: &[ChartPoint]) {
    let total: f64 = points.iter().map(|point| point.y.max(0.0)).sum();
    if total <= 0.0 {
        return;
    }

    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.34;
    let colors = [palette::BLUE, palette::GREEN, palette::AMBER, palette::ROSE];
    let mut angle = -std::f32::consts::FRAC_PI_2;
    for (index, point) in points.iter().enumerate() {
        let sweep = (point.y.max(0.0) / total) as f32 * std::f32::consts::TAU;
        let steps = (sweep.abs() / 0.16).ceil().max(2.0) as usize;
        let mut vertices = Vec::with_capacity(steps + 2);
        vertices.push(center);
        for step in 0..=steps {
            let current = angle + sweep * step as f32 / steps as f32;
            vertices.push(egui::pos2(
                center.x + radius * current.cos(),
                center.y + radius * current.sin(),
            ));
        }
        painter.add(egui::Shape::convex_polygon(
            vertices,
            colors[index % colors.len()],
            Stroke::new(1.0, palette::FIELD),
        ));
        angle += sweep;
    }
}

fn grouped_summary(evaluated: &[Vec<CalcValue>], range: CellRange) -> Vec<GroupSummary> {
    let range = range.normalized();
    if range.width() < 2 {
        return Vec::new();
    }

    let mut groups: HashMap<String, GroupSummary> = HashMap::new();
    for row in range.start.row..=range.end.row {
        let key = evaluated[row][range.start.col].text_value();
        let Some(value) = evaluated[row][range.start.col + 1].numeric_value() else {
            continue;
        };
        let key = if key.is_empty() {
            "(blank)".to_owned()
        } else {
            key
        };
        let entry = groups.entry(key.clone()).or_insert(GroupSummary {
            key,
            count: 0,
            sum: 0.0,
        });
        entry.count += 1;
        entry.sum += value;
    }

    let mut groups: Vec<GroupSummary> = groups.into_values().collect();
    groups.sort_by(|left, right| right.sum.total_cmp(&left.sum));
    groups
}

fn compare_number_rule(value: f64, rule: &ConditionalRule) -> bool {
    match rule.operator {
        CompareOperator::GreaterThan => value > rule.threshold,
        CompareOperator::LessThan => value < rule.threshold,
        CompareOperator::Equal => (value - rule.threshold).abs() <= f64::EPSILON,
        CompareOperator::Between => {
            let low = rule.threshold.min(rule.second_threshold);
            let high = rule.threshold.max(rule.second_threshold);
            value >= low && value <= high
        }
    }
}

fn validation_passes(rule: &ValidationRule, value: &CalcValue) -> bool {
    match rule.kind {
        ValidationKind::NumberBetween => value
            .numeric_value()
            .is_some_and(|number| number >= rule.min && number <= rule.max),
        ValidationKind::DateBetween => {
            matches!(value, CalcValue::Date(days) if (*days as f64) >= rule.min && (*days as f64) <= rule.max)
        }
        ValidationKind::List => {
            let value = value.text_value();
            rule.list
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(value.trim()))
        }
        ValidationKind::NonEmpty => {
            !matches!(value, CalcValue::Empty)
                && !matches!(value, CalcValue::Text(text) if text.trim().is_empty())
        }
    }
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
        return CalcValue::Error(ERR_REF.to_owned());
    }

    if let Some(value) = &cache[row][col] {
        return value.clone();
    }

    let address = CellAddress::new(row, col);
    if visiting.contains(&address) {
        return CalcValue::Error(ERR_CYCLE.to_owned());
    }

    visiting.push(address);
    let raw = cells[row][col].raw.trim();
    let value = if let Some(formula) = raw.strip_prefix('=') {
        let mut parser = FormulaParser::new(formula, cells, named_ranges, cache, visiting);
        match parser.parse() {
            Ok(value) => normalize_calc_value(value),
            Err(error) => CalcValue::Error(error),
        }
    } else {
        parse_typed_literal(raw)
    };
    visiting.pop();
    cache[row][col] = Some(value.clone());
    value
}

fn parse_typed_literal(raw: &str) -> CalcValue {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return CalcValue::Empty;
    }

    match trimmed.to_ascii_uppercase().as_str() {
        "TRUE" => return CalcValue::Boolean(true),
        "FALSE" => return CalcValue::Boolean(false),
        _ => {}
    }

    if let Some(days) = parse_iso_date(trimmed) {
        return CalcValue::Date(days);
    }

    if let Some(rest) = trimmed.strip_prefix('$') {
        if let Some(number) = parse_number_text(rest) {
            return CalcValue::Currency(number);
        }
    }

    if let Some(rest) = trimmed.strip_suffix('%') {
        if let Some(number) = parse_number_text(rest) {
            return CalcValue::Percent(number / 100.0);
        }
    }

    if let Some(number) = parse_number_text(trimmed) {
        CalcValue::Number(number)
    } else {
        CalcValue::Text(trimmed.to_owned())
    }
}

fn normalize_calc_value(value: CalcValue) -> CalcValue {
    match value {
        CalcValue::Number(number) if !number.is_finite() => CalcValue::Error(ERR_NUM.to_owned()),
        CalcValue::Currency(number) if !number.is_finite() => CalcValue::Error(ERR_NUM.to_owned()),
        CalcValue::Percent(number) if !number.is_finite() => CalcValue::Error(ERR_NUM.to_owned()),
        other => other,
    }
}

#[derive(Clone)]
enum FormulaArg {
    Value(CalcValue),
    Range(CellRange),
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

    fn parse(&mut self) -> Result<CalcValue, String> {
        let value = self.comparison()?;
        self.skip_ws();
        if self.pos < self.chars.len() {
            Err(ERR_VALUE.to_owned())
        } else {
            Ok(value)
        }
    }

    fn comparison(&mut self) -> Result<CalcValue, String> {
        let mut left = self.expression()?;
        loop {
            self.skip_ws();
            let operator = if self.consume_str(">=") {
                Some(">=")
            } else if self.consume_str("<=") {
                Some("<=")
            } else if self.consume_str("<>") {
                Some("<>")
            } else if self.consume('>') {
                Some(">")
            } else if self.consume('<') {
                Some("<")
            } else if self.consume('=') {
                Some("=")
            } else {
                None
            };

            if let Some(operator) = operator {
                let right = self.expression()?;
                left = CalcValue::Boolean(compare_values(&left, operator, &right)?);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn expression(&mut self) -> Result<CalcValue, String> {
        let mut value = self.term()?;
        loop {
            self.skip_ws();
            if self.consume('+') {
                let right = self.term()?;
                value = CalcValue::Number(value_to_number(&value)? + value_to_number(&right)?);
            } else if self.consume('-') {
                let right = self.term()?;
                value = CalcValue::Number(value_to_number(&value)? - value_to_number(&right)?);
            } else {
                break;
            }
        }
        Ok(value)
    }

    fn term(&mut self) -> Result<CalcValue, String> {
        let mut value = self.factor()?;
        loop {
            self.skip_ws();
            if self.consume('*') {
                let right = self.factor()?;
                value = CalcValue::Number(value_to_number(&value)? * value_to_number(&right)?);
            } else if self.consume('/') {
                let right = self.factor()?;
                let divisor = value_to_number(&right)?;
                if divisor == 0.0 {
                    return Err(ERR_DIV_ZERO.to_owned());
                }
                value = CalcValue::Number(value_to_number(&value)? / divisor);
            } else {
                break;
            }
        }
        Ok(value)
    }

    fn factor(&mut self) -> Result<CalcValue, String> {
        self.skip_ws();
        if self.consume('-') {
            return Ok(CalcValue::Number(-value_to_number(&self.factor()?)?));
        }
        if self.consume('+') {
            return self.factor();
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<CalcValue, String> {
        self.skip_ws();
        if self.consume('(') {
            let value = self.comparison()?;
            self.expect(')')?;
            return Ok(value);
        }

        if self.peek() == Some('"') {
            return self.string_literal().map(CalcValue::Text);
        }

        if self
            .peek()
            .map_or(false, |ch| ch.is_ascii_digit() || ch == '.')
        {
            return self.number_literal();
        }

        if self.peek().map_or(false, |ch| ch.is_ascii_alphabetic()) {
            let start = self.pos;
            let ident = self.identifier();
            self.skip_ws();
            if self.consume('(') {
                return self.function(&ident);
            }

            match ident.to_ascii_uppercase().as_str() {
                "TRUE" => return Ok(CalcValue::Boolean(true)),
                "FALSE" => return Ok(CalcValue::Boolean(false)),
                _ => {}
            }

            self.pos = start;
            if let Some(address) = self.cell_ref() {
                if self.consume(':') {
                    if let Some(end) = self.cell_ref() {
                        return Ok(CalcValue::Number(kahan_sum(&self.range_numbers(
                            CellRange {
                                start: address,
                                end,
                            },
                        )?)));
                    }
                    return Err(ERR_REF.to_owned());
                }
                return self.cell_value(address);
            }

            self.pos = start;
            let named = self.identifier();
            if let Some(range) = lookup_named_range(self.named_ranges, &named) {
                return Ok(CalcValue::Number(kahan_sum(&self.range_numbers(range)?)));
            }
        }

        Err(ERR_VALUE.to_owned())
    }

    fn function(&mut self, name: &str) -> Result<CalcValue, String> {
        let args = self.arguments()?;
        let upper = name.to_ascii_uppercase();
        match upper.as_str() {
            "SUM" => Ok(CalcValue::Number(kahan_sum(&self.args_numbers(&args)?))),
            "AVG" | "AVERAGE" => {
                let numbers = self.args_numbers(&args)?;
                if numbers.is_empty() {
                    Err(ERR_DIV_ZERO.to_owned())
                } else {
                    Ok(CalcValue::Number(
                        kahan_sum(&numbers) / numbers.len() as f64,
                    ))
                }
            }
            "MIN" => self
                .args_numbers(&args)?
                .into_iter()
                .reduce(f64::min)
                .map(CalcValue::Number)
                .ok_or_else(|| ERR_VALUE.to_owned()),
            "MAX" => self
                .args_numbers(&args)?
                .into_iter()
                .reduce(f64::max)
                .map(CalcValue::Number)
                .ok_or_else(|| ERR_VALUE.to_owned()),
            "COUNT" => Ok(CalcValue::Number(self.args_numbers(&args)?.len() as f64)),
            "MEDIAN" => Ok(CalcValue::Number(median(self.args_numbers(&args)?)?)),
            "STDEV" | "STDEV.S" => Ok(CalcValue::Number(std_dev(self.args_numbers(&args)?, true)?)),
            "STDEV.P" => Ok(CalcValue::Number(std_dev(
                self.args_numbers(&args)?,
                false,
            )?)),
            "VAR" | "VAR.S" => Ok(CalcValue::Number(variance(
                self.args_numbers(&args)?,
                true,
            )?)),
            "VAR.P" => Ok(CalcValue::Number(variance(
                self.args_numbers(&args)?,
                false,
            )?)),
            "PERCENTILE" | "PERCENTILE.INC" => self.percentile_function(&args),
            "QUARTILE" | "QUARTILE.INC" => self.quartile_function(&args),
            "MODE" | "MODE.SNGL" => Ok(CalcValue::Number(mode(self.args_numbers(&args)?)?)),
            "COVARIANCE" | "COVARIANCE.S" => self.covariance_function(&args, true),
            "COVARIANCE.P" => self.covariance_function(&args, false),
            "CORREL" => self.correl(&args),
            "ABS" => Ok(CalcValue::Number(
                value_to_number(&single_arg(&args)?)?.abs(),
            )),
            "ROUND" => self.round(&args, false),
            "ROUNDUP" => self.round(&args, true),
            "IF" => self.if_function(&args),
            "SUMIF" => self.sumif(&args),
            "COUNTIF" => self.countif(&args),
            "VLOOKUP" => self.vlookup(&args),
            "XLOOKUP" => self.xlookup(&args),
            _ => Err(ERR_NAME.to_owned()),
        }
    }

    fn arguments(&mut self) -> Result<Vec<FormulaArg>, String> {
        let mut args = Vec::new();
        self.skip_ws();
        if self.consume(')') {
            return Ok(args);
        }

        loop {
            self.skip_ws();
            if let Some(range) = self.try_range_argument() {
                args.push(FormulaArg::Range(range));
            } else {
                args.push(FormulaArg::Value(self.comparison()?));
            }

            self.skip_ws();
            if self.consume(',') || self.consume(';') {
                continue;
            }
            self.expect(')')?;
            break;
        }

        Ok(args)
    }

    fn args_numbers(&mut self, args: &[FormulaArg]) -> Result<Vec<f64>, String> {
        let mut values = Vec::new();
        for arg in args {
            match arg {
                FormulaArg::Value(value) => {
                    if let Some(number) = value.numeric_value() {
                        values.push(number);
                    }
                }
                FormulaArg::Range(range) => values.extend(self.range_numbers(*range)?),
            }
        }
        Ok(values)
    }

    fn if_function(&mut self, args: &[FormulaArg]) -> Result<CalcValue, String> {
        if args.len() < 2 {
            return Err(ERR_VALUE.to_owned());
        }
        let condition = match args[0] {
            FormulaArg::Value(ref value) => value.truthy()?,
            FormulaArg::Range(range) => !self.range_numbers(range)?.is_empty(),
        };

        if condition {
            self.arg_value(args.get(1))
        } else {
            self.arg_value(args.get(2))
                .or(Ok(CalcValue::Boolean(false)))
        }
    }

    fn sumif(&mut self, args: &[FormulaArg]) -> Result<CalcValue, String> {
        if args.len() < 2 {
            return Err(ERR_VALUE.to_owned());
        }
        let criteria_range = expect_range(args.first())?;
        let criteria = Criteria::from_value(&self.arg_value(args.get(1))?);
        let sum_range = match args.get(2) {
            Some(FormulaArg::Range(range)) => Some(range.normalized()),
            Some(_) => return Err(ERR_VALUE.to_owned()),
            None => None,
        };

        let criteria_range = criteria_range.normalized();
        let mut numbers = Vec::new();
        for row in criteria_range.start.row..=criteria_range.end.row {
            for col in criteria_range.start.col..=criteria_range.end.col {
                let candidate = self.cell_value(CellAddress::new(row, col))?;
                if criteria.matches(&candidate) {
                    let target = if let Some(sum_range) = sum_range {
                        CellAddress::new(
                            sum_range.start.row + (row - criteria_range.start.row),
                            sum_range.start.col + (col - criteria_range.start.col),
                        )
                    } else {
                        CellAddress::new(row, col)
                    };
                    if target.row < ROWS && target.col < COLS {
                        if let Some(number) = self.cell_value(target)?.numeric_value() {
                            numbers.push(number);
                        }
                    }
                }
            }
        }
        Ok(CalcValue::Number(kahan_sum(&numbers)))
    }

    fn countif(&mut self, args: &[FormulaArg]) -> Result<CalcValue, String> {
        if args.len() < 2 {
            return Err(ERR_VALUE.to_owned());
        }
        let range = expect_range(args.first())?.normalized();
        let criteria = Criteria::from_value(&self.arg_value(args.get(1))?);
        let mut count = 0usize;
        for row in range.start.row..=range.end.row {
            for col in range.start.col..=range.end.col {
                if criteria.matches(&self.cell_value(CellAddress::new(row, col))?) {
                    count += 1;
                }
            }
        }
        Ok(CalcValue::Number(count as f64))
    }

    fn vlookup(&mut self, args: &[FormulaArg]) -> Result<CalcValue, String> {
        if args.len() < 3 {
            return Err(ERR_VALUE.to_owned());
        }
        let lookup = self.arg_value(args.first())?;
        let table = expect_range(args.get(1))?.normalized();
        let return_col = value_to_number(&self.arg_value(args.get(2))?)?.round() as usize;
        if return_col == 0 || table.start.col + return_col - 1 > table.end.col {
            return Err(ERR_REF.to_owned());
        }

        for row in table.start.row..=table.end.row {
            let candidate = self.cell_value(CellAddress::new(row, table.start.col))?;
            if compare_values(&candidate, "=", &lookup)? {
                return self.cell_value(CellAddress::new(row, table.start.col + return_col - 1));
            }
        }
        Err(ERR_NA.to_owned())
    }

    fn xlookup(&mut self, args: &[FormulaArg]) -> Result<CalcValue, String> {
        if args.len() < 3 {
            return Err(ERR_VALUE.to_owned());
        }
        let lookup = self.arg_value(args.first())?;
        let lookup_range = expect_range(args.get(1))?.normalized();
        let return_range = expect_range(args.get(2))?.normalized();
        for row in lookup_range.start.row..=lookup_range.end.row {
            for col in lookup_range.start.col..=lookup_range.end.col {
                let candidate = self.cell_value(CellAddress::new(row, col))?;
                if compare_values(&candidate, "=", &lookup)? {
                    let out_row = return_range.start.row + (row - lookup_range.start.row);
                    let out_col = return_range.start.col + (col - lookup_range.start.col);
                    if out_row < ROWS && out_col < COLS {
                        return self.cell_value(CellAddress::new(out_row, out_col));
                    }
                    return Err(ERR_REF.to_owned());
                }
            }
        }
        if let Some(fallback) = args.get(3) {
            self.arg_value(Some(fallback))
        } else {
            Err(ERR_NA.to_owned())
        }
    }

    fn correl(&mut self, args: &[FormulaArg]) -> Result<CalcValue, String> {
        if args.len() < 2 {
            return Err(ERR_VALUE.to_owned());
        }
        let left = match args[0] {
            FormulaArg::Range(range) => self.range_numbers(range)?,
            FormulaArg::Value(_) => return Err(ERR_VALUE.to_owned()),
        };
        let right = match args[1] {
            FormulaArg::Range(range) => self.range_numbers(range)?,
            FormulaArg::Value(_) => return Err(ERR_VALUE.to_owned()),
        };
        Ok(CalcValue::Number(correlation(&left, &right)?))
    }

    fn percentile_function(&mut self, args: &[FormulaArg]) -> Result<CalcValue, String> {
        if args.len() < 2 {
            return Err(ERR_VALUE.to_owned());
        }
        let values = self.arg_numbers(args.first())?;
        let k = value_to_number(&self.arg_value(args.get(1))?)?;
        Ok(CalcValue::Number(percentile(values, k)?))
    }

    fn quartile_function(&mut self, args: &[FormulaArg]) -> Result<CalcValue, String> {
        if args.len() < 2 {
            return Err(ERR_VALUE.to_owned());
        }
        let values = self.arg_numbers(args.first())?;
        let quartile = value_to_number(&self.arg_value(args.get(1))?)?.round();
        if !(0.0..=4.0).contains(&quartile) {
            return Err(ERR_NUM.to_owned());
        }
        Ok(CalcValue::Number(percentile(values, quartile / 4.0)?))
    }

    fn covariance_function(
        &mut self,
        args: &[FormulaArg],
        sample: bool,
    ) -> Result<CalcValue, String> {
        if args.len() < 2 {
            return Err(ERR_VALUE.to_owned());
        }
        let left = self.arg_numbers(args.first())?;
        let right = self.arg_numbers(args.get(1))?;
        Ok(CalcValue::Number(covariance(&left, &right, sample)?))
    }

    fn round(&mut self, args: &[FormulaArg], up: bool) -> Result<CalcValue, String> {
        if args.is_empty() {
            return Err(ERR_VALUE.to_owned());
        }
        let number = value_to_number(&self.arg_value(args.first())?)?;
        let places = args
            .get(1)
            .map(|arg| {
                self.arg_value(Some(arg))
                    .and_then(|value| value_to_number(&value))
            })
            .transpose()?
            .unwrap_or(0.0)
            .round()
            .clamp(0.0, 8.0);
        let factor = 10_f64.powf(places);
        let scaled = number * factor;
        let rounded = if up {
            if scaled >= 0.0 {
                scaled.ceil()
            } else {
                scaled.floor()
            }
        } else {
            scaled.round()
        };
        Ok(CalcValue::Number(rounded / factor))
    }

    fn arg_value(&mut self, arg: Option<&FormulaArg>) -> Result<CalcValue, String> {
        match arg {
            Some(FormulaArg::Value(value)) => Ok(value.clone()),
            Some(FormulaArg::Range(range)) => {
                Ok(CalcValue::Number(kahan_sum(&self.range_numbers(*range)?)))
            }
            None => Ok(CalcValue::Empty),
        }
    }

    fn arg_numbers(&mut self, arg: Option<&FormulaArg>) -> Result<Vec<f64>, String> {
        match arg {
            Some(FormulaArg::Range(range)) => self.range_numbers(*range),
            Some(FormulaArg::Value(value)) => value
                .numeric_value()
                .map(|number| vec![number])
                .ok_or_else(|| ERR_VALUE.to_owned()),
            None => Err(ERR_VALUE.to_owned()),
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
                let value = self.cell_value(CellAddress::new(row, col))?;
                if let Some(number) = value.numeric_value() {
                    numbers.push(number);
                }
            }
        }
        Ok(numbers)
    }

    fn cell_value(&mut self, address: CellAddress) -> Result<CalcValue, String> {
        let address = clamp_address(address);
        match evaluate_cell(
            self.cells,
            self.named_ranges,
            self.cache,
            self.visiting,
            address.row,
            address.col,
        ) {
            CalcValue::Error(error) => Err(error),
            value => Ok(value),
        }
    }

    fn number_literal(&mut self) -> Result<CalcValue, String> {
        let start = self.pos;
        while self
            .peek()
            .map_or(false, |ch| ch.is_ascii_digit() || ch == '.')
        {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let number = parse_number_text(&text).ok_or_else(|| ERR_VALUE.to_owned())?;
        if self.consume('%') {
            Ok(CalcValue::Percent(number / 100.0))
        } else {
            Ok(CalcValue::Number(number))
        }
    }

    fn string_literal(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            self.pos += 1;
            if ch == '"' {
                if self.peek() == Some('"') {
                    value.push('"');
                    self.pos += 1;
                } else {
                    return Ok(value);
                }
            } else {
                value.push(ch);
            }
        }
        Err(ERR_VALUE.to_owned())
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
        let (address, next) = parse_cell_ref_at(&self.chars, self.pos)?;
        self.pos = next;
        Some(address)
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

    fn consume_str(&mut self, expected: &str) -> bool {
        self.skip_ws();
        let expected_chars: Vec<char> = expected.chars().collect();
        if self.chars[self.pos..].starts_with(&expected_chars) {
            self.pos += expected_chars.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(ERR_VALUE.to_owned())
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
}

#[derive(Clone)]
struct Criteria {
    operator: String,
    value: CalcValue,
}

impl Criteria {
    fn from_value(value: &CalcValue) -> Self {
        if let CalcValue::Text(text) = value {
            for operator in [">=", "<=", "<>", ">", "<", "="] {
                if let Some(rest) = text.trim().strip_prefix(operator) {
                    return Self {
                        operator: operator.to_owned(),
                        value: parse_typed_literal(rest.trim()),
                    };
                }
            }
        }

        Self {
            operator: "=".to_owned(),
            value: value.clone(),
        }
    }

    fn matches(&self, candidate: &CalcValue) -> bool {
        compare_values(candidate, &self.operator, &self.value).unwrap_or(false)
    }
}

fn single_arg(args: &[FormulaArg]) -> Result<CalcValue, String> {
    match args.first() {
        Some(FormulaArg::Value(value)) => Ok(value.clone()),
        Some(FormulaArg::Range(_)) => Err(ERR_VALUE.to_owned()),
        None => Err(ERR_VALUE.to_owned()),
    }
}

fn expect_range(arg: Option<&FormulaArg>) -> Result<CellRange, String> {
    match arg {
        Some(FormulaArg::Range(range)) => Ok(*range),
        _ => Err(ERR_VALUE.to_owned()),
    }
}

fn value_to_number(value: &CalcValue) -> Result<f64, String> {
    match value {
        CalcValue::Empty => Ok(0.0),
        CalcValue::Error(error) => Err(error.clone()),
        _ => value.numeric_value().ok_or_else(|| ERR_VALUE.to_owned()),
    }
}

fn compare_values(left: &CalcValue, operator: &str, right: &CalcValue) -> Result<bool, String> {
    if let (Some(left_number), Some(right_number)) = (left.numeric_value(), right.numeric_value()) {
        return Ok(match operator {
            ">" => left_number > right_number,
            "<" => left_number < right_number,
            ">=" => left_number >= right_number,
            "<=" => left_number <= right_number,
            "<>" => (left_number - right_number).abs() > f64::EPSILON,
            "=" => (left_number - right_number).abs() <= f64::EPSILON,
            _ => false,
        });
    }

    let left_text = left.text_value().to_ascii_lowercase();
    let right_text = right.text_value().to_ascii_lowercase();
    Ok(match operator {
        ">" => left_text > right_text,
        "<" => left_text < right_text,
        ">=" => left_text >= right_text,
        "<=" => left_text <= right_text,
        "<>" => left_text != right_text,
        "=" => left_text == right_text,
        _ => false,
    })
}

fn kahan_sum(values: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut compensation = 0.0;
    for value in values {
        let adjusted = value - compensation;
        let next = sum + adjusted;
        compensation = (next - sum) - adjusted;
        sum = next;
    }
    sum
}

fn median(mut values: Vec<f64>) -> Result<f64, String> {
    if values.is_empty() {
        return Err(ERR_VALUE.to_owned());
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        Ok((values[middle - 1] + values[middle]) / 2.0)
    } else {
        Ok(values[middle])
    }
}

fn percentile(mut values: Vec<f64>, k: f64) -> Result<f64, String> {
    if values.is_empty() {
        return Err(ERR_VALUE.to_owned());
    }
    if !(0.0..=1.0).contains(&k) {
        return Err(ERR_NUM.to_owned());
    }
    values.sort_by(f64::total_cmp);
    if values.len() == 1 {
        return Ok(values[0]);
    }
    let rank = k * (values.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        Ok(values[lower])
    } else {
        let weight = rank - lower as f64;
        Ok(values[lower] * (1.0 - weight) + values[upper] * weight)
    }
}

fn mode(values: Vec<f64>) -> Result<f64, String> {
    if values.is_empty() {
        return Err(ERR_NA.to_owned());
    }
    let mut counts: HashMap<i64, (f64, usize)> = HashMap::new();
    for value in values {
        let key = (value * 1_000_000.0).round() as i64;
        let entry = counts.entry(key).or_insert((value, 0));
        entry.1 += 1;
    }
    counts
        .into_values()
        .filter(|(_, count)| *count > 1)
        .max_by_key(|(_, count)| *count)
        .map(|(value, _)| value)
        .ok_or_else(|| ERR_NA.to_owned())
}

fn variance(values: Vec<f64>, sample: bool) -> Result<f64, String> {
    let divisor_adjustment = if sample { 1 } else { 0 };
    if values.len() <= divisor_adjustment {
        return Err(ERR_DIV_ZERO.to_owned());
    }
    let mean = kahan_sum(&values) / values.len() as f64;
    let squared: Vec<f64> = values.iter().map(|value| (value - mean).powi(2)).collect();
    Ok(kahan_sum(&squared) / (values.len() - divisor_adjustment) as f64)
}

fn std_dev(values: Vec<f64>, sample: bool) -> Result<f64, String> {
    Ok(variance(values, sample)?.sqrt())
}

fn covariance(left: &[f64], right: &[f64], sample: bool) -> Result<f64, String> {
    if left.len() != right.len() || left.is_empty() {
        return Err(ERR_NA.to_owned());
    }
    let adjustment = if sample { 1 } else { 0 };
    if left.len() <= adjustment {
        return Err(ERR_DIV_ZERO.to_owned());
    }
    let left_mean = kahan_sum(left) / left.len() as f64;
    let right_mean = kahan_sum(right) / right.len() as f64;
    let products: Vec<f64> = left
        .iter()
        .zip(right.iter())
        .map(|(left_value, right_value)| (left_value - left_mean) * (right_value - right_mean))
        .collect();
    Ok(kahan_sum(&products) / (left.len() - adjustment) as f64)
}

fn correlation(left: &[f64], right: &[f64]) -> Result<f64, String> {
    if left.len() != right.len() || left.len() < 2 {
        return Err(ERR_NA.to_owned());
    }
    let left_mean = kahan_sum(left) / left.len() as f64;
    let right_mean = kahan_sum(right) / right.len() as f64;
    let mut numerator = Vec::with_capacity(left.len());
    let mut left_sq = Vec::with_capacity(left.len());
    let mut right_sq = Vec::with_capacity(left.len());
    for (left_value, right_value) in left.iter().zip(right.iter()) {
        let left_delta = left_value - left_mean;
        let right_delta = right_value - right_mean;
        numerator.push(left_delta * right_delta);
        left_sq.push(left_delta.powi(2));
        right_sq.push(right_delta.powi(2));
    }
    let denominator = (kahan_sum(&left_sq) * kahan_sum(&right_sq)).sqrt();
    if denominator == 0.0 {
        Err(ERR_DIV_ZERO.to_owned())
    } else {
        Ok(kahan_sum(&numerator) / denominator)
    }
}

fn lookup_named_range(named_ranges: &HashMap<String, CellRange>, name: &str) -> Option<CellRange> {
    named_ranges
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, range)| *range)
}

fn collect_dependencies(raw: &str, named_ranges: &HashMap<String, CellRange>) -> Vec<CellAddress> {
    let Some(formula) = raw.trim().strip_prefix('=') else {
        return Vec::new();
    };

    let chars: Vec<char> = formula.chars().collect();
    let mut dependencies = HashSet::new();
    let mut pos = 0;
    while pos < chars.len() {
        if chars[pos] == '"' {
            pos += 1;
            while pos < chars.len() {
                if chars[pos] == '"' {
                    pos += 1;
                    break;
                }
                pos += 1;
            }
            continue;
        }

        if let Some((start, next)) = parse_cell_ref_at(&chars, pos) {
            pos = next;
            if pos < chars.len() && chars[pos] == ':' {
                if let Some((end, after_end)) = parse_cell_ref_at(&chars, pos + 1) {
                    add_range_dependencies(&mut dependencies, CellRange { start, end });
                    pos = after_end;
                    continue;
                }
            }
            dependencies.insert(clamp_address(start));
            continue;
        }

        if chars[pos].is_ascii_alphabetic() || chars[pos] == '_' {
            let start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                pos += 1;
            }
            let identifier: String = chars[start..pos].iter().collect();
            if let Some(range) = lookup_named_range(named_ranges, &identifier) {
                add_range_dependencies(&mut dependencies, range);
            }
            continue;
        }

        pos += 1;
    }

    dependencies.into_iter().collect()
}

fn add_range_dependencies(target: &mut HashSet<CellAddress>, range: CellRange) {
    let range = clamp_range(range).normalized();
    for row in range.start.row..=range.end.row {
        for col in range.start.col..=range.end.col {
            target.insert(CellAddress::new(row, col));
        }
    }
}

fn parse_cell_ref_at(chars: &[char], start: usize) -> Option<(CellAddress, usize)> {
    let mut pos = start;
    let mut col_name = String::new();
    while pos < chars.len() && chars[pos].is_ascii_alphabetic() {
        col_name.push(chars[pos].to_ascii_uppercase());
        pos += 1;
    }

    let digit_start = pos;
    while pos < chars.len() && chars[pos].is_ascii_digit() {
        pos += 1;
    }

    if col_name.is_empty() || digit_start == pos {
        return None;
    }
    let row = chars[digit_start..pos]
        .iter()
        .collect::<String>()
        .parse::<usize>()
        .ok()?
        .checked_sub(1)?;
    let col = column_index(&col_name)?;
    Some((clamp_address(CellAddress::new(row, col)), pos))
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
        CalcValue::Currency(number) => match format {
            NumberFormat::Percent => format!("{:.1}%", number * 100.0),
            _ => format!("${:.2}", number),
        },
        CalcValue::Percent(number) => match format {
            NumberFormat::Currency => format!("${:.2}", number),
            NumberFormat::Number => format!("{:.2}", number),
            _ => format!("{:.1}%", number * 100.0),
        },
        CalcValue::Boolean(value) => {
            if *value {
                "TRUE".to_owned()
            } else {
                "FALSE".to_owned()
            }
        }
        CalcValue::Date(days) => format_date(*days),
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

fn parse_number_text(text: &str) -> Option<f64> {
    text.trim().replace(',', "").parse::<f64>().ok()
}

fn text_to_number(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if let Some(number) = parse_number_text(trimmed) {
        return Some(number);
    }
    if let Some(rest) = trimmed.strip_prefix('$') {
        return parse_number_text(rest);
    }
    if let Some(rest) = trimmed.strip_suffix('%') {
        return parse_number_text(rest).map(|value| value / 100.0);
    }
    parse_iso_date(trimmed).map(|days| days as f64)
}

fn parse_iso_date(text: &str) -> Option<i64> {
    let mut parts = text.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

fn format_date(days: i64) -> String {
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = month as i32;
    let day = day as i32;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146097 + doe - 719468) as i64
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
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
    let chars: Vec<char> = input.trim().chars().collect();
    parse_cell_ref_at(&chars, 0).and_then(|(address, next)| {
        if next == chars.len() {
            Some(address)
        } else {
            None
        }
    })
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
    pub const AMBER: Color32 = Color32::from_rgb(245, 174, 76);
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
            CalcValue::Error(error) => assert_eq!(error, ERR_CYCLE),
            other => panic!("expected cycle error, got {other:?}"),
        }
    }

    #[test]
    fn parses_boolean_currency_percent_and_dates() {
        assert!(matches!(
            parse_typed_literal("TRUE"),
            CalcValue::Boolean(true)
        ));
        assert!(
            matches!(parse_typed_literal("$1,250.50"), CalcValue::Currency(value) if (value - 1250.50).abs() < 0.001)
        );
        assert!(
            matches!(parse_typed_literal("12.5%"), CalcValue::Percent(value) if (value - 0.125).abs() < 0.001)
        );
        assert!(matches!(
            parse_typed_literal("2026-06-04"),
            CalcValue::Date(_)
        ));
    }

    #[test]
    fn evaluates_if_sumif_countif_and_lookup_functions() {
        let mut cells = blank_cells();
        let named_ranges = HashMap::new();
        for (row, value) in ["1", "2", "3", "4"].iter().enumerate() {
            cells[row][0].raw = value.to_string();
        }
        for (row, value) in ["A", "B", "A", "B"].iter().enumerate() {
            cells[row][1].raw = value.to_string();
        }
        cells[0][3].raw = "A".to_owned();
        cells[0][4].raw = "Alpha".to_owned();
        cells[1][3].raw = "B".to_owned();
        cells[1][4].raw = "Beta".to_owned();

        cells[5][0].raw = "=IF(SUM(A1:A4)>9,\"high\",\"low\")".to_owned();
        cells[5][1].raw = "=SUMIF(B1:B4,\"A\",A1:A4)".to_owned();
        cells[5][2].raw = "=COUNTIF(A1:A4,\">2\")".to_owned();
        cells[5][3].raw = "=VLOOKUP(\"B\",D1:E2,2)".to_owned();
        cells[5][4].raw = "=XLOOKUP(\"A\",D1:D2,E1:E2)".to_owned();

        let if_value = evaluate_for_test(&cells, &named_ranges, 5, 0);
        let sumif_value = evaluate_for_test(&cells, &named_ranges, 5, 1);
        let countif_value = evaluate_for_test(&cells, &named_ranges, 5, 2);
        let vlookup_value = evaluate_for_test(&cells, &named_ranges, 5, 3);
        let xlookup_value = evaluate_for_test(&cells, &named_ranges, 5, 4);

        assert!(
            matches!(&if_value, CalcValue::Text(text) if text == "high"),
            "{if_value:?}"
        );
        assert!(
            matches!(sumif_value, CalcValue::Number(value) if (value - 4.0).abs() < 0.001),
            "{sumif_value:?}"
        );
        assert!(
            matches!(countif_value, CalcValue::Number(value) if (value - 2.0).abs() < 0.001),
            "{countif_value:?}"
        );
        assert!(
            matches!(&vlookup_value, CalcValue::Text(text) if text == "Beta"),
            "{vlookup_value:?}"
        );
        assert!(
            matches!(&xlookup_value, CalcValue::Text(text) if text == "Alpha"),
            "{xlookup_value:?}"
        );
    }

    #[test]
    fn evaluates_statistical_functions() {
        let mut cells = blank_cells();
        let named_ranges = HashMap::new();
        for (row, value) in ["1", "2", "3", "4"].iter().enumerate() {
            cells[row][0].raw = value.to_string();
            cells[row][1].raw = (value.parse::<i32>().unwrap() * 2).to_string();
        }

        cells[5][0].raw = "=MEDIAN(A1:A4)".to_owned();
        cells[5][1].raw = "=VAR(A1:A4)".to_owned();
        cells[5][2].raw = "=STDEV(A1:A4)".to_owned();
        cells[5][3].raw = "=CORREL(A1:A4,B1:B4)".to_owned();
        cells[5][4].raw = "=ROUNDUP(1.234,2)".to_owned();
        cells[5][5].raw = "=PERCENTILE(A1:A4,0.75)".to_owned();
        cells[5][6].raw = "=QUARTILE(A1:A4,1)".to_owned();
        cells[5][7].raw = "=COVARIANCE(A1:A4,B1:B4)".to_owned();
        cells[6][0].raw = "2".to_owned();
        cells[6][1].raw = "=MODE(A1:A4,A7)".to_owned();

        assert!(matches!(
            evaluate_for_test(&cells, &named_ranges, 5, 0),
            CalcValue::Number(value) if (value - 2.5).abs() < 0.001
        ));
        assert!(matches!(
            evaluate_for_test(&cells, &named_ranges, 5, 1),
            CalcValue::Number(value) if (value - 1.6666667).abs() < 0.001
        ));
        assert!(matches!(
            evaluate_for_test(&cells, &named_ranges, 5, 2),
            CalcValue::Number(value) if (value - 1.2909944).abs() < 0.001
        ));
        assert!(matches!(
            evaluate_for_test(&cells, &named_ranges, 5, 3),
            CalcValue::Number(value) if (value - 1.0).abs() < 0.001
        ));
        assert!(matches!(
            evaluate_for_test(&cells, &named_ranges, 5, 4),
            CalcValue::Number(value) if (value - 1.24).abs() < 0.001
        ));
        assert!(matches!(
            evaluate_for_test(&cells, &named_ranges, 5, 5),
            CalcValue::Number(value) if (value - 3.25).abs() < 0.001
        ));
        assert!(matches!(
            evaluate_for_test(&cells, &named_ranges, 5, 6),
            CalcValue::Number(value) if (value - 1.75).abs() < 0.001
        ));
        assert!(matches!(
            evaluate_for_test(&cells, &named_ranges, 5, 7),
            CalcValue::Number(value) if (value - 3.3333333).abs() < 0.001
        ));
        assert!(matches!(
            evaluate_for_test(&cells, &named_ranges, 6, 1),
            CalcValue::Number(value) if (value - 2.0).abs() < 0.001
        ));
    }

    #[test]
    fn builds_grouped_summaries_and_validates_data() {
        let mut evaluated = vec![vec![CalcValue::Empty; COLS]; ROWS];
        evaluated[0][0] = CalcValue::Text("A".to_owned());
        evaluated[0][1] = CalcValue::Number(10.0);
        evaluated[1][0] = CalcValue::Text("A".to_owned());
        evaluated[1][1] = CalcValue::Number(15.0);
        evaluated[2][0] = CalcValue::Text("B".to_owned());
        evaluated[2][1] = CalcValue::Number(7.0);

        let groups = grouped_summary(
            &evaluated,
            CellRange {
                start: CellAddress::new(0, 0),
                end: CellAddress::new(2, 1),
            },
        );
        assert_eq!(groups[0].key, "A");
        assert_eq!(groups[0].count, 2);
        assert!((groups[0].sum - 25.0).abs() < 0.001);

        let number_rule = ValidationRule {
            range: CellRange::single(CellAddress::new(0, 0)),
            kind: ValidationKind::NumberBetween,
            min: 0.0,
            max: 1.0,
            list: Vec::new(),
        };
        assert!(validation_passes(&number_rule, &CalcValue::Percent(0.75)));
        assert!(!validation_passes(&number_rule, &CalcValue::Number(2.0)));

        let list_rule = ValidationRule {
            range: CellRange::single(CellAddress::new(0, 0)),
            kind: ValidationKind::List,
            min: 0.0,
            max: 0.0,
            list: vec!["Low".to_owned(), "High".to_owned()],
        };
        assert!(validation_passes(
            &list_rule,
            &CalcValue::Text("high".to_owned())
        ));
        assert!(!validation_passes(
            &list_rule,
            &CalcValue::Text("Medium".to_owned())
        ));
    }

    #[test]
    fn tracks_dependencies_and_recalculates_dirty_dependents() {
        let cc = eframe::CreationContext::_new_kittest(egui::Context::default());
        let mut app = SpreadsheetApp::new(&cc);
        app.clear_workbook();
        app.set_raw(0, 0, "1");
        app.set_raw(0, 1, "2");
        app.set_raw(0, 2, "=A1+B1");

        let values = app.evaluated_cells();
        assert!(matches!(values[0][2], CalcValue::Number(value) if (value - 3.0).abs() < 0.001));

        app.set_raw(0, 0, "5");
        let values = app.evaluated_cells();
        assert!(matches!(values[0][2], CalcValue::Number(value) if (value - 7.0).abs() < 0.001));
        assert_eq!(app.last_recalc_count, 2);
        assert_eq!(
            app.dependencies
                .get(&CellAddress::new(0, 2))
                .cloned()
                .unwrap_or_default()
                .len(),
            2
        );
    }

    #[test]
    fn parses_csv_quotes_and_commas() {
        let parsed = parse_csv("Name,Notes\nAmina,\"ready, waiting\"\nOmar,\"said \"\"yes\"\"\"");

        assert_eq!(parsed[0], vec!["Name", "Notes"]);
        assert_eq!(parsed[1], vec!["Amina", "ready, waiting"]);
        assert_eq!(parsed[2], vec!["Omar", "said \"yes\""]);
    }
}
