use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, FocusHandle,
    Focusable, KeyDownEvent, Window, WindowBounds, WindowOptions,
};
use ropey::Rope;
use std::fs;

struct AnantaSpike {
    rope: Rope,
    cursor: usize,
    file_path: String,
    focus_handle: FocusHandle,
}

impl AnantaSpike {
    fn new(file_path: &str, cx: &mut Context<Self>) -> Self {
        let contents = fs::read_to_string(file_path)
            .unwrap_or_else(|_| format!("// Could not read file: {}", file_path));
        let rope = Rope::from_str(&contents);
        Self {
            rope,
            cursor: 0,
            file_path: file_path.to_string(),
            focus_handle: cx.focus_handle(),
        }
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

        let lines: Vec<_> = self
            .rope
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let line_text = line.to_string().trim_end_matches('\n').to_string();

                let content: gpui::AnyElement = if i == cursor_line {
                    let chars: Vec<char> = line_text.chars().collect();
                    let before: String = chars[..cursor_col.min(chars.len())].iter().collect();
                    let after: String = chars[cursor_col.min(chars.len())..].iter().collect();
                    div()
                        .flex()
                        .flex_row()
                        .child(before)
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(18.0))
                                .bg(rgb(0xf5e0dc)),
                        )
                        .child(after)
                        .into_any_element()
                } else {
                    div()
                        .child(if line_text.is_empty() {
                            " ".to_string()
                        } else {
                            line_text
                        })
                        .into_any_element()
                };

                div()
                    .flex()
                    .flex_row()
                    .child(
                        div()
                            .w(px(48.0))
                            .text_color(rgb(0x6c7086))
                            .child(format!("{}", i + 1)),
                    )
                    .child(div().text_color(rgb(0xcdd6f4)).child(content))
            })
            .collect();

        div()
            .id("root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, _window, cx| {
                this.handle_key(event, cx);
            }))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .child(
                div()
                    .flex()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x181825))
                    .text_color(rgb(0xa6adc8))
                    .text_sm()
                    .child(format!(
                        "{}  ({} lines)  — click window, then type",
                        self.file_path, line_count
                    )),
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
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.0), px(650.0)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_window, cx| cx.new(|cx| AnantaSpike::new("Cargo.toml", cx)),
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
