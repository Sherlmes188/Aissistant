#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod config;
mod icon;
mod platform;

use crate::config::AppConfig;
use crate::platform::ControlEvent;
use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use std::sync::mpsc::{self, Receiver};
use std::thread;

const PANEL_ROUNDING: f32 = 16.0;
const CONTROL_ROUNDING: f32 = 12.0;
const CODE_ROUNDING: f32 = 12.0;
const WINDOW_ROUNDING: f32 = 18.0;
const TITLE_BUTTON_ROUNDING: f32 = 10.0;
const FOREGROUND_POLL_MS: u64 = 120;
const BACKGROUND_POLL_MS: u64 = 1000;

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Aissistant")
        .with_inner_size([720.0, 620.0])
        .with_min_inner_size([480.0, 420.0])
        .with_resizable(true)
        .with_decorations(false)
        .with_transparent(true);

    if let Some(icon) = icon::load_window_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Aissistant",
        options,
        Box::new(|cc| Box::new(AssistantApp::new(cc))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Chat,
    Settings,
}

struct AssistantApp {
    config: AppConfig,
    page: Page,
    question: String,
    answer: String,
    status: String,
    is_loading: bool,
    window_visible: bool,
    focus_question_next_frame: bool,
    allow_quit: bool,
    pending: Option<Receiver<Result<String, String>>>,
    control_rx: Receiver<ControlEvent>,
}

