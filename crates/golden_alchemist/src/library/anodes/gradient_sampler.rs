use std::cmp::Ordering;

use crate::{ColorValue, CompiledNodeEvaluator, NodeEvaluation, RuntimeValue};

use super::support::{float_inputs, lerp};

#[derive(Clone, Debug)]
pub(super) struct GradientStop {
    position: f64,
    color: ColorValue,
}

#[derive(Debug)]
pub(super) struct GradientSamplerEval {
    pub(super) stops: Vec<GradientStop>,
}

impl CompiledNodeEvaluator for GradientSamplerEval {
    fn evaluate(&self, evaluation: &mut NodeEvaluation<'_, '_>) -> Result<Vec<RuntimeValue>, String> {
        let [position] = float_inputs::<1>(evaluation.inputs)?;
        Ok(vec![RuntimeValue::Color(sample_gradient(
            &self.stops,
            position.clamp(0.0, 1.0),
        ))])
    }
}

pub(super) fn parse_gradient(definition: &str) -> Vec<GradientStop> {
    let parsed = definition
        .split(',')
        .filter_map(parse_gradient_stop)
        .collect::<Vec<_>>();
    if parsed.is_empty() {
        return vec![
            GradientStop {
                position: 0.0,
                color: ColorValue::BLACK,
            },
            GradientStop {
                position: 1.0,
                color: ColorValue {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            },
        ];
    }
    let last_index = parsed.len().saturating_sub(1).max(1);
    let mut stops = parsed
        .into_iter()
        .enumerate()
        .map(|(index, (color, position))| GradientStop {
            position: position.unwrap_or(index as f64 / last_index as f64).clamp(0.0, 1.0),
            color,
        })
        .collect::<Vec<_>>();
    stops.sort_by(|left, right| left.position.partial_cmp(&right.position).unwrap_or(Ordering::Equal));
    stops
}

fn parse_gradient_stop(part: &str) -> Option<(ColorValue, Option<f64>)> {
    let mut tokens = part.split_whitespace();
    let color = parse_hex_color(tokens.next()?)?;
    let position = tokens.next().and_then(|token| token.parse::<f64>().ok());
    Some((color, position))
}

fn parse_hex_color(token: &str) -> Option<ColorValue> {
    let value = token.trim().strip_prefix('#')?;
    let expanded;
    let hex = match value.len() {
        3 | 4 => {
            expanded = value
                .chars()
                .flat_map(|character| [character, character])
                .collect::<String>();
            expanded.as_str()
        }
        6 | 8 => value,
        _ => return None,
    };
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    let alpha = if hex.len() >= 8 {
        u8::from_str_radix(&hex[6..8], 16).ok()?
    } else {
        u8::MAX
    };
    Some(ColorValue {
        red: f32::from(red) / 255.0,
        green: f32::from(green) / 255.0,
        blue: f32::from(blue) / 255.0,
        alpha: f32::from(alpha) / 255.0,
    })
}

fn sample_gradient(stops: &[GradientStop], position: f64) -> ColorValue {
    let Some(first) = stops.first() else {
        return ColorValue::BLACK;
    };
    if position <= first.position {
        return first.color;
    }
    for pair in stops.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        if position > right.position {
            continue;
        }
        let width = (right.position - left.position).max(f64::EPSILON);
        let amount = ((position - left.position) / width).clamp(0.0, 1.0);
        return ColorValue {
            red: lerp(f64::from(left.color.red), f64::from(right.color.red), amount) as f32,
            green: lerp(f64::from(left.color.green), f64::from(right.color.green), amount) as f32,
            blue: lerp(f64::from(left.color.blue), f64::from(right.color.blue), amount) as f32,
            alpha: lerp(f64::from(left.color.alpha), f64::from(right.color.alpha), amount) as f32,
        };
    }
    stops.last().map_or(ColorValue::BLACK, |stop| stop.color)
}
