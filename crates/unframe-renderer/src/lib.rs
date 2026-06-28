use egui::{Color32, FontFamily, FontId, RichText, ScrollArea, Sense, Ui};
use unframe_model::{Block, InformationDocument, InlineContent, NoticeKind, OutlineItem};

#[derive(Default)]
pub struct RendererState {
    pub scroll_to_block: Option<usize>,
}

/// Renders the full document content area. Returns a navigation URL if a link was clicked.
pub fn render_document(ui: &mut Ui, doc: &InformationDocument) -> Option<String> {
    let mut nav_url: Option<String> = None;

    ScrollArea::vertical()
        .id_salt("doc_content")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_max_width(720.0);
            ui.add_space(24.0);

            // Title
            ui.label(
                RichText::new(&doc.metadata.title)
                    .font(FontId::new(28.0, FontFamily::Proportional))
                    .strong()
                    .color(Color32::from_rgb(20, 20, 20)),
            );
            ui.add_space(6.0);

            // Byline
            let mut byline_parts = Vec::new();
            if let Some(author) = &doc.metadata.author {
                byline_parts.push(author.clone());
            }
            if let Some(site) = extract_host(&doc.source.final_url) {
                byline_parts.push(site);
            }
            if let Some(pub_at) = &doc.metadata.published_at {
                byline_parts.push(pub_at.format("%b %-d, %Y").to_string());
            }
            if !byline_parts.is_empty() {
                ui.label(
                    RichText::new(byline_parts.join("  ·  "))
                        .font(FontId::new(13.0, FontFamily::Proportional))
                        .color(Color32::from_rgb(120, 120, 120)),
                );
                ui.add_space(16.0);
            }

            ui.separator();
            ui.add_space(16.0);

            for block in &doc.blocks {
                if let Some(url) = render_block(ui, block) {
                    nav_url = Some(url);
                }
                ui.add_space(8.0);
            }

            ui.add_space(32.0);
        });

    nav_url
}

pub fn render_outline(ui: &mut Ui, outline: &[OutlineItem]) -> Option<usize> {
    let mut jump_to: Option<usize> = None;
    let avail_w = ui.available_width();
    for item in outline {
        let indent = (item.level as f32 - 1.0) * 12.0;
        ui.horizontal(|ui| {
            ui.add_space(indent);
            ui.set_max_width(avail_w - indent);
            let display = truncate(&item.text, 36);
            let resp = ui.add(
                egui::Label::new(
                    RichText::new(&display)
                        .font(FontId::new(13.0, FontFamily::Proportional))
                        .color(Color32::from_rgb(60, 100, 180)),
                )
                .sense(Sense::click()),
            );
            if resp.clicked() {
                jump_to = Some(item.block_index);
            }
            if display.ends_with('…') {
                resp.on_hover_text(&item.text);
            }
        });
    }
    jump_to
}

