use log::info;

pub const PIN_KEY: u8 = 2;
pub const KEY_VALUE_MAX: u8 = 4;
pub const DEBOUNCE_MS: u64 = 500;

#[derive(Clone, Debug, Default)]
pub struct KeyDriver {
    key_value: u8,
    last_event_ms: Option<u64>,
}

impl KeyDriver {
    pub fn new() -> Self {
        info!("Initializing key input on pin {}", PIN_KEY);
        Self::default()
    }

    pub fn register_falling_edge(&mut self, now_ms: u64) -> Option<u8> {
        if self
            .last_event_ms
            .is_some_and(|last_event_ms| now_ms.saturating_sub(last_event_ms) <= DEBOUNCE_MS)
        {
            return None;
        }

        self.last_event_ms = Some(now_ms);
        self.key_value = if self.key_value >= KEY_VALUE_MAX {
            0
        } else {
            self.key_value + 1
        };
        Some(self.key_value)
    }

    pub fn key_value(&self) -> u8 {
        self.key_value
    }
}
