pub const RTSP_CONTROL_TOPIC: &str = "rtsp.control";
pub const RTSP_SCHEDULE_TOPIC: &str = "rtsp.schedule";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RtspControlAction {
    StartStream,
    StopStream,
    RestartStream,
    EnableTalkback,
    DisableTalkback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RtspControlCommand {
    pub camera_id: String,
    pub action: RtspControlAction,
    pub correlation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledRtspCommand {
    pub command: RtspControlCommand,
    pub execute_at_epoch_ms: u64,
}

impl RtspControlCommand {
    pub fn new(camera_id: &str, action: RtspControlAction, correlation_id: &str) -> Self {
        Self {
            camera_id: camera_id.to_string(),
            action,
            correlation_id: correlation_id.to_string(),
        }
    }

    pub fn kafka_key(&self) -> String {
        format!("{}:{}", self.camera_id, self.correlation_id)
    }
}

impl ScheduledRtspCommand {
    pub fn new(command: RtspControlCommand, execute_at_epoch_ms: u64) -> Self {
        Self {
            command,
            execute_at_epoch_ms,
        }
    }

    pub fn is_due(&self, now_epoch_ms: u64) -> bool {
        now_epoch_ms >= self.execute_at_epoch_ms
    }

    pub fn delay_ms(&self, now_epoch_ms: u64) -> u64 {
        self.execute_at_epoch_ms.saturating_sub(now_epoch_ms)
    }
}

pub fn due_commands(
    schedule: &[ScheduledRtspCommand],
    now_epoch_ms: u64,
) -> Vec<RtspControlCommand> {
    schedule
        .iter()
        .filter(|entry| entry.is_due(now_epoch_ms))
        .map(|entry| entry.command.clone())
        .collect()
}
