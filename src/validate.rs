const PA_BOOST_MAX: u8 = 20;
const PA_BOOST_RANGE_MAX: u8 = 17;
const PA_BOOST_RANGE_MIN: u8 = 2;
const PA_RFO_MIN: i8 = -4;
const PA_RFO_MAX: i8 = 15;

pub(crate) fn validate_pa_boost(power: u8) -> bool {
    power == PA_BOOST_MAX || (power >= PA_BOOST_RANGE_MIN && power <= PA_BOOST_RANGE_MAX)
}

pub(crate) fn validate_pa_rfo(power: i8) -> bool {
    power >= PA_RFO_MIN && power <= PA_RFO_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_pa_boost_high() {
        assert!(!validate_pa_boost(PA_BOOST_MAX + 1));
    }

    #[test]
    fn validate_pa_boost_range_high() {
        assert!(!validate_pa_boost(PA_BOOST_RANGE_MAX + 1));
    }

    #[test]
    fn validate_pa_boost_range_low() {
        assert!(!validate_pa_boost(PA_BOOST_RANGE_MIN - 1));
    }

    #[test]
    fn validate_pa_boost_range_ok() {
        assert!(validate_pa_boost(PA_BOOST_RANGE_MAX - 3));
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