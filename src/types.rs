use sx127x_common::bits::get_bits;
use sx127x_common::error::Sx127xError;
use crate::registers;
use crate::types::PARamp::*;
use crate::validate;
use crate::validate::{RX_TIMEOUT_SYMBOLS_MAX, RX_TIMEOUT_SYMBOLS_MIN};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Bandwidth {
    Bw7_8kHz = 0x0,
    Bw10_4kHz = 0x1,
    Bw15_6kHz = 0x2,
    Bw20_8kHz = 0x3,
    Bw31_25kHz = 0x4,
    Bw41_7kHz = 0x5,
    Bw62_5kHz = 0x6,
    #[default]
    Bw125kHz = 0x7,
    Bw250kHz = 0x8,
    Bw500kHz = 0x9,
}
impl From<u8> for Bandwidth {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Bandwidth::Bw7_8kHz,
            0x1 => Bandwidth::Bw10_4kHz,
            0x2 => Bandwidth::Bw15_6kHz,
            0x3 => Bandwidth::Bw20_8kHz,
            0x4 => Bandwidth::Bw31_25kHz,
            0x5 => Bandwidth::Bw41_7kHz,
            0x6 => Bandwidth::Bw62_5kHz,
            0x8 => Bandwidth::Bw250kHz,
            0x9 => Bandwidth::Bw500kHz,
            _ => Bandwidth::Bw125kHz,
        }
    }
}
impl Bandwidth {
    pub(crate) fn hz(&self) -> u32 {
        match self {
            Bandwidth::Bw7_8kHz => 7_800,
            Bandwidth::Bw10_4kHz => 10_400,
            Bandwidth::Bw15_6kHz => 15_600,
            Bandwidth::Bw20_8kHz => 20_800,
            Bandwidth::Bw31_25kHz => 31_250,
            Bandwidth::Bw41_7kHz => 41_700,
            Bandwidth::Bw62_5kHz => 62_500,
            Bandwidth::Bw125kHz => 125_000,
            Bandwidth::Bw250kHz => 250_000,
            _ => 500_000
        }
    }

