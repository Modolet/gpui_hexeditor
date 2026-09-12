//! @file row.rs
//! @brief Batched byte-row painting and shared pointer geometry
//! @author modolet <y@xxyx.io>
//! @date 2026-09-12

use super::*;
use gpui_kit::{Bounds, Pixels, Point, TextAlign, TextRun, canvas, fill, point, size};
use std::cell::Cell;

/// Use the same geometry for painting and hit testing, including flex shrink,
/// the inter-byte gaps, and the unused cells in the last HEX row.
#[derive(Clone, Copy)]
struct ByteRowGeometry {
    bounds: Bounds<Pixels>,
    count: usize,
    width: Pixels,
    gap: Pixels,
}

impl ByteRowGeometry {
    fn new(bounds: Bounds<Pixels>, count: usize, max_width: Pixels, gap: Pixels) -> Self {
        let available = (bounds.size.width - gap * count.saturating_sub(1) as f32).max(px(0.));
        let width = (available / count.max(1) as f32).min(max_width);
        Self {
            bounds,
            count,
            width,
            gap,
        }
    }

    fn cell_bounds(self, column: usize) -> Bounds<Pixels> {
        Bounds::new(
            point(
                self.bounds.origin.x + (self.width + self.gap) * column as f32,
                self.bounds.origin.y,
            ),
            size(self.width, self.bounds.size.height),
        )
    }

    fn column_at(self, position: Point<Pixels>) -> Option<usize> {
        if self.count == 0 || self.width <= px(0.) || !self.bounds.contains(&position) {
            return None;
        }
        let local = position.x - self.bounds.origin.x;
        let column = (local / (self.width + self.gap)).floor() as usize;
        (column < self.count && local - (self.width + self.gap) * (column as f32) < self.width)
            .then_some(column)
    }
}

