# Changelog

All notable changes to Podbringer will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2023-03-24

### Added

* Add Gitea Actions workflow for cargo

### Changed

* Update dependencies on `cached` and `youtube_dl`
* Update to `rocket` version 0.5.0-rc.3
* Select MP4 audio streams only (experimental)
* Remove parameters from MIME types to prevent clients tripping over them

### Fixed

* Bump the dependency on `ytextract` (#14)
* Fix typo in the documentation

### Security

* Update dependencies
  ([RUSTSEC-2021-0145](https://rustsec.org/advisories/RUSTSEC-2021-0145.html),
  [RUSTSEC-2020-0016](https://rustsec.org/advisories/RUSTSEC-2020-0016.html),
  [RUSTSEC-2023-0001](https://rustsec.org/advisories/RUSTSEC-2023-0001.html),
  [RUSTSEC-2023-0005](https://rustsec.org/advisories/RUSTSEC-2023-0005.html),
  [RUSTSEC-2023-0018](https://rustsec.org/advisories/RUSTSEC-2023-0018.html),
  [RUSTSEC-2023-0022](https://rustsec.org/advisories/RUSTSEC-2023-0022.html),
  [RUSTSEC-2023-0023](https://rustsec.org/advisories/RUSTSEC-2023-0023.html),
  [RUSTSEC-2023-0024](https://rustsec.org/advisories/RUSTSEC-2023-0024.html))

## [0.3.0] - 2022-12-24

### Added

* Add abstraction that will support multiple back-ends
* Add YouTube back-end for generating feeds of YouTube channels and
  playlists (#5)

### Changed

* Change the name of the `url` to `public_url` in the configuration file
  `Rocket.toml`
* Make feed channel and item images optional
* Simplify how Rocket is launched
* Split off feed generation to a separate module
* Improve documentation

### Fixed

* Some code refactoring

### Security

* Update/bump dependencies

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

[Unreleased]: https://git.luon.net/paul/podbringer/compare/v0.4.0...HEAD
[0.4.0]: https://git.luon.net/paul/podbringer/compare/v0.3.0..v0.4.0
[0.3.0]: https://git.luon.net/paul/podbringer/compare/v0.2.0..v0.3.0
[0.2.0]: https://git.luon.net/paul/podbringer/compare/v0.1.0..v0.2.0
[0.1.0]: https://git.luon.net/paul/podbringer/commits/tag/v0.1.0
