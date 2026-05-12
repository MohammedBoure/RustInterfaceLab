#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{
    self, Align, Color32, FontFamily, FontId, Layout, RichText, Stroke, TextStyle, Vec2,
};
use std::time::Duration;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Modern Ops Studio")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([980.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Modern Ops Studio",
        options,
        Box::new(|cc| Ok(Box::new(ModernOpsApp::new(cc)))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Command,
    Tickets,
    Assets,
    Automations,
    Settings,
}

impl Section {
    fn label(self) -> &'static str {
        match self {
            Self::Command => "Command",
            Self::Tickets => "Tickets",
            Self::Assets => "Assets",
            Self::Automations => "Automations",
            Self::Settings => "Settings",
        }
    }
}

struct Ticket {
    id: u32,
    title: String,
    owner: String,
    severity: &'static str,
    state: &'static str,
}

struct Asset {
    name: &'static str,
    site: &'static str,
    status: &'static str,
    health: f32,
}

struct Event {
    tone: &'static str,
    message: String,
}

struct ModernOpsApp {
    section: Section,
    search: String,
    operator: String,
    ticket_title: String,
    ticket_owner: String,
    selected_severity: usize,
    strict_mode: bool,
    auto_sync: bool,
    dark_boost: bool,
    throughput: f32,
    risk: f32,
    sync_progress: f32,
    ticket_seed: u32,
    tickets: Vec<Ticket>,
    assets: Vec<Asset>,
    events: Vec<Event>,
    samples: Vec<f32>,
}

impl ModernOpsApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_style(&cc.egui_ctx);

        Self {
            section: Section::Command,
            search: String::new(),
            operator: "AES".to_owned(),
            ticket_title: "Delayed settlement batch".to_owned(),
            ticket_owner: "Finance Ops".to_owned(),
            selected_severity: 1,
            strict_mode: true,
            auto_sync: true,
            dark_boost: false,
            throughput: 0.72,
            risk: 0.28,
            sync_progress: 0.41,
            ticket_seed: 7300,
            tickets: vec![
                Ticket {
                    id: 7296,
                    title: "Settlement file drift".to_owned(),
                    owner: "Finance Ops".to_owned(),
                    severity: "High",
                    state: "Investigating",
                },
                Ticket {
                    id: 7297,
                    title: "Scanner queue retries".to_owned(),
                    owner: "Warehouse".to_owned(),
                    severity: "Medium",
                    state: "Queued",
                },
                Ticket {
                    id: 7298,
                    title: "Portal latency spike".to_owned(),
                    owner: "Platform".to_owned(),
                    severity: "Low",
                    state: "Watching",
                },
            ],
            assets: vec![
                Asset {
                    name: "NAS-Backups-02",
                    site: "Infra",
                    status: "Healthy",
                    health: 0.94,
                },
                Asset {
                    name: "POS-Front-003",
                    site: "Retail",
                    status: "Owner stale",
                    health: 0.52,
                },
                Asset {
                    name: "Scanner-WH-009",
                    site: "Warehouse",
                    status: "Idle",
                    health: 0.81,
                },
                Asset {
                    name: "Tablet-QA-017",
                    site: "QA",
                    status: "Pending return",
                    health: 0.63,
                },
            ],
            events: vec![
                Event {
                    tone: "ok",
                    message: "Workspace initialized".to_owned(),
                },
                Event {
                    tone: "warn",
                    message: "Two assets need ownership review".to_owned(),
                },
                Event {
                    tone: "info",
                    message: "Auto sync is ready".to_owned(),
                },
            ],
            samples: vec![0.34, 0.38, 0.47, 0.44, 0.61, 0.58, 0.74, 0.69, 0.78, 0.72, 0.84, 0.80],
        }
    }

    fn add_event(&mut self, tone: &'static str, message: impl Into<String>) {
        self.events.insert(
            0,
            Event {
                tone,
                message: message.into(),
            },
        );
        self.events.truncate(9);
    }

    fn create_ticket(&mut self) {
        self.ticket_seed += 1;
        let severity = ["Low", "Medium", "High", "Critical"][self.selected_severity];
        let title = if self.ticket_title.trim().is_empty() {
            "Untitled operational issue"
        } else {
            self.ticket_title.trim()
        };
        let owner = if self.ticket_owner.trim().is_empty() {
            "Unassigned"
        } else {
            self.ticket_owner.trim()
        };

        self.tickets.insert(
            0,
            Ticket {
                id: self.ticket_seed,
                title: title.to_owned(),
                owner: owner.to_owned(),
                severity,
                state: "New",
            },
        );
        self.risk = (self.risk + severity_weight(severity)).min(0.95);
        self.add_event("ok", format!("Ticket #{} created", self.ticket_seed));
    }

    fn run_sync(&mut self) {
        self.sync_progress = (self.sync_progress + 0.18).min(1.0);
        self.throughput = (self.throughput + 0.06).min(0.98);
        self.samples.rotate_left(1);
        if let Some(last) = self.samples.last_mut() {
            *last = self.throughput;
        }
        self.add_event("info", "Sync cycle completed");
    }

    fn run_audit(&mut self) {
        self.risk = (self.risk - 0.08).max(0.05);
        self.add_event("ok", "Audit completed with policy checks");
    }
}

