use log::info;

pub const DEFAULT_WIDTH_PX: u16 = 480;
pub const DEFAULT_HEIGHT_PX: u16 = 320;
pub const DEFAULT_REFRESH_HZ: u8 = 30;
pub const DISPLAY_CONTROL_TOPIC: &str = "display.control";
pub const DISPLAY_STATUS_TOPIC: &str = "display.status";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayMode {
    Standby,
    LocalPreview,
    RemoteVideoCall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TftDisplayConfig {
    pub width_px: u16,
    pub height_px: u16,
    pub refresh_hz: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TftDisplayState {
    pub mode: DisplayMode,
    pub config: TftDisplayConfig,
    pub backlight_percent: u8,
}

#[derive(Clone, Debug)]
pub struct TftDisplayDriver {
    state: TftDisplayState,
}

impl TftDisplayConfig {
    pub fn new(width_px: u16, height_px: u16, refresh_hz: u8) -> Self {
        Self {
            width_px: width_px.max(1),
            height_px: height_px.max(1),
            refresh_hz: refresh_hz.max(1),
        }
    }
}

impl Default for TftDisplayConfig {
    fn default() -> Self {
        Self::new(DEFAULT_WIDTH_PX, DEFAULT_HEIGHT_PX, DEFAULT_REFRESH_HZ)
    }
}

impl TftDisplayDriver {
    pub fn new(config: TftDisplayConfig) -> Self {
        info!(
            "Initializing TFT display model width_px={} height_px={} refresh_hz={}",
            config.width_px, config.height_px, config.refresh_hz
        );
        Self {
            state: TftDisplayState {
                mode: DisplayMode::Standby,
                config,
                backlight_percent: 50,
            },
        }
    }

    pub fn set_mode(&mut self, mode: DisplayMode) -> TftDisplayState {
        self.state.mode = mode;
        self.state
    }

    pub fn set_backlight(&mut self, backlight_percent: u8) -> TftDisplayState {
        self.state.backlight_percent = backlight_percent.min(100);
        self.state
    }

    pub fn state(&self) -> TftDisplayState {
        self.state
    }
}

impl Default for TftDisplayDriver {
    fn default() -> Self {
        Self::new(TftDisplayConfig::default())
    }
}
