use crate::{ANodeInstance, CompiledNodeEvaluator, NodeEvaluation, RuntimeValue};

use super::support::{
    config_string, delta_seconds, randomized_period, require_inputs, set_state_values, state_values, trigger,
    trigger_fired,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MetronomeMode {
    Frequency,
    Bpm,
    Time,
}

impl MetronomeMode {
    pub(super) fn from_config(instance: &ANodeInstance) -> Self {
        match config_string(instance, "mode", "bpm").as_str() {
            "frequency" => Self::Frequency,
            "time" => Self::Time,
            _ => Self::Bpm,
        }
    }
}

#[derive(Debug)]
pub(super) struct MetronomeEval {
    pub(super) mode: MetronomeMode,
    pub(super) value: f64,
    pub(super) on_ratio: f64,
    pub(super) randomness: f64,
}

impl MetronomeEval {
    fn seed_for(&self, logical_tick: u64) -> i64 {
        self.value.to_bits() as i64 ^ logical_tick as i64
    }
}

impl CompiledNodeEvaluator for MetronomeEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [tap] = require_inputs::<1>(evaluation.inputs)?;
        let dt = delta_seconds(evaluation.ctx.delta_time);
        let state = evaluation.state.first_mut();
        let mut values = state_values(state.as_deref(), 5);
        values[1] += dt;
        if trigger_fired(tap)? {
            if values[2] > 0.0 {
                values[3] = (values[1] - values[2]).max(0.001);
                values[4] = values[3];
            }
            values[2] = values[1];
        }
        let base_period = match self.mode {
            MetronomeMode::Frequency => {
                if self.value.abs() <= f64::EPSILON {
                    1.0
                } else {
                    1.0 / self.value.abs()
                }
            }
            MetronomeMode::Bpm => {
                if self.value.abs() <= f64::EPSILON {
                    0.5
                } else {
                    60.0 / self.value.abs()
                }
            }
            MetronomeMode::Time => self.value.abs().max(0.001),
        };
        if values[4] <= 0.0 {
            values[4] = randomized_period(base_period, self.randomness, self.seed_for(evaluation.ctx.logical_tick));
        }
        let period = if values[3] > 0.0 { values[3] } else { values[4] }.max(0.001);
        values[0] += dt / period;
        let mut fired = false;
        if values[0] >= 1.0 {
            values[0] = values[0].fract();
            values[4] = randomized_period(
                base_period,
                self.randomness,
                self.seed_for(evaluation.ctx.logical_tick + 1),
            );
            fired = true;
        }
        let gate = values[0] < self.on_ratio;
        set_state_values(state, values);
        let edge_id = u64::from(evaluation.exec_node.index() as u32);
        Ok(vec![
            RuntimeValue::Trigger(trigger(fired, edge_id, evaluation.ctx.logical_tick)),
            RuntimeValue::Bool(gate),
        ])
    }
}
