// author: Rob Saunders <hello@robsaunders.io>

#[macro_use]
extern crate vst;

use vst::prelude::*;

use std::{f64::consts::PI, sync::Arc};


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
            0 => format!("{:.2}", self.get_parameter(index) * 5.0),
            1 => format!("{:.2}", self.get_parameter(index) * 2.0 - 1.0),
            2 => format!("{:.2}", self.get_parameter(index) * 5.0 + 1.0),
            _ => "".to_string(),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Gain",
            1 => "Bias",
            2 => "Curve",
            3 => "Feedback",
            _ => "",
        }.to_string()
    }
}

struct Distortion {
    params: Arc<Params>,
    pregain: f32,
    bias: f32,
    curve: f32,
}

pub const TAU: f64 = PI * 2.0;

impl Plugin for Distortion {
    fn new(_host: HostCallback) -> Self {
        let params = Arc::new(Params::default());
        params.transfer.set_parameter(0, 0.2);
        params.transfer.set_parameter(1, 0.5);


        Distortion {
            params,
            pregain: 1.0,
            bias: 0.0,
            curve: 1.0,
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
                0 => self.pregain = value * 5.0,
                1 => self.bias = value * 2.0,
                2 => self.curve = value * 5.0 + 1.0,
                _ => unimplemented!(),
            }
        }

        fn transfer(val: f32, bias: f32, curve: f32) -> f32 {
            (val + bias).signum() * (1.0 - (1.0 / ((val + bias).abs() + 1.0).powf(curve)))
        }

        for (input_buffer, output_buffer) in buffer.zip() {
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                *output_sample = transfer(*input_sample * self.pregain, self.bias, self.curve);
            }
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

plugin_main!(Distortion);
