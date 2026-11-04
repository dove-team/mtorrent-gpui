use gpui::{
    AnyElement, Context, Entity, Focusable, IntoElement, Modifiers, Pixels, Render,
    TestAppContext, VisualTestContext, Window, div, point, prelude::*, px, size,
};
use mtorrent::utils::re_exports::mtorrent_base::pwp::PeerOrigin;
use mtorrent_gpui_core::state::progress::PeerInfo;
use mtorrent_gpui_core::ui::{self, TextInput};
use std::collections::BTreeMap;
use std::ops::Range;

struct Probe {
    build: Box<dyn FnMut() -> AnyElement>,
    width: Pixels,
}

impl Render for Probe {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(self.width)
            .flex()
            .flex_col()
            .child((self.build)())
    }
}

type Rect = (f32, f32, f32, f32);

fn painted(cx: &mut TestAppContext, build: impl FnMut() -> AnyElement + 'static) -> Vec<Rect> {
    let (_view, vcx) = cx.add_window_view(|_, _| Probe {
        build: Box::new(build),
        width: px(900.),
    });
    vcx.run_until_parked();
    vcx.update(|window, _cx| {
        let scale = window.scale_factor();
        let unscale = |value: gpui::ScaledPixels| value.as_f32() / scale;
        window
            .painted_quads()
            .into_iter()
            .map(|quad| {
                (
                    unscale(quad.bounds.origin.x),
                    unscale(quad.bounds.origin.y),
                    unscale(quad.bounds.size.width),
                    unscale(quad.bounds.size.height),
                )
            })
            .collect()
    })
}

fn focus_input(input: &Entity<TextInput>, cx: &mut VisualTestContext) {
    cx.update(|window, app| {
        let handle = input.focus_handle(app);
        handle.focus(window, app);
    });
}

fn bind_input_keys(cx: &mut VisualTestContext) {
    cx.update(|_window, app| app.bind_keys(ui::key_bindings()));
}

fn text(input: &Entity<TextInput>, cx: &mut VisualTestContext) -> String {
    let input = input.clone();
    cx.update(|_window, app| input.read(app).text().to_owned())
}

fn selection(input: &Entity<TextInput>, cx: &mut VisualTestContext) -> Range<usize> {
    let input = input.clone();
    cx.update(|_window, app| input.read(app).selection())
}

fn focused(input: &Entity<TextInput>, cx: &mut VisualTestContext) -> bool {
    let input = input.clone();
    cx.update(|window, app| input.focus_handle(app).is_focused(window))
}

fn sample_peers() -> BTreeMap<String, PeerInfo> {
    BTreeMap::from([
        (
            "198.51.100.7:6881".to_owned(),
            PeerInfo {
                client: Some("qBittorrent 4.6.4".to_owned()),
                origin: PeerOrigin::Dht,
                proto: "TCP (S)".to_owned(),
                downloaded: 1_048_576,
                uploaded: 2_097_152,
            },
        ),
        (
            "203.0.113.19:51413".to_owned(),
            PeerInfo {
                client: None,
                origin: PeerOrigin::Tracker,
                proto: "uTP".to_owned(),
                downloaded: 512,
                uploaded: 0,
            },
        ),
    ])
}

#[gpui::test]
fn progress_bar_fill_tracks_the_ratio(cx: &mut TestAppContext) {
    for (ratio, expected) in [(0.0, 0.0), (0.25, 0.25), (0.5, 0.5), (1.0, 1.0)] {
        let rects = painted(cx, move || {
            ui::progress_block("50.0%".to_owned(), ratio, "状态".to_owned()).into_any_element()
        });
        let mut bars: Vec<f32> = rects
            .iter()
            .filter(|rect| (rect.3 - 8.0).abs() < 0.6)
            .map(|rect| rect.2)
            .collect();
        bars.sort_by(|a, b| a.total_cmp(b));

        if expected == 0.0 {
            assert_eq!(
                bars,
                vec![900.0],
                "a zero width fill is culled, leaving only the track"
            );
            continue;
        }

        assert_eq!(bars.len(), 2, "track and fill at ratio {ratio}");
        assert_eq!(bars[1], 900.0, "the track spans the full width");
        assert!(
            (bars[0] / bars[1] - expected).abs() < 0.01,
            "fill/track = {} at ratio {ratio}, expected {expected}",
            bars[0] / bars[1]
        );
    }
}

fn tallest(rects: Vec<Rect>) -> f32 {
    rects.iter().map(|rect| rect.3).fold(0.0, f32::max)
}

