# Changelog

[`embedded-graphics-simulator`](https://crates.io/crates/embedded-graphics-simulator) is an SDL-based simulator for testing, debugging and developing [`embedded-graphics`](https://crates.io/crates/embedded-graphics) applications.

<!-- next-header -->

## [Unreleased] - ReleaseDate

### Added

- [#28](https://github.com/embedded-graphics/simulator/pull/28) Added `SimulatorDisplay::to_{be,le,ne}_bytes` to convert the display content to raw image data.
- [#29](https://github.com/embedded-graphics/simulator/pull/29) Added support for `EG_SIMULATOR_CHECK`, `EG_SIMULATOR_CHECK_RAW` and `EG_SIMULATOR_DUMP_RAW` environment variables.

### Changed

- **(breaking**) [#29](https://github.com/embedded-graphics/simulator/pull/29) Color types used in `Window::update`and `Window::show_static` must now implement `From<Rgb888>`.

### Fixed

- [#28](https://github.com/embedded-graphics/simulator/pull/28) Fixed panic for zero sized `SimulatorDisplay`s.

## [0.3.0-beta.2] - 2021-05-04

### Added

- [#25](https://github.com/embedded-graphics/simulator/pull/25) Added `OutputImage` to export PNG files and base64 encoded PNGs.
- [#25](https://github.com/embedded-graphics/simulator/pull/25) Added `BinaryColorTheme::Inverted`.

### Changed

- **(breaking)** [#25](https://github.com/embedded-graphics/simulator/pull/25) Removed `SimulatorDisplay::to_image_buffer`. Use `to_rgb_output_image` or `to_grayscale_output_image` instead.

## [0.3.0-beta.1] - 2021-04-24

### Changed

- [#24](https://github.com/embedded-graphics/simulator/pull/24) Upgrade to embedded-graphics 0.7.0-beta.1.

## [0.3.0-alpha.2] - 2021-02-05

### Added

- [#16](https://github.com/embedded-graphics/simulator/pull/16) Re-export `sdl2` types.

## [0.3.0-alpha.1] - 2021-01-07

## [0.2.1] - 2020-07-29

> Note: PR numbers from this point onwards are from the old `embedded-graphics/embedded-graphics` repository. New PR numbers above this note refer to PRs in the `embedded-graphics/simulator` repository.

### Added

- [#298](https://github.com/embedded-graphics/embedded-graphics/pull/298) Added the `with-sdl` option (enabled by default) to allow optionally disabling SDL2 support.
- [#271](https://github.com/embedded-graphics/embedded-graphics/pull/271) Add `MouseMove` event support to simulator.

## [0.2.0] - 2020-03-20

### Added

- **(breaking)** #266 Added [image](https://crates.io/crates/image) support and PNG export. See the `README.md` for information about how to use these features. The API for creating windows was changed to make the output settings independent of the `Window` type. The pixel scaling and theme settings were moved to a new `OutputSettings` struct, that can be built using the `OutputSettingsBuilder`. `WindowBuilder` was removed and replaced by a `Window::new(title, &output_settings)` function.

## [0.2.0-beta.2] - 2020-02-17

### Added

- #183 Added limited mouse and keyboard event handling to the simulator in order to simulate input devices such as touch screens, buttons, or rotary encoders.
- #171 Added a more complex `analog-clock` example to the simulator - [check it out](https://github.com/embedded-graphics/embedded-graphics/tree/embedded-graphics-v0.6.0-alpha.3/simulator/examples/analog-clock.rs) for some more in-depth usage of Embedded Graphics.

### Fixed

- #192 Performance of drawing in the simulator is increased.
- #218 Test README examples in CI and update them to work with latest crate versions.

### Changed

- **(breaking)** The simulator API changed.
- #203 updated simulator screenshots and added them to README

## 0.2.0-alpha.1

### Fixed

- The TGA example in the simulator now draws the image correctly

## 0.1.0

### Changed

- The simulator is now [available on crates.io](https://crates.io/crates/embedded-graphics-simulator) as a standalone crate. You can now create simulated displays for testing out embedded_graphics code or showing off cool examples.
- The builtin simulator now supports colour pixel types, like `RGB565`.

<!-- next-url -->
[unreleased]: https://github.com/embedded-graphics/embedded-graphics-simulator/compare/v0.3.0-beta.2...HEAD
[0.3.0-beta.2]: https://github.com/embedded-graphics/embedded-graphics-simulator/compare/v0.3.0-beta.1...v0.3.0-beta.2

[0.3.0-beta.1]: https://github.com/embedded-graphics/embedded-graphics-simulator/compare/v0.3.0-alpha.2...v0.3.0-beta.1
[0.3.0-alpha.2]: https://github.com/embedded-graphics/embedded-graphics-simulator/compare/v0.3.0-alpha.1...v0.3.0-alpha.2
[0.3.0-alpha.1]: https://github.com/embedded-graphics/simulator/compare/after-split...v0.3.0-alpha.1
[0.2.1]: https://github.com/embedded-graphics/embedded-graphics/compare/embedded-graphics-simulator-v0.2.0...embedded-graphics-simulator-v0.2.1
[0.2.0]: https://github.com/embedded-graphics/embedded-graphics/compare/embedded-graphics-simulator-v0.2.0-beta.2...embedded-graphics-simulator-v0.2.0
[0.2.0-beta.2]: https://github.com/embedded-graphics/embedded-graphics/compare/embedded-graphics-simulator-v0.2.0-alpha.1...embedded-graphics-simulator-v0.2.0-beta.2
