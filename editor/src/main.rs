use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Window,
    WindowBounds, WindowOptions,
};
use ropey::Rope;
use std::fs;

struct AnantaSpike {
    rope: Rope,
    file_path: String,
}

impl AnantaSpike {
    fn new(file_path: &str) -> Self {
        let contents = fs::read_to_string(file_path)
            .unwrap_or_else(|_| format!("// Could not read file: {}", file_path));
        let rope = Rope::from_str(&contents);
        Self {
            rope,
            file_path: file_path.to_string(),
        }
    }
}

impl Render for AnantaSpike {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let line_count = self.rope.len_lines();

        let lines: Vec<_> = self
            .rope
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let line_text = line.to_string().trim_end_matches('\n').to_string();
                div()
                    .flex()
                    .flex_row()
                    .child(
                        div()
                            .w(px(48.0))
                            .text_color(rgb(0x6c7086))
                            .child(format!("{}", i + 1)),
                    )
                    .child(
                        div()
                            .text_color(rgb(0xcdd6f4))
                            .child(if line_text.is_empty() {
                                " ".to_string()
                            } else {
                                line_text
                            }),
                    )
            })
            .collect();

        div()
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
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.0), px(650.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|_cx| AnantaSpike::new("Cargo.toml")),
        )
        .unwrap();
        cx.activate(true);
    });
}
