//! This example demonstrates how to use the channel activity detector (CAD) to determine if the
//! channel is busy or not before initiating transmission. You'll need (at least) one other chip
//! with the same configuration transmitting at a very high frequency to trip the CAD.
//!
//! See: section 1.3 of https://www.semtech.com/uploads/technology/LoRa/cad-ensuring-lora-packets.pdf
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
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use sx127xlora::driver::{Sx127xLora, Sx127xLoraConfig};
use sx127xlora::types::{DeviceMode, Dio3Signal, IRQ};
use common::{heartbeat, LORA_FREQUENCY_HZ};

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

    let mut dio3 = Input::new(p.PIN_18, Pull::Down);

    let mut config = Sx127xLoraConfig::default();
    config.frequency = LORA_FREQUENCY_HZ;
    let mut sx127x = Sx127xLora::new(spi_dev, config).await.unwrap();

    sx127x.set_dio3(Dio3Signal::CadDone).await.unwrap();

    spawner.spawn(heartbeat(Output::new(p.PIN_21, Level::Low))).unwrap();
    sx127x.set_device_mode(DeviceMode::CAD).await.unwrap();

    loop {
        dio3.wait_for_high().await;

        if sx127x.irq_flag(IRQ::CadDetected).await.unwrap() {
            info!("CadDetected triggered! Channel is busy so will retry later.");
            sx127x.clear_irq(IRQ::CadDetected).await.unwrap();
        } else {
            info!("CadDetected not triggered. OK to transmit.");
        }
        sx127x.clear_irq(IRQ::CadDone).await.unwrap();
        Timer::after(embassy_time::Duration::from_millis(250)).await;
        sx127x.set_device_mode(DeviceMode::CAD).await.unwrap();
    }
}