impl eframe::App for ModernOpsApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(700));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_bar")
            .frame(app_frame().inner_margin(egui::Margin::symmetric(18, 14)))
            .show_inside(ui, |ui| self.top_bar(ui));

        egui::Panel::left("side_nav")
            .resizable(false)
            .exact_size(238.0)
            .frame(app_frame().inner_margin(egui::Margin::same(18)))
            .show_inside(ui, |ui| self.side_nav(ui));

        egui::CentralPanel::default()
            .frame(app_frame().inner_margin(egui::Margin::same(18)))
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.main_content(ui));
            });
    }
}

impl ModernOpsApp {
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Modern Ops Studio")
                        .size(24.0)
                        .strong()
                        .color(palette::TEXT),
                );
                ui.label(RichText::new("Live operations console").color(palette::MUTED));
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Run sync").clicked() {
                    self.run_sync();
                }
                if ui.button("Audit").clicked() {
                    self.run_audit();
                }
                ui.add_sized(
                    [210.0, 30.0],
                    egui::TextEdit::singleline(&mut self.search).hint_text("Search workspace"),
                );
            });
        });
    }

    fn side_nav(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label(RichText::new("Workspace").color(palette::MUTED).strong());
        ui.add_space(10.0);

        for section in [
            Section::Command,
            Section::Tickets,
            Section::Assets,
            Section::Automations,
            Section::Settings,
        ] {
            let selected = self.section == section;
            if ui
                .add_sized([200.0, 38.0], egui::Button::selectable(selected, section.label()))
                .clicked()
            {
                self.section = section;
            }
        }

        ui.add_space(24.0);
        card(ui, |ui| {
            ui.label(RichText::new("Operator").color(palette::MUTED));
            ui.text_edit_singleline(&mut self.operator);
            ui.separator();
            ui.checkbox(&mut self.auto_sync, "Auto sync");
            ui.checkbox(&mut self.strict_mode, "Strict validation");
            ui.checkbox(&mut self.dark_boost, "Boost contrast");
        });

        ui.add_space(12.0);
        ui.label(RichText::new("Status").color(palette::MUTED).strong());
        ui.add(
            egui::ProgressBar::new(self.sync_progress)
                .text(format!("{:.0}% synchronized", self.sync_progress * 100.0))
                .desired_width(200.0),
        );
    }

    fn main_content(&mut self, ui: &mut egui::Ui) {
        match self.section {
            Section::Command => self.command_view(ui),
            Section::Tickets => self.ticket_view(ui),
            Section::Assets => self.asset_view(ui),
            Section::Automations => self.automation_view(ui),
            Section::Settings => self.settings_view(ui),
        }
    }

    fn command_view(&mut self, ui: &mut egui::Ui) {
        ui.columns(4, |columns| {
            metric_card(&mut columns[0], "Open tickets", self.tickets.len().to_string(), "+4 today", palette::BLUE);
            metric_card(
                &mut columns[1],
                "Risk posture",
                format!("{:.0}%", self.risk * 100.0),
                "weighted",
                palette::AMBER,
            );
            metric_card(
                &mut columns[2],
                "Throughput",
                format!("{:.0}%", self.throughput * 100.0),
                "pipeline",
                palette::GREEN,
            );
            metric_card(
                &mut columns[3],
                "Assets",
                self.assets.len().to_string(),
                "tracked",
                palette::ROSE,
            );
        });

        ui.add_space(14.0);
        ui.columns(2, |columns| {
            card(&mut columns[0], |ui| {
                ui.heading("Pipeline trend");
                ui.label(RichText::new("Last 12 sync samples").color(palette::MUTED));
                ui.add_space(8.0);
                trend_chart(ui, &self.samples);
            });

            card(&mut columns[1], |ui| {
                ui.heading("Event stream");
                ui.add_space(6.0);
                for event in &self.events {
                    event_row(ui, event);
                }
            });
        });

        ui.add_space(14.0);
        card(ui, |ui| {
            ui.heading("Operational actions");
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Create ticket").clicked() {
                    self.create_ticket();
                }
                if ui.button("Run audit").clicked() {
                    self.run_audit();
                }
                if ui.button("Sync queue").clicked() {
                    self.run_sync();
                }
                if ui.button("Lower risk").clicked() {
                    self.risk = (self.risk - 0.04).max(0.03);
                    self.add_event("ok", "Risk score reduced");
                }
            });
        });
    }

    fn ticket_view(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            card(&mut columns[0], |ui| {
                ui.heading("New ticket");
                ui.label(RichText::new("A compact intake flow for operators.").color(palette::MUTED));
                ui.add_space(8.0);
                ui.label("Title");
                ui.text_edit_singleline(&mut self.ticket_title);
                ui.label("Owner");
                ui.text_edit_singleline(&mut self.ticket_owner);
                ui.label("Severity");
                egui::ComboBox::from_id_salt("severity")
                    .selected_text(["Low", "Medium", "High", "Critical"][self.selected_severity])
                    .show_ui(ui, |ui| {
                        for (index, severity) in ["Low", "Medium", "High", "Critical"].iter().enumerate() {
                            ui.selectable_value(&mut self.selected_severity, index, *severity);
                        }
                    });
                ui.add_space(10.0);
                if ui.button("Create ticket").clicked() {
                    self.create_ticket();
                }
            });

            card(&mut columns[1], |ui| {
                ui.heading("SLA pressure");
                ui.label(RichText::new("Risk and workload change as tickets are added.").color(palette::MUTED));
                ui.add_space(10.0);
                ui.add(egui::ProgressBar::new(self.risk).text(format!("Risk {:.0}%", self.risk * 100.0)));
                ui.add(egui::ProgressBar::new(self.throughput).text(format!("Throughput {:.0}%", self.throughput * 100.0)));
            });
        });

        ui.add_space(14.0);
        card(ui, |ui| {
            ui.heading("Ticket board");
            ui.add_space(8.0);
            egui::Grid::new("ticket_table")
                .striped(true)
                .num_columns(5)
                .spacing(Vec2::new(18.0, 8.0))
                .show(ui, |ui| {
                    table_head(ui, "ID");
                    table_head(ui, "Title");
                    table_head(ui, "Owner");
                    table_head(ui, "Severity");
                    table_head(ui, "State");
                    ui.end_row();

                    for ticket in &self.tickets {
                        ui.label(format!("#{}", ticket.id));
                        ui.label(&ticket.title);
                        ui.label(&ticket.owner);
                        pill(ui, ticket.severity, severity_color(ticket.severity));
                        ui.label(ticket.state);
                        ui.end_row();
                    }
                });
        });
    }

    fn asset_view(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            for (index, asset) in self.assets.iter().enumerate() {
                card(&mut columns[index % 2], |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading(asset.name);
                            ui.label(RichText::new(asset.site).color(palette::MUTED));
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            pill(ui, asset.status, health_color(asset.health));
                        });
                    });
                    ui.add_space(8.0);
                    ui.add(
                        egui::ProgressBar::new(asset.health)
                            .text(format!("Health {:.0}%", asset.health * 100.0)),
                    );
                });
            }
        });
    }

    fn automation_view(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.heading("Automation rules");
            ui.label(RichText::new("Local simulation of policy actions.").color(palette::MUTED));
            ui.add_space(10.0);
            ui.checkbox(&mut self.auto_sync, "Run sync after high severity tickets");
            ui.checkbox(&mut self.strict_mode, "Block incomplete owner metadata");
            ui.checkbox(&mut self.dark_boost, "High contrast review mode");
            ui.add_space(12.0);
            if ui.button("Apply policy").clicked() {
                self.add_event("info", "Automation policy applied");
            }
        });

        ui.add_space(14.0);
        card(ui, |ui| {
            ui.heading("Runbook queue");
            for step in [
                "Validate owners",
                "Refresh asset health",
                "Recalculate SLA",
                "Generate summary",
            ] {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("READY").color(palette::GREEN).strong());
                    ui.label(step);
                });
            }
        });
    }

    fn settings_view(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.heading("Interface settings");
            ui.label(RichText::new("This view is still native desktop Rust through eframe/egui.").color(palette::MUTED));
            ui.add_space(10.0);
            ui.add(egui::Slider::new(&mut self.throughput, 0.1..=1.0).text("Throughput"));
            ui.add(egui::Slider::new(&mut self.risk, 0.0..=1.0).text("Risk"));
            ui.add(egui::Slider::new(&mut self.sync_progress, 0.0..=1.0).text("Sync"));
        });
    }
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(12.0, 8.0);
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(24.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(15.0, FontFamily::Proportional),
    );

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = palette::BG;
    visuals.window_fill = palette::CARD;
    visuals.faint_bg_color = palette::CARD_SOFT;
    visuals.extreme_bg_color = palette::FIELD;
    visuals.selection.bg_fill = palette::BLUE;
    visuals.widgets.inactive.bg_fill = palette::FIELD;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 63, 82);
    visuals.widgets.active.bg_fill = Color32::from_rgb(58, 90, 116);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette::TEXT);
    style.visuals = visuals;
    ctx.set_global_style(style);
}

