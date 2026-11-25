//! Catalog wiring for latched controls on dedicated ADC channels.

use crate::{LadderWire, PhysicalControls};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    all(target_arch = "wasm32", not(target_os = "emscripten")),
    derive(tsify::Tsify)
)]
#[serde(deny_unknown_fields)]
pub struct AdcPositionDial {
    pub control_id: String,
    pub channel: u8,
    pub positions: Vec<PositionBand>,
    pub basis: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    all(target_arch = "wasm32", not(target_os = "emscripten")),
    derive(tsify::Tsify)
)]
#[serde(deny_unknown_fields)]
pub struct PositionBand {
    pub id: String,
    pub band_low: u16,
    pub band_high: u16,
}

pub(crate) fn validate(body: &PhysicalControls) -> Result<(), String> {
    let mut channels = BTreeSet::new();
    let mut controls = BTreeSet::new();
    for dial in &body.adc_position_dials {
        let id = &dial.control_id;
        let control = body
            .controls
            .get(id)
            .ok_or_else(|| format!("unknown ADC control {id}"))?;
        if !controls.insert(id)
            || !channels.insert(dial.channel)
            || body.ladders.iter().any(|ladder| {
                ladder.wire
                    == (LadderWire::Adc {
                        channel: dial.channel,
                    })
            })
        {
            return Err(format!(
                "ADC position control {id} repeats a control or shares a channel"
            ));
        }
        let mut positions = BTreeSet::new();
        for (index, band) in dial.positions.iter().enumerate() {
            if band.band_low > band.band_high
                || body
                    .analog_levels
                    .is_none_or(|levels| band.band_high >= levels)
                || !positions.insert(&band.id)
                || dial.positions[..index].iter().any(|other| {
                    band.band_low <= other.band_high && other.band_low <= band.band_high
                })
            {
                return Err(format!("invalid ADC position band for {id}/{}", band.id));
            }
        }
        let declared = control
            .positions
            .iter()
            .map(|position| &position.id)
            .collect();
        if control.mechanical
            || positions.is_empty()
            || !positions.is_subset(&declared)
            || dial.basis.trim().is_empty()
        {
            return Err(format!(
                "ADC position control {id} requires evidenced bands for documented positions"
            ));
        }
    }
    Ok(())
}
