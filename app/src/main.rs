use std::sync::mpsc::{self, Receiver, Sender};

use eframe::egui;
use egui::{Color32, FontFamily, FontId, RichText};
use url::Url;

use unframe_adapter_html::{adapt, AdapterResult};
use unframe_model::InformationDocument;
use unframe_network::NetworkGateway;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap()),
        )
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Unframe")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Unframe",
        options,
        Box::new(|cc| Ok(Box::new(UnframeApp::new(cc)))),
    )
}

// ── App state ────────────────────────────────────────────────────────────────

#[allow(clippy::large_enum_variant)]
enum LoadResult {
    Success(InformationDocument),
    Unsupported(String),
    Error(String),
}

#[allow(clippy::large_enum_variant)]
enum AppState {
    Idle,
    Loading,
    Loaded(InformationDocument),
    Error { message: String },
}

struct UnframeApp {
    url_input: String,
    state: AppState,
    history: Vec<Url>,
    history_pos: usize,

    tx: Sender<LoadResult>,
    rx: Receiver<LoadResult>,

    gateway: std::sync::Arc<NetworkGateway>,
    runtime: tokio::runtime::Runtime,

    save_status: Option<String>,
}

impl UnframeApp {
    fn new(_cc: &eframe::CreationContext) -> Self {
        let (tx, rx) = mpsc::channel();
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let gateway = std::sync::Arc::new(NetworkGateway::new().expect("network gateway"));
        Self {
            url_input: String::new(),
            state: AppState::Idle,
            history: Vec::new(),
            history_pos: 0,
            tx,
            rx,
            gateway,
            runtime,
            save_status: None,
        }
    }

    fn navigate_to(&mut self, url_str: &str) {
        let url_str = if url_str.starts_with("http://") || url_str.starts_with("https://") {
            url_str.to_string()
        } else {
            format!("https://{url_str}")
        };

        match Url::parse(&url_str) {
            Err(e) => {
                self.state = AppState::Error {
                    message: format!("Invalid URL: {e}"),
                };
            }
            Ok(url) => {
                self.url_input = url.to_string();
                // Truncate forward history
                if self.history_pos < self.history.len() {
                    self.history.truncate(self.history_pos);
                }
                self.history.push(url.clone());
                self.history_pos = self.history.len();

                self.start_load(url);
            }
        }
    }

    fn start_load(&mut self, url: Url) {
        self.state = AppState::Loading;
        let tx = self.tx.clone();
        let gateway = self.gateway.clone();
        self.runtime.spawn(async move {
            let result = async {
                let fetch = gateway.fetch(url).await?;
                let content_type = fetch.content_type.clone();
                if !content_type.contains("html") {
                    return Ok(LoadResult::Unsupported(format!(
                        "Content type '{content_type}' is not supported"
                    )));
                }
                let (html, charset, charset_source) =
                    unframe_parser::detect_charset(&fetch.content_type_raw, &fetch.body);
                let base_url = fetch.final_url.clone();
                let parsed = unframe_parser::parse(&html, &base_url);
                Ok::<_, unframe_network::NetworkError>(match adapt(parsed, &fetch) {
                    AdapterResult::Success(mut doc) | AdapterResult::PartialSuccess(mut doc) => {
                        doc.source.charset = charset;
                        doc.source.charset_source = charset_source;
                        LoadResult::Success(doc)
                    }
                    AdapterResult::Unsupported(reason) => LoadResult::Unsupported(reason),
                    AdapterResult::Failed(e) => LoadResult::Error(e.to_string()),
                })
            }
            .await;

            let load_result = match result {
                Ok(r) => r,
                Err(e) => LoadResult::Error(e.to_string()),
            };
            let _ = tx.send(load_result);
        });
    }

    fn can_go_back(&self) -> bool {
        self.history_pos > 1
    }

    fn can_go_forward(&self) -> bool {
        self.history_pos < self.history.len()
    }

    fn go_back(&mut self) {
        if self.can_go_back() {
            self.history_pos -= 1;
            let url = self.history[self.history_pos - 1].clone();
            self.url_input = url.to_string();
            self.start_load(url);
        }
    }

    fn go_forward(&mut self) {
        if self.can_go_forward() {
            let url = self.history[self.history_pos].clone();
            self.history_pos += 1;
            self.url_input = url.to_string();
            self.start_load(url);
        }
    }
}

