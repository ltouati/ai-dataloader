# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.6.2] - 2024-14-09
## Changed
- bump `tch-rs` to 0.17
- update `itertools` to 0.13
- use new style modules
- implement `ExactSizeIterator` when possible (thanks @AzHicham)
- change the `Fetch` trait to use `&self` instead of `&mut self` (thanks @AzHicham) 

## Added
- add typos to the ci
- add cargo deny to the ci

## [0.6.1] - 2023-13-08
### Fixed
- fix parallelalisation, `install` wasn't sufficient to run things in a multithreaded way.

### Changed
- avoid resetting the thread pools when the number of threads are the same.

## [0.6.0] - 2023-22-05
### Added
- add support for closure as collate function.
- add basic parallelalisation (feat @AzHicham) 

## [0.5.4] - 2023-29-04
### Fixed
- online doc with feature tch are broken

## [0.5.3] - 2023-27-04
### Fixed
- online doc didn't show all features

## [0.5.2] - 2023-27-04
### Added
- basic benchmark with criterion.
### Changed
- improve doc by showing features.

## [0.5.1] - 2023-25-04
### Fixed
- primitive type for `TorchCollate` are now collated in `tch::Tensor` instead of `ndarray`.
- clippy pedantic lints

## [0.5.0] - 2023-18-04
### Changed
- refactor example organisation
- rename features `torch` to `tch`.
- make tch loading on the GPU opt-in
- bunp tch to 0.11.0

## [0.4.0] - 2023-17-03
### Added
- Add tch-rs integration to run on the GPU.

## [0.3.1] - 2022-13-11
### Fixed
- Incorrect bound for iterable dataloader builder, which doesn't allowed provide a custom collate function.
  
## [0.3.0] - 2022-01-11
### Added
- more strict compiler warnings
- `Dataloader` for iterable datasets. A dalaloader working for any type that implements `IntoIterator`
- shuffling support for iterable dataset
- project folder reorganization
- default collate now support reference

## [0.2.1] - 2022-27-09
### Fixed
- Fix link in the doc
- Add missing documentation field in package metadata
## [0.2.0] - 2022-27-09
### Added
- Indexable dataset
- `DefaultCollate` function
- `Dataloader` and `DataloaderBuilder`
- Sequential and random `Sampler`


[Unreleased]: https://github.com/Tudyx/ai-dataloader/compare/v0.6.2...HEAD
[0.6.2]: https://github.com/Tudyx/ai-dataloader/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/Tudyx/ai-dataloader/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/Tudyx/ai-dataloader/compare/v0.5.4...v0.6.0
[0.5.4]: https://github.com/Tudyx/ai-dataloader/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/Tudyx/ai-dataloader/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/Tudyx/ai-dataloader/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/Tudyx/ai-dataloader/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/Tudyx/ai-dataloader/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Tudyx/ai-dataloader/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/Tudyx/ai-dataloader/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Tudyx/ai-dataloader/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/Tudyx/ai-dataloader/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Tudyx/ai-dataloader/compare/v0.1.0...v0.2.0
