use crate::color::extractor::{ColorExtractingStrategy, ColorPalette, ThemeMode};
use crate::utils::image_utils::MyImage;
use std::path::Path;

pub struct NativeColorExtractor;

// pixels below this chroma value are "neutral" — greys, blacks, whites.
// they never become accent colors. they only inform bg/fg and mode detection.
// this is the single most important constant in this file.
const CHROMA_MIN: f64 = 0.09;

// ─────────────────────────────────────────────────────────────
// COLOR SPACE CONVERSIONS (sRGB ↔ OKLCH)
// ─────────────────────────────────────────────────────────────

// undo the gamma curve of sRGB before doing any math
fn linearize(channel: u8) -> f64 {
    let v = channel as f64 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

// re-apply gamma when converting back to sRGB for display
fn delinearize(v: f64) -> u8 {
    let clamped = v.clamp(0.0, 1.0);
    let encoded = if clamped <= 0.0031308 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

// sRGB (u8) → OKLCH (L, C, H)
//
// OKLCH is a perceptual color space where equal numerical distances
// look equally different to the human eye. much better than HSL for this.
//
//   L = lightness  (0.0 = black, 1.0 = white)
//   C = chroma     (0.0 = grey, ~0.4 = very vivid)
//   H = hue angle  (0–360°, like a color wheel)
fn rgb_to_oklch(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    // undo gamma
    let r = linearize(r);
    let g = linearize(g);
    let b = linearize(b);

    // linear RGB → XYZ (D65 illuminant, standard sRGB matrix)
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;

    // XYZ → LMS (cone responses)
    let l = 0.8189330101 * x + 0.3618667424 * y - 0.1288597137 * z;
    let m = 0.0329845436 * x + 0.9293118715 * y + 0.0361456387 * z;
    let s = 0.0482003018 * x + 0.2643662691 * y + 0.6338517070 * z;

    // cube root = perceptual uniformity
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // LMS → OKLab
    let ok_l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let ok_a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let ok_b = 0.0259040371 * l_ + 0.4072165026 * m_ - 0.4321120200 * s_;

    // OKLab → OKLCH (cartesian → polar)
    let chroma = (ok_a * ok_a + ok_b * ok_b).sqrt();
    let hue    = ok_b.atan2(ok_a).to_degrees().rem_euclid(360.0);

    (ok_l, chroma, hue)
}

// OKLCH → sRGB (u8) — reverse pipeline
fn oklch_to_rgb(l: f64, c: f64, h: f64) -> (u8, u8, u8) {
    let h_rad = h.to_radians();
    let ok_a  = c * h_rad.cos();
    let ok_b  = c * h_rad.sin();

    let l_ = l + 0.3963377774 * ok_a + 0.2158037573 * ok_b;
    let m_ = l - 0.1055613458 * ok_a - 0.0638541728 * ok_b;
    let s_ = l - 0.0894841775 * ok_a - 1.2914855480 * ok_b;

    let lv = l_ * l_ * l_;
    let mv = m_ * m_ * m_;
    let sv = s_ * s_ * s_;

    let r =  4.0767416621 * lv - 3.3077115913 * mv + 0.2309699292 * sv;
    let g = -1.2684380046 * lv + 2.6097574011 * mv - 0.3413193965 * sv;
    let b = -0.0041960863 * lv - 0.7034186147 * mv + 1.7076147010 * sv;

    (delinearize(r), delinearize(g), delinearize(b))
}

fn to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

// shortest angular distance between two hue angles, wraps around 0°/360°
fn hue_dist(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

// ─────────────────────────────────────────────────────────────
// THE EXTRACTOR
// ─────────────────────────────────────────────────────────────

impl ColorExtractingStrategy for NativeColorExtractor {
    fn extract(&self, image_path: &Path, _mode: ThemeMode) -> Result<ColorPalette, String> {

        // load image and downsample to 200×200 grid — doing OKLCH math on a
        // full 4K image would be pointlessly slow, 40k pixels is plenty
        let img = MyImage::load(image_path.to_string_lossy().to_string())
            .map_err(|e| e.to_string())?;
        let grid = img.to_sampled_grid();

        // convert every pixel to OKLCH — entering perceptual color space
        let mut all: Vec<(f64, f64, f64)> = Vec::with_capacity(200 * 200);
        for row in &grid {
            for &[r, g, b] in row {
                all.push(rgb_to_oklch(r, g, b));
            }
        }

        // ── SEPARATION ────────────────────────────────────────
        // hard split: neutrals vs chromatic pixels
        //
        // neutrals (greys, blacks, whites) = chroma < CHROMA_MIN
        //   → used ONLY for background, foreground, and mode detection
        //   → NEVER become accent colors
        //
        // chromatic = chroma >= CHROMA_MIN
        //   → these are the image's actual defining colors
        //   → all accent slots come from here
        //
        let neutrals: Vec<_> = all.iter().filter(|p| p.1 < CHROMA_MIN).copied().collect();
        let mut chromatic: Vec<_> = all.iter().filter(|p| p.1 >= CHROMA_MIN).copied().collect();

        // sort chromatic by chroma descending — most vivid pixels float to the top
        // this way the first candidate we find in a bucket is always the best one
        chromatic.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // ── MODE DETECTION ────────────────────────────────────
        // just average lightness over ALL pixels (not just neutrals)
        let avg_l = all.iter().map(|p| p.0).sum::<f64>() / all.len() as f64;
        let theme_mode = if avg_l > 0.5 { ThemeMode::Light } else { ThemeMode::Dark };

        // ── BACKGROUND & FOREGROUND ───────────────────────────
        // pull these from the neutral pool — the actual grey/black/white tones
        // if the image is so vivid it has almost no neutrals, fall back to all pixels

        // bg = darkest neutral (the "black" of the theme)
        let bg = neutrals.iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .or_else(|| all.iter().min_by(|a, b| a.0.partial_cmp(&b.0).unwrap()))
            .copied()
            .unwrap_or((0.12, 0.0, 0.0));

        // fg = brightest neutral (the "white" of the theme)
        let fg = neutrals.iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .or_else(|| all.iter().max_by(|a, b| a.0.partial_cmp(&b.0).unwrap()))
            .copied()
            .unwrap_or((0.92, 0.0, 0.0));

        // ── HUE BUCKETING ────────────────────────────────────
        // divide the hue wheel into 6 buckets of 60° each
        // one per standard terminal accent color (red, yellow, green, cyan, blue, magenta)
        //
        // for each bucket:
        //   1. find the most vivid (highest chroma) unused pixel in that hue range
        //   2. if the bucket is empty (that hue doesn't exist in this image),
        //      find the most vivid unused pixel closest in hue to this bucket's center
        //      → this is way better than falling back to grey
        //
        // we track `used` to avoid the same pixel winning two adjacent buckets
        let mut used = vec![false; chromatic.len()];
        let mut accents: Vec<(f64, f64, f64)> = Vec::with_capacity(6);

        for bucket in 0..6usize {
            let lo     = bucket as f64 * 60.0;
            let hi     = lo + 60.0;
            let center = lo + 30.0;

            // first try: best vivid pixel squarely in this hue range
            let primary = chromatic.iter().enumerate()
                .filter(|(i, px)| !used[*i] && px.2 >= lo && px.2 < hi)
                .max_by(|(_, a), (_, b)| a.1.partial_cmp(&b.1).unwrap());

            if let Some((idx, &best)) = primary {
                used[idx] = true;
                // small chroma boost — makes it pop a bit more in terminal themes
                accents.push((best.0, (best.1 * 1.2).min(0.4), best.2));
                continue;
            }

            // this hue range doesn't exist in the image
            // score remaining pixels by: chroma * hue_proximity
            // → prefers vivid colors that are also close to what we wanted
            // → much better than a grey placeholder
            let fallback = chromatic.iter().enumerate()
                .filter(|(i, _)| !used[*i])
                .max_by(|(_, a), (_, b)| {
                    let score_a = a.1 * (1.0 - hue_dist(a.2, center) / 180.0);
                    let score_b = b.1 * (1.0 - hue_dist(b.2, center) / 180.0);
                    score_a.partial_cmp(&score_b).unwrap()
                });

            if let Some((idx, &best)) = fallback {
                used[idx] = true;
                accents.push((best.0, (best.1 * 1.2).min(0.4), best.2));
            } else {
                // image is basically monochrome — synthesize a neutral for this slot
                // this path should almost never happen in practice
                accents.push((0.5, 0.0, center));
            }
        }

        // ── ASSEMBLE THE 16-COLOR PALETTE ────────────────────
        //
        // terminal color slot layout:
        //   0        → black (background)
        //   1–6      → red, yellow, green, cyan, blue, magenta (normal)
        //   7        → white (foreground)
        //   8        → bright black (comments, inactive text)
        //   9–14     → bright versions of 1–6
        //   15       → bright white
        //
        let mut colors: [String; 16] = Default::default();

        // slot 0: background
        let (r, g, b) = oklch_to_rgb(bg.0, bg.1, bg.2);
        colors[0] = to_hex(r, g, b);

        // slots 1–6: accent colors
        for (i, &(l, c, h)) in accents.iter().enumerate() {
            let (r, g, b) = oklch_to_rgb(l, c, h);
            colors[i + 1] = to_hex(r, g, b);
        }

        // slot 7: foreground
        let (r, g, b) = oklch_to_rgb(fg.0, fg.1, fg.2);
        colors[7] = to_hex(r, g, b);

        // slot 8: bright black — background shifted lighter
        // terminals render this as the "dim" color for comments etc.
        let (r, g, b) = oklch_to_rgb((bg.0 + 0.15).min(0.95), bg.1, bg.2);
        colors[8] = to_hex(r, g, b);

        // slots 9–14: bright accent variants — same hue and chroma, just lighter
        for (i, &(l, c, h)) in accents.iter().enumerate() {
            let (r, g, b) = oklch_to_rgb((l + 0.10).min(0.95), c, h);
            colors[i + 9] = to_hex(r, g, b);
        }

        // slot 15: bright white — fg pushed brighter, slightly desaturated
        let (r, g, b) = oklch_to_rgb((fg.0 + 0.06).min(1.0), fg.1 * 0.4, fg.2);
        colors[15] = to_hex(r, g, b);

        let background = colors[0].clone();
        let foreground = colors[7].clone();

        Ok(ColorPalette { mode: theme_mode, colors, background, foreground })
    }
}
