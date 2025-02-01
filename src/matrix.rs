use serde::{Deserialize, Serialize};
use std::fmt::Debug;


pub type ColVector<T, const ROWS: usize> = Matrix<T, ROWS, 1>;
// pub type RowVector<T, const COLS: usize> = Matrix<T, 1, COLS>;


#[derive(Clone)]
pub struct Matrix<T, const ROWS: usize, const COLS: usize>{
    data: [[T; COLS]; ROWS],
}
impl<T, const ROWS: usize, const COLS: usize> Matrix<T, ROWS, COLS>{
    
    
    pub fn get_unchecked(&self, row: usize, col: usize) -> &T {
        &self.data[row][col]
    }
    // pub fn get(&self, row: usize, col: usize) -> Option<&T> {
    //     self.data.get(row).and_then(|r| r.get(col))
    // }
    

    
    pub fn to_vecs(&self) -> Vec<Vec<T>> where T: Clone {
        self.data.iter().map(|r| r.to_vec()).collect()
    }
    pub fn from_vecs(vecs: Vec<Vec<T>>) -> Self
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
    
    // pub fn new_from_one_val(val: T) -> Self where T: Copy {
    //     Matrix {
    //         data: [[val; COLS]; ROWS]
    //     }
    // }
    
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
    
    
    
    
    pub fn mul<const RHSCOLS: usize>(&self, rhs: &Matrix<T, COLS, RHSCOLS>) -> Matrix<T, ROWS, RHSCOLS>
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
    
    pub fn add(&self, rhs: &Matrix<T, ROWS, COLS>) -> Matrix<T, ROWS, COLS>
        where T: Copy + std::ops::Add<T, Output = T>,
        <T as std::ops::Add>::Output: Debug,
        <T as std::ops::Add>::Output: Copy,
    {
        Matrix::new_from_generator(
            |i, j| self.data[i][j] + rhs.data[i][j]
        )
    }
}

impl<const ROWS: usize, const COLS: usize> Serialize for Matrix<f32, ROWS, COLS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
        S: serde::Serializer,
    {
        self.to_vecs().serialize(serializer)
    }
}

impl<'de, const ROWS: usize, const COLS: usize> Deserialize<'de> for Matrix<f32, ROWS, COLS> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
        D: serde::Deserializer<'de>,
    {

        let vecs = Vec::<Vec<f32>>::deserialize(deserializer)?;
        Ok(Matrix::from_vecs(vecs))
    }
}
