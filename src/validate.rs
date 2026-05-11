const PA_HP: u8 = 20;
const PA_MAX: u8 = 17;
const PA_MIN: u8 = 2;
const PA_RFO_MIN: i8 = -4;
const PA_RFO_MAX: i8 = 15;

pub(crate) fn validate_pa_power(power: u8) -> bool {
    power == PA_HP || (power >= PA_MIN && power <= PA_MAX)
}

pub(crate) fn validate_pa_rfo(power: i8) -> bool {
    power >= PA_RFO_MIN && power <= PA_RFO_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_pa_boost_high() {
        assert!(!validate_pa_power(PA_HP + 1));
    }

    #[test]
    fn validate_pa_boost_range_high() {
        assert!(!validate_pa_power(PA_MAX + 1));
    }

    #[test]
    fn validate_pa_boost_range_low() {
        assert!(!validate_pa_power(PA_MIN - 1));
    }

    #[test]
    fn validate_pa_boost_range_ok() {
        assert!(validate_pa_power(PA_MAX - 3));
    }

    #[test]
    fn validate_pa_rfo_high() {
        assert!(!validate_pa_rfo(PA_RFO_MAX + 1));
    }

    #[test]
    fn validate_pa_rfo_low() {
        assert!(!validate_pa_rfo(PA_RFO_MIN - 1));
    }

    #[test]
    fn validate_pa_rfo_ok() {
        assert!(validate_pa_rfo(PA_RFO_MIN + 2));
    }
}