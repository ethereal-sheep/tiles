#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) enum AnchorCorner {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
    CenterLeft,
    CenterRight,
    Center,
}

impl AnchorCorner {
    pub(crate) fn corner_offset(
        &self,
        box_w: f32,
        box_h: f32,
        box_offset_x: f32,
        box_offset_y: f32,
    ) -> (f32, f32) {
        let half_w = box_w / 2.0;
        let half_h = box_h / 2.0;

        match &self {
            AnchorCorner::TopLeft => (-box_offset_x, -box_offset_y),
            AnchorCorner::TopRight => (-box_w - box_offset_x, -box_offset_y),
            AnchorCorner::BottomLeft => (-box_offset_x, -box_h - box_offset_y),
            AnchorCorner::BottomRight => (-box_w - box_offset_x, -box_h - box_offset_y),
            AnchorCorner::TopCenter => (-half_w - box_offset_x, -box_offset_y),
            AnchorCorner::BottomCenter => (-half_w - box_offset_x, -box_h - box_offset_y),
            AnchorCorner::CenterLeft => (-box_offset_x, -half_h - box_offset_y),
            AnchorCorner::CenterRight => (-box_w - box_offset_x, -half_h - box_offset_y),
            AnchorCorner::Center => (-half_w - box_offset_x, -half_h - box_offset_y),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_left_is_zero_offset() {
        assert_eq!(
            AnchorCorner::TopLeft.corner_offset(10.0, 20.0, 0.0, 0.0),
            (0.0, 0.0)
        );
    }

    #[test]
    fn center_straddles_box() {
        assert_eq!(
            AnchorCorner::Center.corner_offset(10.0, 20.0, 0.0, 0.0),
            (-5.0, -10.0)
        );
    }

    #[test]
    fn bottom_right_offsets_full_box() {
        assert_eq!(
            AnchorCorner::BottomRight.corner_offset(10.0, 20.0, 0.0, 0.0),
            (-10.0, -20.0)
        );
    }
}
