use crate::ui::theme::DARK;
use gpui::{FontWeight, IntoElement, div, prelude::*, px, relative, rgb};

pub fn task_card(title: String, body: impl IntoElement, actions: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .w_full()
        .rounded_md()
        .bg(rgb(DARK.surface))
        .text_color(rgb(DARK.text))
        .child(div().text_lg().font_weight(FontWeight::BOLD).child(title))
        .child(body)
        .child(div().flex().flex_row().gap_4().items_center().child(actions))
}

pub fn progress_block(percent: String, ratio: f32, status_line: String) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .w_full()
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .w_full()
                .child(div().text_xs().text_color(rgb(DARK.muted)).child(status_line))
                .child(div().text_xs().child(percent)),
        )
        .child(
            div()
                .relative()
                .w_full()
                .h(px(8.))
                .rounded_md()
                .bg(rgb(DARK.track))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .h(px(8.))
                        .w(relative(ratio.clamp(0.0, 1.0)))
                        .rounded_md()
                        .bg(rgb(DARK.progress)),
                ),
        )
}
