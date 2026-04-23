pub use sx127x_common::registers::*;

// RegOpMode ---------------------------------------------------------------------------------------
pub const OP_MODE_LONG_RANGE_MODE_MASK: u8 = 0x80;
pub const OP_MODE_ACCESS_SHARED_REG_MASK: u8 = 0x40;
pub const OP_MODE_LOW_FREQUENCY_MODE_ON_MASK: u8 = 0x08;
pub const OP_MODE_MODE_MASK: u8 = 0x07;

// -------------------------------------------------------------------------------------------------
pub const FIFO_ADDR_PTR: u8 = 0x0d;
pub const FIFO_TX_BASE_ADDR: u8 = 0x0e;
pub const FIFO_RX_BASE_ADDR: u8 = 0x0f;
pub const FIFO_RX_CURRENT_ADDR: u8 = 0x10;
pub const IRQ_FLAGS_MASK: u8 = 0x11;

// RegIrqFlags -------------------------------------------------------------------------------------
pub const IRQ_FLAGS: u8 = 0x12;
pub const IRQ_FLAGS_CAD_DETECTED_MASK: u8 = 0x01;
pub const IRQ_FLAGS_FHSS_CHANGE_CHANNEL_MASK: u8 = 0x02;
pub const IRQ_FLAGS_CAD_DONE_MASK: u8 = 0x04;
pub const IRQ_FLAGS_TX_DONE_MASK: u8 = 0x08;
pub const IRQ_FLAGS_VALID_HEADER_MASK: u8 = 0x10;
pub const IRQ_FLAGS_PAYLOAD_CRC_ERROR_MASK: u8 = 0x20;
pub const IRQ_FLAGS_RX_DONE_MASK: u8 = 0x40;
pub const IRQ_FLAGS_RX_TIMEOUT_MASK: u8 = 0x80;


pub const RX_NB_BYTES: u8 = 0x13;
pub const RX_HEADER_CNT_VALUE_MSB: u8 = 0x14;
pub const RX_HEADER_CNT_VALUE_LSB: u8 = 0x15;
pub const RX_PACKET_CNT_VALUE_MSB: u8 = 0x16;
pub const RX_PACKET_CNT_VALUE_LSB: u8 = 0x17;
pub const MODEM_STAT: u8 = 0x18;
pub const PKT_SNR_VALUE: u8 = 0x19;
pub const PKT_RSSI_VALUE: u8 = 0x1a;
pub const RSSI_VALUE: u8 = 0x1b;

// RegHopChannel -----------------------------------------------------------------------------------
pub const HOP_CHANNEL: u8 = 0x1c;
pub const HOP_CHANNEL_PLL_TIMEOUT_MASK: u8 = 0x80;
pub const HOP_CHANNEL_CRC_ON_PAYLOAD_MASK: u8 = 0x40;
pub const HOP_CHANNEL_FHSS_PRESENT_CHANNEL_MASK: u8 = 0x3f;

// RegModemConfig1 ---------------------------------------------------------------------------------
pub const MODEM_CONFIG_1: u8 = 0x1d;
pub const MODEM_CONFIG_1_BW_MASK: u8 = 0xf0;
pub const MODEM_CONFIG_1_CODING_RATE_MASK: u8 = 0x0e;
pub const MODEM_CONFIG_1_IMPLICIT_HEADER_MODE_ON_MASK: u8 = 0x01;

// RegModemConfig2 ---------------------------------------------------------------------------------
pub const MODEM_CONFIG_2: u8 = 0x1e;
pub const MODEM_CONFIG_2_SPREADING_FACTOR_MASK: u8 = 0xf0;
pub const MODEM_CONFIG_2_TX_CONTINUOUS_MODE_MASK: u8 = 0x08;
pub const MODEM_CONFIG_2_RX_PAYLOAD_CRC_ON_MASK: u8 = 0x04;
pub const MODEM_CONFIG_2_SYMB_TIMEOUT_MASK: u8 = 0x03;

// -------------------------------------------------------------------------------------------------
pub const SYMB_TIMEOUT_LSB: u8 = 0x1f;
pub const PREAMBLE_MSB: u8 = 0x20;
pub const PREAMBLE_LSB: u8 = 0x21;
pub const PAYLOAD_LENGTH: u8 = 0x22;
pub const MAX_PAYLOAD_LENGTH: u8 = 0x23;
pub const HOP_PERIOD: u8 = 0x24;
pub const FIFO_RX_BYTE_ADDR: u8 = 0x25;

// RegModemConfig3 ---------------------------------------------------------------------------------
pub const MODEM_CONFIG_3: u8 = 0x26;
pub const MODEM_CONFIG_3_LOW_DATA_RATE_OPTIMIZE_MASK: u8 = 0x08;

// -------------------------------------------------------------------------------------------------
pub const FEI_MSB: u8 = 0x28;
pub const FEI_MID: u8 = 0x29;
pub const FEI_LSB: u8 = 0x2a;
pub const RSSI_WIDEBAND: u8 = 0x2c;
pub const IF_FREQ_2: u8 = 0x2f;
pub const IF_FREQ_1: u8 = 0x30;

// RegDetectOptimize -------------------------------------------------------------------------------
pub const DETECT_OPTIMIZE: u8 = 0x31;
pub const DETECT_OPTIMIZE_DETECTION_OPTIMIZE_MASK: u8 = 0x07;

// RegInvertIQ -------------------------------------------------------------------------------------
pub const INVERT_IQ: u8 = 0x33;
pub const INVERT_IQ_RX_MASK: u8 = 0x40;
pub const INVERT_IQ_TX_MASK: u8 = 0x1;

// -------------------------------------------------------------------------------------------------
// RegImageCal: this is a FSK/OOK reg needed for calibration (hence only pub(crate))
pub(crate) const IMAGE_CAL: u8 = 0x3b;
pub const HIGH_BW_OPTIMIZE_1: u8 = 0x36;
pub const DETECTION_THRESHOLD: u8 = 0x37;
pub const SYNC_WORD: u8 = 0x39;
pub const HIGH_BW_OPTIMIZE_2: u8 = 0x3a;
pub const INVERT_IQ_2: u8 = 0x3b;