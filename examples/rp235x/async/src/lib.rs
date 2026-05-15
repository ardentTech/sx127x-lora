#![no_std]

use embassy_rp::gpio::Output;
use embassy_time::Timer;

pub const LORA_FREQUENCY_HZ: u32 = 915_000_000;

#[embassy_executor::task]
pub async fn heartbeat(mut pin: Output<'static>) {
    loop {
        pin.set_high();
        Timer::after(embassy_time::Duration::from_millis(250)).await;
        pin.set_low();
        Timer::after(embassy_time::Duration::from_millis(750)).await;
    }
}