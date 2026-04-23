//! This async example demonstrates how to use the LoRa modem in RXCONTINUOUS mode and handle the
//! RxDone interrupt on DIO0. You will need a second dual_modem chip in range and with the same settings
//! to handle tx.
#![no_std]
#![no_main]

use core::convert::Infallible;
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_embedded_hal::shared_bus::SpiDeviceError;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{Async, Config, Error, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use {defmt_rtt as _, panic_probe as _};
use sx127xlora::driver::{Sx127xLora, Sx127xLoraConfig};
use sx127xlora::types::{Dio0Signal, Interrupt};

const FREQUENCY_HZ: u32 = 915_000_000;

#[embassy_executor::main]
async fn main(_task_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let miso = p.PIN_12;
    let mosi = p.PIN_11;
    let sck = p.PIN_10;
    let cs = Output::new(p.PIN_13, Level::High);

    let spi = Spi::new(p.SPI1, sck, mosi, miso, p.DMA_CH0, p.DMA_CH1, Config::default());
    let spi_bus: Mutex<NoopRawMutex, Spi<SPI1, Async>> = Mutex::new(spi);
    let spi_dev = SpiDevice::new(&spi_bus, cs);

    let mut dio0 = Input::new(p.PIN_15, Pull::Down);

    let mut config = Sx127xLoraConfig::default();
    config.frequency = FREQUENCY_HZ;
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.expect("driver init failed :(");

    sx127x.set_dio0(Dio0Signal::RxDone).await.expect("enable_dio0 failed :(");
    sx127x.receive(None).await.expect("receive failed :(");

    loop {
        info!("waiting for RxDone...");
        dio0.wait_for_high().await;
        info!("RxDone triggered!");
        sx127x.clear_interrupt(Interrupt::RxDone).await.expect("clear interrupt RxDone failed :(");
        match sx127x.read_rx_data().await {
            Ok(buf) => {
                let len: usize = buf.iter().filter(|c| **c != 0).count();
                info!("rx buffer: {:a}", buf[..len])
            },
            Err(_) => error!("read_rx_data failed :(")
        }
        match sx127x.last_packet_rssi().await {
            Ok(byte) => info!("last rx packet RSSI: {}", byte),
            Err(_) => error!("last_packet_rssi failed :(")
        }
        match sx127x.last_packet_snr().await {
            Ok(byte) => info!("last rx packet SNR: {}", byte),
            Err(_) => error!("last_packet_snr failed :(")
        }
    }
}