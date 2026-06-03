use tiles::font::Font;

#[derive(Clone)]
pub struct PaneStyle {
    pub border: bool,
    pub corner_radius: u8,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub title_background_color: [f32; 4],
    pub title_text_color: [f32; 4],
    pub text_color: [f32; 4],
    pub font: &'static Font,
    pub padding: f32,
    pub element_spacing: f32,
}

impl Default for PaneStyle {
    fn default() -> Self {
        Self {
            border: true,
            corner_radius: 1,
            background_color: [0.10, 0.10, 0.12, 0.95],
            border_color: [0.3, 0.3, 0.35, 1.0],
            title_background_color: [0.16, 0.16, 0.20, 1.0],
            title_text_color: [0.9, 0.9, 0.9, 1.0],
            text_color: [0.8, 0.8, 0.8, 1.0],
            font: &tiles::font::TINY5_4X5,
            padding: 3.0,
            element_spacing: 2.0,
        }
    }
}

pub struct PaneConfig {
    pub(crate) size: Option<(f32, f32)>,
    pub(crate) movable: bool,
    pub(crate) resizable: bool,
    pub(crate) closable: bool,
    pub(crate) title: Option<String>,
    pub(crate) style: Option<PaneStyle>,
    pub(crate) position: Option<(f32, f32)>,
}

impl PaneConfig {
    pub fn new() -> Self {
        Self {
            size: None,
            movable: false,
            resizable: false,
            closable: false,
            title: None,
            style: None,
            position: None,
        }
    }

    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.size = Some((w, h));
        self
    }

    pub fn movable(mut self, movable: bool) -> Self {
        self.movable = movable;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn style(mut self, style: PaneStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }
}

impl Default for PaneConfig {
    fn default() -> Self {
        Self::new()
    }
}
