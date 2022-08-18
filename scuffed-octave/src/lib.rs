// author: Rob Saunders <hello@robsaunders.io>

#[macro_use]
extern crate vst;

use vst::prelude::*;

use std::{f64::consts::PI, sync::Arc};

const NUM_PARAMS: usize = 0;

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
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            _ => "",
        }.to_string()
    }
}

struct PopbackShift {
    params: Arc<Params>,
}

pub const TAU: f64 = PI * 2.0;

impl Plugin for PopbackShift {
    fn new(_host: HostCallback) -> Self {
        let params = Arc::new(Params::default());

        PopbackShift {
            params: params.clone(),
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
                _ => unimplemented!(),
            }
        }

        for (input_buffer, output_buffer) in buffer.zip() {
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                *output_sample = input_sample.abs()
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
