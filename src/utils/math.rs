/// Round an f32 to one decimal. To be used for rounding times only.
pub fn rounded_f32(n: f32) -> f32 {
    (n * 10.0).trunc() / 10.0
}
