use super::{Dataset, GetItem};
use crate::collate::Collate;
use crate::sampler::HasLength;
use ndarray::{Array, Axis, Dimension, RemoveAxis};

/// Basic dataset than can contains 2 `ndarray` of any dimension
#[derive(Debug, PartialEq, Hash, Eq)]
pub struct NdarrayDataset<A1, A2, D1, D2>
where
    A1: Clone,
    A2: Clone,
    D1: Dimension + RemoveAxis,
    D2: Dimension + RemoveAxis,
{
    /// The content of the dataset
    pub ndarrays: (Array<A1, D1>, Array<A2, D2>),
}
impl<A1, A2, D1, D2, T> Dataset<T> for NdarrayDataset<A1, A2, D1, D2>
where
    A1: Clone,
    A2: Clone,
    D1: Dimension + RemoveAxis,
    D2: Dimension + RemoveAxis,
    T: Collate<Vec<Self::Output>>,
{
}

impl<A1, A2, D1, D2> Clone for NdarrayDataset<A1, A2, D1, D2>
where
    A1: Clone,
    A2: Clone,
    D1: Dimension + RemoveAxis,
    D2: Dimension + RemoveAxis,
{
    fn clone(&self) -> Self {
        NdarrayDataset {
            ndarrays: self.ndarrays.clone(),
        }
    }
}

impl<A1, A2, D1, D2> HasLength for NdarrayDataset<A1, A2, D1, D2>
where
    A1: Clone,
    A2: Clone,
    D1: Dimension + RemoveAxis,
    D2: Dimension + RemoveAxis,
{
    fn len(&self) -> usize {
        self.ndarrays.0.shape()[0]
    }
}
impl<A1, A2, D1, D2> GetItem for NdarrayDataset<A1, A2, D1, D2>
where
    A1: Clone,
    A2: Clone,
    D1: Dimension + RemoveAxis,
    D2: Dimension + RemoveAxis,
{
    type Output = (
        Array<A1, <D1 as Dimension>::Smaller>,
        Array<A2, <D2 as Dimension>::Smaller>,
    );
    fn get_item(&self, index: usize) -> Self::Output {
        (
            self.ndarrays.0.index_axis(Axis(0), index).into_owned(),
            self.ndarrays.1.index_axis(Axis(0), index).into_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{arr0, array};
    #[test]
    fn ndarray_dataset() {
        let dataset = NdarrayDataset {
            ndarrays: (array![1, 2], array![3, 4]),
        };
        assert_eq!(dataset.get_item(0), (arr0(1), arr0(3)));
        assert_eq!(dataset.get_item(1), (arr0(2), arr0(4)));
    }
}