fn app_frame() -> egui::Frame {
    egui::Frame::default().fill(palette::BG)
}

fn card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .fill(palette::CARD)
        .stroke(Stroke::new(1.0, palette::BORDER))
        .inner_margin(egui::Margin::same(14))
        .show(ui, add_contents);
}

fn metric_card(ui: &mut egui::Ui, title: &str, value: String, detail: &str, accent: Color32) {
    card(ui, |ui| {
        ui.label(RichText::new(title).color(palette::MUTED));
        ui.label(RichText::new(value).size(30.0).strong().color(accent));
        ui.label(RichText::new(detail).color(palette::MUTED));
    });
}

fn event_row(ui: &mut egui::Ui, event: &Event) {
    ui.horizontal(|ui| {
        let color = match event.tone {
            "ok" => palette::GREEN,
            "warn" => palette::AMBER,
            _ => palette::BLUE,
        };
        pill(ui, event.tone.to_uppercase(), color);
        ui.label(&event.message);
    });
}

fn table_head(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).color(palette::MUTED).strong());
}

fn pill(ui: &mut egui::Ui, text: impl Into<String>, color: Color32) {
    let text = text.into();
    egui::Frame::default()
        .fill(color.gamma_multiply(0.22))
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.65)))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(color).strong());
        });
}

