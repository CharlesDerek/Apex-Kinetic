use apex_kinetic_vision_node::rtsp_control::{
    due_commands, RtspControlAction, RtspControlCommand, ScheduledRtspCommand, RTSP_CONTROL_TOPIC,
    RTSP_SCHEDULE_TOPIC,
};

#[test]
fn rtsp_control_command_builds_stable_kafka_key() {
    let command =
        RtspControlCommand::new("front-door", RtspControlAction::RestartStream, "ticket-42");

    assert_eq!(RTSP_CONTROL_TOPIC, "rtsp.control");
    assert_eq!(RTSP_SCHEDULE_TOPIC, "rtsp.schedule");
    assert_eq!(command.kafka_key(), "front-door:ticket-42");
}

#[test]
fn scheduled_rtsp_commands_report_due_state_and_delay() {
    let command = RtspControlCommand::new("garage", RtspControlAction::StartStream, "wake");
    let scheduled = ScheduledRtspCommand::new(command.clone(), 10_000);

    assert!(!scheduled.is_due(9_999));
    assert_eq!(scheduled.delay_ms(9_500), 500);
    assert_eq!(scheduled.delay_ms(10_500), 0);
    assert!(scheduled.is_due(10_000));
}

#[test]
fn due_command_filter_only_returns_ready_controls() {
    let schedule = vec![
        ScheduledRtspCommand::new(
            RtspControlCommand::new("front", RtspControlAction::StartStream, "a"),
            1_000,
        ),
        ScheduledRtspCommand::new(
            RtspControlCommand::new("rear", RtspControlAction::StopStream, "b"),
            2_000,
        ),
    ];

    assert_eq!(
        due_commands(&schedule, 1_500),
        vec![RtspControlCommand::new(
            "front",
            RtspControlAction::StartStream,
            "a"
        )]
    );
}