fn render_block(ui: &mut Ui, block: &Block) -> Option<String> {
    match block {
        Block::Heading { level, text, .. } => {
            let size = match level {
                1 => 24.0,
                2 => 20.0,
                3 => 17.0,
                4 => 15.0,
                _ => 14.0,
            };
            ui.add_space(if *level <= 2 { 12.0 } else { 6.0 });
            ui.label(
                RichText::new(text)
                    .font(FontId::new(size, FontFamily::Proportional))
                    .strong()
                    .color(Color32::from_rgb(20, 20, 20)),
            );
            None
        }

        Block::Paragraph { text } => {
            ui.label(
                RichText::new(text)
                    .font(FontId::new(15.0, FontFamily::Proportional))
                    .color(Color32::from_rgb(40, 40, 40)),
            );
            None
        }

        Block::Text { text } => {
            ui.label(
                RichText::new(text)
                    .font(FontId::new(15.0, FontFamily::Proportional))
                    .color(Color32::from_rgb(40, 40, 40)),
            );
            None
        }

        Block::List { ordered, items } => render_list(ui, items, *ordered, 0),

        Block::RichParagraph { segments } => {
            let mut nav: Option<String> = None;
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 1.0;
                for seg in segments {
                    match seg {
                        InlineContent::Text { text } => {
                            ui.label(
                                RichText::new(text)
                                    .font(FontId::new(15.0, FontFamily::Proportional))
                                    .color(Color32::from_rgb(40, 40, 40)),
                            );
                        }
                        InlineContent::Link { text, href } => {
                            let resp = ui
                                .add(
                                    egui::Label::new(
                                        RichText::new(text)
                                            .color(Color32::from_rgb(30, 80, 200))
                                            .font(FontId::new(15.0, FontFamily::Proportional)),
                                    )
                                    .sense(Sense::click()),
                                )
                                .on_hover_text(href.as_str());
                            if resp.clicked() {
                                nav = Some(href.clone());
                            }
                        }
                    }
                }
            });
            nav
        }

        Block::Quote { text, attribution } => {
            egui::Frame::none()
                .inner_margin(egui::Margin {
                    left: 12.0,
                    right: 0.0,
                    top: 6.0,
                    bottom: 6.0,
                })
                .show(ui, |ui| {
                    ui.visuals_mut().override_text_color = Some(Color32::from_rgb(80, 80, 80));
                    let full_text = if let Some(attr) = attribution {
                        format!("{text}\n— {attr}")
                    } else {
                        text.clone()
                    };
                    ui.label(
                        RichText::new(full_text)
                            .font(FontId::new(15.0, FontFamily::Proportional))
                            .italics(),
                    );
                });
            None
        }

        Block::CodeBlock { code, language } => {
            let label = language.as_deref().unwrap_or("code");
            egui::Frame::none()
                .fill(Color32::from_rgb(245, 245, 245))
                .rounding(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(label)
                            .font(FontId::new(11.0, FontFamily::Monospace))
                            .color(Color32::from_rgb(150, 150, 150)),
                    );
                    ui.add_space(4.0);
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.label(
                            RichText::new(code)
                                .font(FontId::new(13.0, FontFamily::Monospace))
                                .color(Color32::from_rgb(30, 30, 30)),
                        );
                    });
                });
            None
        }

        Block::ImageReference { src, alt, .. } => {
            let alt_text = alt.as_deref().unwrap_or("Image");
            egui::Frame::none()
                .fill(Color32::from_rgb(240, 240, 240))
                .rounding(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("\u{1F4F7} {alt_text}"))
                            .font(FontId::new(13.0, FontFamily::Proportional))
                            .color(Color32::from_rgb(100, 100, 100)),
                    );
                    ui.label(
                        RichText::new(truncate(src, 60))
                            .font(FontId::new(11.0, FontFamily::Monospace))
                            .color(Color32::from_rgb(150, 150, 150)),
                    );
                    if ui.small_button("Load").clicked() {
                        // TODO: trigger image load via channel
                    }
                });
            None
        }

        Block::Table { rows } => render_table(ui, rows),

        Block::Divider => {
            ui.separator();
            None
        }

        Block::Notice { kind, text } => {
            let (bg, fg) = match kind {
                NoticeKind::Info => (
                    Color32::from_rgb(235, 244, 255),
                    Color32::from_rgb(30, 80, 160),
                ),
                NoticeKind::Warning => (
                    Color32::from_rgb(255, 248, 225),
                    Color32::from_rgb(160, 100, 0),
                ),
                NoticeKind::Error => (
                    Color32::from_rgb(255, 235, 235),
                    Color32::from_rgb(180, 30, 30),
                ),
                NoticeKind::Unsupported => (
                    Color32::from_rgb(245, 245, 245),
                    Color32::from_rgb(100, 100, 100),
                ),
            };
            egui::Frame::none()
                .fill(bg)
                .rounding(4.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(text)
                            .color(fg)
                            .font(FontId::new(14.0, FontFamily::Proportional)),
                    );
                });
            None
        }

        Block::Link { text, href } => {
            let resp = ui.add(
                egui::Label::new(
                    RichText::new(text)
                        .color(Color32::from_rgb(30, 80, 200))
                        .font(FontId::new(15.0, FontFamily::Proportional)),
                )
                .sense(Sense::click()),
            );
            if resp.clicked() {
                return Some(href.clone());
            }
            resp.on_hover_text(href);
            None
        }

        Block::LinkList { items } => {
            let mut nav = None;
            for item in items {
                if let Some(url) = render_block(ui, item) {
                    nav = Some(url);
                }
            }
            nav
        }

        Block::ListItem { text, href, .. } => {
            if let Some(url) = href {
                let resp = ui
                    .add(
                        egui::Label::new(
                            RichText::new(format!("• {text}"))
                                .color(Color32::from_rgb(30, 80, 200))
                                .font(FontId::new(15.0, FontFamily::Proportional)),
                        )
                        .sense(Sense::click()),
                    )
                    .on_hover_text(url.as_str());
                if resp.clicked() {
                    return Some(url.clone());
                }
            } else {
                ui.label(
                    RichText::new(format!("• {text}"))
                        .font(FontId::new(15.0, FontFamily::Proportional)),
                );
            }
            None
        }

        Block::TableRow { .. } | Block::TableCell { .. } => None, // rendered inside render_table
    }
}

