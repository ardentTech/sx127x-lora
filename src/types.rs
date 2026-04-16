use crate::registers;

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
            0x7 => Bandwidth::Bw125kHz,
            0x8 => Bandwidth::Bw250kHz,
            _ => Bandwidth::Bw500kHz,
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
            0x1 => CodingRate::Cr4_5,
            0x2 => CodingRate::Cr4_6,
            0x3 => CodingRate::Cr4_7,
            _ => CodingRate::Cr4_8,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeviceMode {
    SLEEP = 0x0,
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
            0x1 => DeviceMode::STDBY,
            0x2 => DeviceMode::FSTX,
            0x3 => DeviceMode::TX,
            0x4 => DeviceMode::FSRX,
            0x5 => DeviceMode::RXCONTINUOUS,
            0x6 => DeviceMode::RXSINGLE,
            _ => DeviceMode::CAD,
        }
    }
}

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

#[derive(Clone, Copy, PartialEq)]
pub enum Interrupt {
    CadDetected,
    FhssChangeChannel,
    CadDone,
    TxDone,
    ValidHeader,
    PayloadCrcError,
    RxDone,
    RxTimeout,
}
impl Interrupt {

    pub(crate) fn lsb_offset(&self) -> u8 {
        self.mask() >> 1
    }

    pub(crate) fn mask(&self) -> u8 {
        match self {
            Interrupt::CadDetected => registers::IRQ_FLAGS_CAD_DETECTED_MASK,
            Interrupt::FhssChangeChannel => registers::IRQ_FLAGS_FHSS_CHANGE_CHANNEL_MASK,
            Interrupt::CadDone => registers::IRQ_FLAGS_CAD_DONE_MASK,
            Interrupt::TxDone => registers::IRQ_FLAGS_TX_DONE_MASK,
            Interrupt::ValidHeader => registers::IRQ_FLAGS_VALID_HEADER_MASK,
            Interrupt::PayloadCrcError => registers::IRQ_FLAGS_PAYLOAD_CRC_ERROR_MASK,
            Interrupt::RxDone => registers::IRQ_FLAGS_RX_DONE_MASK,
            Interrupt::RxTimeout => registers::IRQ_FLAGS_RX_TIMEOUT_MASK,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InvertIQConfig {
    pub rx_path: bool,
    pub tx_path: bool,
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModemStatus {
    SignalDetected = 0x0,
    SignalSynchronized = 0x1,
    RxOnGoing = 0x4,
    HeaderInfoValid = 0x8,
    ModemClear = 0x16,
}
impl From<u8> for ModemStatus {
    fn from(value: u8) -> Self {
        match value {
            0x0 => ModemStatus::SignalDetected,
            0x1 => ModemStatus::SignalSynchronized,
            0x4 => ModemStatus::RxOnGoing,
            0x8 => ModemStatus::HeaderInfoValid,
            _ => ModemStatus::ModemClear,
        }
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

#[derive(Clone, Copy, PartialEq)]
pub enum RxStatus {
    ModemClear,
    HeaderInfoValid,
    RxOnGoing,
    SignalSynchronized,
    SignalDetected,
    Unknown,
}

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
            0x7 => SpreadingFactor::Sf7,
            0x8 => SpreadingFactor::Sf8,
            0x9 => SpreadingFactor::Sf9,
            0xa => SpreadingFactor::Sf10,
            0xb => SpreadingFactor::Sf11,
            _ => SpreadingFactor::Sf12,
        }
    }
}