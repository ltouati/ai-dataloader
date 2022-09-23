use std::marker::PhantomData;

use crate::{
    collate::{Collate, DefaultCollate},
    dataloader::builder::DataLoaderBuilder,
    fetch::{Fetcher, MapDatasetFetcher},
    sampler::{BatchIterator, BatchSampler},
    sampler::{Sampler, SequentialSampler},
    Dataset, Len,
};

pub mod builder;

// TODO: Generic over batch or over sampler?

// The collate function could have been a `Fn(Vec<D::Sample>) -> T` or a `fn(Vec<D::Sample>) -> T`, it would have allowed
// to pass directly closure or function to construct a `Dataloader`.
// The main drawback is that you can't (as i'm aware of) pass a default value
// to a generic Fn (or to the return type of fn), which made the API less ergonomic. Indeed, in that case, you have to precise
// the collate function to construct the dataloader each time, and in most our use cases we just just want the default one.

/// Data loader. Combines a dataset and a sampler, and provides an iterable over the given dataset.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct DataLoader<D, S = SequentialSampler, C = DefaultCollate>
where
    D: Dataset,
    S: Sampler,
    C: Collate<D::Sample>,
{
    /// Dataset from which to load the data.
    dataset: D,
    /// Return a batch of indices at a time.
    batch_sampler: BatchSampler<S>,
    /// Just here because collate has no data.
    phantom: PhantomData<C>,
}

impl<D> DataLoader<D, SequentialSampler, DefaultCollate>
where
    D: Dataset,
    DefaultCollate: Collate<D::Sample>,
{
    pub fn builder(dataset: D) -> DataLoaderBuilder<D, SequentialSampler, DefaultCollate> {
        DataLoaderBuilder::new(dataset)
    }
}

impl<D, S> DataLoader<D, S, DefaultCollate>
where
    D: Dataset,
    DefaultCollate: Collate<D::Sample>,
    S: Sampler,
{
    pub fn with_sampler(dataset: D, sampler: S, batch_size: usize) -> Self {
        Self::new_complete(dataset, sampler, DefaultCollate, batch_size)
    }
}

impl<D, S, C> DataLoader<D, S, C>
where
    D: Dataset,
    S: Sampler,
    C: Collate<D::Sample>,
{
    pub fn new_complete(
        dataset: D,
        sampler: S,
        _collate: C,
        batch_size: usize,
    ) -> DataLoader<D, S, C> {
        DataLoader {
            dataset,
            batch_sampler: BatchSampler {
                sampler,
                batch_size,
                drop_last: false,
            },
            phantom: PhantomData,
        }
    }
    // todo return self?
    pub fn drop_last(mut self, activate: bool) -> Self {
        self.batch_sampler.drop_last = activate;
        self
    }
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_sampler.batch_size = batch_size;
        self
    }

    pub fn with_collate_fn(mut self, _collate_fn: C) -> Self {
        self.phantom = PhantomData;
        self
    }

    /// Return not owning iterator over the dataloader.
    pub fn iter(&self) -> SingleProcessDataLoaderIter<D, S, C> {
        SingleProcessDataLoaderIter::new(self)
    }
}

impl<D, S, C> Len for DataLoader<D, S, C>
where
    D: Dataset,
    S: Sampler,
    C: Collate<D::Sample>,
{
    /// Return the number of batch that contain the dataloader.
    fn len(&self) -> usize {
        self.batch_sampler.len()
    }
}

/// Iterate over the dataloader with a single thread.
pub struct SingleProcessDataLoaderIter<'dataset, D, S = SequentialSampler, C = DefaultCollate>
where
    D: Dataset,
    S: Sampler,
    C: Collate<D::Sample>,
{
    sampler_iter: BatchIterator<S::IntoIter>,
    num_yielded: u64,
    data_fetcher: MapDatasetFetcher<'dataset, D, C>,
}

