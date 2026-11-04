use crate::host::WindowChrome;
use crate::ui::theme::DARK;
use gpui::{FontWeight, IntoElement, div, prelude::*, rgb};

pub fn header(
    title: &'static str,
    subtitle: String,
    actions: impl IntoElement,
    chrome: &WindowChrome,
) -> impl IntoElement {
    let title_block = div()
        .flex_1()
        .flex()
        .flex_col()
        .justify_center()
        .child(div().text_lg().font_weight(FontWeight::BOLD).child(title))
        .child(div().text_xs().text_color(rgb(DARK.muted)).child(subtitle));

    let title_block = match &chrome.drag_region {
        Some(mark_drag) => mark_drag(title_block),
        None => title_block,
    };

    let row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap_4()
        .px_4()
        .py_1()
        .bg(rgb(DARK.surface))
        .text_color(rgb(DARK.text))
        .child(title_block)
        .child(div().flex().flex_row().items_center().child(actions));

    match &chrome.buttons {
        Some(build) => row.child(build()),
        None => row,
    }
}
