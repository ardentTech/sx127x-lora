//! This async example demonstrates the RxTimeout interrupt being triggered on DIO1 when a packet
//! doesn't arrive before the user-defined timeout (in symbols).
#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use sx127xlora::driver::{Sx127xLora, Sx127xLoraConfig, RX_TIMEOUT_MIN_SYMBOLS};
use sx127xlora::types::{Dio1Signal, Interrupt};

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

    let mut dio1 = Input::new(p.PIN_14, Pull::Down);

    let mut config = Sx127xLoraConfig::default();
    config.frequency = FREQUENCY_HZ;
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.expect("driver init failed :(");

    sx127x.set_dio1(Dio1Signal::RxTimeout).await.expect("enable_dio1 failed");

    loop {
        sx127x.receive(Some(RX_TIMEOUT_MIN_SYMBOLS)).await.expect("receive init failed :(");
        info!("waiting for timeout...");
        dio1.wait_for_high().await;
        info!("RxTimeout triggered!");
        sx127x.clear_interrupt(Interrupt::RxTimeout).await.expect("clear interrupt RxTimeout failed :(");
        Timer::after(embassy_time::Duration::from_millis(3_000)).await;
        info!("looping around");
    }
}