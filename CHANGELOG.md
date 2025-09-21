# Changelog

All notable changes to Podbringer will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.6] - 2025-09-21

### Changed

* Enable email notifications for all Forgejo Action workflows

### Security

* Updated dependencies; fixes security advisories:
  * [RUSTSEC-2025-0022](https://rustsec.org/advisories/RUSTSEC-2025-0022.html)
  * [RUSTSEC-2025-0024](https://rustsec.org/advisories/RUSTSEC-2025-0024.html)
  * [RUSTSEC-2025-0047](https://rustsec.org/advisories/RUSTSEC-2025-0047.html)
  * [RUSTSEC-2025-0055](https://rustsec.org/advisories/RUSTSEC-2025-0055.html)

## [0.5.5] - 2025-03-23

### Added

* Add Renovate config with recommended settings

### Changed

* Update the dependencies on `cached`, `reqwest` and `thiserror`

### Fixed

* Fix typos in documentation and comments

### Security

* Updated dependencies, fixes security advisories:
  * [RUSTSEC-2025-0004](https://rustsec.org/advisories/RUSTSEC-2025-0004)
  * [RUSTSEC-2025-0009](https://rustsec.org/advisories/RUSTSEC-2025-0009)

## [0.5.4] - 2024-07-26

### Changed

* Switch to Forgejo Actions; add audit workflow
* Update dependency on `rocket_dyn_templates`
* Update dependency on `youtube_dl`

### Security

* Update dependencies, fixes security advisories:
  * [RUSTSEC-2024-0019](https://rustsec.org/advisories/RUSTSEC-2024-0019)
  * [RUSTSEC-2024-0332](https://rustsec.org/advisories/RUSTSEC-2024-0332)
  * [RUSTSEC-2024-0336](https://rustsec.org/advisories/RUSTSEC-2024-0336)
  * [RUSTSEC-2024-0357](https://rustsec.org/advisories/RUSTSEC-2024-0357)

## [0.5.3] - 2024-02-27

### Changed

* Update dependency on `cached`

### Security

* Update dependencies, fixes security advisories:
  * [RUSTSEC-2024-0003](https://rustsec.org/advisories/RUSTSEC-2024-0003)
  * [RUSTSEC-2023-0072](https://rustsec.org/advisories/RUSTSEC-2024-0072)
  * [RUSTSEC-2023-0074](https://rustsec.org/advisories/RUSTSEC-2024-0072)

### Fixed

* Handle paging information begin absent; fixes short feeds for Mixcloud (#17)

## [0.5.2] - 2023-11-03

### Security

* Update dependencies
  ([RUSTSEC-2020-0071](https://rustsec.org/advisories/RUSTSEC-2020-0071.html))

### Changed

* Switch to Rocket 0.5 RC4
* Update dependency on `cached`

## [0.5.1] - 2023-08-25

### Changed

* Bump the dependency on `youtube_dl`
* Update release Gitea Actions workflow; add separate job to release Debian
  package to the new repository

### Security

* Update dependencies
  ([RUSTSEC-2023-0034](https://rustsec.org/advisories/RUSTSEC-2023-0034),
  [RUSTSEC-2023-0044](https://rustsec.org/advisories/RUSTSEC-2023-0044),
  [RUSTSEC-2023-0052](https://rustsec.org/advisories/RUSTSEC-2023-0052))

## [0.5.0] - 2023-06-08

### Added

* Add full release Gitea Actions workflow

### Changed

* Simplify GItea Actions check and lint workflow

### Fixed

* Differentiate between publish and update time for items

## [0.4.1] - 2023-04-11

### Changed

* Select only direct HTTP MP4 audio streams for the Mixcloud back-end

## [0.4.0] - 2023-03-24

### Added

* Add Gitea Actions workflow for cargo

### Changed

* Update dependencies on `cached` and `youtube_dl`
* Update to `rocket` version 0.5.0-rc.3
* Select only MP4 audio streams for the YouTube back-end (experimental)
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

[Unreleased]: https://git.luon.net/paul/podbringer/compare/v0.5.6...HEAD
[0.5.6]: https://git.luon.net/paul/podbringer/compare/v0.5.5..v0.5.6
[0.5.5]: https://git.luon.net/paul/podbringer/compare/v0.5.4..v0.5.5
[0.5.4]: https://git.luon.net/paul/podbringer/compare/v0.5.3..v0.5.4
[0.5.3]: https://git.luon.net/paul/podbringer/compare/v0.5.2..v0.5.3
[0.5.2]: https://git.luon.net/paul/podbringer/compare/v0.5.1..v0.5.2
[0.5.1]: https://git.luon.net/paul/podbringer/compare/v0.5.0..v0.5.1
[0.5.0]: https://git.luon.net/paul/podbringer/compare/v0.4.1..v0.5.0
[0.4.1]: https://git.luon.net/paul/podbringer/compare/v0.4.0..v0.4.1
[0.4.0]: https://git.luon.net/paul/podbringer/compare/v0.3.0..v0.4.0
[0.3.0]: https://git.luon.net/paul/podbringer/compare/v0.2.0..v0.3.0
[0.2.0]: https://git.luon.net/paul/podbringer/compare/v0.1.0..v0.2.0
[0.1.0]: https://git.luon.net/paul/podbringer/commits/tag/v0.1.0