fn render_list(ui: &mut Ui, items: &[Block], ordered: bool, depth: usize) -> Option<String> {
    let mut nav_url: Option<String> = None;
    for (i, item) in items.iter().enumerate() {
        match item {
            Block::ListItem {
                text,
                href,
                children,
            } => {
                let bullet = if ordered {
                    format!("{}.", i + 1)
                } else {
                    "•".into()
                };
                let mut item_nav: Option<String> = None;
                ui.horizontal(|ui| {
                    ui.add_space(depth as f32 * 16.0 + 8.0);
                    if let Some(url) = href {
                        let resp = ui
                            .add(
                                egui::Label::new(
                                    RichText::new(format!("{bullet} {text}"))
                                        .color(Color32::from_rgb(30, 80, 200))
                                        .font(FontId::new(15.0, FontFamily::Proportional)),
                                )
                                .sense(Sense::click()),
                            )
                            .on_hover_text(url.as_str());
                        if resp.clicked() {
                            item_nav = Some(url.clone());
                        }
                    } else {
                        ui.label(
                            RichText::new(format!("{bullet} {text}"))
                                .font(FontId::new(15.0, FontFamily::Proportional))
                                .color(Color32::from_rgb(40, 40, 40)),
                        );
                    }
                });
                if item_nav.is_some() {
                    nav_url = item_nav;
                }
                for child in children {
                    if let Block::List {
                        ordered: child_ordered,
                        items: child_items,
                    } = child
                    {
                        if let Some(url) = render_list(ui, child_items, *child_ordered, depth + 1) {
                            nav_url = Some(url);
                        }
                    }
                }
            }
            Block::List {
                ordered: sub_ordered,
                items: sub_items,
            } => {
                if let Some(url) = render_list(ui, sub_items, *sub_ordered, depth + 1) {
                    nav_url = Some(url);
                }
            }
            _ => {}
        }
    }
    nav_url
}

fn render_table(ui: &mut Ui, rows: &[Block]) -> Option<String> {
    let mut nav_url: Option<String> = None;
    egui::Frame::none()
        .fill(Color32::from_rgb(252, 252, 252))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(220, 220, 220)))
        .rounding(4.0)
        .inner_margin(4.0)
        .show(ui, |ui| {
            let col_count = rows
                .first()
                .and_then(|r| {
                    if let Block::TableRow { cells, .. } = r {
                        Some(cells.len())
                    } else {
                        None
                    }
                })
                .unwrap_or(1)
                .max(1);

            egui::Grid::new(ui.next_auto_id())
                .striped(true)
                .min_col_width(60.0)
                .max_col_width(400.0)
                .show(ui, |ui| {
                    for row in rows {
                        if let Block::TableRow { cells, is_header } = row {
                            for cell in cells {
                                if let Block::TableCell { content } = cell {
                                    ui.vertical(|ui| {
                                        for block in content {
                                            match block {
                                                Block::Link { text, href } => {
                                                    let font_size = 13.0;
                                                    let rt = if *is_header {
                                                        RichText::new(text)
                                                            .strong()
                                                            .color(Color32::from_rgb(30, 80, 200))
                                                            .font(FontId::new(
                                                                font_size,
                                                                FontFamily::Proportional,
                                                            ))
                                                    } else {
                                                        RichText::new(text)
                                                            .color(Color32::from_rgb(30, 80, 200))
                                                            .font(FontId::new(
                                                                font_size,
                                                                FontFamily::Proportional,
                                                            ))
                                                    };
                                                    let resp = ui
                                                        .add(
                                                            egui::Label::new(rt)
                                                                .sense(Sense::click()),
                                                        )
                                                        .on_hover_text(href.as_str());
                                                    if resp.clicked() {
                                                        nav_url = Some(href.clone());
                                                    }
                                                }
                                                Block::Text { text }
                                                | Block::Paragraph { text } => {
                                                    let rt = if *is_header {
                                                        RichText::new(text).strong().font(
                                                            FontId::new(
                                                                13.0,
                                                                FontFamily::Proportional,
                                                            ),
                                                        )
                                                    } else {
                                                        RichText::new(text).font(FontId::new(
                                                            13.0,
                                                            FontFamily::Proportional,
                                                        ))
                                                    };
                                                    ui.label(rt);
                                                }
                                                other => {
                                                    if let Some(url) = render_block(ui, other) {
                                                        nav_url = Some(url);
                                                    }
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                            for _ in cells.len()..col_count {
                                ui.label("");
                            }
                            ui.end_row();
                        }
                    }
                });
        });
    nav_url
}

fn extract_host(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