impl HexEditor {
    fn render_byte_pane(
        &self,
        row: usize,
        pane: ActivePane,
        style: HexEditorStyle,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let bytes = self.state.row_bytes(row);
        let row_start = row * self.state.bytes_per_row();
        let (count, max_width, id) = match pane {
            ActivePane::Hex => (self.state.bytes_per_row(), px(36.), "hex-row"),
            ActivePane::Ascii => (bytes.len(), px(12.), "ascii-row"),
        };
        let cells = (0..count)
            .map(|column| {
                let offset = row_start + column;
                let text = match (pane, bytes.get(column)) {
                    (ActivePane::Hex, Some(byte)) => format!("{byte:02X}"),
                    (ActivePane::Ascii, Some(byte)) => {
                        if byte.is_ascii_graphic() || *byte == b' ' {
                            char::from(*byte)
                        } else {
                            '·'
                        }
                        .to_string()
                    }
                    _ => String::new(),
                };
                // Match the original style precedence: cursor > dirty > selection.
                let decoration = if offset == self.state.cursor_offset() {
                    Some((style.current_cell_background, Some(style.focus_ring)))
                } else if self.state.is_dirty_offset(offset) {
                    Some((style.dirty_cell_background, Some(style.dirty_cell_border)))
                } else if self.state.is_selected(offset) {
                    Some((style.selection_background, None))
                } else {
                    None
                };
                (text, decoration)
            })
            .collect::<Vec<_>>();
        let geometry = Rc::new(Cell::new(ByteRowGeometry::new(
            Bounds::default(),
            count,
            max_width,
            px(2.),
        )));
        let for_down = geometry.clone();
        let for_move = geometry.clone();
        div()
            .id((id, row))
            .debug_selector(move || format!("{id}-{row}"))
            .size_full()
            .font_family("DejaVu Sans Mono")
            .text_xs()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui_kit::MouseDownEvent, window, cx| {
                    let Some(column) = for_down.get().column_at(event.position) else {
                        return;
                    };
                    let offset = this.state.row_col_to_offset(row, column);
                    if event.click_count >= 2 {
                        this.state.set_active_pane(pane);
                        this.state.select_row(row);
                        window.focus(&this.focus_handle, cx);
                        cx.notify();
                    } else if event.modifiers.shift {
                        this.shift_click_select(offset, pane, window, cx);
                    } else {
                        this.begin_pointer_selection(offset, pane, false, window, cx);
                    }
                }),
            )
            .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                if event.dragging()
                    && let Some(column) = for_move.get().column_at(event.position)
                {
                    let offset = this.state.row_col_to_offset(row, column);
                    this.update_pointer_selection(offset, cx);
                }
            }))
            .child(
                canvas(
                    move |bounds, window, _| {
                        let layout = ByteRowGeometry::new(
                            bounds,
                            count,
                            max_width,
                            window.rem_size() * 0.125,
                        );
                        geometry.set(layout);
                        let font = window.text_style().font();
                        cells
                            .into_iter()
                            .enumerate()
                            .map(|(column, (text, decoration))| {
                                let cell = layout.cell_bounds(column);
                                let run = TextRun {
                                    len: text.len(),
                                    font: font.clone(),
                                    color: style.text,
                                    background_color: None,
                                    underline: None,
                                    strikethrough: None,
                                };
                                let line = window.text_system().shape_line(
                                    text.into(),
                                    window.text_style().font_size.to_pixels(window.rem_size()),
                                    &[run],
                                    None,
                                );
                                let decoration = decoration.map(|(background, border)| {
                                    let quad =
                                        fill(cell, background).corner_radii(style.cell_radius);
                                    match border {
                                        Some(color) => {
                                            quad.border_widths(px(1.)).border_color(color)
                                        }
                                        None => quad,
                                    }
                                });
                                (cell, line, decoration)
                            })
                            .collect::<Vec<_>>()
                    },
                    move |bounds, cells, window, cx| {
                        // Separate non-overlapping backgrounds from text to preserve draw order.
                        window.paint_layer(bounds, |window| {
                            for (_, _, decoration) in &cells {
                                if let Some(quad) = decoration {
                                    window.paint_quad(quad.clone());
                                }
                            }
                        });
                        window.paint_layer(bounds, |window| {
                            for (cell, line, _) in &cells {
                                let origin = point(
                                    cell.origin.x + (cell.size.width - line.width) / 2.,
                                    cell.origin.y,
                                );
                                window.with_content_mask(
                                    Some(gpui_kit::ContentMask { bounds: *cell }),
                                    |window| {
                                        let _ = line.paint(
                                            origin,
                                            cell.size.height,
                                            TextAlign::Left,
                                            None,
                                            window,
                                            cx,
                                        );
                                    },
                                );
                            }
                        });
                    },
                )
                .size_full(),
            )
    }

    pub(super) fn render_row(
        &self,
        row: usize,
        style: HexEditorStyle,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let row_start = row * self.state.bytes_per_row();
        div()
            .flex()
            .flex_row()
            .items_center()
            // The virtual list's declared row size must match the actual layout.
            .h(style.row_height)
            .w_full()
            .flex_shrink_0()
            .px_1p5()
            .gap_1p5()
            .child(
                div()
                    .w(px(72.))
                    .flex_shrink_0()
                    .font_family("DejaVu Sans Mono")
                    .text_xs()
                    .text_color(style.muted_text)
                    .child(format!("{row_start:08X}")),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.render_byte_pane(row, ActivePane::Hex, style, cx)),
            )
            .when(self.config.show_ascii, |this| {
                this.child(
                    div()
                        .w(px(144.))
                        .flex_shrink_0()
                        .h_full()
                        .child(self.render_byte_pane(row, ActivePane::Ascii, style, cx)),
                )
            })
            .when(
                self.state.offset_to_row_col(self.state.cursor_offset()).0 == row,
                |this| this.bg(style.current_row_background),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_testing_tracks_shrink_gaps_and_scrolled_origin() {
        let bounds = Bounds::new(point(px(30.), px(-7.)), size(px(302.), px(19.)));
        let geometry = ByteRowGeometry::new(bounds, 16, px(36.), px(2.));
        assert_eq!(geometry.width, px(17.));
        assert_eq!(geometry.column_at(point(px(30.), px(0.))), Some(0));
        assert_eq!(geometry.column_at(point(px(47.5), px(0.))), None);
        assert_eq!(geometry.column_at(point(px(315.), px(0.))), Some(15));
        assert_eq!(geometry.column_at(point(px(30.), px(13.))), None);
        assert_eq!(geometry.column_at(point(px(333.), px(0.))), None);
    }

    #[test]
    fn short_ascii_rows_do_not_stretch_or_hit_unused_space() {
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(144.), px(19.)));
        let geometry = ByteRowGeometry::new(bounds, 3, px(12.), px(2.));
        assert_eq!(geometry.width, px(12.));
        assert_eq!(geometry.column_at(point(px(30.), px(8.))), Some(2));
        assert_eq!(geometry.column_at(point(px(70.), px(8.))), None);
        let empty = ByteRowGeometry::new(bounds, 0, px(12.), px(2.));
        assert_eq!(empty.column_at(point(px(0.), px(8.))), None);
    }
}
