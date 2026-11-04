use crate::state::format::format_bytes;
use crate::state::progress::{PeerInfo, origin_label};
use crate::ui::theme::DARK;
use gpui::{IntoElement, div, prelude::*, rgb};
use rust_i18n::t;
use std::collections::BTreeMap;

fn peer_cells(peers: &BTreeMap<String, PeerInfo>) -> Vec<(String, String, String, String, String, String)> {
    peers
        .iter()
        .map(|(address, peer)| {
            let client = match &peer.client {
                Some(client) => client.clone(),
                None => t!("peers.unknown_client").into_owned(),
            };
            (
                address.clone(),
                client,
                origin_label(peer.origin).into_owned(),
                peer.proto.clone(),
                format_bytes(peer.downloaded),
                format_bytes(peer.uploaded),
            )
        })
        .collect()
}

fn heading(text: String) -> impl IntoElement {
    div().text_xs().text_color(rgb(DARK.muted)).child(text)
}

pub fn peer_table(peers: &BTreeMap<String, PeerInfo>) -> impl IntoElement {
    let cells = peer_cells(peers);

    let grid = div().grid().grid_cols(6).gap_4().w_full().child(
        heading(t!("peers.address").into_owned()),
    );
    let grid = grid
        .child(heading(t!("peers.client").into_owned()))
        .child(heading(t!("peers.origin").into_owned()))
        .child(heading(t!("peers.protocol").into_owned()))
        .child(heading(t!("peers.downloaded").into_owned()))
        .child(heading(t!("peers.uploaded").into_owned()));

    let grid = cells.into_iter().fold(grid, |grid, row| {
        grid
            .child(div().text_sm().child(row.0))
            .child(div().text_sm().text_color(rgb(DARK.muted)).child(row.1))
            .child(div().text_sm().text_color(rgb(DARK.muted)).child(row.2))
            .child(div().text_sm().child(row.3))
            .child(div().text_sm().child(row.4))
            .child(div().text_sm().child(row.5))
    });

    grid
}

pub fn peer_section(peers: &BTreeMap<String, PeerInfo>) -> impl IntoElement {
    let rows = peer_cells(peers)
        .into_iter()
        .map(|(address, client, origin, proto, downloaded, uploaded)| {
            div()
                .flex()
                .flex_col()
                .gap_4()
                .w_full()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_4()
                        .w_full()
                        .child(div().text_sm().child(address))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(DARK.muted))
                                .child(format!("{client} · {origin} · {proto}")),
                        ),
                )
                .child(div().text_xs().text_color(rgb(DARK.muted)).child(format!(
                    "↓ {} / ↑ {}",
                    downloaded, uploaded
                )))
        })
        .collect::<Vec<_>>();

    div()
        .flex()
        .flex_col()
        .gap_4()
        .w_full()
        .children(rows)
}