impl AssistantApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);
        configure_style(&cc.egui_ctx);
        let (control_tx, control_rx) = mpsc::channel();
        let config = AppConfig::load();
        let tray_icon = icon::ensure_tray_icon().map(|path| path.to_string_lossy().to_string());
        platform::start_control_thread(
            control_tx,
            cc.egui_ctx.clone(),
            config.hotkey.clone(),
            tray_icon,
        );

        Self {
            config,
            page: Page::Chat,
            question: String::new(),
            answer: String::new(),
            status: "Ready".to_string(),
            is_loading: false,
            window_visible: true,
            focus_question_next_frame: true,
            allow_quit: false,
            pending: None,
            control_rx,
        }
    }

    fn send_question(&mut self, ctx: &egui::Context) {
        if self.is_loading {
            return;
        }

        let question = self.question.trim().to_string();
        if question.is_empty() {
            self.status = "Type a question first".to_string();
            return;
        }

        if let Err(err) = self.config.save() {
            self.status = format!("Config save failed: {err}");
            return;
        }

        let config = self.config.clone();
        let (tx, rx) = mpsc::channel();

        self.answer.clear();
        self.status = "Thinking...".to_string();
        self.is_loading = true;
        self.pending = Some(rx);

        thread::spawn(move || {
            let result = api::ask(&config, &question).map_err(|err| err.to_string());
            let _ = tx.send(result);
        });

        ctx.request_repaint();
    }

    fn poll_pending(&mut self, ctx: &egui::Context, foreground: bool) {
        let Some(rx) = &self.pending else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(answer)) => {
                self.answer = answer;
                self.status = "Done".to_string();
                self.is_loading = false;
                self.pending = None;
            }
            Ok(Err(err)) => {
                self.answer = err;
                self.status = "Request failed".to_string();
                self.is_loading = false;
                self.pending = None;
            }
            Err(mpsc::TryRecvError::Empty) => {
                let interval = if foreground {
                    FOREGROUND_POLL_MS
                } else {
                    BACKGROUND_POLL_MS
                };
                ctx.request_repaint_after(std::time::Duration::from_millis(interval));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.status = "Worker disconnected".to_string();
                self.is_loading = false;
                self.pending = None;
            }
        }
    }

    fn poll_control_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.control_rx.try_recv() {
            match event {
                ControlEvent::ShowWindow => {
                    self.show_window(ctx);
                }
                ControlEvent::ToggleWindow => {
                    if self.window_visible {
                        self.hide_window(ctx);
                    } else {
                        self.show_window(ctx);
                    }
                }
                ControlEvent::QuitRequested => {
                    self.allow_quit = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn hide_window(&mut self, ctx: &egui::Context) {
        self.window_visible = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }

    fn show_window(&mut self, ctx: &egui::Context) {
        self.window_visible = true;
        self.page = Page::Chat;
        self.focus_question_next_frame = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.request_repaint();
    }

    fn close_or_hide(&mut self, ctx: &egui::Context) {
        if self.config.close_to_tray {
            self.hide_window(ctx);
        } else {
            self.allow_quit = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn show_title_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let title_height = 44.0;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), title_height),
            egui::Sense::click_and_drag(),
        );

        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(10.0),
            egui::Color32::from_rgb(17, 20, 24),
        );
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(39, 45, 52)),
        );

        if response.drag_started() {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        let mut title_ui = ui.child_ui(
            rect.shrink2(egui::vec2(14.0, 7.0)),
            egui::Layout::left_to_right(egui::Align::Center),
        );

        title_ui.label(
            egui::RichText::new("Aissistant")
                .size(18.0)
                .strong()
                .color(egui::Color32::from_rgb(238, 241, 245)),
        );
        title_ui.label(
            egui::RichText::new(&self.config.hotkey)
                .small()
                .color(egui::Color32::from_rgb(126, 136, 148)),
        );

        title_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if title_button(ui, "x", egui::Color32::from_rgb(163, 72, 82)).clicked() {
                self.close_or_hide(ctx);
            }
            if title_button(ui, "-", egui::Color32::from_rgb(67, 74, 84)).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
        });
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(2.0);
            if tab_button(ui, "Chat", self.page == Page::Chat) {
                self.page = Page::Chat;
                self.focus_question_next_frame = true;
            }
            if tab_button(ui, "Settings", self.page == Page::Settings) {
                self.page = Page::Settings;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let color = if self.is_loading {
                    egui::Color32::from_rgb(245, 178, 92)
                } else {
                    egui::Color32::from_rgb(119, 205, 156)
                };
                ui.label(egui::RichText::new(&self.status).color(color));
            });
        });
    }

    fn show_chat(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(6.0);

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(24, 28, 32))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(56, 64, 73)))
            .rounding(egui::Rounding::same(PANEL_ROUNDING))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Question")
                        .strong()
                        .color(egui::Color32::from_rgb(226, 230, 235)),
                );

                let question = egui::TextEdit::multiline(&mut self.question)
                    .id_source("question_input")
                    .desired_rows(4)
                    .hint_text("Ask for a Linux command, Python snippet, or quick explanation...")
                    .frame(false);
                let response = ui.add_sized([ui.available_width(), 108.0], question);
                if self.focus_question_next_frame {
                    response.request_focus();
                    self.focus_question_next_frame = false;
                }

                let send_by_enter = response.has_focus()
                    && ctx.input(|input| {
                        input.key_pressed(egui::Key::Enter) && !input.modifiers.ctrl
                    });

                if send_by_enter {
                    trim_enter_insert(&mut self.question);
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let send_text = if self.is_loading {
                        "Sending..."
                    } else {
                        "Send"
                    };
                    let send_clicked = ui
                        .add_enabled(
                            !self.is_loading,
                            egui::Button::new(send_text)
                                .fill(egui::Color32::from_rgb(38, 142, 107))
                                .rounding(egui::Rounding::same(CONTROL_ROUNDING))
                                .min_size(egui::vec2(82.0, 32.0)),
                        )
                        .clicked();

                    ui.label(
                        egui::RichText::new("Enter to send, Ctrl+Enter for newline")
                            .small()
                            .color(egui::Color32::from_rgb(145, 151, 160)),
                    );

                    if send_clicked || send_by_enter {
                        self.send_question(ctx);
                    }
                });
            });

        ui.add_space(12.0);
        ui.label(
            egui::RichText::new("Answer")
                .strong()
                .color(egui::Color32::from_rgb(226, 230, 235)),
        );

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(20, 23, 27))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 55, 64)))
            .rounding(egui::Rounding::same(PANEL_ROUNDING))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.answer.trim().is_empty() {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("The answer will appear here.")
                                    .color(egui::Color32::from_rgb(121, 128, 138)),
                            );
                        } else {
                            render_answer(ui, &self.answer);
                        }
                    });
            });
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(8.0);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(24, 28, 32))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(56, 64, 73)))
                    .rounding(egui::Rounding::same(PANEL_ROUNDING))
                    .inner_margin(egui::Margin::same(18.0))
                    .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("API")
                        .strong()
                        .color(egui::Color32::from_rgb(226, 230, 235)),
                );
                ui.add_space(8.0);

                settings_text(ui, "Base URL", &mut self.config.base_url, false);
                settings_text(ui, "API Key", &mut self.config.api_key, true);
                settings_text(ui, "Model", &mut self.config.model, false);

                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("System Prompt")
                        .strong()
                        .color(egui::Color32::from_rgb(226, 230, 235)),
                );
                ui.add_sized(
                    [ui.available_width(), 120.0],
                    egui::TextEdit::multiline(&mut self.config.system_prompt),
                );

                ui.add_space(12.0);
                ui.checkbox(
                    &mut self.config.close_to_tray,
                    "Close button hides to system tray",
                );
                settings_text(ui, "Global Hotkey", &mut self.config.hotkey, false);
                ui.label(
                    egui::RichText::new("Examples: Ctrl+Space, Alt+Space, Ctrl+Alt+A. Click the tray icon to toggle the window too.")
                        .small()
                        .color(egui::Color32::from_rgb(145, 151, 160)),
                );

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new("Save Settings")
                                .fill(egui::Color32::from_rgb(38, 142, 107))
                                .rounding(egui::Rounding::same(CONTROL_ROUNDING))
                                .min_size(egui::vec2(120.0, 32.0)),
                        )
                        .clicked()
                    {
                        self.status = match platform::update_hotkey(&self.config.hotkey) {
                            Ok(()) => match self.config.save() {
                                Ok(()) => "Settings saved".to_string(),
                                Err(err) => format!("Save failed: {err}"),
                            },
                            Err(err) => format!("Hotkey failed: {err}"),
                        };
                    }

                    if ui
                        .add(
                            egui::Button::new("Quit App")
                                .fill(egui::Color32::from_rgb(78, 58, 62))
                                .rounding(egui::Rounding::same(CONTROL_ROUNDING))
                                .min_size(egui::vec2(96.0, 32.0)),
                        )
                        .clicked()
                    {
                        self.allow_quit = true;
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                    });
            });
    }
}

