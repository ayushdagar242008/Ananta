use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, FocusHandle,
    Focusable, KeyDownEvent, MouseButton, SharedString, Window, WindowBounds, WindowOptions,
};
use ropey::Rope;
use std::fs;
use std::path::PathBuf;
use tree_sitter::Parser;

fn color_for_kind(kind: &str) -> Option<u32> {
    const KEYWORDS: &[&str] = &[
        "fn", "let", "pub", "struct", "impl", "use", "mut", "return", "if", "else",
        "match", "for", "while", "loop", "const", "static", "enum", "trait", "mod",
        "self", "Self", "super", "crate", "as", "in", "where", "move", "async",
        "await", "dyn", "ref", "unsafe", "true", "false",
    ];

    if kind.contains("comment") {
        Some(0x6c7086)
    } else if matches!(kind, "string_literal" | "raw_string_literal" | "char_literal") {
        Some(0xa6e3a1)
    } else if matches!(kind, "integer_literal" | "float_literal") {
        Some(0xfab387)
    } else if matches!(kind, "type_identifier" | "primitive_type") {
        Some(0xf9e2af)
    } else if KEYWORDS.contains(&kind) {
        Some(0xcba6f7)
    } else {
        None
    }
}

fn collect_highlights(node: tree_sitter::Node, out: &mut Vec<(usize, usize, u32)>) {
    if let Some(color) = color_for_kind(node.kind()) {
        out.push((node.start_byte(), node.end_byte(), color));
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_highlights(child, out);
    }
}

fn list_source_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir("src")
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

struct AnantaSpike {
    rope: Rope,
    cursor: usize,
    file_path: String,
    focus_handle: FocusHandle,
    files: Vec<PathBuf>,
    byte_colors: Vec<Option<u32>>,
}

impl AnantaSpike {
    fn new(cx: &mut Context<Self>) -> Self {
        let files = list_source_files();
        let mut this = Self {
            rope: Rope::new(),
            cursor: 0,
            file_path: String::new(),
            focus_handle: cx.focus_handle(),
            files,
            byte_colors: Vec::new(),
        };
        let initial = this
            .files
            .iter()
            .find(|p| p.to_string_lossy().ends_with("main.rs"))
            .cloned()
            .or_else(|| this.files.first().cloned());
        if let Some(path) = initial {
            this.open_file(path);
        }
        this
    }

    fn open_file(&mut self, path: PathBuf) {
        let contents = fs::read_to_string(&path).unwrap_or_default();
        self.rope = Rope::from_str(&contents);
        self.file_path = path.to_string_lossy().to_string();
        self.cursor = 0;
        self.reparse();
    }

    fn reparse(&mut self) {
        let text = self.rope.to_string();
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE.into();
        if parser.set_language(&language).is_err() {
            self.byte_colors = vec![None; text.len() + 1];
            return;
        }
        let tree = match parser.parse(&text, None) {
            Some(t) => t,
            None => {
                self.byte_colors = vec![None; text.len() + 1];
                return;
            }
        };

        let mut highlights = Vec::new();
        collect_highlights(tree.root_node(), &mut highlights);

        let mut byte_colors = vec![None; text.len() + 1];
        for (start, end, color) in highlights {
            for b in start..end.min(byte_colors.len()) {
                byte_colors[b] = Some(color);
            }
        }
        self.byte_colors = byte_colors;
    }

    fn handle_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let mods = &event.keystroke.modifiers;

        match key {
            "backspace" => {
                if self.cursor > 0 {
                    self.rope.remove(self.cursor - 1..self.cursor);
                    self.cursor -= 1;
                }
            }
            "delete" => {
                if self.cursor < self.rope.len_chars() {
                    self.rope.remove(self.cursor..self.cursor + 1);
                }
            }
            "enter" => {
                self.rope.insert_char(self.cursor, '\n');
                self.cursor += 1;
            }
            "space" => {
                self.rope.insert_char(self.cursor, ' ');
                self.cursor += 1;
            }
            "left" => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            "right" => {
                if self.cursor < self.rope.len_chars() {
                    self.cursor += 1;
                }
            }
            "up" => {
                let line = self.rope.char_to_line(self.cursor);
                if line > 0 {
                    let col = self.cursor - self.rope.line_to_char(line);
                    let prev_line_start = self.rope.line_to_char(line - 1);
                    let prev_line_len = self.rope.line(line - 1).len_chars();
                    self.cursor = prev_line_start + col.min(prev_line_len);
                }
            }
            "down" => {
                let line = self.rope.char_to_line(self.cursor);
                if line + 1 < self.rope.len_lines() {
                    let col = self.cursor - self.rope.line_to_char(line);
                    let next_line_start = self.rope.line_to_char(line + 1);
                    let next_line_len = self.rope.line(line + 1).len_chars();
                    self.cursor = next_line_start + col.min(next_line_len);
                }
            }
            _ => {
                if !mods.control && !mods.platform && key.chars().count() == 1 {
                    if let Some(ch) = key.chars().next() {
                        self.rope.insert_char(self.cursor, ch);
                        self.cursor += 1;
                    }
                }
            }
        }

