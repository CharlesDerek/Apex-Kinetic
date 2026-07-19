use log::info;

pub const DEFAULT_SAMPLE_RATE_HZ: u32 = 16_000;
pub const DEFAULT_CHANNELS: u8 = 1;
pub const MAX_VOLUME_PERCENT: u8 = 100;
pub const AUDIO_CONTROL_TOPIC: &str = "audio.control";
pub const AUDIO_STATUS_TOPIC: &str = "audio.status";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioRoute {
    LocalSpeaker,
    RemoteTalkback,
    TwoWayCall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpeakerConfig {
    pub sample_rate_hz: u32,
    pub channels: u8,
    pub volume_percent: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpeakerState {
    pub route: AudioRoute,
    pub config: SpeakerConfig,
    pub muted: bool,
}

#[derive(Clone, Debug)]
pub struct SpeakerDriver {
    state: SpeakerState,
}

impl SpeakerConfig {
    pub fn new(sample_rate_hz: u32, channels: u8, volume_percent: u8) -> Self {
        Self {
            sample_rate_hz,
            channels: channels.max(1),
            volume_percent: volume_percent.min(MAX_VOLUME_PERCENT),
        }
    }
}

impl Default for SpeakerConfig {
    fn default() -> Self {
        Self::new(DEFAULT_SAMPLE_RATE_HZ, DEFAULT_CHANNELS, 50)
    }
}

impl SpeakerDriver {
    pub fn new(config: SpeakerConfig) -> Self {
        info!(
            "Initializing speaker route with sample_rate_hz={} channels={} volume_percent={}",
            config.sample_rate_hz, config.channels, config.volume_percent
        );
        Self {
            state: SpeakerState {
                route: AudioRoute::LocalSpeaker,
                config,
                muted: false,
            },
        }
    }

    pub fn set_volume(&mut self, volume_percent: u8) -> SpeakerState {
        self.state.config.volume_percent = volume_percent.min(MAX_VOLUME_PERCENT);
        self.state
    }

    pub fn set_muted(&mut self, muted: bool) -> SpeakerState {
        self.state.muted = muted;
        self.state
    }

    pub fn route_audio(&mut self, route: AudioRoute) -> SpeakerState {
        self.state.route = route;
        self.state
    }

    pub fn state(&self) -> SpeakerState {
        self.state
    }
}

impl Default for SpeakerDriver {
    fn default() -> Self {
        Self::new(SpeakerConfig::default())
    }
}