impl eframe::App for AssistantApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_control_events(ctx);

        let minimized = ctx.input(|input| input.viewport().minimized.unwrap_or(false));
        let foreground = self.window_visible && !minimized;
        self.poll_pending(ctx, foreground);

        if ctx.input(|input| input.viewport().close_requested()) && !self.allow_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_or_hide(ctx);
        }

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.hide_window(ctx);
        }

        if !foreground {
            return;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(13, 15, 18))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 41, 48)))
                    .rounding(egui::Rounding::same(WINDOW_ROUNDING))
                    .show(ui, |ui| {
                        self.show_title_bar(ui, ctx);

                        egui::Frame::none()
                            .fill(egui::Color32::TRANSPARENT)
                            .inner_margin(egui::Margin::symmetric(18.0, 14.0))
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
                                self.show_header(ui);
                                ui.separator();

                                match self.page {
                                    Page::Chat => self.show_chat(ui, ctx),
                                    Page::Settings => self.show_settings(ui),
                                }
                            });
                    });
            });
    }
}

fn title_button(ui: &mut egui::Ui, text: &str, color: egui::Color32) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .size(15.0)
                .strong()
                .color(egui::Color32::from_rgb(235, 238, 242)),
        )
        .fill(color)
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(1.25)))
        .rounding(egui::Rounding::same(TITLE_BUTTON_ROUNDING))
        .min_size(egui::vec2(34.0, 28.0)),
    )
}