        self.reparse();
        cx.notify();
    }
}

impl Focusable for AnantaSpike {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AnantaSpike {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let line_count = self.rope.len_lines();
        let cursor_line = self.rope.char_to_line(self.cursor);
        let cursor_col = self.cursor - self.rope.line_to_char(cursor_line);

        let sidebar_items: Vec<_> = self
            .files
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let path_clone = path.clone();
                let is_active = path.to_string_lossy() == self.file_path;
                div()
                    .id(SharedString::from(format!("file-{}", idx)))
                    .px_3()
                    .py_1()
                    .text_sm()
                    .text_color(if is_active {
                        rgb(0xf5e0dc)
                    } else {
                        rgb(0xa6adc8)
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            this.open_file(path_clone.clone());
                            cx.notify();
                        }),
                    )
                    .child(name)
            })
            .collect();

        let sidebar = div()
            .flex()
            .flex_col()
            .w(px(200.0))
            .h_full()
            .bg(rgb(0x181825))
            .py_2()
            .children(sidebar_items);

        let rope_snapshot = self.rope.clone();
        let byte_colors = &self.byte_colors;

        let lines: Vec<_> = rope_snapshot
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let line_text = line.to_string().trim_end_matches('\n').to_string();
                let line_char_start = rope_snapshot.line_to_char(i);

                let mut spans: Vec<gpui::AnyElement> = Vec::new();
                let mut current_text = String::new();
                let mut current_color: Option<u32> = None;

                let flush = |text: &mut String, color: Option<u32>, out: &mut Vec<gpui::AnyElement>| {
                    if !text.is_empty() {
                        out.push(
                            div()
                                .text_color(rgb(color.unwrap_or(0xcdd6f4)))
                                .child(text.clone())
                                .into_any_element(),
                        );
                        text.clear();
                    }
                };

                for (offset, ch) in line_text.chars().enumerate() {
                    if i == cursor_line && offset == cursor_col {
                        flush(&mut current_text, current_color, &mut spans);
                        spans.push(
                            div()
                                .w(px(2.0))
                                .h(px(18.0))
                                .bg(rgb(0xf5e0dc))
                                .into_any_element(),
                        );
                    }
                    let global_char_idx = line_char_start + offset;
                    let byte_idx = rope_snapshot.char_to_byte(global_char_idx);
                    let color = byte_colors.get(byte_idx).copied().flatten();
                    if color != current_color {
                        flush(&mut current_text, current_color, &mut spans);
                        current_color = color;
                    }
                    current_text.push(ch);
                }
                if i == cursor_line && cursor_col == line_text.chars().count() {
                    flush(&mut current_text, current_color, &mut spans);
                    spans.push(
                        div()
                            .w(px(2.0))
                            .h(px(18.0))
                            .bg(rgb(0xf5e0dc))
                            .into_any_element(),
                    );
                }
                flush(&mut current_text, current_color, &mut spans);

                div()
                    .flex()
                    .flex_row()
                    .child(
                        div()
                            .w(px(48.0))
                            .text_color(rgb(0x6c7086))
                            .child(format!("{}", i + 1)),
                    )
                    .child(div().flex().flex_row().children(spans))
            })
            .collect();

        div()
            .id("root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, _window, cx| {
                this.handle_key(event, cx);
            }))
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .child(sidebar)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .h_full()
                    .child(
                        div()
                            .flex()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x11111b))
                            .text_color(rgb(0xa6adc8))
                            .text_sm()
                            .child(format!("{}  ({} lines)", self.file_path, line_count)),
                    )
                    .child(
                        div()
                            .id("buffer-scroll")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .overflow_y_scroll()
                            .p_4()
                            .font_family("monospace")
                            .text_size(px(14.0))
                            .children(lines),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_window, cx| cx.new(|cx| AnantaSpike::new(cx)),
            )
            .unwrap();

        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle(cx));
            })
            .ok();

        cx.activate(true);
    });
}