    pub(crate) fn khz(&self) -> f32 {
        self.hz() as f32 / 1000.0
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CodingRate {
    #[default]
    Cr4_5 = 0x1,
    Cr4_6 = 0x2,
    Cr4_7 = 0x3,
    Cr4_8 = 0x4,
}
impl From<u8> for CodingRate {
    fn from(value: u8) -> Self {
        match value {
            0x2 => CodingRate::Cr4_6,
            0x3 => CodingRate::Cr4_7,
            0x4 => CodingRate::Cr4_8,
            _ => CodingRate::Cr4_5,
        }
    }
}
impl Into<f32> for CodingRate {
    fn into(self) -> f32 {
        4f32 / (match self {
            CodingRate::Cr4_5 => 5f32,
            CodingRate::Cr4_6 => 6f32,
            CodingRate::Cr4_7 => 7f32,
            CodingRate::Cr4_8 => 8f32,
        })
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DeviceMode {
    SLEEP = 0x0,
    #[default]
    STDBY = 0x1,
    FSTX = 0x2,
    TX = 0x3,
    FSRX = 0x4,
    RXCONTINUOUS = 0x5,
    RXSINGLE = 0x6,
    CAD = 0x7
}
impl From<u8> for DeviceMode {
    fn from(value: u8) -> Self {
        match value {
            0x0 => DeviceMode::SLEEP,
            0x2 => DeviceMode::FSTX,
            0x3 => DeviceMode::TX,
            0x4 => DeviceMode::FSRX,
            0x5 => DeviceMode::RXCONTINUOUS,
            0x6 => DeviceMode::RXSINGLE,
            0x7 => DeviceMode::CAD,
            _ => DeviceMode::STDBY,
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dio0Signal {
    #[default]
    RxDone = 0x0,
    TxDone = 0x1,
    CadDone = 0x2,
    None = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dio1Signal {
    #[default]
    RxTimeout = 0x0,
    FhssChangeChannel = 0x1,
    CadDetected = 0x2,
    None = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dio2Signal {
    #[default]
    FhssChangeChannel = 0x0,
    None = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dio3Signal {
    #[default]
    CadDone = 0x0,
    ValidHeader = 0x1,
    PayloadCrcError = 0x2,
    None = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dio4Signal {
    #[default]
    CadDetected = 0x0,
    PllLock = 0x1,
    None = 0x3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dio5Signal {
    #[default]
    ModeReady = 0x0,
    ClkOut = 0x1,
    None = 0x3,
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum HeaderMode {
    #[default]
    Explicit = 0x0,
    Implicit = 0x1,
}
impl From<u8> for HeaderMode {
    fn from(value: u8) -> Self {
        match value {
            0x0 => HeaderMode::Explicit,
            _ => HeaderMode::Implicit,
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct HopChannel {
    pll_timeout: bool,
    crc_on_payload: bool,
    fhss_present_channel: u8
}
impl From<u8> for HopChannel {
    fn from(value: u8) -> Self {
        Self {
            pll_timeout: get_bits(value, registers::HOP_CHANNEL_PLL_TIMEOUT_MASK, registers::HOP_CHANNEL_PLL_TIMEOUT_OFFSET) == 1,
            crc_on_payload: get_bits(value, registers::HOP_CHANNEL_CRC_ON_PAYLOAD_MASK, registers::HOP_CHANNEL_CRC_ON_PAYLOAD_OFFSET) == 1,
            fhss_present_channel: get_bits(value, registers::HOP_CHANNEL_FHSS_PRESENT_CHANNEL_MASK, registers::HOP_CHANNEL_FHSS_PRESENT_CHANNEL_OFFSET)
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq)]
pub enum IRQ {
    CadDetected,
    FhssChangeChannel,
    CadDone,
    TxDone,
    ValidHeader,
    PayloadCrcError,
    RxDone,
    RxTimeout,
}

impl IRQ {
    pub(crate) fn mask(&self) -> u8 {
        match self {
            IRQ::CadDetected => registers::IRQ_FLAGS_CAD_DETECTED_MASK,
            IRQ::FhssChangeChannel => registers::IRQ_FLAGS_FHSS_CHANGE_CHANNEL_MASK,
            IRQ::CadDone => registers::IRQ_FLAGS_CAD_DONE_MASK,
            IRQ::TxDone => registers::IRQ_FLAGS_TX_DONE_MASK,
            IRQ::ValidHeader => registers::IRQ_FLAGS_VALID_HEADER_MASK,
            IRQ::PayloadCrcError => registers::IRQ_FLAGS_PAYLOAD_CRC_ERROR_MASK,
            IRQ::RxDone => registers::IRQ_FLAGS_RX_DONE_MASK,
            IRQ::RxTimeout => registers::IRQ_FLAGS_RX_TIMEOUT_MASK,
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct InvertIQ {
    pub rx_path: bool,
    pub tx_path: bool,
}
impl From<u8> for InvertIQ {
    fn from(value: u8) -> Self {
        Self {
            rx_path: get_bits(value, registers::INVERT_IQ_RX_MASK, 6) == 1,
            tx_path: get_bits(value, registers::INVERT_IQ_TX_MASK, 0) == 1,
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LnaGain {
    #[default]
    G1 = 0x1,
    G2 = 0x2,
    G3 = 0x3,
    G4 = 0x4,
    G5 = 0x5,
    G6 = 0x6
}
impl From<u8> for LnaGain {
    fn from(value: u8) -> Self {
        match value {
            0x2 => LnaGain::G2,
            0x3 => LnaGain::G3,
            0x4 => LnaGain::G4,
            0x5 => LnaGain::G5,
            0x6 => LnaGain::G6,
            _ => LnaGain::G1,
        }
    }
}

pub struct Lna {
    pub boost_hf: bool,
    pub gain: LnaGain,
}
impl Lna {
    pub fn new(boost_hf: bool, gain: LnaGain) -> Self {
        Self { boost_hf, gain }
    }
}
impl Default for Lna {
    fn default() -> Self {
        Self { boost_hf: false, gain: LnaGain::default() }
    }
}
impl From<u8> for Lna {
    fn from(value: u8) -> Self {
        Self {
            boost_hf: get_bits(value, registers::LNA_BOOST_HF_MASK, registers::LNA_BOOST_HF_OFFSET) == 1,
            gain: LnaGain::from(get_bits(value, registers::LNA_GAIN_MASK, registers::LNA_GAIN_OFFSET))
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ModemStatus {
    SignalDetected,
    SignalSynchronized,
    RxOnGoing,
    HeaderInfoValid,
    #[default]
    ModemClear,
}
impl From<u8> for ModemStatus {
    fn from(value: u8) -> Self {
        match value {
            registers::MODEM_STAT_MODEM_STATUS_SIGNAL_DETECTED => ModemStatus::SignalDetected,
            registers::MODEM_STAT_MODEM_STATUS_SIGNAL_SYNCHRONIZED => ModemStatus::SignalSynchronized,
            registers::MODEM_STAT_MODEM_STATUS_RX_ONGOING_MASK => ModemStatus::RxOnGoing,
            registers::MODEM_STAT_MODEM_STATUS_HEADER_INFO_VALID_MASK => ModemStatus::HeaderInfoValid,
            _ => ModemStatus::ModemClear,
        }
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Ocp {
    pub on: bool,
    pub imax: u8,
}
impl Ocp {
    pub fn new(on: bool, imax: u8) -> Self {
        Self { on, imax }
    }
}
impl Default for Ocp {
    fn default() -> Self {
        // TODO should these go in common since reg, masks and offsets are in there?
        Self { on: true, imax: 100 }
    }
}

// -------------------------------------------------------------------------------------------------
// TODO min/max getters?
pub struct PowerAmplifier(pub(crate) u8);
impl PowerAmplifier {
    pub fn new(power: u8) -> Result<Self, Sx127xError<()>> {
        if !validate::pa_power(power) {
            return Err(Sx127xError::InvalidInput)
        }
        Ok(PowerAmplifier(power))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PARamp {
    Ms3_4 = 0x0,
    Ms2 = 0x1,
    Ms1 = 0x2,
    Us500 = 0x3,
    Us250 = 0x4,
    Us125 = 0x5,
    Us100 = 0x6,
    Us62 = 0x7,
    Us50 = 0x8,
    #[default]
    Us40 = 0x9,
    Us31 = 0xa,
    Us25 = 0xb,
    Us20 = 0xc,
    Us15 = 0xd,
    Us12 = 0xe,
    Us10 = 0xf,
}
impl From<u8> for PARamp {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Ms3_4,
            0x1 => Ms2,
            0x2 => Ms1,
            0x3 => Us500,
            0x4 => Us250,
            0x5 => Us125,
            0x6 => Us100,
            0x7 => Us62,
            0x8 => Us50,
            0xa => Us31,
            0xb => Us25,
            0xc => Us20,
            0xd => Us15,
            0xe => Us12,
            0xf => Us10,
            _ => Us40,
        }
    }
}

pub struct PARFO(pub(crate) i8);

impl PARFO {
    pub fn new(power: i8) -> Result<Self, Sx127xError<()>> {
        if !validate::pa_rfo(power) {
            return Err(Sx127xError::InvalidInput)
        }
        Ok(PARFO(power))
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PLLBandwidth {
    Bw75kHz = 0x0,
    Bw150kHz = 0x1,
    Bw225kHz = 0x2,
    #[default]
    Bw300kHz = 0x3,
}

// -------------------------------------------------------------------------------------------------
pub struct PreambleLength(pub(crate) u16);
impl PreambleLength {
    pub fn new(length: u16) -> Result<Self, Sx127xError<()>> {
        if !validate::preamble_length(length) {
            return Err(Sx127xError::InvalidInput)
        }
        Ok(PreambleLength(length))
    }
}

// -------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SpreadingFactor {
    /// Only implicit header mode is possible with Sf6.
    Sf6 = 0x6,
    #[default]
    Sf7 = 0x7,
    Sf8 = 0x8,
    Sf9 = 0x9,
    Sf10 = 0xa,
    Sf11 = 0xb,
    Sf12 = 0xc,
}
impl From<u8> for SpreadingFactor {
    fn from(value: u8) -> Self {
        match value {
            0x6 => SpreadingFactor::Sf6,
            0x8 => SpreadingFactor::Sf8,
            0x9 => SpreadingFactor::Sf9,
            0xa => SpreadingFactor::Sf10,
            0xb => SpreadingFactor::Sf11,
            0xc => SpreadingFactor::Sf12,
            _ => SpreadingFactor::Sf7,
        }
    }
}

// -------------------------------------------------------------------------------------------------
pub struct TimeoutSymbols(pub(crate) u16);

impl TimeoutSymbols {
    pub fn new(timeout: u16) -> Result<Self, Sx127xError<()>> {
        if !validate::rx_timeout_symbols(timeout) {
            return Err(Sx127xError::InvalidInput)
        }
        Ok(TimeoutSymbols(timeout))
    }

    pub fn max() -> Self {
        TimeoutSymbols(RX_TIMEOUT_SYMBOLS_MAX)
    }

    pub fn min() -> Self {
        TimeoutSymbols(RX_TIMEOUT_SYMBOLS_MIN)
    }
}