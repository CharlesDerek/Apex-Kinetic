//! Authority and watchdog model for a future physical motor adapter.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopReason {
    NoCommand,
    StaleCommand,
    LostController,
    LowPower,
    Obstacle,
    StaleGeneration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MotorDecision {
    pub left: i8,
    pub right: i8,
    pub stop_reason: Option<StopReason>,
}

pub struct SafetyController {
    generation: u64,
    command: Option<(i8, i8, u64)>,
    max_age_ms: u64,
    min_voltage_mv: u32,
    min_distance_mm: u32,
}

impl SafetyController {
    pub fn new(max_age_ms: u64, min_voltage_mv: u32, min_distance_mm: u32) -> Self {
        Self {
            generation: 0,
            command: None,
            max_age_ms: max_age_ms.max(1),
            min_voltage_mv,
            min_distance_mm,
        }
    }

    pub fn advance_generation(&mut self, generation: u64) -> bool {
        if generation <= self.generation {
            return false;
        }
        self.generation = generation;
        self.command = None;
        true
    }

    pub fn command(
        &mut self,
        generation: u64,
        left: i8,
        right: i8,
        observed_ms: u64,
    ) -> Result<(), StopReason> {
        if generation != self.generation || generation == 0 {
            return Err(StopReason::StaleGeneration);
        }
        self.command = Some((left, right, observed_ms));
        Ok(())
    }

    pub fn decide(
        &self,
        now_ms: u64,
        controller_connected: bool,
        voltage_mv: Option<u32>,
        distance_mm: Option<u32>,
    ) -> MotorDecision {
        let stop = |reason| MotorDecision {
            left: 0,
            right: 0,
            stop_reason: Some(reason),
        };
        if !controller_connected {
            return stop(StopReason::LostController);
        }
        if voltage_mv.is_none_or(|voltage| voltage < self.min_voltage_mv) {
            return stop(StopReason::LowPower);
        }
        if distance_mm.is_none_or(|distance| distance < self.min_distance_mm) {
            return stop(StopReason::Obstacle);
        }
        let Some((left, right, observed_ms)) = self.command else {
            return stop(StopReason::NoCommand);
        };
        if observed_ms > now_ms || now_ms - observed_ms > self.max_age_ms {
            return stop(StopReason::StaleCommand);
        }
        MotorDecision {
            left,
            right,
            stop_reason: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fences_stale_authority_and_stops_on_failed_dependencies() {
        let mut controller = SafetyController::new(500, 11000, 300);
        assert_eq!(
            controller
                .decide(100, true, Some(12000), Some(500))
                .stop_reason,
            Some(StopReason::NoCommand)
        );
        assert!(controller.advance_generation(2));
        assert_eq!(
            controller.command(1, 80, 80, 100),
            Err(StopReason::StaleGeneration)
        );
        controller.command(2, 80, 80, 100).unwrap();
        assert_eq!(
            controller.decide(200, true, Some(12000), Some(500)).left,
            80
        );
        assert_eq!(
            controller
                .decide(601, true, Some(12000), Some(500))
                .stop_reason,
            Some(StopReason::StaleCommand)
        );
        assert_eq!(
            controller
                .decide(200, false, Some(12000), Some(500))
                .stop_reason,
            Some(StopReason::LostController)
        );
        assert_eq!(
            controller
                .decide(200, true, Some(10000), Some(500))
                .stop_reason,
            Some(StopReason::LowPower)
        );
        assert_eq!(
            controller
                .decide(200, true, Some(12000), Some(100))
                .stop_reason,
            Some(StopReason::Obstacle)
        );
        assert!(controller.advance_generation(3));
        assert_eq!(
            controller
                .decide(200, true, Some(12000), Some(500))
                .stop_reason,
            Some(StopReason::NoCommand)
        );
        assert!(!controller.advance_generation(2));
    }
}
