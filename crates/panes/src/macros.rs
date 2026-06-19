#[macro_export]
macro_rules! ui {
    ($buf:ident { $($body:tt)* }) => {
        $crate::ui!(@go $buf; $($body)*);
    };

    (@go $buf:ident; fill_rect($w:expr, $h:expr, $color:expr) ; $($rest:tt)*) => {
        $crate::widgets::fill_rect(&mut $buf, $w, $h, $color);
        $crate::ui!(@go $buf; $($rest)*);
    };

    (@go $buf:ident; spacer($w:expr, $h:expr) ; $($rest:tt)*) => {
        $crate::widgets::spacer(&mut $buf, $w, $h);
        $crate::ui!(@go $buf; $($rest)*);
    };

    (@go $buf:ident; let $var:ident = button($w:expr, $h:expr, $color:expr, $hover:expr) ; $($rest:tt)*) => {
        let $var = $crate::widgets::button(&mut $buf, $w, $h, $color, $hover);
        $crate::ui!(@go $buf; $($rest)*);
    };

    (@go $buf:ident; panel($pad:expr, $color:expr, $axis:expr, $gap:expr) { $($children:tt)* } $($rest:tt)*) => {
        $crate::widgets::panel(&mut $buf, $pad, $color, $axis, $gap, |buf| {
            $crate::ui!(@go buf; $($children)*);
        });
        $crate::ui!(@go $buf; $($rest)*);
    };

    (@go $buf:ident; @ if $cond:ident { $($body:tt)* } $($rest:tt)*) => {
        if $cond { $crate::ui!(@go $buf; $($body)*); }
        $crate::ui!(@go $buf; $($rest)*);
    };

    (@go $buf:ident; @ if {$cond:expr} { $($body:tt)* } $($rest:tt)*) => {
        if $cond { $crate::ui!(@go $buf; $($body)*); }
        $crate::ui!(@go $buf; $($rest)*);
    };

    (@go $buf:ident; @ for $pat:pat in $iter:ident { $($body:tt)* } $($rest:tt)*) => {
        for $pat in $iter { $crate::ui!(@go $buf; $($body)*); }
        $crate::ui!(@go $buf; $($rest)*);
    };

    (@go $buf:ident; @ for $pat:pat in {$iter:expr} { $($body:tt)* } $($rest:tt)*) => {
        for $pat in $iter { $crate::ui!(@go $buf; $($body)*); }
        $crate::ui!(@go $buf; $($rest)*);
    };

    (@go $buf:ident; | $b:ident | { $($code:tt)* } $($rest:tt)*) => {
        { let $b = &mut $buf; $($code)* }
        $crate::ui!(@go $buf; $($rest)*);
    };

    (@go $buf:ident;) => {};
}

#[cfg(test)]
mod tests {
    use crate::{Axis, UiBuffer};
    use tiles::Color;

    const RED: Color = Color::linear(1.0, 0.0, 0.0, 1.0);
    const BLUE: Color = Color::linear(0.0, 0.0, 1.0, 1.0);
    const GREY: Color = Color::linear(0.3, 0.3, 0.3, 1.0);

    #[test]
    fn macro_fill_rect() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        ui!(buf {
            fill_rect(4, 3, RED);
        });
        assert_eq!(buf.cells().len(), 12);
    }

    #[test]
    fn macro_panel_with_children() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        ui!(buf {
            panel(2, GREY, Axis::Column, 0) {
                fill_rect(3, 3, RED);
                fill_rect(3, 3, BLUE);
            }
        });
        // Inner: 3x6, panel: 7x10
        assert_eq!(buf.cursor(), (0, 10));
    }

    #[test]
    fn macro_button_binding() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        buf.input.mouse_screen_pos = glam::Vec2::new(200.0, 200.0);
        buf.input.prev_mouse_screen_pos = glam::Vec2::new(200.0, 200.0);
        ui!(buf {
            let hit = button(5, 3, RED, BLUE);
        });
        assert!(!hit.is_hovered());
        assert_eq!(buf.cells().len(), 15);
    }

    #[test]
    fn macro_if_control_flow() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        let show = true;
        ui!(buf {
            @ if show {
                fill_rect(2, 2, RED);
            }
        });
        assert_eq!(buf.cells().len(), 4);

        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        let show = false;
        ui!(buf {
            @ if show {
                fill_rect(2, 2, RED);
            }
        });
        assert_eq!(buf.cells().len(), 0);
    }

    #[test]
    fn macro_for_loop() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        let colors = [RED, BLUE, GREY];
        ui!(buf {
            @ for c in colors {
                fill_rect(2, 2, c);
            }
        });
        assert_eq!(buf.cells().len(), 12);
        assert_eq!(buf.cursor(), (0, 6));
    }

    #[test]
    fn macro_raw_escape() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        ui!(buf {
            |b| {
                b.advance(10, 10);
            }
        });
        assert_eq!(buf.cursor(), (0, 10));
    }
}
