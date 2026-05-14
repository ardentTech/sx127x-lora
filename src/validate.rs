const PA_HP: u8 = 20;
const PA_MAX: u8 = 17;
const PA_MIN: u8 = 2;
const PA_RFO_MIN: i8 = -4;
const PA_RFO_MAX: i8 = 15;
const PREAMBLE_LENGTH_MIN: u16 = 6;
pub const RX_TIMEOUT_SYMBOLS_MIN: u16 = 4;
pub const RX_TIMEOUT_SYMBOLS_MAX: u16 = 1023;

pub(crate) fn pa_power(power: u8) -> bool {
    power == PA_HP || (power >= PA_MIN && power <= PA_MAX)
}

pub(crate) fn pa_rfo(power: i8) -> bool {
    power >= PA_RFO_MIN && power <= PA_RFO_MAX
}

pub(crate) fn preamble_length(length: u16) -> bool {
    length >= PREAMBLE_LENGTH_MIN
}

pub(crate) fn rx_timeout_symbols(symbols: u16) -> bool {
    symbols >= RX_TIMEOUT_SYMBOLS_MIN && symbols <= RX_TIMEOUT_SYMBOLS_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_pa_power_high() {
        assert!(!pa_power(PA_HP + 1));
    }

    #[test]
    fn validate_pa_power_range_high() {
        assert!(!pa_power(PA_MAX + 1));
    }

    #[test]
    fn validate_pa_power_range_low() {
        assert!(!pa_power(PA_MIN - 1));
    }

    #[test]
    fn validate_pa_power_range_ok() {
        assert!(pa_power(PA_MAX - 3));
    }

    #[test]
    fn validate_pa_rfo_high() {
        assert!(!pa_rfo(PA_RFO_MAX + 1));
    }

    #[test]
    fn validate_pa_rfo_low() {
        assert!(!pa_rfo(PA_RFO_MIN - 1));
    }

    #[test]
    fn validate_pa_rfo_ok() {
        assert!(pa_rfo(PA_RFO_MIN + 2));
    }

    #[test]
    fn validate_preamble_length_low() {
        assert!(!preamble_length(PREAMBLE_LENGTH_MIN - 1));
    }

    #[test]
    fn validate_preamble_length_ok() {
        assert!(preamble_length(PREAMBLE_LENGTH_MIN));
    }

    #[test]
    fn validate_rx_timeout_symbols_low() {
        assert!(!rx_timeout_symbols(RX_TIMEOUT_SYMBOLS_MIN - 1));
    }

    #[test]
    fn validate_rx_timeout_symbols_high() {
        assert!(!rx_timeout_symbols(RX_TIMEOUT_SYMBOLS_MAX + 1));
    }

    #[test]
    fn validate_rx_timeout_symbols_ok() {
        assert!(rx_timeout_symbols(RX_TIMEOUT_SYMBOLS_MIN + 1));
    }
}