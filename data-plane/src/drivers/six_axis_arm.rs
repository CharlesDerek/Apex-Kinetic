use log::info;

pub const AXIS_COUNT: usize = 6;
pub const ARM_CONTROL_TOPIC: &str = "arm.control";
pub const ARM_STATUS_TOPIC: &str = "arm.status";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArmAxis {
    Base,
    Shoulder,
    Elbow,
    WristPitch,
    WristRoll,
    Gripper,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JointLimit {
    pub min_degrees: i16,
    pub max_degrees: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JointCommand {
    pub axis: ArmAxis,
    pub target_degrees: i16,
    pub speed_percent: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArmPose {
    pub joints: [JointCommand; AXIS_COUNT],
}

#[derive(Clone, Debug)]
pub struct SixAxisArmDriver {
    limits: [JointLimit; AXIS_COUNT],
}

impl ArmAxis {
    pub fn index(self) -> usize {
        match self {
            ArmAxis::Base => 0,
            ArmAxis::Shoulder => 1,
            ArmAxis::Elbow => 2,
            ArmAxis::WristPitch => 3,
            ArmAxis::WristRoll => 4,
            ArmAxis::Gripper => 5,
        }
    }
}

impl SixAxisArmDriver {
    pub fn new(limits: [JointLimit; AXIS_COUNT]) -> Self {
        info!("Initializing six-axis arm command model");
        Self { limits }
    }

    pub fn command(&self, axis: ArmAxis, target_degrees: i16, speed_percent: u8) -> JointCommand {
        let limit = self.limits[axis.index()];
        JointCommand {
            axis,
            target_degrees: target_degrees.clamp(limit.min_degrees, limit.max_degrees),
            speed_percent: speed_percent.min(100),
        }
    }

    pub fn neutral_pose(&self, speed_percent: u8) -> ArmPose {
        ArmPose {
            joints: [
                self.command(ArmAxis::Base, 0, speed_percent),
                self.command(ArmAxis::Shoulder, 0, speed_percent),
                self.command(ArmAxis::Elbow, 0, speed_percent),
                self.command(ArmAxis::WristPitch, 0, speed_percent),
                self.command(ArmAxis::WristRoll, 0, speed_percent),
                self.command(ArmAxis::Gripper, 0, speed_percent),
            ],
        }
    }
}

impl Default for SixAxisArmDriver {
    fn default() -> Self {
        Self::new([
            JointLimit {
                min_degrees: -180,
                max_degrees: 180,
            },
            JointLimit {
                min_degrees: -90,
                max_degrees: 120,
            },
            JointLimit {
                min_degrees: -135,
                max_degrees: 135,
            },
            JointLimit {
                min_degrees: -90,
                max_degrees: 90,
            },
            JointLimit {
                min_degrees: -180,
                max_degrees: 180,
            },
            JointLimit {
                min_degrees: 0,
                max_degrees: 90,
            },
        ])
    }
}
