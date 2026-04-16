//! This async example demonstrates how to use the LoRa modem in RXSINGLE mode with a timeout. The
//! RxDone interrupt on DIO0 is wired to pin 15, and the RxTimeout interrupt on DIO1 is wired to pin
//! 14. You will need a second dual_modem chip in range and with the same settings to handle tx before
//! the timeout.
#![no_std]
#![no_main]

use defmt::{error, info, panic};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::SPI1;
use embassy_rp::spi::{Async, Config, Spi};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use {defmt_rtt as _, panic_probe as _};
use sx127x_lora::driver::{Sx127xLora, Sx127xLoraConfig, RX_TIMEOUT_MAX_SYMBOLS};
use sx127x_lora::types::{Dio0Signal, Dio1Signal, Interrupt};

const FREQUENCY_HZ: u32 = 915_000_000;

#[embassy_executor::task]
async fn dio1_task(mut dio1: Input<'static>) {
    loop {
        dio1.wait_for_high().await;
        panic!("RxTimeout triggered :(");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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
    sx127x.set_dio1(Dio1Signal::RxTimeout).await.expect("enable_dio1 failed :(");

    spawner.spawn(dio1_task(Input::new(p.PIN_14, Pull::Down))).unwrap();

    sx127x.receive(Some(RX_TIMEOUT_MAX_SYMBOLS)).await.expect("receive failed :(");

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
        info!("looping around");
    }
}