use crate::cli::ColorBoost;

pub(super) fn color_gains(boost: ColorBoost) -> (f32, f32, f32) {
    match boost {
        ColorBoost::None => (1.0, 1.0, 1.0),
        ColorBoost::Red => (1.28, 0.94, 0.94),
        ColorBoost::Green => (1.05, 1.85, 1.08),
        ColorBoost::Blue => (1.05, 1.12, 1.95),
    }
}

pub(super) fn apply_color_overdrive(r: &mut f32, g: &mut f32, b: &mut f32, boost: ColorBoost) {
    match boost {
        ColorBoost::None | ColorBoost::Red => {}
        ColorBoost::Green => {
            *r = *r * 1.10 + 18.0;
            *g = *g * 1.58 + 44.0;
            *b = *b * 1.16 + 20.0;

            let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
            let shoulder = ((lum - 120.0) / 110.0).clamp(0.0, 1.0);
            *r += shoulder * 20.0;
            *g += shoulder * 56.0;
            *b += shoulder * 26.0;
        }
        ColorBoost::Blue => {
            *r = *r * 1.10 + 16.0;
            *g = *g * 1.18 + 22.0;
            *b = *b * 1.62 + 48.0;

            let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
            let shoulder = ((lum - 118.0) / 105.0).clamp(0.0, 1.0);
            *r += shoulder * 18.0;
            *g += shoulder * 24.0;
            *b += shoulder * 62.0;
        }
    }
}

pub(super) fn apply_matrix_tone(r: &mut f32, g: &mut f32, b: &mut f32, boost: ColorBoost) {
    let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
    let (strength, tint_r, tint_g, tint_b, lift) = match boost {
        ColorBoost::Green => (0.44, 0.33, 1.30, 0.64, 19.0),
        ColorBoost::Blue => (0.44, 0.36, 0.98, 1.32, 19.0),
        ColorBoost::None | ColorBoost::Red => (0.22, 0.36, 1.12, 0.70, 9.0),
    };

    let target_r = lum * tint_r + lift * 0.25;
    let target_g = lum * tint_g + lift;
    let target_b = lum * tint_b + lift * 0.45;

    *r = *r * (1.0 - strength) + target_r * strength;
    *g = *g * (1.0 - strength) + target_g * strength;
    *b = *b * (1.0 - strength) + target_b * strength;

    let shadow = ((112.0 - lum) / 112.0).clamp(0.0, 1.0) * strength;
    *r *= 1.0 - shadow * 0.25;
    *g *= 1.0 + shadow * 0.18;
    *b *= 1.0 + shadow * 0.08;
}

pub(super) fn apply_highlight_recovery(r: &mut f32, g: &mut f32, b: &mut f32, boost: ColorBoost) {
    let max_c = (*r).max(*g).max(*b);
    let min_c = (*r).min(*g).min(*b);
    let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
    let clipping = ((max_c - 225.0) / 60.0).clamp(0.0, 1.0);
    if clipping <= 0.0 {
        return;
    }

    let whiteness = (1.0 - ((max_c - min_c) / 90.0).clamp(0.0, 1.0)).powf(0.75);
    let amount = (clipping * (0.70 + 0.30 * whiteness)).clamp(0.0, 1.0);
    let compress = 1.0 - amount * 0.55;

    compress_highlights(r, 215.0, compress);
    compress_highlights(g, 215.0, compress);
    compress_highlights(b, 215.0, compress);

    let (tint_r, tint_g, tint_b, offset) = match boost {
        ColorBoost::Green => (0.42, 1.00, 0.68, 18.0),
        ColorBoost::Blue => (0.46, 0.90, 1.00, 18.0),
        ColorBoost::None | ColorBoost::Red => (0.44, 0.98, 0.74, 14.0),
    };
    let tint_rv = lum * tint_r + offset * 0.32;
    let tint_gv = lum * tint_g + offset;
    let tint_bv = lum * tint_b + offset * 0.55;

    *r = *r * (1.0 - amount) + tint_rv * amount;
    *g = *g * (1.0 - amount) + tint_gv * amount;
    *b = *b * (1.0 - amount) + tint_bv * amount;
}

pub(super) fn apply_saturation(r: &mut f32, g: &mut f32, b: &mut f32, saturation: f32) {
    let lum = *r * 0.2126 + *g * 0.7152 + *b * 0.0722;
    *r = lum + (*r - lum) * saturation;
    *g = lum + (*g - lum) * saturation;
    *b = lum + (*b - lum) * saturation;
}

fn compress_highlights(channel: &mut f32, start: f32, factor: f32) {
    if *channel <= start {
        return;
    }
    *channel = start + (*channel - start) * factor;
}