#[gpui::test]
fn peer_grid_packs_cells_into_rows_not_a_single_column(cx: &mut TestAppContext) {
    let grid = tallest(painted(cx, {
        let peers = sample_peers();
        move || {
            ui::task_card(
                "ubuntu.iso".to_owned(),
                ui::peer_table(&peers),
                div().child("actions"),
            )
            .into_any_element()
        }
    }));

    let one_column = tallest(painted(cx, || {
        let stack = div().flex().flex_col().gap_4().w_full().children(
            (0..18)
                .map(|_| div().text_sm().child("198.51.100.7:6881"))
                .collect::<Vec<_>>(),
        );
        ui::task_card(
            "ubuntu.iso".to_owned(),
            stack,
            div().child("actions"),
        )
        .into_any_element()
    }));

    assert!(
        grid < one_column,
        "a six column grid ({grid}) must be shorter than the same 18 cells stacked ({one_column})"
    );

    let cards = tallest(painted(cx, {
        let peers = sample_peers();
        move || {
            ui::task_card(
                "ubuntu.iso".to_owned(),
                ui::peer_section(&peers),
                div().child("actions"),
            )
            .into_any_element()
        }
    }));
    assert!(
        cards > 0.0 && cards < one_column,
        "the narrow peer cards should also lay out compactly ({cards})"
    );
}

#[gpui::test]
fn task_card_paints_one_background_at_the_origin(cx: &mut TestAppContext) {
    let rects = painted(cx, || {
        ui::task_card(
            "ubuntu.iso".to_owned(),
            div().child("body"),
            div().child("actions"),
        )
        .into_any_element()
    });
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].0, 0.0);
    assert_eq!(rects[0].1, 0.0);
    assert_eq!(rects[0].2, 900.0);
    assert!(rects[0].3 > 100.0);
}

#[gpui::test]
fn typed_text_and_ime_replacement_update_the_value(cx: &mut TestAppContext) {
    let (input, vcx) = cx.add_window_view(|_, cx| TextInput::new("磁力链接或文件路径：", cx));
    vcx.run_until_parked();
    bind_input_keys(vcx);
    focus_input(&input, vcx);

    vcx.simulate_input("abc123");
    assert_eq!(text(&input, vcx), "abc123");

    vcx.simulate_keystrokes("ctrl-a");
    assert_eq!(selection(&input, vcx), 0..6);

    vcx.simulate_input("漢字");
    assert_eq!(text(&input, vcx), "漢字");

    vcx.simulate_keystrokes("end");
    vcx.simulate_input("Z");
    assert_eq!(text(&input, vcx), "漢字Z");

    vcx.simulate_keystrokes("backspace");
    assert_eq!(text(&input, vcx), "漢字");

    vcx.simulate_keystrokes("ctrl-a");
    vcx.simulate_keystrokes("ctrl-c");
    let copied = vcx.update(|_window, app| app.read_from_clipboard().and_then(|item| item.text()));
    assert_eq!(copied, Some("漢字".to_owned()));

    vcx.simulate_keystrokes("ctrl-a");
    vcx.simulate_keystrokes("delete");
    assert_eq!(text(&input, vcx), "");
}

#[gpui::test]
fn long_magnet_link_types_into_the_input(cx: &mut TestAppContext) {
    let (input, vcx) = cx.add_window_view(|_, cx| TextInput::new("ph", cx));
    vcx.run_until_parked();
    bind_input_keys(vcx);
    focus_input(&input, vcx);

    let link = "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=ubuntu.iso";
    vcx.simulate_input(link);
    assert_eq!(text(&input, vcx), link);
    assert_eq!(selection(&input, vcx), link.len()..link.len());

    vcx.simulate_keystrokes("ctrl-a");
    vcx.simulate_input("x");
    assert_eq!(text(&input, vcx), "x");
}

#[gpui::test]
fn clicking_the_input_focuses_it_and_keystrokes_land(cx: &mut TestAppContext) {
    let (input, vcx) = cx.add_window_view(|_, cx| TextInput::new("磁力链接或文件路径：", cx));
    vcx.run_until_parked();
    bind_input_keys(vcx);

    assert!(!focused(&input, vcx));
    vcx.simulate_click(point(px(20.), px(12.)), Modifiers::default());
    vcx.run_until_parked();
    assert!(focused(&input, vcx));

    vcx.simulate_input("ab");
    assert_eq!(text(&input, vcx), "ab");
}

#[gpui::test]
fn viewport_resize_flips_the_breakpoint(cx: &mut TestAppContext) {
    let (input, vcx) = cx.add_window_view(|_, cx| TextInput::new("ph", cx));
    vcx.update(|window, _cx| {
        assert!(window.viewport_size().width >= px(ui::theme::NARROW_WIDTH))
    });
    vcx.simulate_resize(size(px(600.), px(800.)));
    vcx.run_until_parked();
    vcx.update(|window, _cx| {
        assert!(window.viewport_size().width < px(ui::theme::NARROW_WIDTH))
    });
    bind_input_keys(vcx);
    focus_input(&input, vcx);
    vcx.simulate_input("x");
    assert_eq!(text(&input, vcx), "x");
}
