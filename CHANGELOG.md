<!-- next-header -->

## [Unreleased] - ReleaseDate

## Added
- ~~when running `disguise` subcommand, if masking asset can't fit the file inside, all assets are tried in turn until one is found that fits, otherwise it gives up.~~
- Integrate `disguise` command with HTTP client to pull images on the fly for masking

## Fixed
- don't panic when attempting to decrypt a message that wasn't encrypted to begin with

## [0.4.2] - 2022-07-06

## [0.4.1] - 2022-07-06

## Fixed
- assets not found when using disguise

## Removed
- assets to ensure crate is within crates.io max upload size 

## [0.4.0] - 2022-07-06

## Added
- disguise functionality to mask all files in a directory through the `disguise` subcommand
- better error handling
- logging - level modified through environment variable `RUST_LOG`
 
## Changed
- CLI interface to have subcommands
  - example from previous usage
  "echo my message | stegosaurust -o out.png input.png"
  is now
  "echo my message | stegosaurust enc -o out.png input.png"

## [0.3.1] - 2022-06-27

## Added
- bit distribution to spread message evenly throughout image
- improved error handling

## [0.3.0] - 2022-06-11

## Added
- documentation for key components
- compression/decompression for data being encoded

## Modified
- minor refactoring in places, no behvaioural differences

## [0.2.4] - 2022-06-09

## Fixed
- status badge for publish point to right branch

## [0.2.3] - 2022-06-09

### Reverted
- binary upload to github

## [0.2.2] - 2022-06-09

### Added
- added a changelog to the project
- enhanced continuous integration and deployment with:
  - version bumping
  - partially automated changelog management
  - release binaries to github

<!-- next-url -->
[Unreleased]: https://github.com/jj-style/stegosaurust/compare/v0.4.2...HEAD
[0.4.2]: https://github.com/jj-style/stegosaurust/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/jj-style/stegosaurust/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/jj-style/stegosaurust/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/jj-style/stegosaurust/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/jj-style/stegosaurust/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/jj-style/stegosaurust/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/jj-style/stegosaurust/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/jj-style/stegosaurust/compare/v0.2.1...v0.2.2