fn tab_button(ui: &mut egui::Ui, text: &str, selected: bool) -> bool {
    ui.add(
        egui::Button::new(text)
            .fill(if selected {
                egui::Color32::from_rgb(42, 130, 104)
            } else {
                egui::Color32::from_rgb(31, 35, 40)
            })
            .stroke(egui::Stroke::new(
                1.0,
                if selected {
                    egui::Color32::from_rgb(72, 178, 142)
                } else {
                    egui::Color32::from_rgb(49, 55, 61)
                },
            ))
            .rounding(egui::Rounding::same(CONTROL_ROUNDING)),
    )
    .clicked()
}

fn settings_text(ui: &mut egui::Ui, label: &str, value: &mut String, password: bool) {
    ui.label(
        egui::RichText::new(label)
            .color(egui::Color32::from_rgb(174, 181, 190))
            .small(),
    );
    let edit = egui::TextEdit::singleline(value).password(password);
    ui.add_sized([ui.available_width(), 30.0], edit);
}

fn trim_enter_insert(text: &mut String) {
    while text.ends_with('\n') || text.ends_with('\r') {
        text.pop();
    }
}

#[derive(Debug)]
enum AnswerBlock {
    Paragraph(String),
    Code { lang: String, code: String },
    Formula(String),
}

fn parse_answer(text: &str) -> Vec<AnswerBlock> {
    let mut blocks = Vec::new();
    let mut paragraph = Vec::new();
    let mut code_lang = String::new();
    let mut code = Vec::new();
    let mut formula = Vec::new();
    let mut in_code = false;
    let mut in_formula = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code {
                blocks.push(AnswerBlock::Code {
                    lang: code_lang.trim().to_string(),
                    code: code.join("\n"),
                });
                code.clear();
                code_lang.clear();
                in_code = false;
            } else {
                flush_paragraph(&mut blocks, &mut paragraph);
                code_lang = trimmed.trim_start_matches("```").to_string();
                in_code = true;
            }
            continue;
        }

        if in_code {
            code.push(line.to_string());
            continue;
        }

        if trimmed == "$$" {
            if in_formula {
                blocks.push(AnswerBlock::Formula(formula.join("\n")));
                formula.clear();
                in_formula = false;
            } else {
                flush_paragraph(&mut blocks, &mut paragraph);
                in_formula = true;
            }
            continue;
        }

        if in_formula {
            formula.push(line.to_string());
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut blocks, &mut paragraph);
        } else {
            paragraph.push(line.to_string());
        }
    }

    if in_code {
        blocks.push(AnswerBlock::Code {
            lang: code_lang.trim().to_string(),
            code: code.join("\n"),
        });
    }
    if in_formula {
        blocks.push(AnswerBlock::Formula(formula.join("\n")));
    }
    flush_paragraph(&mut blocks, &mut paragraph);

    blocks
}

fn flush_paragraph(blocks: &mut Vec<AnswerBlock>, paragraph: &mut Vec<String>) {
    if !paragraph.is_empty() {
        blocks.push(AnswerBlock::Paragraph(paragraph.join("\n")));
        paragraph.clear();
    }
}

fn render_answer(ui: &mut egui::Ui, text: &str) {
    for block in parse_answer(text) {
        match block {
            AnswerBlock::Paragraph(text) => render_paragraph(ui, &text),
            AnswerBlock::Code { lang, code } => render_code_block(ui, &lang, &code),
            AnswerBlock::Formula(formula) => render_formula(ui, &formula),
        }
        ui.add_space(8.0);
    }
}

fn render_paragraph(ui: &mut egui::Ui, text: &str) {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("### ") {
            ui.label(
                egui::RichText::new(trimmed.trim_start_matches("### "))
                    .strong()
                    .color(egui::Color32::from_rgb(231, 236, 242)),
            );
        } else if trimmed.starts_with("## ") {
            ui.label(
                egui::RichText::new(trimmed.trim_start_matches("## "))
                    .heading()
                    .color(egui::Color32::from_rgb(231, 236, 242)),
            );
        } else if trimmed.starts_with("# ") {
            ui.label(
                egui::RichText::new(trimmed.trim_start_matches("# "))
                    .heading()
                    .strong()
                    .color(egui::Color32::from_rgb(231, 236, 242)),
            );
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("-").color(egui::Color32::from_rgb(119, 205, 156)));
                ui.label(inline_job(&trimmed[2..]));
            });
        } else {
            ui.label(inline_job(line));
        }
    }
}

