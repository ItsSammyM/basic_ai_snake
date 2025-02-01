use serde::{Deserialize, Serialize};
use std::fmt::Debug;


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
    second: Layer<12, 12>,
    third: Layer<12, 4>,
}
const STARTING_VALUE: f32 = 6.0;
const LEARNING_RATE: f32 = 0.5;
pub type NetInput = ColVector<f32, 12>;
pub type NetOutput = ColVector<f32, 4>;
impl Network{
    pub fn new(rng: &mut impl rand::Rng) -> Network{
        Network{
            first: Layer::new(rng),
            second: Layer::new(rng),
            third: Layer::new(rng),
        }
    }
    fn forward(&self, input: NetInput) -> NetOutput {
        self.third.forward(
            self.second.forward(
                self.first.forward(input)
            )
        )
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

#[derive(Clone)]
pub struct Matrix<T, const ROWS: usize, const COLS: usize>{
    data: [[T; COLS]; ROWS],
}
impl<T, const ROWS: usize, const COLS: usize> Matrix<T, ROWS, COLS>{
    fn get_unchecked(&self, row: usize, col: usize) -> &T {
        &self.data[row][col]
    }
    // fn get(&self, row: usize, col: usize) -> Option<&T> {
    //     self.data.get(row).and_then(|r| r.get(col))
    // }
    // fn new_from_one_val(val: T) -> Self where T: Copy {
    //     Matrix {
    //         data: [[val; COLS]; ROWS]
    //     }
    // }
    fn to_vecs(&self) -> Vec<Vec<T>> where T: Clone {
        self.data.iter().map(|r| r.to_vec()).collect()
    }
    fn from_vecs(vecs: Vec<Vec<T>>) -> Self
        where
        T: Copy,
        [T; COLS]: TryFrom<Vec<T>>,
        [[T; COLS]; ROWS]: TryFrom<Vec<[T; COLS]>>,
        <[T; COLS] as TryFrom<Vec<T>>>::Error: Debug,
        <[[T; COLS]; ROWS] as TryFrom<Vec<[T; COLS]>>>::Error: Debug,
    {
        Matrix {
            data: vecs.iter().map(|r| r.as_slice().try_into().unwrap()).collect::<Vec<[T; COLS]>>().try_into().unwrap()
        }
    }
    pub fn new_from_slice(data: [[T; COLS]; ROWS]) -> Self {
        Self{data}
    }
    /// (ROW, COL)
    pub fn new_from_generator(mut generator: impl FnMut(usize, usize)->T) -> Self
        where
        T: Copy + Debug,
        [T; COLS]: TryFrom<Vec<T>>,
        [[T; COLS]; ROWS]: TryFrom<Vec<[T; COLS]>>,
        <[T; COLS] as TryFrom<Vec<T>>>::Error: Debug,
        <[[T; COLS]; ROWS] as TryFrom<Vec<[T; COLS]>>>::Error: Debug,
    {
        Matrix {
            data:
            (0..ROWS)
                .map(|i|
                    (0..COLS)
                        .map(|j| generator(i, j))
                        .collect::<Vec<T>>()
                        .try_into()
                        .unwrap()
                )
                .collect::<Vec<[T; COLS]>>()
                .try_into()
                .unwrap()
        }
    }
    fn mul<const RHSCOLS: usize>(&self, rhs: &Matrix<T, COLS, RHSCOLS>) -> Matrix<T, ROWS, RHSCOLS>
        where T: Copy + Debug + std::ops::Mul<T, Output = T> + std::ops::AddAssign<<T as std::ops::Mul>::Output>,
    {
        Matrix::new_from_generator(
            |i, j| {
                let mut sum: T = self.data[i][0] * rhs.data[0][j];
                for k in 0..COLS {
                    sum += self.data[i][k] * rhs.data[k][j];
                }
                sum
            }
        )
    }
    fn add(&self, rhs: &Matrix<T, ROWS, COLS>) -> Matrix<T, ROWS, COLS>
        where T: Copy + std::ops::Add<T, Output = T>,
        <T as std::ops::Add>::Output: Debug,
        <T as std::ops::Add>::Output: Copy,
    {
        Matrix::new_from_generator(
            |i, j| self.data[i][j] + rhs.data[i][j]
        )
    }
}
pub type ColVector<T, const ROWS: usize> = Matrix<T, ROWS, 1>;
//type RowVector<T, const COLS: usize> = Matrix<T, 1, COLS>;

impl<const ROWS: usize, const COLS: usize> Serialize for Matrix<f32, ROWS, COLS>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
        S: serde::Serializer,
    {
        self.to_vecs().serialize(serializer)
    }
}
impl<'de, const ROWS: usize, const COLS: usize> Deserialize<'de> for Matrix<f32, ROWS, COLS>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
        D: serde::Deserializer<'de>,
    {

        let vecs = Vec::<Vec<f32>>::deserialize(deserializer)?;
        Ok(Matrix::from_vecs(vecs))
    }
}