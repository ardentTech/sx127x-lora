# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `set_pll_bandwidth` method
- `invert_iq` getter
- `rx_coding_rate` getter
- `pa_ramp` getter
- `Ocp` struct, `ocp` getter
- `Lna` struct and `lna` getter
- `hop_channel` getter
- `PowerAmplifier` tuple struct
- `PARFO` tuple struct
- error check of received payload length compared to buffer size
- `PreambleLength` and `TimeoutSymbols` types

### Changed

- `set_invert_iq` parameter is `InvertIQ` struct instead of bools
- `set_lna_gain` to `set_lna`
- set lf mode on/off when calling `set_frequency()`
- `set_pa_boost` to `set_power_amplifier`
- `set_pa_boost` parameter type from `u8` to `PowerAmplifier`
- `set_pa_rfo` parameter type forom `i8` to `PARFO`
- rename `Interrupt` to `IRQ` for type and driver methods

### Fixed

- crate import errors in examples after unexpected rename for 0.1.0
- `modem_status` getter bug that didn't mask off last rx coding rate before conversion to enum
- hard-coded register mask offsets

### Removed

- `DEFAULT_FREQUENCY_HZ` constant
- `assert!` from driver
- unused `RxStatus` enum

## [0.1.1] - 2026-04-23

### Fixed

- crate import errors in examples after unexpected rename for 0.1.0

## [0.1.0] - 2026-04-22

### Added

- `lora` and `fsk` cargo features
- `valid_rx_headers` method
- `valid_rx_packets` method
- `last_packet_snr` method
- `set_invert_iq` method
- `bandwidth`, `coding_rate`, `spreading_factor` methods
- `symbol_rate`, `data_rate` methods
- make SPI `write` method public
- `set_temp_monitor` method
- `set_lna_gain` method
- `set_ocp` method
- `set_crc` method and make `set_coding_rate` public
- `rssi`, `last_packet_rssi` methods
- `set_low_data_rate_optimize` method
- `set_preamble_length` method
- `set_pa_boost`, `set_pa_rfo` methods
- `frequency_error_indication_hz`, `frequency_error_indication_ppm` methods
- `set_pa_ramp` method
- update RegInvertIQ2 according to rx_path or tx_path inversion
- `set_sync_word` method
- `set_max_payload_length` method
- `rssi_wideband` method
- `last_rx_byte_addr` method
- `set_hop_period` method
- `optimize_rx_response` method for errata 2.3
- DIO2..DIO5 signals and setters
- `half_duplex` feature flag
- remove `frf` calculation and move to `sx127x-common`
- pub re-export `Sx127xError`
- make `device_mode` getter public