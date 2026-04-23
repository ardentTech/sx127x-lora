# Sx127x-LoRa
`#![no_std]`, `async`-first driver for the LoRa modem on the Semtech SX127X transceiver built on top of the Rust
[embedded-hal](https://github.com/rust-embedded/embedded-hal).

### Cargo Features

- `defmt`: include deferred formatting logging functionality
- `half_duplex`: use the full data buffer size of 256 bytes for RX or TX, at the cost of full duplex RX and TX with 128 byte buffers.
- `sync`: modem sync implementation

### Roadmap

- [ ] async (in-progress)
- [ ] sync

### TODO

- [ ] LowFrequencyModeOn bit of RegOpMode
- [ ] frequency hopping example

### Examples

* [LoRa RP235x async](https://github.com/ardentTech/sx127x-lora/tree/main/examples/rp235x/async)

### Resources

* [Datasheet](https://semtech.my.salesforce.com/sfc/p/E0000000JelG/a/2R0000001Rbr/6EfVZUorrpoKFfvaF_Fkpgp5kzjiNyiAbqcpqh9qSjE)
* [Errata](https://semtech.my.salesforce.com/sfc/p/E0000000JelG/a/2R000000HSPv/sqi9xX0gs6hgzl2LoPwCK0TS9GDPlMwsXmcNzJCMHjw)

### License

* [MIT](https://github.com/ardentTech/sx127x-lora/blob/main/LICENSE-MIT)
* [Apache](https://github.com/ardentTech/sx127x-lora/blob/main/LICENSE-APACHE)