fn trend_chart(ui: &mut egui::Ui, values: &[f32]) {
    let desired = Vec2::new(ui.available_width(), 190.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, egui::CornerRadius::same(14), palette::FIELD);
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(14),
        Stroke::new(1.0, palette::BORDER),
        egui::StrokeKind::Outside,
    );

    if values.len() < 2 {
        return;
    }

    let left = rect.left() + 18.0;
    let right = rect.right() - 18.0;
    let top = rect.top() + 18.0;
    let bottom = rect.bottom() - 18.0;

    for line in 0..4 {
        let y = top + ((bottom - top) / 3.0) * line as f32;
        painter.line_segment(
            [egui::pos2(left, y), egui::pos2(right, y)],
            Stroke::new(1.0, Color32::from_rgb(39, 50, 66)),
        );
    }

    let step = (right - left) / (values.len() - 1) as f32;
    let mut points = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let x = left + step * index as f32;
        let y = bottom - value.clamp(0.0, 1.0) * (bottom - top);
        points.push(egui::pos2(x, y));
    }

    painter.add(egui::Shape::line(points.clone(), Stroke::new(3.0, palette::BLUE)));
    for point in points {
        painter.circle_filled(point, 4.0, palette::GREEN);
    }
}

fn severity_weight(severity: &str) -> f32 {
    match severity {
        "Critical" => 0.18,
        "High" => 0.12,
        "Medium" => 0.07,
        _ => 0.03,
    }
}

fn severity_color(severity: &str) -> Color32 {
    match severity {
        "Critical" => palette::ROSE,
        "High" => palette::AMBER,
        "Medium" => palette::BLUE,
        _ => palette::GREEN,
    }
}

fn health_color(health: f32) -> Color32 {
    if health > 0.8 {
        palette::GREEN
    } else if health > 0.6 {
        palette::AMBER
    } else {
        palette::ROSE
    }
}

mod palette {
    use eframe::egui::Color32;

    pub const BG: Color32 = Color32::from_rgb(16, 21, 30);
    pub const CARD: Color32 = Color32::from_rgb(25, 32, 43);
    pub const CARD_SOFT: Color32 = Color32::from_rgb(30, 39, 52);
    pub const FIELD: Color32 = Color32::from_rgb(21, 27, 37);
    pub const BORDER: Color32 = Color32::from_rgb(55, 68, 86);
    pub const TEXT: Color32 = Color32::from_rgb(235, 241, 247);
    pub const MUTED: Color32 = Color32::from_rgb(145, 158, 174);
    pub const BLUE: Color32 = Color32::from_rgb(88, 166, 255);
    pub const GREEN: Color32 = Color32::from_rgb(76, 203, 137);
    pub const AMBER: Color32 = Color32::from_rgb(245, 174, 76);
    pub const ROSE: Color32 = Color32::from_rgb(239, 99, 119);
}
