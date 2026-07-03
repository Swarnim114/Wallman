use crate::color::extractor::{ColorExtractingStrategy, ColorPalette, ThemeMode};
use crate::utils::image_utils::MyImage;
use std::path::Path;

pub struct NativeColorExtractor;

// only pixels above this chroma get into the accent pool
// below this = too grey/muted to be a defining color
const CHROMA_MIN: f64 = 0.09;

// two accent colors this close in hue are basically duplicates
const MIN_HUE_GAP: f64 = 30.0;

// the minimum noticeability score any pixel can have, regardless of color
// this is the "floor" — even pure black gets this much per pixel
//
// lower = blacks/greys need MORE coverage to show up in the palette
// higher = blacks/greys can dominate even with moderate coverage
//
// sensible range: 0.01 (very low, only extreme coverage wins)
//                 0.05 (default, black needs ~80% of image to beat vivid red)
//                 0.15 (high, even small dark regions influence the palette a lot)
const BASE_NOTICEABILITY: f64 = 0.01;

// ─────────────────────────────────────────────────────────────
// HUMAN NOTICEABILITY SCORE
// ─────────────────────────────────────────────────────────────

// how much would a human "notice" this one pixel?
//
// formula: 0.05 + chroma × bell_curve(lightness)
//
// the 0.05 floor is important — it means even pure black gets a non-zero
// score. so if 95% of an image is black, black's aggregate score is
// 38,000 × 0.05 = 1,900 which beats a handful of vivid pixels.
//
// the bell_curve peaks at L=0.5 and drops to 0 at L=0 and L=1,
// which models how mid-lightness colors pop more than extremes.
//
// example scores:
//   pure black  (L=0.02, C=0.00) → 0.05 + 0.0           = 0.050
//   dark navy   (L=0.18, C=0.06) → 0.05 + 0.06 × 0.77   = 0.096
//   vivid red   (L=0.45, C=0.28) → 0.05 + 0.28 × 0.99   = 0.327
//   yellow-green(L=0.78, C=0.22) → 0.05 + 0.22 × 0.83   = 0.233
//   near-white  (L=0.96, C=0.01) → 0.05 + 0.01 × 0.39   = 0.054
fn noticeability(l: f64, c: f64) -> f64 {
    let bell = (4.0 * l * (1.0 - l)).sqrt();
    BASE_NOTICEABILITY + c * bell
}

// compute the noticeability-weighted average OKLCH of a slice of pixels
// this answers: "what color does this region look like to a human overall?"
//
// used for bg (dark band average) and fg (bright band average)
// so that coverage counts — 10,000 black pixels pull the average towards
// black more than 100 vivid navy pixels pull it towards navy
fn weighted_avg_oklch(pixels: &[(f64, f64, f64)]) -> Option<(f64, f64, f64)> {
    let total_w: f64 = pixels.iter().map(|p| noticeability(p.0, p.1)).sum();
    if total_w == 0.0 { return None; }

    let avg_l = pixels.iter().map(|p| p.0 * noticeability(p.0, p.1)).sum::<f64>() / total_w;
    let avg_c = pixels.iter().map(|p| p.1 * noticeability(p.0, p.1)).sum::<f64>() / total_w;

    // circular mean for hue — handles the 0°/360° wraparound correctly
    // (simple average would give wrong answer for e.g. 350° and 10°)
    let sin_h: f64 = pixels.iter().map(|p| p.2.to_radians().sin() * noticeability(p.0, p.1)).sum::<f64>() / total_w;
    let cos_h: f64 = pixels.iter().map(|p| p.2.to_radians().cos() * noticeability(p.0, p.1)).sum::<f64>() / total_w;
    let avg_h = sin_h.atan2(cos_h).to_degrees().rem_euclid(360.0);

    Some((avg_l, avg_c, avg_h))
}

// ─────────────────────────────────────────────────────────────
// COLOR SPACE CONVERSIONS (sRGB ↔ OKLCH)
// ─────────────────────────────────────────────────────────────

