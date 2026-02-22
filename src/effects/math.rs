use image::RgbImage;

pub(super) fn random_shift(seed: u64, y: u64, max_shift: i32, salt: u64) -> i32 {
    if max_shift <= 0 {
        return 0;
    }
    let span = (max_shift as u64) * 2 + 1;
    let n = hash64(seed ^ (y.wrapping_mul(0x9E37_79B9_7F4A_7C15)) ^ salt) % span;
    n as i32 - max_shift
}

pub(super) fn random_unit(seed: u64, x: u64, y: u64, salt: u64) -> f32 {
    let bits = hash64(
        seed ^ (x.wrapping_mul(0x9E37_79B9_7F4A_7C15))
            ^ (y.wrapping_mul(0xBF58_476D_1CE4_E5B9))
            ^ salt,
    );
    let value = (bits >> 40) as u32;
    value as f32 / 16_777_215.0
}

fn hash64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub(super) fn sample_rgb(img: &RgbImage, x: i32, y: i32) -> [u8; 3] {
    let max_x = img.width().saturating_sub(1) as i32;
    let max_y = img.height().saturating_sub(1) as i32;

    let xx = x.clamp(0, max_x) as u32;
    let yy = y.clamp(0, max_y) as u32;
    img.get_pixel(xx, yy).0
}

pub(super) fn clamp_to_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}
