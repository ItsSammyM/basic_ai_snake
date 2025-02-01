use serde::{Deserialize, Serialize};

use crate::matrix::{ColVector, Matrix};

fn sigmoid(x: f32) -> f32 {
    if x > 0.0 {
        x
    } else {
        x / 100.0
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Network {
    first: Layer<12, 12>,
    second: Layer<12, 4>,
}
const STARTING_VALUE: f32 = 6.0;
const LEARNING_RATE: f32 = 0.5;
pub type NetInput = ColVector<f32, 12>;
pub type NetOutput = ColVector<f32, 4>;


impl Network{
    pub fn new(rng: &mut impl rand::Rng) -> Network{
        Network{
            first: Layer::new(rng),
            second: Layer::new(rng)
        }
    }
    fn forward(&self, input: NetInput) -> NetOutput {
        let out = self.first.forward(input);
        let out = self.second.forward(out);
        out
    }
    pub fn choice_with_highest_confidence(&self, input: NetInput)->usize{
        let output = self.forward(input);
        let mut highest_confidence = -std::f32::INFINITY;
        let mut highest_confidence_index = 0;
        for i in 0..4{
            let confidence = *output.get_unchecked(i, 0);
            if confidence > highest_confidence{
                highest_confidence = confidence;
                highest_confidence_index = i;
            }
        }
        highest_confidence_index
    }
    pub fn randomly_edit(&mut self, rng: &mut impl rand::Rng) {
        self.first.randomly_edit(rng);
        self.first.randomly_edit(rng);
    }
}



#[derive(Clone, Serialize, Deserialize)]
struct Layer<const IN: usize, const OUT: usize> {
    matrix: Matrix<f32, OUT, IN>,
    bias: ColVector<f32, OUT>,
}
impl<const IN: usize, const OUT: usize> Layer<IN, OUT> {
    fn new(rng: &mut impl rand::Rng) -> Self {
        Layer {
            matrix: Matrix::new_from_generator(
                |_, _| rng.gen_range(-STARTING_VALUE..STARTING_VALUE)
            ),
            bias: ColVector::new_from_generator(
                |_, _| rng.gen_range(-STARTING_VALUE..STARTING_VALUE)
            )
        }
    }
    fn forward(&self, input: ColVector<f32, IN>) -> ColVector<f32, OUT> {
        let mut output = self.matrix.mul(&input);
        output = output.add(&self.bias);
        output = ColVector::new_from_generator(|i, _| sigmoid(*output.get_unchecked(i, 0)));
        output
    }
    fn randomly_edit(&mut self, rng: &mut impl rand::Rng) {
        self.matrix = Matrix::new_from_generator(
            |_, _| {
                let mut val = *self.matrix.get_unchecked(0, 0);
                val += rng.gen_range(-LEARNING_RATE..LEARNING_RATE);
                val
            }
        );
        self.bias = ColVector::new_from_generator(
            |_, _| {
                let mut val = *self.bias.get_unchecked(0, 0);
                val += rng.gen_range(-LEARNING_RATE..LEARNING_RATE);
                val
            }
        );
    }
}

