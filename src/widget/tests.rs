//! @file tests.rs
//! @brief Byte-row pointer and editing regression tests
//! @author modolet <y@xxyx.io>
//! @date 2026-09-12

use gpui_kit::component::Root;
use gpui_kit::{AppContext, TestAppContext, px, size};

#[gpui_kit::test]
fn batched_memory_rows_preserve_pointer_editing_after_scroll_and_resize(cx: &mut TestAppContext) {
    use crate::{HexEditor, HexEditorConfig};
    use gpui_kit::{Bounds, Modifiers, MouseButton, VisualTestContext, point};

    // A single editor gives stable debug selectors without the two page axes.
    cx.update(gpui_kit::component::init);
    let mut editor = None;
    let window = cx.open_window(size(px(800.), px(260.)), |window, cx| {
        let view = cx.new(|cx| {
            HexEditor::new(
                vec![0; 515],
                HexEditorConfig {
                    show_toolbar: false,
                    ..Default::default()
                }
                .fill_container(true),
                cx,
            )
        });
        editor = Some(view.clone());
        Root::new(view, window, cx)
    });
    let editor = editor.unwrap();
    cx.run_until_parked();
    let mut v = VisualTestContext::from_window(window.into(), cx);
    let cell = |bounds: Bounds<gpui_kit::Pixels>, column: usize, count: usize, max_width: f32| {
        let width = ((bounds.size.width - px(2. * count.saturating_sub(1) as f32)) / count as f32)
            .min(px(max_width));
        point(
            bounds.origin.x + (width + px(2.)) * column as f32 + width / 2.,
            bounds.center().y,
        )
    };
    let repaint = |v: &mut VisualTestContext| {
        v.update(|window, cx| {
            window.draw(cx).clear(cx);
        })
    };
    repaint(&mut v);
    let hex0 = v.debug_bounds("hex-row-0").unwrap();
    let hex1 = v.debug_bounds("hex-row-1").unwrap();
    assert_eq!(hex0.size.height, px(19.));
    assert_eq!(hex1.origin.y - hex0.origin.y, px(19.));
    v.simulate_click(cell(hex0, 3, 16, 36.), Modifiers::none());
    assert_eq!(
        editor.read_with(&v, |editor, _| editor.cursor_offset()),
        3,
        "HEX bounds: {hex0:?}"
    );
    v.simulate_keystrokes("a b");
    assert_eq!(
        editor.read_with(&v, |editor, _| editor.bytes_cloned()[3]),
        0xAB
    );

    repaint(&mut v);
    v.simulate_mouse_down(cell(hex0, 1, 16, 36.), MouseButton::Left, Modifiers::none());
    v.simulate_mouse_move(cell(hex1, 2, 16, 36.), MouseButton::Left, Modifiers::none());
    v.simulate_mouse_up(cell(hex1, 2, 16, 36.), MouseButton::Left, Modifiers::none());
    v.simulate_keystrokes("ctrl-c");
    assert_eq!(
        v.update(|_, cx| cx
            .read_from_clipboard()
            .unwrap()
            .text()
            .unwrap()
            .split_whitespace()
            .count()),
        18
    );

    repaint(&mut v);
    let ascii1 = v.debug_bounds("ascii-row-1").unwrap();
    v.simulate_click(cell(ascii1, 1, 16, 12.), Modifiers::none());
    v.simulate_keystrokes("z");
    assert_eq!(
        editor.read_with(&v, |editor, _| editor.bytes_cloned()[17]),
        b'z'
    );
    v.simulate_keystrokes("ctrl-z");
    assert_eq!(
        editor.read_with(&v, |editor, _| editor.bytes_cloned()[17]),
        0
    );
    v.simulate_click(cell(ascii1, 1, 16, 12.), Modifiers::none());
    v.simulate_click(
        cell(hex1, 4, 16, 36.),
        Modifiers {
            shift: true,
            ..Modifiers::none()
        },
    );
    v.simulate_keystrokes("ctrl-c");
    assert_eq!(
        v.update(|_, cx| cx
            .read_from_clipboard()
            .unwrap()
            .text()
            .unwrap()
            .split_whitespace()
            .count()),
        4
    );
    v.simulate_event(gpui_kit::MouseDownEvent {
        position: cell(hex1, 4, 16, 36.),
        button: MouseButton::Left,
        click_count: 2,
        ..Default::default()
    });
    v.simulate_mouse_up(cell(hex1, 4, 16, 36.), MouseButton::Left, Modifiers::none());
    v.simulate_keystrokes("ctrl-c");
    assert_eq!(
        v.update(|_, cx| cx
            .read_from_clipboard()
            .unwrap()
            .text()
            .unwrap()
            .split_whitespace()
            .count()),
        16
    );

    v.simulate_resize(size(px(640.), px(260.)));
    v.update(|window, cx| {
        gpui_kit::component::Theme::change(gpui_kit::component::ThemeMode::Light, Some(window), cx)
    });
    repaint(&mut v);
    v.simulate_event(gpui_kit::ScrollWheelEvent {
        position: point(px(300.), px(120.)),
        delta: gpui_kit::ScrollDelta::Pixels(point(px(0.), px(-10_000.))),
        ..Default::default()
    });
    repaint(&mut v);
    let last = v.debug_bounds("hex-row-32").unwrap();
    // Clicking a blank HEX slot in the short final row still clamps to the final byte.
    v.simulate_click(cell(last, 15, 16, 36.), Modifiers::none());
    assert_eq!(
        editor.read_with(&v, |editor, _| editor.cursor_offset()),
        514
    );
    v.simulate_keystrokes("7 7");
    assert_eq!(
        editor.read_with(&v, |editor, _| editor.bytes_cloned()[514]),
        0x77
    );
    v.simulate_keystrokes("ctrl-z");
    assert_eq!(
        editor.read_with(&v, |editor, _| editor.bytes_cloned()[514]),
        0
    );
}
