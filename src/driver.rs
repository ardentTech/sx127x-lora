#[cfg(feature = "defmt")]
use defmt::error;

use embedded_hal_async::spi::SpiDevice;
pub use sx127x_common::error::Sx127xError;
use sx127x_common::{DEFAULT_FREQUENCY_HZ, FSTEP};
use sx127x_common::bits::{get_bits, set_bits};
use sx127x_common::error::Sx127xError::InvalidState;
use sx127x_common::spi::Sx127xSpi;
use crate::registers::*;
use crate::types::*;
use crate::calculate;

#[cfg(feature = "half_duplex")]
const PAYLOAD_SIZE: usize = 256;
#[cfg(not(feature = "half_duplex"))]
const PAYLOAD_SIZE: usize = 128;
// identifies silicon Version 1b, which applies to errata
const PRODUCTION_VERSION: u8 = 0x12;
const LF_MAX_HZ: u32 = 525_000_000;
const HF_MIN_HZ: u32 = 779_000_000;

pub struct Sx127xLoraConfig {
    pub bandwidth: Bandwidth,
    pub coding_rate: CodingRate,
    pub frequency: u32, // Hz
    pub spreading_factor: SpreadingFactor,
}
impl Default for Sx127xLoraConfig {
    fn default() -> Self {
        Self {
            bandwidth: Bandwidth::default(),
            coding_rate: CodingRate::default(),
            frequency: DEFAULT_FREQUENCY_HZ,
            spreading_factor: SpreadingFactor::default(),
        }
    }
}

/// Sx127x driver with LoRa modem.
pub struct Sx127xLora<SPI> {
    pub spi: Sx127xSpi<SPI>
}
impl<SPI: SpiDevice> Sx127xLora<SPI> {
    pub async fn new(spi: SPI, config: Sx127xLoraConfig) -> Result<Sx127xLora<SPI>, Sx127xError<SPI::Error>> {
        let mut driver = Self { spi: Sx127xSpi::new(spi) };

        driver.set_long_range_mode(true).await?;
        driver.set_bandwidth(config.bandwidth).await?;
        driver.set_coding_rate(config.coding_rate).await?;
        driver.set_frequency(config.frequency).await?;
        driver.set_spreading_factor(config.spreading_factor).await?;

        Ok(driver)
    }

    /// Gets the bandwidth.
    ///
    /// See: datasheet section 4.1.1.4
    pub async fn bandwidth(&mut self) -> Result<Bandwidth, Sx127xError<SPI::Error>> {
        Ok(Bandwidth::from((self.read(MODEM_CONFIG_1).await? & MODEM_CONFIG_1_BW_MASK) >> MODEM_CONFIG_1_BW_OFFSET))
    }