impl eframe::App for UnframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll async result
        if let Ok(result) = self.rx.try_recv() {
            self.state = match result {
                LoadResult::Success(doc) => AppState::Loaded(doc),
                LoadResult::Unsupported(reason) => AppState::Error {
                    message: format!("Unsupported: {reason}"),
                },
                LoadResult::Error(msg) => AppState::Error { message: msg },
            };
            ctx.request_repaint();
        }

        // Repaint while loading
        if matches!(self.state, AppState::Loading) {
            ctx.request_repaint();
        }

        // ── Top toolbar ──────────────────────────────────────────────────────
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // Logo
                ui.label(
                    RichText::new("U")
                        .font(FontId::new(18.0, FontFamily::Proportional))
                        .strong()
                        .color(Color32::from_rgb(30, 80, 200)),
                );
                ui.label(
                    RichText::new("Unframe")
                        .font(FontId::new(14.0, FontFamily::Proportional))
                        .strong(),
                );
                ui.add_space(8.0);

                // Back / Forward
                let can_back = self.can_go_back();
                let can_fwd = self.can_go_forward();
                if ui.add_enabled(can_back, egui::Button::new("←")).clicked() {
                    self.go_back();
                }
                if ui.add_enabled(can_fwd, egui::Button::new("→")).clicked() {
                    self.go_forward();
                }
                ui.add_space(4.0);

                // URL bar
                let url_response = ui.add(
                    egui::TextEdit::singleline(&mut self.url_input)
                        .desired_width(ui.available_width() - 360.0)
                        .hint_text("Enter URL…")
                        .font(FontId::new(14.0, FontFamily::Proportional)),
                );
                if url_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let url = self.url_input.clone();
                    self.navigate_to(&url);
                }

                ui.add_space(8.0);

                // Stats badges (from current doc)
                if let AppState::Loaded(doc) = &self.state {
                    let audit = &doc.audit;
                    badge(
                        ui,
                        &format!("{} req", audit.total_requests),
                        Color32::from_rgb(220, 230, 255),
                    );
                    badge(
                        ui,
                        &human_bytes(audit.total_bytes),
                        Color32::from_rgb(220, 255, 230),
                    );
                    badge(
                        ui,
                        &format!("{} scripts", audit.scripts_executed),
                        Color32::from_rgb(255, 240, 220),
                    );
                    let adapter_label = if audit.adapter_name.is_empty() {
                        "No adapter".to_string()
                    } else {
                        format!("{} v{}", audit.adapter_name, audit.adapter_version)
                    };
                    badge(ui, &adapter_label, Color32::from_rgb(235, 235, 255));
                }
            });
            ui.add_space(4.0);
        });

        // ── Bottom action bar ────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("actions").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if ui.button("↗ Open source").clicked() {
                        if let Some(url) = self.history.get(self.history_pos.saturating_sub(1)) {
                            // open in system browser
                            let _ = open::that(url.as_str());
                        }
                    }
                    ui.separator();
                    if ui.button("⟳ Refresh").clicked() {
                        if let Some(url) = self
                            .history
                            .get(self.history_pos.saturating_sub(1))
                            .cloned()
                        {
                            self.start_load(url);
                        }
                    }
                    ui.separator();
                    if ui.button("📂 Open document").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Unframe document", &["json"])
                            .pick_file()
                        {
                            match load_document_from_file(&path) {
                                Ok(doc) => {
                                    self.url_input = format!("file://{}", path.display());
                                    self.state = AppState::Loaded(doc);
                                    self.save_status = None;
                                }
                                Err(e) => {
                                    self.state = AppState::Error {
                                        message: format!("Could not open: {e}"),
                                    };
                                }
                            }
                        }
                    }
                    ui.separator();
                    if ui.button("💾 Save document").clicked() {
                        if let AppState::Loaded(doc) = &self.state {
                            let suggested =
                                suggest_filename(&doc.metadata.title, &doc.source.requested_url);
                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name(&suggested)
                                .add_filter("Unframe document", &["json"])
                                .save_file()
                            {
                                self.save_status = save_document_atomic(doc, &path);
                            }
                        }
                    }
                    if let Some(err) = &self.save_status {
                        ui.separator();
                        ui.label(
                            RichText::new(format!("⚠ {err}"))
                                .font(FontId::new(12.0, FontFamily::Proportional))
                                .color(Color32::from_rgb(180, 40, 40)),
                        );
                    }
                });
            });
            ui.add_space(4.0);
        });

        // ── Left outline sidebar ─────────────────────────────────────────────
        egui::SidePanel::left("outline")
            .resizable(true)
            .default_width(200.0)
            .width_range(120.0..=300.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Outline")
                        .strong()
                        .font(FontId::new(13.0, FontFamily::Proportional)),
                );
                ui.separator();
                ui.add_space(4.0);

                if let AppState::Loaded(doc) = &self.state {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        unframe_renderer::render_outline(ui, &doc.outline);
                    });
                }
            });

        // ── Right document passport ──────────────────────────────────────────
        egui::SidePanel::right("passport")
            .resizable(true)
            .default_width(220.0)
            .width_range(160.0..=320.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Document passport")
                        .strong()
                        .font(FontId::new(13.0, FontFamily::Proportional)),
                );
                ui.separator();

                if let AppState::Loaded(doc) = &self.state {
                    render_passport(ui, doc);
                }
            });

        // ── Central content area ─────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| match &self.state {
            AppState::Idle => {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("Enter a URL to begin")
                            .font(FontId::new(18.0, FontFamily::Proportional))
                            .color(Color32::from_rgb(150, 150, 150)),
                    );
                });
            }
            AppState::Loading => {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label(
                        RichText::new("Loading…")
                            .font(FontId::new(16.0, FontFamily::Proportional))
                            .color(Color32::from_rgb(100, 100, 100)),
                    );
                });
            }
            AppState::Loaded(doc) => {
                let doc = doc.clone();
                if let Some(nav_url) = unframe_renderer::render_document(ui, &doc) {
                    self.navigate_to(&nav_url);
                }
            }
            AppState::Error { message } => {
                let msg = message.clone();
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(
                            RichText::new("Could not load page")
                                .font(FontId::new(20.0, FontFamily::Proportional))
                                .strong()
                                .color(Color32::from_rgb(180, 40, 40)),
                        );
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(&msg)
                                .font(FontId::new(14.0, FontFamily::Proportional))
                                .color(Color32::from_rgb(100, 100, 100)),
                        );
                    });
                });
            }
        });
    }
}

