pub(crate) fn data_rate(symbol_rate: f32, spreading_factor: f32, coding_rate: f32) -> u16 {
    (symbol_rate * spreading_factor * coding_rate) as u16
}

pub(crate) fn fei_hz(fei: i32, bandwidth_khz: f32) -> f64 {
    ((fei * 2i32.pow(24) / (32 * 10i32.pow(6))) as f64) * ((bandwidth_khz / 500f32) as f64)
}

pub(crate) fn fei_ppm(hz: f64, frf: u32) -> f64 {
    hz * (10u32.pow(6) / frf) as f64
}

pub(crate) fn frf(hz: u32, fstep: f32) -> u32 {
    ((hz as f32) / fstep) as u32
}

pub(crate) fn ocp_trim(imax: u8) -> u8 {
    if imax < 130 {
        (imax - 45) / 5
    } else {
        (imax + 30) / 10
    }
}

pub(crate) fn symbol_rate(bandwidth: u32, spreading_factor: u32) -> u32 {
    bandwidth / 2u32.pow(spreading_factor)
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use super::*;

    #[test]
    fn data_rate_ok() {
        let res = data_rate(1953f32, 6f32, 0.8f32);
        assert_eq!(res, 9374u16);
    }

    #[test]
    fn fei_new_neg_fei_hz_ok() {
        let res = fei_hz(-2i32, 16f32);
        assert_relative_eq!(res, -0.032, epsilon=1e-3);
    }

    #[test]
    fn fei_new_pos_fei_hz_ok() {
        let res = fei_hz(8i32, 16f32);
        assert_relative_eq!(res, 0.128, epsilon=1e-3);
    }

    #[test]
    fn fei_new_neg_fei_ppm_ok() {
        let fei_hz = fei_hz(-4i32, 16f32);
        let fei_ppm = fei_ppm(fei_hz, 32u32);
        assert_relative_eq!(fei_ppm, -2000.0, epsilon=1e-3);
    }

    #[test]
    fn fei_new_pos_fei_ppm_ok() {
        let fei_hz = fei_hz(8i32, 16f32);
        let fei_ppm = fei_ppm(fei_hz, 32u32);
        assert_relative_eq!(fei_ppm, 4000.0, epsilon=1e-3);
    }

    #[test]
    fn frf_ok() {
        let res = frf(434_000_000, (32_000_000f32) / (2u32.pow(19) as f32));
        assert_eq!(res, 0x6c8000);
    }

    #[test]
    fn ocp_trim_high_ok() {
        let res = ocp_trim(140);
        assert_eq!(res, 17);
    }

    #[test]
    fn ocp_trim_low_ok() {
        let res = ocp_trim(129);
        assert_eq!(res, 16);
    }

    #[test]
    fn symbol_rate_ok() {
        let bandwidth = 125_000u32;
        let spreading_factor = 7u32;
        let symbol_rate = symbol_rate(bandwidth, spreading_factor);
        assert_eq!(symbol_rate, 976u32);
    }
}