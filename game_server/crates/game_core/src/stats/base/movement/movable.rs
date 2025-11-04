use crate::{consts::FIXED_UPDATE_TICK_RATE, stats::*};
use serde::Deserialize;

#[derive(Clone, Component, Debug, Default, Deserialize, PartialEq, Reflect)]
pub struct Movable {
    move_type: MovementStat,
    prev_move_type: MovementStat,
    speed: MovementStats,
    steps: usize,
}

impl Movable {
    // 2.4 get from https://habr.com/ru/articles/814529/
    const STEPS_TO_RUN: usize = (FIXED_UPDATE_TICK_RATE / 2.4) as usize;

    pub fn new(move_speed: MovementStats) -> Self {
        Movable {
            move_type: MovementStat::default(),
            prev_move_type: MovementStat::default(),
            speed: move_speed,
            steps: Default::default(),
        }
    }

    pub fn movement_stats(&self) -> &MovementStats {
        &self.speed
    }

    pub fn movement_stats_mut(&mut self) -> &mut MovementStats {
        &mut self.speed
    }

    pub fn running(&self) -> bool {
        self.move_type == MovementStat::Run
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn step(&mut self) {
        self.steps += 1;
    }

    pub fn reset_steps(&mut self) {
        self.steps = Default::default();
    }

    pub fn move_type(&self) -> MovementStat {
        self.move_type
    }

    pub fn is_moving(&self) -> bool {
        self.steps > 0
    }

    pub fn is_running(&self) -> bool {
        self.move_type == MovementStat::Run
    }

    pub fn is_flying(&self) -> bool {
        self.move_state() == MoveState::Air
    }

    pub fn in_water(&self) -> bool {
        self.move_state() == MoveState::Water
    }

    pub fn exiting_water(&self) -> bool {
        let prev_move_mode = MoveState::from(self.prev_move_type);
        let current_move_mode = MoveState::from(self.move_type);
        prev_move_mode == MoveState::Water && current_move_mode == MoveState::Ground
    }

    pub fn move_state(&self) -> MoveState {
        MoveState::from(self.move_type)
    }

    pub fn set_move_type(&mut self, move_type: MovementStat) {
        self.prev_move_type = self.move_type;
        self.move_type = move_type;
    }

    pub fn speed_stat(&self, move_type: MovementStat) -> u32 {
        self.speed.get(move_type)
    }

    pub fn set_speed_stat(&mut self, move_type: MovementStat, speed: u32) {
        self.speed.insert(move_type, speed);
    }

    pub fn speed(&self) -> u32 {
        match self.move_type {
            MovementStat::Run => {
                if self.steps < Movable::STEPS_TO_RUN {
                    self.speed.get(MovementStat::Walk)
                } else {
                    self.speed.get(self.move_type)
                }
            }
            _ => self.speed.get(self.move_type),
        }
    }

    pub fn multiplier(&self, base: &MovementStats) -> f64 {
        let base_speed = match self.move_type {
            MovementStat::Run => base.get(MovementStat::Run),
            _ => base.get(MovementStat::Walk),
        };
        self.speed.get(self.move_type) as f64 * (1.0 / base_speed as f64)
    }
}

impl From<&BaseClassStats> for Movable {
    fn from(stats: &BaseClassStats) -> Self {
        Movable {
            move_type: MovementStat::default(),
            prev_move_type: MovementStat::default(),
            speed: stats.base_speed.clone(),
            steps: Default::default(),
        }
    }
}
