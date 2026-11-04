use gpui::{Div, Stateful, WindowControlArea, div, prelude::*, px, rgb};
use mtorrent_gpui_core::{WindowChrome, ui::theme::DARK};
use std::rc::Rc;

pub fn chrome() -> WindowChrome {
    WindowChrome {
        drag_region: Some(Rc::new(|block: Div| {
            block.window_control_area(WindowControlArea::Drag)
        })),
        buttons: Some(Rc::new(|| window_controls().into_any_element())),
    }
}

fn window_button(id: &'static str, area: WindowControlArea, glyph: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .w(px(44.))
        .h(px(36.))
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .window_control_area(area)
        .hover(|style| style.bg(rgb(DARK.quiet_hover)))
        .child(glyph)
}

fn window_controls() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .child(window_button("win-minimize", WindowControlArea::Min, "﹣"))
        .child(window_button("win-maximize", WindowControlArea::Max, "□"))
        .child(window_button("win-close", WindowControlArea::Close, "✕"))
}
