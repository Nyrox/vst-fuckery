// author: Rob Saunders <hello@robsaunders.io>

#[macro_use]
extern crate vst;

use vst::prelude::*;

use std::{f64::consts::PI, sync::Arc};

const MAX_DELAY: f32 = 500.0;
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
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Shift Time",
            1 => "Dry gain",
            2 => "Wet gain",
            3 => "Feedback",
            _ => "",
        }.to_string()
    }
}

struct PopbackShift {
    popback: Vec<f32>,
    idx: usize,
    params: Arc<Params>,
    dry_gain: f32,
    wet_gain: f32,
    feedback: f32,
}

pub const TAU: f64 = PI * 2.0;

impl Plugin for PopbackShift {
    fn new(_host: HostCallback) -> Self {
        let params = Arc::new(Params::default());
        params.transfer.set_parameter(0, 0.1);
        params.transfer.set_parameter(1, 1.0);
        params.transfer.set_parameter(2, 0.7);

        PopbackShift {
            params: params.clone(),
            popback: vec![0.0; (1.0 * MAX_DELAY * 44100.0 / 1000.0) as usize],
            idx: 0,
            dry_gain: 1.0,
            wet_gain: 0.0,
            feedback: 0.0,
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
                    self.popback.resize((value * MAX_DELAY * 44100.0 / 1000.0) as usize, 0.0);
                    self.idx = 0;
                },
                1 => self.dry_gain = value * 2.0 - 1.0,
                2 => self.wet_gain = value * 2.0 - 1.0,
                3 => self.feedback = value,
                _ => unimplemented!(),
            }
        }

        for (input_buffer, output_buffer) in buffer.zip() {
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                let popback_sample = if self.popback.len() == 0 {
                    *input_sample
                } else {
                    let _p = self.popback[self.idx];
                    self.popback[self.idx] = *input_sample + _p * self.feedback;
                    self.idx = self.idx + 1;
                    if self.idx >= self.popback.len() {
                        self.idx = 0;
                    }
                    _p
                };

                *output_sample = *input_sample * self.dry_gain
                    + popback_sample * self.wet_gain;
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