impl<'dataset, D, S, C> SingleProcessDataLoaderIter<'dataset, D, S, C>
where
    D: Dataset,
    S: Sampler,
    C: Collate<D::Sample>,
{
    fn new(loader: &DataLoader<D, S, C>) -> SingleProcessDataLoaderIter<D, S, C> {
        SingleProcessDataLoaderIter {
            sampler_iter: loader.batch_sampler.iter(),
            num_yielded: 0,
            data_fetcher: MapDatasetFetcher {
                dataset: &loader.dataset,
                collate_fn: PhantomData,
            },
        }
    }
    fn next_index(&mut self) -> Option<Vec<usize>> {
        self.sampler_iter.next()
    }
    fn next_data(&mut self) -> Option<C::Output> {
        let index = self.next_index();
        if let Some(index) = index {
            let data = self.data_fetcher.fetch(index);
            return Some(data);
        }
        None
    }
}
impl<'dataset, D, S, C> Iterator for SingleProcessDataLoaderIter<'dataset, D, S, C>
where
    D: Dataset,
    S: Sampler,
    C: Collate<D::Sample>,
{
    type Item = C::Output;
    fn next(&mut self) -> Option<Self::Item> {
        let data = self.next_data();

        if let Some(data) = data {
            self.num_yielded += 1;
            return Some(data);
        }
        None
    }
}
impl<'dataset, D, S, C> IntoIterator for &'dataset DataLoader<D, S, C>
where
    D: Dataset,
    S: Sampler,
    C: Collate<D::Sample>,
{
    type Item = C::Output;
    type IntoIter = SingleProcessDataLoaderIter<'dataset, D, S, C>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collate::NoOpCollate;
    use crate::dataset::NdarrayDataset;
    use crate::sampler::RandomSampler;
    use crate::sampler::SequentialSampler;
    use crate::Len;
    use ndarray::{arr0, array, Array, Array1, Array4, Axis, Ix1, Ix4, Slice};
    use ndarray_rand::rand_distr::{Normal, Uniform};
    use ndarray_rand::RandomExt;
    use std::collections::HashMap;

    #[test]
    fn len() {
        let dataset = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let dataloader = DataLoader::builder(dataset)
            .batch_size(2)
            .drop_last()
            .build();
        assert_eq!(dataloader.len(), dataloader.batch_sampler.len());
        assert_eq!(dataloader.len(), 5);
    }

    #[test]
    fn one_dimension_basic() {
        let dataset = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        // why type annotation is required even if we provide a dataset parameter?
        let dataloader = DataLoader::builder(dataset).batch_size(2).build();

        let mut iter = dataloader.iter();
        assert_eq!(iter.next(), Some(array![1, 2]));
        assert_eq!(iter.next(), Some(array![3, 4]));
        assert_eq!(iter.next(), Some(array![5, 6]));
        assert_eq!(iter.next(), Some(array![7, 8]));
        assert_eq!(iter.next(), Some(array![9, 10]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn two_iteration() {
        let dataset = vec![1, 2, 3, 4];
        let dataloader = DataLoader::builder(dataset).batch_size(2).build();

        let mut iter = dataloader.iter();
        assert_eq!(iter.next(), Some(array![1, 2]));
        assert_eq!(iter.next(), Some(array![3, 4]));
        assert_eq!(iter.next(), None);
        let mut iter = dataloader.iter();
        assert_eq!(iter.next(), Some(array![1, 2]));
        assert_eq!(iter.next(), Some(array![3, 4]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn one_dimension_basic_string() {
        let dataset = vec![String::from("a"), String::from("b")];
        let dataloader = DataLoader::builder(dataset).build();

        let mut iter = dataloader.iter();
        assert_eq!(iter.next(), Some(vec![String::from("a")]));
        assert_eq!(iter.next(), Some(vec![String::from("b")]));
        assert_eq!(iter.next(), None);
    }
    #[test]
    fn collate() {
        let dataset = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let dataloader = DataLoader::builder(dataset)
            .batch_size(2)
            .with_collate_fn(NoOpCollate)
            .build();

        let mut iter = dataloader.iter();

        assert_eq!(iter.next(), Some(vec![1, 2]));
        assert_eq!(iter.next(), Some(vec![3, 4]));
        assert_eq!(iter.next(), Some(vec![5, 6]));
        assert_eq!(iter.next(), Some(vec![7, 8]));
        assert_eq!(iter.next(), Some(vec![9, 10]));
        assert_eq!(iter.next(), None);
    }
    struct TestDataLoader<S: Sampler> {
        loader: DataLoader<NdarrayDataset<f64, f64, Ix4, Ix1>, S>,
        data: Array4<f64>,
        labels: Array1<f64>,
        dataset: NdarrayDataset<f64, f64, Ix4, Ix1>,
    }
    enum TestDataLoaderData {
        Sequential(TestDataLoader<SequentialSampler>),
        Random(TestDataLoader<RandomSampler>),
    }
    fn get_loader_with_dummy_data(batch_size: usize, shuffle: bool) -> TestDataLoaderData {
        // We use a normal distribution for the random numbers
        let normal: Normal<f64> = Normal::new(0.0, 1.0).unwrap();
        // We create a 4-dimensional array populated with random value
        let data = Array::random((100, 2, 3, 5), normal);
        // We create a 1-dimensional array populated with random value
        let labels = Array::random(100, Uniform::<f64>::new(0., 50.));
        // Basic Test dataset
        let dataset = NdarrayDataset {
            ndarrays: (data.clone(), labels.clone()),
        };

        if shuffle {
            let loader = DataLoader::builder(dataset.clone())
                .batch_size(batch_size)
                .shuffle()
                .build();

            TestDataLoaderData::Random(TestDataLoader {
                loader,
                data,
                labels,
                dataset,
            })
        } else {
            let loader = DataLoader::builder(dataset.clone())
                .batch_size(batch_size)
                .build();

            TestDataLoaderData::Sequential(TestDataLoader {
                loader,
                data,
                labels,
                dataset,
            })
        }
    }

    #[test]
    fn sequential_non_batch() {
        let batch_size = 1;
        let test_dataloader_data = tests::get_loader_with_dummy_data(batch_size, false);
        let test_data;
        if let TestDataLoaderData::Sequential(test_dataloader_data) = test_dataloader_data {
            test_data = test_dataloader_data;
        } else {
            panic!("Excpected a Sequential loader")
        }
        let mut current_idx = 0;

        for (idx, (sample, target)) in test_data.loader.iter().enumerate() {
            assert_eq!(
                sample,
                test_data
                    .data
                    .slice_axis(Axis(0), Slice::from(idx..idx + batch_size))
            );
            assert_eq!(
                target,
                test_data
                    .labels
                    .slice_axis(Axis(0), Slice::from(idx..idx + batch_size))
            );
            current_idx = idx;
        }
        assert_eq!(current_idx, test_data.dataset.len() - 1);
    }

    #[test]
    fn sequential_batch() {
        let batch_size = 2;
        let test_dataloader_data = tests::get_loader_with_dummy_data(2, false);
        let test_data;
        if let TestDataLoaderData::Sequential(test_dataloader_data) = test_dataloader_data {
            test_data = test_dataloader_data;
        } else {
            panic!("Expected a sequential loader")
        }

        let mut current_i = 0;

        for (i, (sample, target)) in test_data.loader.iter().enumerate() {
            let idx = i * batch_size;
            assert_eq!(
                sample,
                test_data
                    .data
                    .slice_axis(Axis(0), Slice::from(idx..idx + batch_size))
            );
            assert_eq!(
                target,
                test_data
                    .labels
                    .slice_axis(Axis(0), Slice::from(idx..idx + batch_size))
            );
            current_i = i;
        }
        assert_eq!(current_i, (test_data.dataset.len() - 1) / batch_size);
    }

    #[test]
    fn shuffle_non_batch() {
        let test_dataloader_data = tests::get_loader_with_dummy_data(1, true);
        let test_data;
        if let TestDataLoaderData::Random(test_dataloader_data) = test_dataloader_data {
            test_data = test_dataloader_data;
        } else {
            panic!("Expected a random loader")
        }
        // 2 maps to keep track on what we have iterated.
        let mut found_data: HashMap<_, _> = (0..test_data.data.len())
            .zip(vec![0; test_data.data.len()])
            .collect();
        let mut found_labels: HashMap<_, _> = (0..test_data.labels.len())
            .zip(vec![0; test_data.labels.len()])
            .collect();
        let mut current_i = 0;
        for (i, (sample, target)) in test_data.loader.iter().enumerate() {
            current_i = i;
            let mut current_data_point_idx = 0;
            // We iterate over the original data, finding the data corresponding to the one the dataloader just yield us
            for (data_point_idx, data_point) in test_data.data.outer_iter().enumerate() {
                current_data_point_idx = data_point_idx;
                // We need to take the inner of the sample (It's not automatically done like in python)
                if data_point == sample.index_axis(Axis(0), 0) {
                    assert_eq!(found_data[&data_point_idx], 0);
                    *found_data.get_mut(&data_point_idx).unwrap() += 1;
                    break;
                }
            }

            assert_eq!(
                arr0(target[0]),
                test_data.labels.index_axis(Axis(0), current_data_point_idx)
            );
            *found_labels.get_mut(&current_data_point_idx).unwrap() += 1;
            assert_eq!(found_data.values().sum::<usize>(), i + 1);
            assert_eq!(found_labels.values().sum::<usize>(), i + 1);
        }
        assert_eq!(current_i, test_data.dataset.len() - 1)
    }

    #[test]
    fn shuffle_batch() {
        let batch_size = 2;
        let test_dataloader_data = tests::get_loader_with_dummy_data(batch_size, true);
        let test_data;
        if let TestDataLoaderData::Random(test_dataloader_data) = test_dataloader_data {
            test_data = test_dataloader_data;
        } else {
            panic!("Expected a random loader")
        }
        let mut found_data: HashMap<_, _> = (0..test_data.data.len())
            .zip(vec![0; test_data.data.len()])
            .collect();
        let mut found_labels: HashMap<_, _> = (0..test_data.labels.len())
            .zip(vec![0; test_data.labels.len()])
            .collect();
        let mut current_i = 0;
        for (i, (batch_samples, batch_targets)) in test_data.loader.iter().enumerate() {
            current_i = i;
            for (sample, target) in batch_samples.outer_iter().zip(batch_targets) {
                let mut current_data_point_idx = 0;
                for (data_point_idx, data_point) in test_data.data.outer_iter().enumerate() {
                    current_data_point_idx = data_point_idx;
                    if data_point == sample {
                        assert_eq!(found_data[&data_point_idx], 0);
                        *found_data.get_mut(&data_point_idx).unwrap() += 1;
                        break;
                    }
                }
                assert_eq!(
                    arr0(target),
                    test_data.labels.index_axis(Axis(0), current_data_point_idx)
                );
                *found_labels.get_mut(&current_data_point_idx).unwrap() += 1;
            }
            assert_eq!(found_data.values().sum::<usize>(), (i + 1) * batch_size);
            assert_eq!(found_labels.values().sum::<usize>(), (i + 1) * batch_size);
        }
        assert_eq!(current_i, (test_data.dataset.len() - 1) / batch_size)
    }
}