fn inline_job(text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let normal = TextFormat {
        font_id: egui::FontId::new(14.0, egui::FontFamily::Proportional),
        color: egui::Color32::from_rgb(218, 223, 229),
        ..Default::default()
    };
    let code = TextFormat {
        font_id: egui::FontId::new(13.0, egui::FontFamily::Monospace),
        color: egui::Color32::from_rgb(245, 197, 122),
        background: egui::Color32::from_rgb(37, 42, 48),
        ..Default::default()
    };
    let formula = TextFormat {
        font_id: egui::FontId::new(13.0, egui::FontFamily::Monospace),
        color: egui::Color32::from_rgb(142, 196, 255),
        background: egui::Color32::from_rgb(30, 39, 51),
        ..Default::default()
    };

    let mut rest = text;
    while let Some(pos) = rest.find(['`', '$']) {
        let (before, after) = rest.split_at(pos);
        job.append(before, 0.0, normal.clone());

        if let Some(stripped) = after.strip_prefix('`') {
            if let Some(end) = stripped.find('`') {
                let (inner, tail) = stripped.split_at(end);
                job.append(inner, 0.0, code.clone());
                rest = &tail[1..];
            } else {
                job.append(after, 0.0, normal.clone());
                return job;
            }
        } else if let Some(stripped) = after.strip_prefix('$') {
            if let Some(end) = stripped.find('$') {
                let (inner, tail) = stripped.split_at(end);
                job.append(inner, 0.0, formula.clone());
                rest = &tail[1..];
            } else {
                job.append(after, 0.0, normal.clone());
                return job;
            }
        }
    }

    job.append(rest, 0.0, normal);
    job
}

fn render_code_block(ui: &mut egui::Ui, lang: &str, code: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(10, 12, 15))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 55, 64)))
        .rounding(egui::Rounding::same(CODE_ROUNDING))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if lang.is_empty() { "code" } else { lang })
                        .small()
                        .color(egui::Color32::from_rgb(145, 151, 160)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Copy").clicked() {
                        ui.output_mut(|output| output.copied_text = code.to_string());
                    }
                });
            });
            ui.separator();
            ui.add(
                egui::Label::new(
                    egui::RichText::new(code)
                        .monospace()
                        .color(egui::Color32::from_rgb(226, 230, 235)),
                )
                .selectable(true),
            );
        });
}

fn render_formula(ui: &mut egui::Ui, formula: &str) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(22, 31, 42))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(56, 86, 115)))
        .rounding(egui::Rounding::same(CODE_ROUNDING))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(formula)
                    .monospace()
                    .color(egui::Color32::from_rgb(167, 211, 255)),
            );
        });
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for font_path in [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ] {
        if let Ok(bytes) = std::fs::read(font_path) {
            fonts
                .font_data
                .insert("windows_cjk".to_string(), egui::FontData::from_owned(bytes));

            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts
                    .families
                    .entry(family)
                    .or_default()
                    .insert(0, "windows_cjk".to_string());
            }
            break;
        }
    }
    ctx.set_fonts(fonts);
}

fn configure_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(13, 15, 18);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(31, 35, 40);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 47, 54);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(42, 130, 104);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(CONTROL_ROUNDING);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(CONTROL_ROUNDING);
    style.visuals.widgets.active.rounding = egui::Rounding::same(CONTROL_ROUNDING);
    style.visuals.widgets.open.rounding = egui::Rounding::same(CONTROL_ROUNDING);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(42, 130, 104);
    style.visuals.hyperlink_color = egui::Color32::from_rgb(142, 196, 255);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.text_styles = [
        (
            egui::TextStyle::Heading,
            egui::FontId::new(21.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Body,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Monospace,
            egui::FontId::new(13.0, egui::FontFamily::Monospace),
        ),
        (
            egui::TextStyle::Button,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        ),
        (
            egui::TextStyle::Small,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
        ),
    ]
    .into();
    ctx.set_style(style);
}
