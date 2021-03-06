# Changelog

All notable changes to Podbringer will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2022-05-27

### Added

* Add support for paging, i.e. retrieving more that 50 past items (#9)
* Introduce the `limit` parameter to get more/less than 50 feed items
* Add caching; all Mixcloud user, cloudcasts and download URL requests are
  cached for 24 hours (#3)

### Changed

* Implemented proper error logging and handling (#6)
* Replaces own youtube-dl command running implementation by `youtub_dl`
  crate (#8)
* Several code and documentation improvements & fixes

### Removed

* Drop dependencies on some unnecessary/unused crates

## [0.1.0] - 2022-05-24

Initial release.

[Unreleased]: https://git.luon.net/paul/podbringer/compare/v0.2.0...HEAD
[0.2.0]: https://git.luon.net/paul/podbringer/compare/v0.1.0..v0.2.0
[0.1.0]: https://git.luon.net/paul/podbringer/commits/tag/v0.1.0
