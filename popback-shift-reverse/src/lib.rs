// author: Rob Saunders <hello@robsaunders.io>

#[macro_use]
extern crate vst;

use vst::prelude::*;

use std::{f64::consts::PI, sync::Arc};

const MAX_DELAY: f32 = 1000.0;
const NUM_PARAMS: usize = 4;

struct Params {
    transfer: ParameterTransfer,
}

impl Default for Params {
    fn default() -> Params {
        Params {
            transfer: ParameterTransfer::new(NUM_PARAMS),
        }
    }
}

impl PluginParameters for Params {
    fn set_parameter(&self, index: i32, value: f32) {
        self.transfer.set_parameter(index as usize, value);
    }

    fn get_parameter(&self, index: i32) -> f32 {
        self.transfer.get_parameter(index as usize)
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", self.get_parameter(index) * MAX_DELAY),
            1 => format!("{:.2}", self.get_parameter(index) * 2.0 - 1.0),
            2 => format!("{:.2}", self.get_parameter(index) * 2.0 - 1.0),
            3 => format!("{:.2}", self.get_parameter(index) * MAX_DELAY),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Record period",
            1 => "Dry gain",
            2 => "Wet gain",
            _ => "",
        }.to_string()
    }
}

struct PopbackShift {
    swap_buffers: (Vec<f32>, Vec<f32>),
    idx: usize,
    params: Arc<Params>,
    dry_gain: f32,
    wet_gain: f32,
    fade_ms: f32,
}

pub const TAU: f64 = PI * 2.0;

impl Plugin for PopbackShift {
    fn new(_host: HostCallback) -> Self {
        let params = Arc::new(Params::default());
        params.transfer.set_parameter(0, 0.5);
        params.transfer.set_parameter(1, 1.0);
        params.transfer.set_parameter(2, 0.7);
        params.transfer.set_parameter(3, 0.2);

        PopbackShift {
            params: params.clone(),
            swap_buffers: (
                vec![0.0; (1.0 * MAX_DELAY * 44100.0 / 1000.0) as usize],
                vec![0.0; (1.0 * MAX_DELAY * 44100.0 / 1000.0) as usize]
            ),
            idx: 0,
            dry_gain: 0.5,
            wet_gain: 0.7,
            fade_ms: 0.0,
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "PopbackShift".to_string(),
            vendor: "SOUNDBOKS".to_string(),
            unique_id: 12502,
            category: Category::Effect,
            inputs: 1,
            outputs: 1,
            parameters: NUM_PARAMS as i32,
            initial_delay: 0,
            ..Info::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        for (p, value) in self.params.transfer.iterate(true) {
            match p {
                0 => {
                    self.swap_buffers.0.resize((value * MAX_DELAY * 44100.0 / 1000.0) as usize, 0.0);
                    self.swap_buffers.1.resize((value * MAX_DELAY * 44100.0 / 1000.0) as usize, 0.0);
                    self.idx = 0;
                },
                1 => self.dry_gain = value * 2.0 - 1.0,
                2 => self.wet_gain = value * 2.0 - 1.0,
                3 => self.fade_ms = value * MAX_DELAY,
                _ => unimplemented!(),
            }
        }

        for (input_buffer, output_buffer) in buffer.zip() {
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                if self.idx >= self.swap_buffers.0.len() {
                    std::mem::swap(&mut self.swap_buffers.0, &mut self.swap_buffers.1);
                    self.idx = 0;
                }

                let (record, playback) = &mut self.swap_buffers;
                record[self.idx] = *input_sample;
                
                let wet = playback[playback.len() - (1 + self.idx)] * f32::min(
                    (self.idx as f32 / 44100.0 * self.fade_ms).max(1.0), // fade in
                    ((playback.len() - self.idx) as f32 / 44100.0 * self.fade_ms).max(1.0) // fade out
                );

                *output_sample = wet * self.wet_gain + *input_sample * self.dry_gain;
                self.idx += 1;
            }
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

plugin_main!(PopbackShift);
