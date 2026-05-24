use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub resizable: bool,
    pub vsync: bool,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub steps_per_second: u32,
    #[serde(skip)]
    pub(crate) file_path: Option<PathBuf>,
    #[serde(skip)]
    pub(crate) no_file: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title: String::from("Tiles"),
            resizable: true,
            vsync: true,
            viewport_width: 128.0,
            viewport_height: 128.0,
            steps_per_second: 120,
            file_path: None,
            no_file: false,
        }
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder {
            config: Config::default(),
        }
    }

    pub(crate) fn save(&self) {
        if self.no_file {
            return;
        }
        let path = self.resolve_path();
        let content = toml::to_string_pretty(self).unwrap_or_default();
        let _ = std::fs::write(path, content);
    }

    fn resolve_path(&self) -> PathBuf {
        self.file_path.clone().unwrap_or_else(|| PathBuf::from("tiles.toml"))
    }
}

pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn width(mut self, width: u32) -> Self {
        self.config.width = width;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.config.height = height;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.config.resizable = resizable;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> Self {
        self.config.vsync = vsync;
        self
    }

    pub fn viewport(mut self, width: f32, height: f32) -> Self {
        self.config.viewport_width = width;
        self.config.viewport_height = height;
        self
    }

    pub fn steps_per_second(mut self, steps: u32) -> Self {
        self.config.steps_per_second = steps;
        self
    }

    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.file_path = Some(path.into());
        self
    }

    pub fn no_file(mut self) -> Self {
        self.config.no_file = true;
        self
    }

    pub fn build(mut self) -> Config {
        if !self.config.no_file {
            self.config = merge_with_file(self.config);
        }
        self.config
    }
}

fn merge_with_file(defaults: Config) -> Config {
    let path = defaults.file_path.clone().unwrap_or_else(|| PathBuf::from("tiles.toml"));
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return defaults,
    };
    let file_values: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return defaults,
    };

    let mut config = defaults;
    if let Some(table) = file_values.as_table() {
        if let Some(v) = table.get("width").and_then(|v| v.as_integer()) {
            config.width = v as u32;
        }
        if let Some(v) = table.get("height").and_then(|v| v.as_integer()) {
            config.height = v as u32;
        }
        if let Some(v) = table.get("title").and_then(|v| v.as_str()) {
            config.title = v.to_owned();
        }
        if let Some(v) = table.get("resizable").and_then(|v| v.as_bool()) {
            config.resizable = v;
        }
        if let Some(v) = table.get("vsync").and_then(|v| v.as_bool()) {
            config.vsync = v;
        }
        if let Some(v) = table.get("viewport_width").and_then(|v| v.as_float()) {
            config.viewport_width = v as f32;
        }
        if let Some(v) = table.get("viewport_height").and_then(|v| v.as_float()) {
            config.viewport_height = v as f32;
        }
        if let Some(v) = table.get("steps_per_second").and_then(|v| v.as_integer()) {
            config.steps_per_second = v as u32;
        }
    }
    config.file_path = Some(path);
    config
}
