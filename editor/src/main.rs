use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Window,
    WindowBounds, WindowOptions,
};

struct AnantaSpike;

impl Render for AnantaSpike {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .justify_center()
            .items_center()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            .text_xl()
            .child("Ananta is alive.")
            .child(
                div()
                    .mt_4()
                    .text_sm()
                    .text_color(rgb(0x9399b2))
                    .child("GPUI window rendering — week 1 spike"),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|_cx| AnantaSpike),
        )
        .unwrap();
        cx.activate(true);
    });
}