// undo sRGB gamma before doing any math — raw u8 values aren't linear light
fn linearize(channel: u8) -> f64 {
    let v = channel as f64 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

// re-apply gamma when going back to screen values
fn delinearize(v: f64) -> u8 {
    let clamped = v.clamp(0.0, 1.0);
    let encoded = if clamped <= 0.0031308 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

// sRGB → OKLCH
// L = lightness (0–1), C = chroma (0 = grey, ~0.4 = vivid), H = hue (0–360°)
fn rgb_to_oklch(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = linearize(r);
    let g = linearize(g);
    let b = linearize(b);

    // linear RGB → XYZ (D65)
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;

    // XYZ → LMS
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

// OKLCH → sRGB — reverse path, needed for generating bright variants
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

// shortest angular distance between two hue angles, handles 0°/360° wrap
fn hue_dist(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 360.0;
    if d > 180.0 { 360.0 - d } else { d }
}

// ─────────────────────────────────────────────────────────────
// THE EXTRACTOR
// ─────────────────────────────────────────────────────────────

impl ColorExtractingStrategy for NativeColorExtractor {
    fn extract(&self, image_path: &Path, _mode: ThemeMode) -> Result<ColorPalette, String> {

        // load and downsample — 200×200 = 40k pixels, more than enough
        let img = MyImage::load(image_path.to_string_lossy().to_string())
            .map_err(|e| e.to_string())?;
        let grid = img.to_sampled_grid();

        let mut all: Vec<(f64, f64, f64)> = Vec::with_capacity(200 * 200);
        for row in &grid {
            for &[r, g, b] in row {
                all.push(rgb_to_oklch(r, g, b));
            }
        }

        // ── MODE DETECTION ─────────────────────────────────────
        let avg_l = all.iter().map(|p| p.0).sum::<f64>() / all.len() as f64;
        let theme_mode = if avg_l > 0.5 { ThemeMode::Light } else { ThemeMode::Dark };

        // ── BACKGROUND & FOREGROUND ────────────────────────────
        //
        // we split pixels into a dark band (bottom 20% by lightness) and a
        // bright band (top 20%), then compute the noticeability-weighted average
        // OKLCH of each band.
        //
        // this means COVERAGE counts — if the dark band is 95% pure black,
        // the weighted average will be very dark and nearly achromatic.
        // if it's 60% black + 40% dark navy, the navy pulls the average toward blue.
        // we're not picking one outlier pixel, we're asking "what does this band
        // look like overall to a human?"
        //
        let mut sorted_by_l: Vec<f64> = all.iter().map(|p| p.0).collect();
        sorted_by_l.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let dark_cutoff  = sorted_by_l[all.len() / 5];      // 20th percentile
        let light_cutoff = sorted_by_l[all.len() * 4 / 5];  // 80th percentile

        let dark_band:  Vec<_> = all.iter().filter(|p| p.0 <= dark_cutoff).copied().collect();
        let light_band: Vec<_> = all.iter().filter(|p| p.0 >= light_cutoff).copied().collect();

        let bg = weighted_avg_oklch(&dark_band).unwrap_or((0.12, 0.02, 250.0));
        let fg = weighted_avg_oklch(&light_band).unwrap_or((0.90, 0.02, 80.0));

        // ── ACCENT POOL ────────────────────────────────────────
        // only vivid pixels (C >= CHROMA_MIN) are candidates for accent colors
        let mut chromatic: Vec<_> = all.iter()
            .filter(|p| p.1 >= CHROMA_MIN)
            .copied()
            .collect();

        // ── BUCKET SCORING (coverage × noticeability) ──────────
        //
        // for each of the 6 hue buckets, sum up noticeability across all its pixels
        // this is the core formula the user described:
        //   bucket_score = Σ noticeability(L, C) for all chromatic pixels in that hue range
        //
        // a hue with 2000 moderately vivid pixels beats a hue with 10 very vivid pixels
        // because area × noticeability > raw vividness
        let mut bucket_scores = [0.0f64; 6];
        for px in &chromatic {
            let idx = (px.2 / 60.0) as usize % 6;
            bucket_scores[idx] += noticeability(px.0, px.1);
        }

        // process buckets in order of their score — most "humanly important" hue goes first
        // this matters for the fallback pool: if two buckets are empty and borrow from the
        // same remaining pixels, the higher-scoring bucket gets first pick
        let mut bucket_order: Vec<usize> = (0..6).collect();
        bucket_order.sort_by(|&a, &b| bucket_scores[b].partial_cmp(&bucket_scores[a]).unwrap());

        // sort chromatic by noticeability descending — best candidates first
        chromatic.sort_by(|a, b| {
            noticeability(b.0, b.1).partial_cmp(&noticeability(a.0, a.1)).unwrap()
        });

        let mut used = vec![false; chromatic.len()];
        // accents indexed by BUCKET (0=red, 1=yellow, 2=green, 3=cyan, 4=blue, 5=magenta)
        let mut accents: Vec<Option<(f64, f64, f64)>> = vec![None; 6];

        for &bucket in &bucket_order {
            let lo     = bucket as f64 * 60.0;
            let hi     = lo + 60.0;
            let center = lo + 30.0;

            // primary: most noticeable unused pixel in this hue range
            let primary = chromatic.iter().enumerate()
                .filter(|(i, px)| !used[*i] && px.2 >= lo && px.2 < hi)
                .max_by(|(_, a), (_, b)| {
                    noticeability(a.0, a.1).partial_cmp(&noticeability(b.0, b.1)).unwrap()
                });

            if let Some((idx, &best)) = primary {
                used[idx] = true;
                accents[bucket] = Some((best.0, (best.1 * 1.2).min(0.4), best.2));
                continue;
            }

            // this hue doesn't exist in the image — borrow from pool
            // score = noticeability × hue-proximity (prefer vivid AND close to what we wanted)
            let fallback = chromatic.iter().enumerate()
                .filter(|(i, _)| !used[*i])
                .max_by(|(_, a), (_, b)| {
                    let score_a = noticeability(a.0, a.1) * (1.0 - hue_dist(a.2, center) / 180.0);
                    let score_b = noticeability(b.0, b.1) * (1.0 - hue_dist(b.2, center) / 180.0);
                    score_a.partial_cmp(&score_b).unwrap()
                });

            if let Some((idx, &best)) = fallback {
                used[idx] = true;
                accents[bucket] = Some((best.0, (best.1 * 1.2).min(0.4), best.2));
            } else {
                // truly monochrome image — synthesize a neutral placeholder
                accents[bucket] = Some((0.5, 0.0, center));
            }
        }

        // flatten accents: unwrap each Option (they're all Some at this point)
        let mut accents: Vec<(f64, f64, f64)> = accents.into_iter()
            .map(|a| a.unwrap_or((0.5, 0.0, 0.0)))
            .collect();

        // ── DEDUPLICATION ──────────────────────────────────────
        // if two accent slots are within MIN_HUE_GAP degrees of each other,
        // they'll look almost identical in a terminal theme — replace the less
        // vivid one with the best remaining chromatic color that keeps enough separation
        //
        // this loop repeats until no more duplicates exist (usually 0–1 iterations)
        loop {
            let mut replaced = false;

            'pairs: for i in 0..accents.len() {
                for j in (i + 1)..accents.len() {
                    if hue_dist(accents[i].2, accents[j].2) < MIN_HUE_GAP {
                        // the less noticeable one gets replaced
                        let ni = noticeability(accents[i].0, accents[i].1);
                        let nj = noticeability(accents[j].0, accents[j].1);
                        let replace = if ni <= nj { i } else { j };

                        // find the best unused pixel that's far enough from all kept accents
                        let candidate = chromatic.iter().enumerate()
                            .filter(|(u_idx, _)| !used[*u_idx])
                            .filter(|(_, px)| {
                                accents.iter().enumerate()
                                    .filter(|(k, _)| *k != replace)
                                    .all(|(_, other)| hue_dist(px.2, other.2) >= MIN_HUE_GAP)
                            })
                            .max_by(|(_, a), (_, b)| {
                                noticeability(a.0, a.1).partial_cmp(&noticeability(b.0, b.1)).unwrap()
                            });

                        if let Some((u_idx, &best)) = candidate {
                            used[u_idx] = true;
                            accents[replace] = (best.0, (best.1 * 1.2).min(0.4), best.2);
                            replaced = true;
                            break 'pairs;
                        }
                        break 'pairs;
                    }
                }
            }

            if !replaced { break; }
        }

        // ── ASSEMBLE 16-COLOR PALETTE ──────────────────────────
        //
        // 0        → black  (background)
        // 1–6      → accent colors (one per hue bucket)
        // 7        → white  (foreground)
        // 8        → bright black (comments, inactive UI)
        // 9–14     → brighter versions of 1–6
        // 15       → bright white
        //
        let mut colors: [String; 16] = Default::default();

        let (r, g, b) = oklch_to_rgb(bg.0, bg.1, bg.2);
        colors[0] = to_hex(r, g, b);

        for (i, &(l, c, h)) in accents.iter().enumerate() {
            let (r, g, b) = oklch_to_rgb(l, c, h);
            colors[i + 1] = to_hex(r, g, b);
        }

        let (r, g, b) = oklch_to_rgb(fg.0, fg.1, fg.2);
        colors[7] = to_hex(r, g, b);

        // bright black: bg lightness bumped up — used for dimmed/comment text
        let (r, g, b) = oklch_to_rgb((bg.0 + 0.15).min(0.95), bg.1, bg.2);
        colors[8] = to_hex(r, g, b);

        // bright accents: same hue and chroma, just lighter
        for (i, &(l, c, h)) in accents.iter().enumerate() {
            let (r, g, b) = oklch_to_rgb((l + 0.10).min(0.95), c, h);
            colors[i + 9] = to_hex(r, g, b);
        }

        // bright white: fg pushed slightly brighter, chroma softened
        let (r, g, b) = oklch_to_rgb((fg.0 + 0.06).min(1.0), fg.1 * 0.4, fg.2);
        colors[15] = to_hex(r, g, b);

        let background = colors[0].clone();
        let foreground = colors[7].clone();

        // secondary background: bg shifted a bit lighter
        // this becomes the surface/panel color in a full theme (like Catppuccin's "surface0")
        let (r, g, b) = oklch_to_rgb((bg.0 + 0.09).min(0.95), bg.1, bg.2);
        let secondary_background = to_hex(r, g, b);

        // secondary foreground: fg shifted a bit dimmer
        // used for comments, subtext, anything that should feel "less important"
        let (r, g, b) = oklch_to_rgb((fg.0 - 0.12).max(0.05), fg.1 * 0.85, fg.2);
        let secondary_foreground = to_hex(r, g, b);

        Ok(ColorPalette {
            mode: theme_mode,
            colors,
            background,
            secondary_background,
            foreground,
            secondary_foreground,
        })
    }
}