fn render_passport(ui: &mut egui::Ui, doc: &InformationDocument) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let audit = &doc.audit;

        passport_row(ui, "Source", &truncate_url(&doc.source.requested_url, 30));
        if doc.source.final_url != doc.source.requested_url {
            passport_row(ui, "Final URL", &truncate_url(&doc.source.final_url, 30));
        }
        passport_row(
            ui,
            "Retrieved",
            &doc.source
                .retrieved_at
                .format("%b %-d, %Y  %H:%M")
                .to_string(),
        );
        passport_row(ui, "Content type", &doc.source.content_type);
        passport_row(ui, "Charset", &doc.source.charset);
        passport_row(ui, "Charset source", &doc.source.charset_source);
        passport_row(ui, "Resource size", &human_bytes(audit.total_bytes));
        passport_row(ui, "Requests made", &audit.total_requests.to_string());
        passport_row(ui, "Blocked requests", &audit.blocked_count.to_string());
        passport_row(
            ui,
            "Scripts discovered",
            &audit.scripts_discovered.to_string(),
        );
        passport_row(ui, "Scripts executed", &audit.scripts_executed.to_string());
        passport_row(
            ui,
            "Third-party requests",
            &format!("{} (0%)", audit.third_party_requests),
        );
        if !audit.adapter_name.is_empty() {
            passport_row(
                ui,
                "Adapter used",
                &format!("{} v{}", audit.adapter_name, audit.adapter_version),
            );
        }
        if let Some(lang) = &doc.metadata.language {
            passport_row(ui, "Language", lang);
        }

        if !doc.warnings.is_empty() {
            ui.add_space(8.0);
            ui.label(
                RichText::new("Warnings")
                    .strong()
                    .font(FontId::new(12.0, FontFamily::Proportional)),
            );
            for w in &doc.warnings {
                ui.label(
                    RichText::new(format!("⚠ {w}"))
                        .font(FontId::new(12.0, FontFamily::Proportional))
                        .color(Color32::from_rgb(180, 100, 0)),
                );
            }
        }
    });
}

fn passport_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(label)
            .font(FontId::new(11.0, FontFamily::Proportional))
            .color(Color32::from_rgb(130, 130, 130)),
    );
    ui.label(
        RichText::new(value)
            .font(FontId::new(13.0, FontFamily::Proportional))
            .color(Color32::from_rgb(30, 30, 30)),
    );
    ui.separator();
}

fn badge(ui: &mut egui::Ui, text: &str, bg: Color32) {
    egui::Frame::none()
        .fill(bg)
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .font(FontId::new(12.0, FontFamily::Proportional))
                    .color(Color32::from_rgb(40, 40, 40)),
            );
        });
}

fn human_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn truncate_url(url: &str, max: usize) -> String {
    if url.len() <= max {
        url.to_string()
    } else {
        format!("{}…", &url[..max])
    }
}

fn suggest_filename(title: &str, url: &str) -> String {
    let base: String = if !title.is_empty() {
        title.to_string()
    } else {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_else(|| "document".to_string())
    };
    let safe: String = base
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(60)
        .collect();
    format!("{safe}.unframe.json")
}

fn save_document_atomic(doc: &InformationDocument, path: &std::path::Path) -> Option<String> {
    let json = match serde_json::to_string_pretty(doc) {
        Ok(j) => j,
        Err(e) => return Some(format!("Serialization error: {e}")),
    };
    let tmp = path.with_extension("unframe.tmp");
    if let Err(e) = std::fs::write(&tmp, json.as_bytes()) {
        return Some(format!("Write error: {e}"));
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Some(format!("Save error: {e}"));
    }
    None
}

fn load_document_from_file(path: &std::path::Path) -> Result<InformationDocument, String> {
    let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc: InformationDocument =
        serde_json::from_str(&data).map_err(|e| format!("JSON parse error: {e}"))?;
    if doc.model_version != unframe_model::MODEL_VERSION {
        return Err(format!(
            "Unsupported model version: {} (this client supports {})",
            doc.model_version,
            unframe_model::MODEL_VERSION
        ));
    }
    doc.validate().map_err(|e| e.to_string())?;
    Ok(doc)
}