    /// Triggers the IQ and RSSI calibration when set in Standby mode. Takes ~10ms.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn calibrate(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut image_cal = self.read(IMAGE_CAL).await?;
        image_cal |= IMAGE_CAL_IMAGE_CAL_START_MASK;
        self.write(IMAGE_CAL, image_cal).await
    }

    /// Clears an irq.
    pub async fn clear_irq(&mut self, irq: IRQ) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS).await?;
        self.write(IRQ_FLAGS, byte | irq.mask()).await
    }

    /// Gets the cyclic error coding rate.
    ///
    /// See: datasheet section 4.1.1.3
    pub async fn coding_rate(&mut self) -> Result<CodingRate, Sx127xError<SPI::Error>> {
        let byte = self.read(MODEM_CONFIG_1).await?;
        Ok(CodingRate::from(get_bits(byte, MODEM_CONFIG_1_CODING_RATE_MASK, MODEM_CONFIG_1_CODING_RATE_OFFSET)))
    }

    /// Calculates the data rate in bits/s.
    pub async fn data_rate(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let coding_rate: f32 = self.coding_rate().await?.into();
        let symbol_rate = self.symbol_rate().await? as f32;
        let spreading_factor = (self.spreading_factor().await? as u8) as f32;
        Ok(calculate::data_rate(symbol_rate, spreading_factor, coding_rate))
    }

    /// Gets the device mode.
    ///
    /// See: datasheet table 16
    pub async fn device_mode(&mut self) -> Result<DeviceMode, Sx127xError<SPI::Error>> {
        let op_mode = self.read(OP_MODE).await?;
        Ok(DeviceMode::from(get_bits(op_mode, OP_MODE_MODE_MASK, OP_MODE_MODE_OFFSET)))
    }

    /// Gets the carrier frequency in Hz.
    ///
    /// See: datasheet section 4.1.4
    pub async fn frequency(&mut self) -> Result<u32, Sx127xError<SPI::Error>> {
        let msb = self.read(FRF_MSB).await? as u32;
        let mid = self.read(FRF_MID).await? as u32;
        let lsb = self.read(FRF_LSB).await? as u32;
        Ok((msb << 16) | (mid << 8) | lsb)
    }

    /// Gets the frequency error indication (FEI) in Hz.
    ///
    /// See: datasheet section 4.1.5
    pub async fn frequency_error_indication_hz(&mut self) -> Result<f64, Sx127xError<SPI::Error>> {
        let msb = self.read(FEI_MSB).await?;
        let mid = self.read(FEI_MID).await?;
        let lsb = self.read(FEI_LSB).await?;
        let fei = (((msb as u32) << 16) | ((mid as u32) << 8) | (lsb as u32)) as i32;
        let bandwidth = self.bandwidth().await?;

        Ok(calculate::fei_hz(fei, bandwidth.khz()))
    }

    /// Gets the frequency error indication (FEI) in PPM.
    ///
    /// See: datasheet section 4.1.5
    pub async fn frequency_error_indication_ppm(&mut self) -> Result<f64, Sx127xError<SPI::Error>> {
        let hz = self.frequency_error_indication_hz().await?;
        let frf = self.frequency().await?;
        Ok(calculate::fei_ppm(hz, frf))
    }

    /// Gets the FHSS start channel.
    ///
    /// See: datasheet section 4.1.1.6
    pub async fn hop_channel(&mut self) -> Result<HopChannel, Sx127xError<SPI::Error>> {
        Ok(HopChannel::from(self.read(HOP_CHANNEL).await?))
    }

    /// Gets an irq flag.
    pub async fn irq_flag(&mut self, irq: IRQ) -> Result<bool, Sx127xError<SPI::Error>> {
        Ok(self.read(IRQ_FLAGS).await? & irq.mask() == 1)
    }

    /// Gets the RX data buffer pointer.
    ///
    /// See: datasheet pages 41-42
    pub async fn last_rx_byte_addr(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.read(FIFO_RX_BYTE_ADDR).await
    }

    /// Gets the over-current protection (OCP) on/off.
    ///
    /// See: datasheet section 3.4.4
    pub async fn ocp(&mut self) -> Result<Ocp, Sx127xError<SPI::Error>> {
        let byte = self.read(OCP).await?;
        Ok(Ocp::new(
            get_bits(byte, OCP_ON_MASK, OCP_ON_OFFSET) == 1,
            calculate::ocp_imax(get_bits(byte, OCP_TRIM_MASK, OCP_TRIM_OFFSET))
        ))
    }

    /// Optimize receiver intermediate frequency to mitigate spurious reception of LoRa signal.
    ///
    /// See: errata section 2.3
    pub async fn optimize_rx_response(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_device_mode(DeviceMode::STDBY).await?;

        let bandwidth = self.bandwidth().await?;
        self.optimize_rx_response_frf_offset(bandwidth).await?;
        self.optimize_rx_response_detect_optimize(bandwidth).await?;
        self.optimize_rx_response_if(bandwidth).await
    }

    /// Gets the rise/fall time of the power amplifier (PA).
    ///
    /// See: datasheet section 2.1.2.3
    pub async fn pa_ramp(&mut self) -> Result<PARamp, Sx127xError<SPI::Error>> {
        Ok(PARamp::from(get_bits(self.read(PA_RAMP).await?, PA_RAMP_MASK, PA_RAMP_OFFSET)))
    }

    /// Sets the DIO0 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn set_dio0(&mut self, signal: Dio0Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio_mapping(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO0_MASK, DIO_MAPPING_1_DIO0_OFFSET).await
    }

    /// Sets the DIO1 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn set_dio1(&mut self, signal: Dio1Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio_mapping(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO1_MASK, DIO_MAPPING_1_DIO1_OFFSET).await
    }

    /// Sets the DIO2 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn set_dio2(&mut self, signal: Dio2Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio_mapping(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO2_MASK, DIO_MAPPING_1_DIO2_OFFSET).await
    }

    /// Sets the DIO3 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn set_dio3(&mut self, signal: Dio3Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio_mapping(DIO_MAPPING_1, signal as u8, DIO_MAPPING_1_DIO3_MASK, DIO_MAPPING_1_DIO3_OFFSET).await
    }

    /// Sets the DIO4 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn set_dio4(&mut self, signal: Dio4Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio_mapping(DIO_MAPPING_2, signal as u8, DIO_MAPPING_2_DIO4_MASK, DIO_MAPPING_2_DIO4_OFFSET).await
    }

    /// Sets the DIO5 pin signal source.
    ///
    /// See: datasheet table 18
    pub async fn set_dio5(&mut self, signal: Dio5Signal) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_dio_mapping(DIO_MAPPING_2, signal as u8, DIO_MAPPING_2_DIO5_MASK, DIO_MAPPING_2_DIO5_OFFSET).await
    }

    /// Gets received signal strength indicator (RSSI) of last packet received.
    ///
    /// See: datasheet section 3.5.5
    pub async fn last_packet_rssi(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.read(PKT_RSSI_VALUE).await
    }

    /// Gets estimation of signal-to-noise ratio (SNR) in dB on last packet received.
    ///
    /// See: datasheet section 3.5.5
    pub async fn last_packet_snr(&mut self) -> Result<i8, Sx127xError<SPI::Error>> {
        Ok(self.read(PKT_SNR_VALUE).await? as i8 >> 2)
    }

    /// Gets the gain and high frequency boost for the low noise receiver amplifier (LNA).
    ///
    /// See: datasheet page 110
    pub async fn lna(&mut self) -> Result<Lna, Sx127xError<SPI::Error>> {
        Ok(Lna::from(self.read(LNA).await?))
    }

    /// Masks an irq.
    ///
    /// See: datasheet section 4.1.2.4
    pub async fn mask_irq(&mut self, irq: IRQ) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS_MASK).await?;
        self.write(IRQ_FLAGS_MASK, byte | irq.mask()).await
    }

    /// Gets the modem status.
    ///
    /// See: datasheet section 2.0.2
    pub async fn modem_status(&mut self) -> Result<ModemStatus, Sx127xError<SPI::Error>> {
        Ok(ModemStatus::try_from(self.read(MODEM_STAT).await? & MODEM_STAT_MODEM_STATUS_MASK).map_err(|_| InvalidState)?)
    }

    /// TODO document
    pub async fn optimize_rx_response_if(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        if bandwidth != Bandwidth::Bw500kHz {
            self.write(IF_FREQ_2, match bandwidth {
                Bandwidth::Bw7_8kHz => 0x48,
                Bandwidth::Bw10_4kHz | Bandwidth::Bw15_6kHz | Bandwidth::Bw20_8kHz | Bandwidth::Bw31_25kHz | Bandwidth::Bw41_7kHz => 0x44,
                _ => 0x40
            }).await?;

            self.write(IF_FREQ_1, 0x00).await?;
        }
        Ok(())
    }

    /// Gets N bytes from the FIFO buffer, depending upon the `half_duplex` feature flag.
    ///
    /// See: datasheet figure 10
    pub async fn read_rx_data(&mut self) -> Result<[u8; PAYLOAD_SIZE], Sx127xError<SPI::Error>> {
        let reg_hop_channel = self.read(HOP_CHANNEL).await?;
        let crc_on_payload = get_bits(reg_hop_channel, HOP_CHANNEL_CRC_ON_PAYLOAD_MASK, HOP_CHANNEL_CRC_ON_PAYLOAD_OFFSET) == 1;

        let irq_flags_bits = self.read(IRQ_FLAGS).await? >> 4;
        let rx_packet_termination_ok = if crc_on_payload {
            irq_flags_bits & 0xf == 0
        } else {
            irq_flags_bits & 0xc == 0 && irq_flags_bits & 0x1 == 0
        };
        if !rx_packet_termination_ok {
            return Err(Sx127xError::PacketTermination)
        }

        let rx_fifo_addr = self.read(FIFO_RX_CURRENT_ADDR).await?;
        self.write(FIFO_ADDR_PTR, rx_fifo_addr).await?;

        let num_bytes = self.read(RX_NB_BYTES).await?;
        if num_bytes > PAYLOAD_SIZE as u8 {
            #[cfg(feature = "defmt")]
            error!("received {} bytes but buffer size is only {} bytes", num_bytes, PAYLOAD_SIZE);

            return Err(Sx127xError::InvalidPayloadLength)
        }

        let mut buffer = [0; PAYLOAD_SIZE];
        for i in 0..PAYLOAD_SIZE {
            let byte = self.read(FIFO).await?;
            buffer[i] = byte;
        }
        Ok(buffer)
    }

    /// Enables receive mode and searches for a preamble. If a `timeout` is specified, the device
    /// enter RXSINGLE mode, else RXCONTINUOUS mode.
    ///
    /// See: datasheet pages 40-42
    pub async fn receive(&mut self, timeout: Option<TimeoutSymbols>) -> Result<(), Sx127xError<SPI::Error>> {
        let device_mode = self.device_mode().await?;
        #[cfg(feature = "half_duplex")]
        {
            if device_mode == DeviceMode::RXSINGLE || device_mode == DeviceMode::RXCONTINUOUS || device_mode == DeviceMode::TX {
                return Err(InvalidState)
            }
        }
        #[cfg(not(feature = "half_duplex"))]
        {
            if device_mode == DeviceMode::RXSINGLE || device_mode == DeviceMode::RXCONTINUOUS {
                return Err(InvalidState)
            }
        }

        self.set_device_mode(DeviceMode::STDBY).await?;
        let mut mode = DeviceMode::RXCONTINUOUS;

        if let Some(timeout) = timeout {
            mode = DeviceMode::RXSINGLE;

            let mut modem_config_2 = self.read(MODEM_CONFIG_2).await?;
            set_bits(&mut modem_config_2, (timeout.0 >> 8) as u8, MODEM_CONFIG_2_SYMB_TIMEOUT_MASK, MODEM_CONFIG_2_SYMB_TIMEOUT_OFFSET);
            self.write(MODEM_CONFIG_2, modem_config_2).await?;

            self.write(SYMB_TIMEOUT_LSB, (timeout.0 & 0xff) as u8).await?;
        }

        self.write(FIFO_RX_BASE_ADDR, 0x00).await?;
        self.write(FIFO_ADDR_PTR, FIFO_RX_BASE_ADDR).await?;
        self.set_device_mode(mode).await
    }

    /// Gets the received signal strength indicator (RSSI).
    ///
    /// See: datasheet section 3.5.5
    pub async fn rssi(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.read(RSSI_VALUE).await
    }

    /// Gets the received signal strength indicator (RSSI) wideband measurement.
    pub async fn rssi_wideband(&mut self) -> Result<u8, Sx127xError<SPI::Error>> {
        self.read(RSSI_WIDEBAND).await
    }

    /// Gets the coding rate of the last header received.
    pub async fn rx_coding_rate(&mut self) -> Result<CodingRate, Sx127xError<SPI::Error>> {
        Ok(CodingRate::from(get_bits(self.read(MODEM_STAT).await?, MODEM_STAT_RX_CODING_RATE_MASK, MODEM_STAT_RX_CODING_RATE_OFFSET)))
    }

    /// Sets the bandwidth and then optimizes the sensitivity of the modem.
    ///
    /// See: datasheet section 4.1.1.4
    pub async fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, bandwidth as u8, MODEM_CONFIG_1_BW_MASK, MODEM_CONFIG_1_BW_OFFSET);
        self.write(MODEM_CONFIG_1, byte).await?;
        self.optimize_bandwidth().await
    }

    /// Sets the cyclic error coding rate.
    ///
    /// See: datasheet section 4.1.1.3
    pub async fn set_coding_rate(&mut self, coding_rate: CodingRate) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, coding_rate as u8, MODEM_CONFIG_1_CODING_RATE_MASK, MODEM_CONFIG_1_CODING_RATE_OFFSET);
        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets CRC generation and check on payload on/off.
    ///
    /// See: section 4.1.1.6
    pub async fn set_crc(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_2).await?;
        set_bits(&mut byte, on as u8, MODEM_CONFIG_2_RX_PAYLOAD_CRC_ON_MASK, MODEM_CONFIG_2_RX_PAYLOAD_CRC_ON_OFFSET);
        self.write(MODEM_CONFIG_2, byte).await
    }

    /// Sets the device mode.
    ///
    /// See: datasheet table 16
    pub async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(OP_MODE).await?;
        set_bits(&mut byte, device_mode as u8, OP_MODE_MODE_MASK, OP_MODE_MODE_OFFSET);
        self.write(OP_MODE, byte).await
    }

    /// Sets the carrier frequency. It's imperative that you check regulations for your area (e.g.
    /// 902-928 MHz for the United States).
    ///
    /// See: datasheet section 4.1.4
    pub async fn set_frequency(&mut self, hz: u32) -> Result<(), Sx127xError<SPI::Error>> {
        let frf = sx127x_common::calculate::frf(hz, FSTEP);
        self.write(FRF_MSB, (frf >> 16) as u8).await?;
        self.write(FRF_MID, (frf >> 8) as u8).await?;
        self.write(FRF_LSB, frf as u8).await?;

        if hz < LF_MAX_HZ {
            self.set_low_frequency_mode(true).await?;
        } else if hz > HF_MIN_HZ {
            self.set_low_frequency_mode(false).await?;
        }
        self.calibrate().await
    }

    /// Sets the header mode to explicit or implicit.
    ///
    /// See: datasheet section 4.1.1.6
    pub async fn set_header_mode(&mut self, mode: HeaderMode) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_1).await?;
        set_bits(&mut byte, mode as u8, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK, MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_OFFSET);
        self.write(MODEM_CONFIG_1, byte).await
    }

    /// Sets the symbol periods between frequency hops.
    ///
    /// See: datasheet section 4.1.1.8
    pub async fn set_hop_period(&mut self, period: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.write(HOP_PERIOD, period).await
    }

    /// Sets the invert IQ configuration.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn set_invert_iq(&mut self, invert_iq: InvertIQ) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(INVERT_IQ).await?;
        set_bits(&mut byte, invert_iq.rx_path as u8, INVERT_IQ_RX_MASK, INVERT_IQ_RX_OFFSET);
        set_bits(&mut byte, invert_iq.tx_path as u8, INVERT_IQ_TX_MASK, INVERT_IQ_TX_OFFSET);

        // optimize
        self.write(INVERT_IQ_2, if invert_iq.rx_path || invert_iq.tx_path { INVERT_IQ_2_ON } else { INVERT_IQ_2_OFF }).await?;
        self.write(INVERT_IQ, byte).await
    }

    /// Gets the invert IQ configuration.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn invert_iq(&mut self) -> Result<InvertIQ, Sx127xError<SPI::Error>> {
        Ok(InvertIQ::from(self.read(INVERT_IQ).await?))
    }

    /// Sets the gain and high frequency boost for the low noise receiver amplifier (LNA).
    ///
    /// See: datasheet page 110
    pub async fn set_lna(&mut self, lna: Lna) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(LNA).await?;
        set_bits(&mut byte, lna.boost_hf as u8, LNA_BOOST_HF_MASK, LNA_BOOST_HF_OFFSET);
        set_bits(&mut byte, lna.gain as u8, LNA_GAIN_MASK, LNA_GAIN_OFFSET);
        self.write(LNA, byte).await
    }

    /// Sets the low data rate optimization.Its use is mandated when the symbol duration exceeds
    /// 16ms.
    ///
    /// See: datasheet section 4.1.1.6
    pub async fn set_low_data_rate_optimize(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(MODEM_CONFIG_3).await?;
        set_bits(&mut byte, on as u8, MODEM_CONFIG_3_LOW_DATA_RATE_OPTIMIZE_MASK, MODEM_CONFIG_3_LOW_DATA_RATE_OPTIMIZE_OFFSET);
        self.write(MODEM_CONFIG_3, byte).await
    }

    /// Sets the maximum payload length.
    ///
    /// If header payload length exceeds value a header CRC error is generated. Allows filtering of
    /// packet with a bad size.
    pub async fn set_max_payload_length(&mut self, payload_length: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.write(MAX_PAYLOAD_LENGTH, payload_length).await
    }

    /// Sets the over-current protection (OCP) on/off.
    ///
    /// See: datasheet section 3.4.4
    pub async fn set_ocp(&mut self, ocp: Ocp) -> Result<(), Sx127xError<SPI::Error>> {
        self.write(OCP, ((ocp.on as u8) << OCP_ON_OFFSET) | calculate::ocp_trim(ocp.imax)).await
    }

    /// Sets the power amplifier (PA) to PA_HP on the PA_BOOST pin.
    ///
    /// See: datasheet section 3.4.2
    pub async fn set_power_amplifier(&mut self, pa: PowerAmplifier) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(PA_CONFIG).await?;
        set_bits(&mut byte, 1, PA_CONFIG_PA_SELECT_MASK, PA_CONFIG_PA_SELECT_OFFSET);
        set_bits(&mut byte, 7, PA_CONFIG_MAX_POWER_MASK, PA_CONFIG_MAX_POWER_OFFSET);
        set_bits(&mut byte, if pa.0 == 20 { 15 } else { pa.0 - 2 }, PA_CONFIG_OUTPUT_POWER_MASK, PA_CONFIG_OUTPUT_POWER_OFFSET);
        self.write(PA_CONFIG, byte).await?;

        self.write(PA_DAC, if pa.0 == 20 { 0x87 } else { 0x84 }).await?;
        self.set_ocp(Ocp::new(true, if pa.0 == 20 { 120 } else { 87 })).await
    }

    /// Gets the
    pub async fn power_amplifier(&mut self) -> Result<PowerAmplifier, Sx127xError<SPI::Error>> {
        if self.read(PA_DAC).await? == 0x87 {
            Ok(PowerAmplifier { 0: 20 })
        } else {
            let output_power = get_bits(self.read(PA_CONFIG).await?, PA_CONFIG_OUTPUT_POWER_MASK, PA_CONFIG_OUTPUT_POWER_OFFSET);
            Ok(PowerAmplifier { 0: output_power + 2 })
        }
    }

    /// Sets the rise/fall time of the power amplifier (PA).
    pub async fn set_pa_ramp(&mut self, pa_ramp: PARamp) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(PA_RAMP).await?;
        self.write(PA_RAMP, byte | pa_ramp as u8).await
    }

    /// Sets the power amplifier (PA) to PA_HF on the RFO_HF pin or PA_LF on the RFO_LF pin.
    ///
    /// See: datasheet section 3.4.2
    pub async fn set_pa_rfo(&mut self, pa_rfo: PARFO) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(PA_CONFIG).await?;
        set_bits(&mut byte, 0, PA_CONFIG_PA_SELECT_MASK, PA_CONFIG_PA_SELECT_OFFSET);
        if (-4..=0).contains(&pa_rfo.0) {
            set_bits(&mut byte, 0, PA_CONFIG_MAX_POWER_MASK, PA_CONFIG_MAX_POWER_OFFSET);
            set_bits(&mut byte, (pa_rfo.0 as f32 + 4.2) as u8, PA_CONFIG_OUTPUT_POWER_MASK, PA_CONFIG_OUTPUT_POWER_OFFSET);
        } else {
            set_bits(&mut byte, 7, PA_CONFIG_MAX_POWER_MASK, PA_CONFIG_MAX_POWER_OFFSET);
            set_bits(&mut byte, pa_rfo.0 as u8, PA_CONFIG_OUTPUT_POWER_MASK, PA_CONFIG_OUTPUT_POWER_OFFSET);
        }

        self.write(PA_DAC, 0x84).await?;
        self.set_ocp(Ocp::new(false, 0)).await
    }

    /// Sets the PLL bandwidth.
    ///
    /// See: datasheet section 3
    pub async fn set_pll_bandwidth(&mut self, bandwidth: PLLBandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(PLL).await?;
        set_bits(&mut byte, bandwidth as u8, PLL_PLL_BANDWIDTH_MASK, PLL_PLL_BANDWIDTH_OFFSET);
        self.write(PLL, byte).await
    }

    /// Sets the preamble length, minus 4 symbols of fixed overhead. A `length` of 6, which is the
    /// minimum valid preamble length, will yield a total of 10 symbols, and a `length` of 65535
    /// will yield a total of 65539 symbols.
    ///
    /// See: datasheet section 4.1.1.6
    pub async fn set_preamble_length(&mut self, preamble_length: PreambleLength) -> Result<(), Sx127xError<SPI::Error>> {
        self.write(PREAMBLE_MSB, (preamble_length.0 >> 8) as u8).await?;
        self.write(PREAMBLE_LSB, (preamble_length.0 & 0xff) as u8).await
    }

    /// Sets the spreading factor.
    ///
    /// See: datasheet section 4.1.1.2
    pub async fn set_spreading_factor(&mut self, spreading_factor: SpreadingFactor) -> Result<(), Sx127xError<SPI::Error>> {
        let mut modem_config_2 = self.read(MODEM_CONFIG_2).await?;
        set_bits(&mut modem_config_2, spreading_factor as u8, MODEM_CONFIG_2_SPREADING_FACTOR_MASK, MODEM_CONFIG_2_SPREADING_FACTOR_OFFSET);
        self.write(MODEM_CONFIG_2, modem_config_2).await?;

        let mut detect_optimize = self.read(DETECT_OPTIMIZE).await?;
        detect_optimize &= !DETECT_OPTIMIZE_DETECTION_OPTIMIZE_MASK;

        if spreading_factor == SpreadingFactor::Sf6 {
            self.set_header_mode(HeaderMode::Implicit).await?;
            detect_optimize |= DETECT_OPTIMIZE_DETECTION_OPTIMIZE_SF6;
            self.write(DETECTION_THRESHOLD, DETECTION_THRESHOLD_SF6).await?;
        } else {
            detect_optimize |= DETECT_OPTIMIZE_DETECTION_OPTIMIZE_SF7_TO_SF12;
            self.write(DETECTION_THRESHOLD, DETECTION_THRESHOLD_SF7_TO_SF12).await?;
        }
        self.write(DETECT_OPTIMIZE, detect_optimize).await
    }

    /// Sets the LoRa sync word.
    pub async fn set_sync_word(&mut self, sync_word: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.write(SYNC_WORD, sync_word).await
    }

    /// Sets the temperature monitor operation flag. This will switch to the FSK/OOK modem,
    /// set/unset the temp monitor flag, then switch back to the LoRa modem before returning.
    ///
    /// See: datasheet section 2.1.3.8
    pub async fn set_temp_monitor(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        self.set_long_range_mode(false).await?;
        let image_cal = self.read(IMAGE_CAL).await?;
        self.write(IMAGE_CAL, image_cal | !on as u8).await?;
        self.set_long_range_mode(true).await
    }

    /// Gets the spreading factor.
    pub async fn spreading_factor(&mut self) -> Result<SpreadingFactor, Sx127xError<SPI::Error>> {
        Ok(SpreadingFactor::from(get_bits(self.read(MODEM_CONFIG_2).await?, MODEM_CONFIG_2_SPREADING_FACTOR_MASK, MODEM_CONFIG_2_SPREADING_FACTOR_OFFSET)))
    }

    /// Calculates the symbol rate in chips/s.
    pub async fn symbol_rate(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let bandwidth = self.bandwidth().await?;
        let spreading_factor = self.spreading_factor().await?;

        Ok(calculate::symbol_rate(bandwidth.hz(), spreading_factor as u32) as u16)
    }

    /// Transmits a `payload` of up to 255 bytes. Will automatically transition to STDBY when done.
    ///
    /// See: datasheet figure 9
    pub async fn transmit(&mut self, payload: &[u8]) -> Result<(), Sx127xError<SPI::Error>> {
        let payload_len = payload.len();
        if payload_len > PAYLOAD_SIZE {
            return Err(Sx127xError::InvalidPayloadLength);
        }

        let device_mode = self.device_mode().await?;
        #[cfg(feature = "half_duplex")]
        {
            if device_mode == DeviceMode::RXSINGLE || device_mode == DeviceMode::RXCONTINUOUS {
                return Err(InvalidState)
            }
            self.write(FIFO_TX_BASE_ADDR, 0x00).await?;
        }
        #[cfg(not(feature = "half_duplex"))]
        {
            self.write(FIFO_TX_BASE_ADDR, 0x80).await?;
        }

        if device_mode == DeviceMode::TX {
            return Err(InvalidState)
        }

        self.set_device_mode(DeviceMode::STDBY).await?;
        self.write(FIFO_ADDR_PTR, FIFO_TX_BASE_ADDR).await?;
        for &byte in payload.iter().take(PAYLOAD_SIZE) {
            self.write(FIFO, byte).await?;
        }
        self.write(PAYLOAD_LENGTH, payload.len() as u8).await?;
        self.set_device_mode(DeviceMode::TX).await
    }

    /// Unmasks an irq.
    ///
    /// See: datasheet section 4.1.2.4
    pub async fn unmask_irq(&mut self, irq: IRQ) -> Result<(), Sx127xError<SPI::Error>> {
        let byte = self.read(IRQ_FLAGS_MASK).await?;
        self.write(IRQ_FLAGS_MASK, byte & !irq.mask()).await
    }

    /// Gets the number of valid headers received since last transition into Rx mode.
    pub async fn valid_rx_headers(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let msb = self.read(RX_HEADER_CNT_VALUE_MSB).await? as u16;
        let lsb = self.read(RX_HEADER_CNT_VALUE_LSB).await? as u16;
        Ok((msb << 8) | lsb)
    }

    /// Gets the number of valid packets received since last transition into Rx mode.
    pub async fn valid_rx_packets(&mut self) -> Result<u16, Sx127xError<SPI::Error>> {
        let msb = self.read(RX_PACKET_CNT_VALUE_MSB).await? as u16;
        let lsb = self.read(RX_PACKET_CNT_VALUE_LSB).await? as u16;
        Ok((msb << 8) | lsb)
    }

    // PRIVATE -------------------------------------------------------------------------------------

    async fn set_dio_mapping(&mut self, register: u8, value: u8, mask: u8, offset: u8) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(register).await?;
        set_bits(&mut byte, value, mask, offset);
        self.write(register, byte).await
    }

    // Selects the LoRa modem when `on` == true, and the FSK/OOK modem when `on` == false.
    async fn set_long_range_mode(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        // also clears the FIFO buffer
        self.set_device_mode(DeviceMode::SLEEP).await?;

        let mut op_mode = self.read(OP_MODE).await?;
        set_bits(&mut op_mode, on as u8, OP_MODE_LONG_RANGE_MODE_MASK, OP_MODE_LONG_RANGE_MODE_OFFSET);
        self.write(OP_MODE, op_mode).await?;

        self.set_device_mode(DeviceMode::STDBY).await
    }

    // see: errata section 2.1
    async fn optimize_bandwidth(&mut self) -> Result<(), Sx127xError<SPI::Error>> {
        if self.read(VERSION).await? != PRODUCTION_VERSION { return Ok(()) } // noop for engineering samples

        match self.frequency().await? {
            410_000_000..=525_000_000 => {
                self.write(HIGH_BW_OPTIMIZE_1, 0x02).await?;
                self.write(HIGH_BW_OPTIMIZE_2, 0x7f).await?;
            }
            862_000_000..=1_020_000_000 => {
                self.write(HIGH_BW_OPTIMIZE_1, 0x02).await?;
                self.write(HIGH_BW_OPTIMIZE_2, 0x64).await?;
            }
            _ => {
                self.write(HIGH_BW_OPTIMIZE_1, 0x03).await?;
            }
        }
        Ok(())
    }

    async fn optimize_rx_response_detect_optimize(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        let mut detect_optimize = self.read(DETECT_OPTIMIZE).await?;
        set_bits(&mut detect_optimize, if bandwidth == Bandwidth::Bw500kHz { 1 } else { 0 }, DETECT_OPTIMIZE_AUTOMATIC_IF_ON_MASK, DETECT_OPTIMIZE_AUTOMATIC_IF_ON_OFFSET);
        self.write(DETECT_OPTIMIZE, detect_optimize).await
    }

    async fn optimize_rx_response_frf_offset(&mut self, bandwidth: Bandwidth) -> Result<(), Sx127xError<SPI::Error>> {
        match bandwidth {
            Bandwidth::Bw7_8kHz => self.set_frequency_offset(7_8000).await,
            Bandwidth::Bw10_4kHz => self.set_frequency_offset(10_4000).await,
            Bandwidth::Bw15_6kHz => self.set_frequency_offset(15_6000).await,
            Bandwidth::Bw20_8kHz => self.set_frequency_offset(20_8000).await,
            Bandwidth::Bw31_25kHz => self.set_frequency_offset(31_2000).await,
            Bandwidth::Bw41_7kHz => self.set_frequency_offset(41_6000).await,
            _ => Ok(())
        }
    }

    async fn read(&mut self, addr: u8) -> Result<u8, Sx127xError<SPI::Error>> {
        self.spi.read(addr).await
    }

    async fn set_frequency_offset(&mut self, offset: i32) -> Result<(), Sx127xError<SPI::Error>> {
        let mut frequency = self.frequency().await?;
        if offset > 0 {
            frequency += offset as u32;
        } else {
            frequency -= offset as u32;
        }
        self.set_frequency(frequency).await
    }

    async fn set_low_frequency_mode(&mut self, on: bool) -> Result<(), Sx127xError<SPI::Error>> {
        let mut byte = self.read(OP_MODE).await?;
        set_bits(&mut byte, on as u8, OP_MODE_LOW_FREQUENCY_MODE_ON_MASK, OP_MODE_LOW_FREQUENCY_MODE_ON_OFFSET);
        self.write(OP_MODE, byte).await
    }

    async fn write(&mut self, addr: u8, data: u8) -> Result<(), Sx127xError<SPI::Error>> {
        self.spi.write(addr, data).await
    }
}