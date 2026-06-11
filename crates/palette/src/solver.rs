use tiles::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Hsl,
    Hsv,
    Oklch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distribution {
    Uniform,
    Normal { sigma: f32 },
    Symmetric { concentration: f32, floor: f32 },
    Cluster,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionError {
    BucketCountZero,
    BucketCountExceedsColors,
}

impl std::fmt::Display for PartitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BucketCountZero => write!(f, "bucket count must be at least 1"),
            Self::BucketCountExceedsColors => {
                write!(f, "bucket count exceeds number of colors")
            }
        }
    }
}

impl std::error::Error for PartitionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HuePartitionError {
    BucketCountZero,
}

impl std::fmt::Display for HuePartitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BucketCountZero => write!(f, "bucket count must be at least 1"),
        }
    }
}

impl std::error::Error for HuePartitionError {}

pub struct LightnessPartition {
    num_buckets: usize,
    color_space: ColorSpace,
    distribution: Distribution,
    fuzziness: f32,
}

impl LightnessPartition {
    pub fn new(num_buckets: usize) -> Self {
        Self {
            num_buckets,
            color_space: ColorSpace::Oklch,
            distribution: Distribution::Uniform,
            fuzziness: 0.0,
        }
    }

    pub fn color_space(mut self, color_space: ColorSpace) -> Self {
        self.color_space = color_space;
        self
    }

    pub fn distribution(mut self, distribution: Distribution) -> Self {
        self.distribution = distribution;
        self
    }

    pub fn fuzziness(mut self, fuzziness: f32) -> Self {
        self.fuzziness = fuzziness;
        self
    }

    pub fn partition(self, colors: &[Color]) -> Result<Vec<Vec<Color>>, PartitionError> {
        if colors.is_empty() {
            if self.num_buckets == 0 {
                return Err(PartitionError::BucketCountZero);
            }
            return Ok(vec![Vec::new(); self.num_buckets]);
        }
        Ok(self.build(colors)?.sort(colors))
    }

    pub fn build(self, colors: &[Color]) -> Result<LightnessBuckets, PartitionError> {
        if self.num_buckets == 0 {
            return Err(PartitionError::BucketCountZero);
        }
        if !colors.is_empty() && self.num_buckets > colors.len() {
            return Err(PartitionError::BucketCountExceedsColors);
        }

        let lightnesses: Vec<f32> = colors
            .iter()
            .map(|c| lightness(c, self.color_space))
            .collect();
        let boundaries = self.compute_boundaries_from(&lightnesses);

        Ok(LightnessBuckets {
            boundaries,
            color_space: self.color_space,
            fuzziness: self.fuzziness,
        })
    }

    fn compute_boundaries_from(&self, lightnesses: &[f32]) -> Vec<f32> {
        let min_l = lightnesses.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_l = lightnesses
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);

        match self.distribution {
            Distribution::Uniform | Distribution::Normal { .. } => {
                compute_boundaries(self.num_buckets, self.distribution, min_l, max_l)
            }
            Distribution::Symmetric {
                concentration,
                floor,
            } => {
                let mut sorted: Vec<f32> = lightnesses.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let counts = symmetric_counts(self.num_buckets, sorted.len(), concentration, floor);
                rank_boundaries(&sorted, &counts, min_l, max_l)
            }
            Distribution::Cluster => {
                let mut sorted: Vec<f32> = lightnesses.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let breaks = jenks_breaks(&sorted, self.num_buckets);
                rank_boundaries(&sorted, &breaks, min_l, max_l)
            }
        }
    }
}

pub struct LightnessBuckets {
    boundaries: Vec<f32>,
    color_space: ColorSpace,
    fuzziness: f32,
}

impl LightnessBuckets {
    pub fn num_buckets(&self) -> usize {
        self.boundaries.len() - 1
    }

    pub fn sort(&self, colors: &[Color]) -> Vec<Vec<Color>> {
        let num_buckets = self.num_buckets();
        let mut buckets = vec![Vec::new(); num_buckets];

        for color in colors.iter() {
            let l = lightness(color, self.color_space);
            let idx = find_bucket(&self.boundaries, l);
            buckets[idx].push(*color);

            if self.fuzziness > 0.0 && num_buckets > 1 {
                let bucket_width = self.boundaries[idx + 1] - self.boundaries[idx];
                let fuzz_zone = self.fuzziness * bucket_width * 0.5;
                let dist_to_lower = l - self.boundaries[idx];
                let dist_to_upper = self.boundaries[idx + 1] - l;

                if dist_to_lower < fuzz_zone && idx > 0 {
                    buckets[idx - 1].push(*color);
                }
                if dist_to_upper < fuzz_zone && idx < num_buckets - 1 {
                    buckets[idx + 1].push(*color);
                }
            }
        }

        buckets
    }
}

pub struct HuePartition {
    color_space: ColorSpace,
    num_buckets: usize,
    chroma_threshold: f32,
    fuzziness: f32,
    offset: Option<f32>,
}

impl HuePartition {
    pub fn new(num_buckets: usize) -> Self {
        Self {
            color_space: ColorSpace::Oklch,
            num_buckets,
            chroma_threshold: 0.02,
            fuzziness: 0.0,
            offset: None,
        }
    }

    pub fn color_space(mut self, color_space: ColorSpace) -> Self {
        self.color_space = color_space;
        self
    }

    pub fn chroma_threshold(mut self, chroma_threshold: f32) -> Self {
        self.chroma_threshold = chroma_threshold;
        self
    }

    pub fn fuzziness(mut self, fuzziness: f32) -> Self {
        self.fuzziness = fuzziness;
        self
    }

    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn partition(self, colors: &[Color]) -> Result<Vec<Vec<Color>>, HuePartitionError> {
        Ok(self.build(colors)?.sort(colors))
    }

    pub fn build(self, colors: &[Color]) -> Result<HueBuckets, HuePartitionError> {
        if self.num_buckets == 0 {
            return Err(HuePartitionError::BucketCountZero);
        }

        let hues: Vec<f32> = colors
            .iter()
            .filter(|c| compute_chroma(c, self.color_space) >= self.chroma_threshold)
            .map(|c| hue(c, self.color_space))
            .collect();

        let offset = self
            .offset
            .unwrap_or_else(|| find_largest_gap_midpoint(&hues));

        Ok(HueBuckets {
            offset,
            num_buckets: self.num_buckets,
            color_space: self.color_space,
            chroma_threshold: self.chroma_threshold,
            fuzziness: self.fuzziness,
        })
    }
}

pub struct HueBuckets {
    offset: f32,
    num_buckets: usize,
    color_space: ColorSpace,
    chroma_threshold: f32,
    fuzziness: f32,
}

impl HueBuckets {
    pub fn num_buckets(&self) -> usize {
        self.num_buckets
    }

    pub fn sort(&self, colors: &[Color]) -> Vec<Vec<Color>> {
        let mut buckets = vec![Vec::new(); self.num_buckets];
        let mut achromatic = Vec::new();
        let bucket_width = 360.0 / self.num_buckets as f32;
        let fuzz_zone = self.fuzziness * bucket_width * 0.5;

        for color in colors.iter() {
            let chroma = compute_chroma(color, self.color_space);
            if chroma < self.chroma_threshold {
                achromatic.push(*color);
                continue;
            }

            let hue_val = hue(color, self.color_space);
            let rotated = (hue_val - self.offset - bucket_width * 0.5 + 360.0) % 360.0;
            let bucket_idx = ((rotated / 360.0) * self.num_buckets as f32).floor() as usize;
            let bucket_idx = bucket_idx.min(self.num_buckets - 1);
            buckets[bucket_idx].push(*color);

            if fuzz_zone > 0.0 && self.num_buckets > 1 {
                let bucket_start = bucket_idx as f32 * bucket_width;
                let bucket_end = bucket_start + bucket_width;
                let dist_to_lower = rotated - bucket_start;
                let dist_to_upper = bucket_end - rotated;

                if dist_to_lower < fuzz_zone && bucket_idx > 0 {
                    buckets[bucket_idx - 1].push(*color);
                } else if dist_to_lower < fuzz_zone && bucket_idx == 0 {
                    buckets[self.num_buckets - 1].push(*color);
                }

                if dist_to_upper < fuzz_zone && bucket_idx < self.num_buckets - 1 {
                    buckets[bucket_idx + 1].push(*color);
                } else if dist_to_upper < fuzz_zone && bucket_idx == self.num_buckets - 1 {
                    buckets[0].push(*color);
                }
            }
        }

        buckets.push(achromatic);
        buckets
    }
}

fn compute_boundaries(
    num_buckets: usize,
    distribution: Distribution,
    min_l: f32,
    max_l: f32,
) -> Vec<f32> {
    let range = max_l - min_l;
    if range <= 0.0 {
        return (0..=num_buckets)
            .map(|i| min_l + i as f32 * f32::EPSILON)
            .collect();
    }

    match distribution {
        Distribution::Uniform => (0..=num_buckets)
            .map(|i| min_l + (i as f32 / num_buckets as f32) * range)
            .collect(),
        Distribution::Normal { sigma } => {
            let cdf_lo = normal_cdf(0.0, 0.5, sigma);
            let cdf_hi = normal_cdf(1.0, 0.5, sigma);
            let mut boundaries = Vec::with_capacity(num_buckets + 1);
            for i in 0..=num_buckets {
                let t = i as f32 / num_buckets as f32;
                let warped = (normal_cdf(t, 0.5, sigma) - cdf_lo) / (cdf_hi - cdf_lo);
                boundaries.push(min_l + warped * range);
            }
            boundaries
        }
        Distribution::Symmetric { .. } | Distribution::Cluster => unreachable!(),
    }
}

fn find_bucket(boundaries: &[f32], l: f32) -> usize {
    let num_buckets = boundaries.len() - 1;
    for i in 1..num_buckets {
        if l < boundaries[i] {
            return i - 1;
        }
    }
    num_buckets - 1
}

fn rank_boundaries(sorted_vals: &[f32], counts: &[usize], min_l: f32, max_l: f32) -> Vec<f32> {
    let num_buckets = counts.len();
    let mut boundaries = Vec::with_capacity(num_buckets + 1);
    boundaries.push(min_l);

    let mut offset = 0;
    for b in 0..num_buckets - 1 {
        offset += counts[b];
        let last = sorted_vals[offset - 1];
        let first = sorted_vals[offset];
        boundaries.push((last + first) / 2.0);
    }

    boundaries.push(max_l);
    boundaries
}

fn jenks_breaks(sorted_vals: &[f32], num_buckets: usize) -> Vec<usize> {
    let n = sorted_vals.len();

    let mut variance = vec![vec![f64::MAX; n]; num_buckets];
    let mut backtrack = vec![vec![0usize; n]; num_buckets];

    let mut sum = 0.0_f64;
    let mut sum_sq = 0.0_f64;
    for i in 0..n {
        sum += sorted_vals[i] as f64;
        sum_sq += (sorted_vals[i] as f64) * (sorted_vals[i] as f64);
        let count = (i + 1) as f64;
        variance[0][i] = sum_sq - (sum * sum) / count;
    }

    for k in 1..num_buckets {
        for i in k..n {
            let mut s = 0.0_f64;
            let mut ssq = 0.0_f64;
            for j in (k..=i).rev() {
                s += sorted_vals[j] as f64;
                ssq += (sorted_vals[j] as f64) * (sorted_vals[j] as f64);
                let count = (i - j + 1) as f64;
                let ssd = ssq - (s * s) / count;
                let candidate = variance[k - 1][j - 1] + ssd;
                if candidate < variance[k][i] {
                    variance[k][i] = candidate;
                    backtrack[k][i] = j;
                }
            }
        }
    }

    let mut breaks = vec![0usize; num_buckets];
    breaks[num_buckets - 1] = n;
    let mut k = num_buckets - 1;
    let mut i = n - 1;
    while k > 0 {
        breaks[k - 1] = backtrack[k][i];
        i = backtrack[k][i] - 1;
        k -= 1;
    }

    let mut counts = Vec::with_capacity(num_buckets);
    let mut start = 0;
    for &end in &breaks {
        counts.push(end - start);
        start = end;
    }
    counts
}

fn symmetric_counts(
    num_buckets: usize,
    total: usize,
    concentration: f32,
    floor: f32,
) -> Vec<usize> {
    let center = (num_buckets - 1) as f32 / 2.0;
    let half = num_buckets / 2;
    let odd = num_buckets % 2 == 1;

    let unique = half + if odd { 1 } else { 0 };
    let mut weights = Vec::with_capacity(unique);
    for i in 0..unique {
        let dist = if center == 0.0 {
            0.0
        } else {
            ((i as f32 - center) / center).abs()
        };
        let w = floor + (1.0 - floor) * (1.0 - dist).powf(concentration);
        weights.push(w);
    }

    // Sum of full symmetric weight set
    let mut full_sum: f32 = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        if odd && i == half {
            full_sum += w;
        } else {
            full_sum += w * 2.0;
        }
    }

    // Assign counts: each bucket gets at least 1
    let available = total - num_buckets;
    let half_counts: Vec<usize> = weights
        .iter()
        .map(|&w| 1 + (w / full_sum * available as f32).floor() as usize)
        .collect();

    // Build full symmetric array
    let mut counts = vec![0usize; num_buckets];
    for i in 0..half {
        counts[i] = half_counts[i];
        counts[num_buckets - 1 - i] = half_counts[i];
    }
    if odd {
        counts[half] = half_counts[half];
    }

    // Fix remainder — distribute symmetrically from center outward
    let mut assigned: usize = counts.iter().sum();
    let mut ring = 0usize; // 0 = center (if odd), then pairs outward
    while assigned < total {
        if odd && ring == 0 {
            counts[half] += 1;
            assigned += 1;
            ring += 1;
        } else {
            let pair_idx = if odd { ring } else { ring + 1 };
            let left = half.wrapping_sub(pair_idx);
            let right = half + pair_idx - if odd { 0 } else { 1 };
            if right >= num_buckets || left >= num_buckets {
                ring = 0;
                continue;
            }
            if assigned + 2 <= total {
                counts[left] += 1;
                counts[right] += 1;
                assigned += 2;
            } else {
                // Only 1 left, give to center if odd
                if odd {
                    counts[half] += 1;
                    assigned += 1;
                } else {
                    counts[left] += 1;
                    counts[right] += 1;
                    assigned += 2;
                }
            }
            ring += 1;
            if (odd && ring > half) || (!odd && ring >= half) {
                ring = 0;
            }
        }
    }

    counts
}

fn normal_cdf(x: f32, mean: f32, sigma: f32) -> f32 {
    0.5 * (1.0 + erf((x - mean) / (sigma * std::f32::consts::SQRT_2)))
}

fn erf(x: f32) -> f32 {
    // Abramowitz & Stegun approximation 7.1.26
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * y
}

fn lightness(color: &Color, color_space: ColorSpace) -> f32 {
    match color_space {
        ColorSpace::Hsl => hsl_lightness(color),
        ColorSpace::Hsv => hsv_value(color),
        ColorSpace::Oklch => oklch_lightness(color),
    }
}

fn linear_to_srgb(l: f32) -> f32 {
    if l <= 0.0031308 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    }
}

fn hsl_lightness(color: &Color) -> f32 {
    let r = linear_to_srgb(color.r);
    let g = linear_to_srgb(color.g);
    let b = linear_to_srgb(color.b);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    (max + min) / 2.0
}

fn oklch_lightness(color: &Color) -> f32 {
    let l = 0.4122214708 * color.r + 0.5363325363 * color.g + 0.0514459929 * color.b;
    let m = 0.2119034982 * color.r + 0.6806995451 * color.g + 0.1073969566 * color.b;
    let s = 0.0883024619 * color.r + 0.2817188376 * color.g + 0.6299787005 * color.b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_
}

fn hsv_value(color: &Color) -> f32 {
    let r = linear_to_srgb(color.r);
    let g = linear_to_srgb(color.g);
    let b = linear_to_srgb(color.b);
    r.max(g).max(b)
}

fn hue(color: &Color, color_space: ColorSpace) -> f32 {
    match color_space {
        ColorSpace::Hsl | ColorSpace::Hsv => rgb_hue(color),
        ColorSpace::Oklch => oklch_hue(color),
    }
}

fn compute_chroma(color: &Color, color_space: ColorSpace) -> f32 {
    match color_space {
        ColorSpace::Hsl | ColorSpace::Hsv => {
            let r = linear_to_srgb(color.r);
            let g = linear_to_srgb(color.g);
            let b = linear_to_srgb(color.b);
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            max - min
        }
        ColorSpace::Oklch => {
            let l = 0.4122214708 * color.r + 0.5363325363 * color.g + 0.0514459929 * color.b;
            let m = 0.2119034982 * color.r + 0.6806995451 * color.g + 0.1073969566 * color.b;
            let s = 0.0883024619 * color.r + 0.2817188376 * color.g + 0.6299787005 * color.b;

            let l_ = l.cbrt();
            let m_ = m.cbrt();
            let s_ = s.cbrt();

            let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
            let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;
            (a * a + b * b).sqrt()
        }
    }
}

fn rgb_hue(color: &Color) -> f32 {
    let r = linear_to_srgb(color.r);
    let g = linear_to_srgb(color.g);
    let b = linear_to_srgb(color.b);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta < 1e-6 {
        return 0.0;
    }

    let h = if (max - r).abs() < 1e-6 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < 1e-6 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    (h + 360.0) % 360.0
}

fn oklch_hue(color: &Color) -> f32 {
    let l = 0.4122214708 * color.r + 0.5363325363 * color.g + 0.0514459929 * color.b;
    let m = 0.2119034982 * color.r + 0.6806995451 * color.g + 0.1073969566 * color.b;
    let s = 0.0883024619 * color.r + 0.2817188376 * color.g + 0.6299787005 * color.b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    let h = b.atan2(a).to_degrees();
    (h + 360.0) % 360.0
}

fn find_largest_gap_midpoint(hues: &[f32]) -> f32 {
    if hues.len() <= 1 {
        return 0.0;
    }

    let mut sorted: Vec<f32> = hues.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut max_gap = 0.0_f32;
    let mut gap_start = 0.0_f32;

    for i in 1..sorted.len() {
        let gap = sorted[i] - sorted[i - 1];
        if gap > max_gap {
            max_gap = gap;
            gap_start = sorted[i - 1];
        }
    }

    // Wrap-around gap
    let wrap_gap = (360.0 - sorted[sorted.len() - 1]) + sorted[0];
    if wrap_gap > max_gap {
        gap_start = sorted[sorted.len() - 1];
        max_gap = wrap_gap;
    }

    (gap_start + max_gap / 2.0) % 360.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- builder tests ---

    #[test]
    fn lightness_builder_defaults() {
        let black = Color::hex(0x000000);
        let white = Color::hex(0xFFFFFF);
        let colors = vec![black, white];
        let result = LightnessPartition::new(2).partition(&colors).unwrap();
        assert_eq!(result[0], vec![black]);
        assert_eq!(result[1], vec![white]);
    }

    #[test]
    fn lightness_builder_color_space() {
        let black = Color::hex(0x000000);
        let white = Color::hex(0xFFFFFF);
        let colors = vec![black, white];
        let result = LightnessPartition::new(2)
            .color_space(ColorSpace::Hsv)
            .partition(&colors)
            .unwrap();
        assert_eq!(result[0], vec![black]);
        assert_eq!(result[1], vec![white]);
    }

    #[test]
    fn lightness_builder_distribution() {
        let colors: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let uniform = LightnessPartition::new(5).partition(&colors).unwrap();
        let normal = LightnessPartition::new(5)
            .distribution(Distribution::Normal { sigma: 0.3 })
            .partition(&colors)
            .unwrap();
        assert!(normal[2].len() > uniform[2].len());
    }

    #[test]
    fn lightness_builder_fuzziness() {
        let colors: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let no_fuzz = LightnessPartition::new(5).partition(&colors).unwrap();
        let with_fuzz = LightnessPartition::new(5)
            .fuzziness(0.5)
            .partition(&colors)
            .unwrap();
        let total_no: usize = no_fuzz.iter().map(|b| b.len()).sum();
        let total_with: usize = with_fuzz.iter().map(|b| b.len()).sum();
        assert!(total_with > total_no);
    }

    #[test]
    fn lightness_builder_error_zero_buckets() {
        let colors = vec![Color::hex(0x000000)];
        let result = LightnessPartition::new(0).partition(&colors);
        assert_eq!(result, Err(PartitionError::BucketCountZero));
    }

    #[test]
    fn lightness_builder_error_exceeds_colors() {
        let colors = vec![Color::hex(0xFF0000), Color::hex(0x00FF00)];
        let result = LightnessPartition::new(3).partition(&colors);
        assert_eq!(result, Err(PartitionError::BucketCountExceedsColors));
    }

    #[test]
    fn hue_builder_defaults() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
        ];
        let result = HuePartition::new(3).partition(&colors).unwrap();
        assert_eq!(result.len(), 4);
        let chromatic_total: usize = result[..3].iter().map(|b| b.len()).sum();
        assert_eq!(chromatic_total, 3);
        assert!(result[3].is_empty());
    }

    #[test]
    fn hue_builder_color_space() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
        ];
        let result = HuePartition::new(3)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        let chromatic_total: usize = result[..3].iter().map(|b| b.len()).sum();
        assert_eq!(chromatic_total, 3);
    }

    #[test]
    fn hue_builder_chroma_threshold() {
        let colors = vec![Color::hex(0xFF0000), Color::hex(0x808080)];
        let low_thresh = HuePartition::new(2)
            .chroma_threshold(0.01)
            .partition(&colors)
            .unwrap();
        let high_thresh = HuePartition::new(2)
            .chroma_threshold(0.99)
            .partition(&colors)
            .unwrap();
        assert_eq!(high_thresh[2].len(), 2);
        assert_eq!(low_thresh[2].len(), 1);
    }

    #[test]
    fn hue_builder_fuzziness() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0xFF4400),
            Color::hex(0x00FF00),
            Color::hex(0x00FF44),
            Color::hex(0x0000FF),
            Color::hex(0x4400FF),
        ];
        let no_fuzz = HuePartition::new(3).partition(&colors).unwrap();
        let with_fuzz = HuePartition::new(3)
            .fuzziness(1.0)
            .partition(&colors)
            .unwrap();
        let total_no: usize = no_fuzz[..3].iter().map(|b| b.len()).sum();
        let total_with: usize = with_fuzz[..3].iter().map(|b| b.len()).sum();
        assert!(total_with > total_no);
    }

    #[test]
    fn hue_builder_error_zero_buckets() {
        let colors = vec![Color::hex(0xFF0000)];
        let result = HuePartition::new(0).partition(&colors);
        assert_eq!(result, Err(HuePartitionError::BucketCountZero));
    }

    // --- sort tests ---

    #[test]
    fn lightness_sort_matches_partition() {
        let colors: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let partition_result = LightnessPartition::new(4).partition(&colors).unwrap();
        let buckets = LightnessPartition::new(4).build(&colors).unwrap();
        let sort_result = buckets.sort(&colors);
        assert_eq!(partition_result, sort_result);
    }

    #[test]
    fn lightness_sort_subset_uses_full_boundaries() {
        let full: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let buckets = LightnessPartition::new(4).build(&full).unwrap();
        let dark: Vec<Color> = full[..5].to_vec();
        let result = buckets.sort(&dark);
        assert!(result[3].is_empty());
        let total: usize = result.iter().map(|b| b.len()).sum();
        assert_eq!(total, 5);
    }

    #[test]
    fn lightness_sort_preserves_all_colors() {
        let colors: Vec<Color> = (0..30)
            .map(|i| {
                let v = (i as f32 / 29.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let buckets = LightnessPartition::new(5).build(&colors).unwrap();
        let result = buckets.sort(&colors);
        let total: usize = result.iter().map(|b| b.len()).sum();
        assert_eq!(total, 30);
    }

    #[test]
    fn lightness_sort_with_fuzziness() {
        let colors: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let no_fuzz = LightnessPartition::new(4).build(&colors).unwrap();
        let with_fuzz = LightnessPartition::new(4)
            .fuzziness(0.5)
            .build(&colors)
            .unwrap();
        let total_no: usize = no_fuzz.sort(&colors).iter().map(|b| b.len()).sum();
        let total_with: usize = with_fuzz.sort(&colors).iter().map(|b| b.len()).sum();
        assert!(total_with > total_no);
    }

    #[test]
    fn hue_sort_matches_partition() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
            Color::hex(0x808080),
        ];
        let partition_result = HuePartition::new(3).offset(0.0).partition(&colors).unwrap();
        let buckets = HuePartition::new(3).offset(0.0).build(&colors).unwrap();
        let sort_result = buckets.sort(&colors);
        assert_eq!(partition_result, sort_result);
    }

    #[test]
    fn hue_sort_subset_uses_full_offset() {
        let full = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
            Color::hex(0xFFFF00),
            Color::hex(0xFF00FF),
            Color::hex(0x00FFFF),
        ];
        let buckets = HuePartition::new(3).build(&full).unwrap();
        let subset = vec![Color::hex(0xFF0000), Color::hex(0x0000FF)];
        let result = buckets.sort(&subset);
        let chromatic_total: usize = result[..3].iter().map(|b| b.len()).sum();
        assert_eq!(chromatic_total, 2);
        let non_empty: Vec<_> = result[..3].iter().filter(|b| !b.is_empty()).collect();
        assert_eq!(non_empty.len(), 2);
    }

    #[test]
    fn hue_sort_preserves_all_colors() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
            Color::hex(0x808080),
            Color::hex(0x333333),
        ];
        let buckets = HuePartition::new(4).build(&colors).unwrap();
        let result = buckets.sort(&colors);
        let total: usize = result.iter().map(|b| b.len()).sum();
        assert_eq!(total, 5);
    }

    #[test]
    fn hue_sort_achromatic_to_last_bucket() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x808080),
            Color::hex(0x000000),
        ];
        let buckets = HuePartition::new(3).build(&colors).unwrap();
        let result = buckets.sort(&colors);
        assert_eq!(result.len(), 4);
        assert_eq!(result[3].len(), 2);
    }

    #[test]
    fn hue_sort_with_fuzziness() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0xFF4400),
            Color::hex(0x00FF00),
            Color::hex(0x00FF44),
            Color::hex(0x0000FF),
            Color::hex(0x4400FF),
        ];
        let no_fuzz = HuePartition::new(3).offset(0.0).build(&colors).unwrap();
        let with_fuzz = HuePartition::new(3)
            .offset(0.0)
            .fuzziness(1.0)
            .build(&colors)
            .unwrap();
        let total_no: usize = no_fuzz.sort(&colors)[..3].iter().map(|b| b.len()).sum();
        let total_with: usize = with_fuzz.sort(&colors)[..3].iter().map(|b| b.len()).sum();
        assert!(total_with > total_no);
    }

    #[test]
    fn lightness_buckets_num_buckets() {
        let colors: Vec<Color> = (0..10)
            .map(|i| {
                let v = (i as f32 / 9.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let buckets = LightnessPartition::new(7).build(&colors).unwrap();
        assert_eq!(buckets.num_buckets(), 7);
    }

    #[test]
    fn hue_buckets_num_buckets() {
        let colors = vec![Color::hex(0xFF0000), Color::hex(0x00FF00)];
        let buckets = HuePartition::new(6).build(&colors).unwrap();
        assert_eq!(buckets.num_buckets(), 6);
    }

    // --- lightness partition tests ---

    #[test]
    fn empty_input_returns_empty_buckets() {
        let result = LightnessPartition::new(3).partition(&[]).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|b| b.is_empty()));
    }

    #[test]
    fn black_goes_to_first_bucket() {
        let black = Color::hex(0x000000);
        let colors = vec![black, Color::hex(0xFFFFFF), Color::hex(0x808080)];
        let result = LightnessPartition::new(3)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        assert!(result[0].contains(&black));
    }

    #[test]
    fn white_goes_to_last_bucket() {
        let white = Color::hex(0xFFFFFF);
        let colors = vec![Color::hex(0x000000), white, Color::hex(0x808080)];
        let result = LightnessPartition::new(3)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        assert!(result[2].contains(&white));
    }

    #[test]
    fn partitions_dark_mid_light() {
        let dark = Color::hex(0x111111);
        let mid = Color::hex(0x888888);
        let light = Color::hex(0xEEEEEE);
        let colors = vec![dark, mid, light];
        let result = LightnessPartition::new(3)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        assert!(result[0].contains(&dark));
        assert!(result[1].contains(&mid));
        assert!(result[2].contains(&light));
    }

    #[test]
    fn single_bucket_collects_all() {
        let colors = vec![
            Color::hex(0x000000),
            Color::hex(0x808080),
            Color::hex(0xFFFFFF),
        ];
        let result = LightnessPartition::new(1)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 3);
    }

    #[test]
    fn dark_and_light_in_correct_buckets() {
        let white = Color::hex(0xFFFFFF);
        let black = Color::hex(0x000000);
        let light_gray = Color::hex(0xCCCCCC);
        let dark_gray = Color::hex(0x111111);
        let colors = vec![white, black, light_gray, dark_gray];
        let result = LightnessPartition::new(2)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        assert!(result[0].contains(&black));
        assert!(result[0].contains(&dark_gray));
        assert!(result[1].contains(&white));
        assert!(result[1].contains(&light_gray));
    }

    #[test]
    fn normal_distribution_middle_bucket_wider() {
        let boundaries_uniform = compute_boundaries(5, Distribution::Uniform, 0.0, 1.0);
        let boundaries_normal =
            compute_boundaries(5, Distribution::Normal { sigma: 0.2 }, 0.0, 1.0);
        let mid_uniform = boundaries_uniform[3] - boundaries_uniform[2];
        let mid_normal = boundaries_normal[3] - boundaries_normal[2];
        assert!(mid_normal > mid_uniform);
    }

    #[test]
    fn normal_distribution_edge_buckets_narrower() {
        let boundaries_uniform = compute_boundaries(5, Distribution::Uniform, 0.0, 1.0);
        let boundaries_normal =
            compute_boundaries(5, Distribution::Normal { sigma: 0.2 }, 0.0, 1.0);
        let first_uniform = boundaries_uniform[1] - boundaries_uniform[0];
        let first_normal = boundaries_normal[1] - boundaries_normal[0];
        assert!(first_normal < first_uniform);
    }

    #[test]
    fn normal_distribution_boundaries_monotonic() {
        let boundaries = compute_boundaries(7, Distribution::Normal { sigma: 0.15 }, 0.0, 1.0);
        for i in 1..boundaries.len() {
            assert!(boundaries[i] > boundaries[i - 1]);
        }
    }

    #[test]
    fn no_empty_buckets_with_spread_colors() {
        let colors = vec![
            Color::hex(0x333333),
            Color::hex(0x555555),
            Color::hex(0x777777),
            Color::hex(0x999999),
            Color::hex(0xBBBBBB),
        ];
        let result = LightnessPartition::new(5)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        for bucket in &result {
            assert!(!bucket.is_empty());
        }
    }

    #[test]
    fn hsl_lightness_pure_red() {
        let c = Color::hex(0xFF0000);
        let l = hsl_lightness(&c);
        assert!((l - 0.5).abs() < 0.01);
    }

    #[test]
    fn hsv_value_bounds() {
        let black = Color::hex(0x000000);
        let white = Color::hex(0xFFFFFF);
        assert!(hsv_value(&black) < 0.01);
        assert!((hsv_value(&white) - 1.0).abs() < 0.01);
    }

    #[test]
    fn oklch_lightness_bounds() {
        let black = Color::hex(0x000000);
        let white = Color::hex(0xFFFFFF);
        assert!(oklch_lightness(&black) < 0.01);
        assert!((oklch_lightness(&white) - 1.0).abs() < 0.02);
    }

    #[test]
    fn symmetric_counts_are_mirrored() {
        let counts = symmetric_counts(5, 20, 1.0, 0.0);
        assert_eq!(counts[0], counts[4]);
        assert_eq!(counts[1], counts[3]);
        assert_eq!(counts.iter().sum::<usize>(), 20);
    }

    #[test]
    fn symmetric_counts_center_has_most() {
        let counts = symmetric_counts(5, 20, 2.0, 0.0);
        assert!(counts[2] >= counts[1]);
        assert!(counts[1] >= counts[0]);
    }

    #[test]
    fn symmetric_partition_produces_symmetric_bucket_sizes() {
        let colors: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let result = LightnessPartition::new(5)
            .color_space(ColorSpace::Hsl)
            .distribution(Distribution::Symmetric {
                concentration: 1.0,
                floor: 0.0,
            })
            .partition(&colors)
            .unwrap();
        assert_eq!(result[0].len(), result[4].len());
        assert_eq!(result[1].len(), result[3].len());
    }

    #[test]
    fn symmetric_high_concentration_spikes_center() {
        let counts_low = symmetric_counts(7, 30, 0.5, 0.0);
        let counts_high = symmetric_counts(7, 30, 3.0, 0.0);
        assert!(counts_high[3] > counts_low[3]);
    }

    #[test]
    fn symmetric_all_buckets_nonempty() {
        let colors: Vec<Color> = (0..15)
            .map(|i| {
                let v = (i as f32 / 14.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let result = LightnessPartition::new(5)
            .color_space(ColorSpace::Hsl)
            .distribution(Distribution::Symmetric {
                concentration: 2.0,
                floor: 0.0,
            })
            .partition(&colors)
            .unwrap();
        for bucket in &result {
            assert!(!bucket.is_empty());
        }
    }

    #[test]
    fn symmetric_preserves_total_count() {
        let colors: Vec<Color> = (0..64).map(|i| Color::hex(i * 0x040404)).collect();
        let result = LightnessPartition::new(7)
            .distribution(Distribution::Symmetric {
                concentration: 1.5,
                floor: 0.0,
            })
            .partition(&colors)
            .unwrap();
        let total: usize = result.iter().map(|b| b.len()).sum();
        assert_eq!(total, 64);
    }

    #[test]
    fn symmetric_floor_lifts_edge_counts() {
        let counts_no_floor = symmetric_counts(7, 30, 2.0, 0.0);
        let counts_with_floor = symmetric_counts(7, 30, 2.0, 0.3);
        assert!(counts_with_floor[0] > counts_no_floor[0]);
        assert_eq!(counts_with_floor.iter().sum::<usize>(), 30);
        assert_eq!(counts_with_floor[0], counts_with_floor[6]);
        assert_eq!(counts_with_floor[1], counts_with_floor[5]);
        assert_eq!(counts_with_floor[2], counts_with_floor[4]);
    }

    #[test]
    fn symmetric_floor_one_equals_uniform() {
        let counts = symmetric_counts(5, 20, 2.0, 1.0);
        for &c in &counts {
            assert_eq!(c, 4);
        }
    }

    #[test]
    fn cluster_groups_similar_lightness() {
        let darks = [
            Color::hex(0x101010),
            Color::hex(0x141414),
            Color::hex(0x181818),
        ];
        let lights = [
            Color::hex(0xE0E0E0),
            Color::hex(0xE8E8E8),
            Color::hex(0xF0F0F0),
        ];
        let colors = vec![
            darks[0], darks[1], darks[2], lights[0], lights[1], lights[2],
        ];
        let result = LightnessPartition::new(2)
            .color_space(ColorSpace::Hsl)
            .distribution(Distribution::Cluster)
            .partition(&colors)
            .unwrap();
        for d in &darks {
            assert!(result[0].contains(d));
        }
        for l in &lights {
            assert!(result[1].contains(l));
        }
    }

    #[test]
    fn cluster_preserves_total_count() {
        let colors: Vec<Color> = (0..30)
            .map(|i| {
                let v = (i as f32 / 29.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let result = LightnessPartition::new(5)
            .distribution(Distribution::Cluster)
            .partition(&colors)
            .unwrap();
        let total: usize = result.iter().map(|b| b.len()).sum();
        assert_eq!(total, 30);
    }

    #[test]
    fn cluster_all_buckets_nonempty() {
        let colors: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let result = LightnessPartition::new(4)
            .color_space(ColorSpace::Hsl)
            .distribution(Distribution::Cluster)
            .partition(&colors)
            .unwrap();
        for bucket in &result {
            assert!(!bucket.is_empty());
        }
    }

    #[test]
    fn cluster_three_distinct_groups() {
        let colors = vec![
            Color::hex(0x0A0A0A),
            Color::hex(0x0F0F0F),
            Color::hex(0x777777),
            Color::hex(0x808080),
            Color::hex(0xF0F0F0),
            Color::hex(0xFAFAFA),
        ];
        let result = LightnessPartition::new(3)
            .color_space(ColorSpace::Hsv)
            .distribution(Distribution::Cluster)
            .partition(&colors)
            .unwrap();
        assert_eq!(result[0].len(), 2);
        assert_eq!(result[1].len(), 2);
        assert_eq!(result[2].len(), 2);
    }

    // --- hue partition tests ---

    #[test]
    fn hue_empty_input_returns_empty_buckets_plus_achromatic() {
        let result = HuePartition::new(3).partition(&[]).unwrap();
        assert_eq!(result.len(), 4);
        assert!(result.iter().all(|b| b.is_empty()));
    }

    #[test]
    fn hue_grays_go_to_achromatic_bucket() {
        let black = Color::hex(0x000000);
        let gray = Color::hex(0x808080);
        let white = Color::hex(0xFFFFFF);
        let colors = vec![black, gray, white];
        let result = HuePartition::new(3)
            .color_space(ColorSpace::Hsl)
            .chroma_threshold(0.05)
            .partition(&colors)
            .unwrap();
        let achromatic = &result[3];
        assert_eq!(achromatic.len(), 3);
        assert!(achromatic.contains(&black));
        assert!(achromatic.contains(&gray));
        assert!(achromatic.contains(&white));
    }

    #[test]
    fn hue_separates_red_green_blue() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
        ];
        let result = HuePartition::new(3)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        assert_eq!(result.len(), 4);
        let chromatic_counts: Vec<usize> = result[..3].iter().map(|b| b.len()).collect();
        assert_eq!(chromatic_counts.iter().sum::<usize>(), 3);
        assert!(chromatic_counts.iter().all(|&c| c == 1));
        assert!(result[3].is_empty());
    }

    #[test]
    fn hue_similar_hues_share_bucket() {
        let red = Color::hex(0xFF0000);
        let red_orange = Color::hex(0xFF3300);
        let blue = Color::hex(0x0000FF);
        let blue_ish = Color::hex(0x0033FF);
        let colors = vec![red, red_orange, blue, blue_ish];
        let result = HuePartition::new(2)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        let bucket_with_red = result[..2].iter().find(|b| b.contains(&red)).unwrap();
        assert!(bucket_with_red.contains(&red_orange));
        let bucket_with_blue = result[..2].iter().find(|b| b.contains(&blue)).unwrap();
        assert!(bucket_with_blue.contains(&blue_ish));
    }

    #[test]
    fn hue_preserves_total_count() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
            Color::hex(0x808080),
            Color::hex(0xFFFF00),
        ];
        let result = HuePartition::new(4).partition(&colors).unwrap();
        let total: usize = result.iter().map(|b| b.len()).sum();
        assert_eq!(total, 5);
    }

    #[test]
    fn hue_all_colors_present_once() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
            Color::hex(0x808080),
            Color::hex(0xFFFF00),
            Color::hex(0xFF00FF),
        ];
        let result = HuePartition::new(3)
            .color_space(ColorSpace::Hsl)
            .chroma_threshold(0.05)
            .partition(&colors)
            .unwrap();
        let all: Vec<Color> = result.iter().flatten().copied().collect();
        assert_eq!(all.len(), 6);
        for c in &colors {
            assert!(all.contains(c));
        }
    }

    #[test]
    fn hue_oklch_separates_warm_cool() {
        let warm1 = Color::hex(0xFF4400);
        let warm2 = Color::hex(0xFF0000);
        let cool1 = Color::hex(0x0066FF);
        let cool2 = Color::hex(0x0099FF);
        let colors = vec![warm1, warm2, cool1, cool2];
        let result = HuePartition::new(2).partition(&colors).unwrap();
        let bucket_with_warm = result[..2].iter().find(|b| b.contains(&warm1)).unwrap();
        assert!(bucket_with_warm.contains(&warm2));
        let bucket_with_cool = result[..2].iter().find(|b| b.contains(&cool1)).unwrap();
        assert!(bucket_with_cool.contains(&cool2));
    }

    #[test]
    fn hue_wrap_around_keeps_reds_together() {
        let red1 = Color::hex(0xFF1A1A);
        let red2 = Color::hex(0xFF0033);
        let colors = vec![red1, red2, Color::hex(0x00FF00), Color::hex(0x0000FF)];
        let result = HuePartition::new(3)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        let bucket_with_first_red = result[..3].iter().find(|b| b.contains(&red1)).unwrap();
        assert!(bucket_with_first_red.contains(&red2));
    }

    // --- fuzziness tests ---

    #[test]
    fn lightness_fuzz_zero_no_duplicates() {
        let colors: Vec<Color> = (0..10)
            .map(|i| {
                let v = (i as f32 / 9.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let result = LightnessPartition::new(5)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        let total: usize = result.iter().map(|b| b.len()).sum();
        assert_eq!(total, 10);
    }

    #[test]
    fn lightness_fuzz_increases_total_count() {
        let colors: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let no_fuzz = LightnessPartition::new(5)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        let with_fuzz = LightnessPartition::new(5)
            .color_space(ColorSpace::Hsl)
            .fuzziness(0.5)
            .partition(&colors)
            .unwrap();
        let total_no: usize = no_fuzz.iter().map(|b| b.len()).sum();
        let total_with: usize = with_fuzz.iter().map(|b| b.len()).sum();
        assert!(total_with > total_no);
    }

    #[test]
    fn lightness_fuzz_boundary_color_appears_in_two_buckets() {
        let mid = Color::hex(0x808080);
        let colors = vec![Color::hex(0x000000), mid, Color::hex(0xFFFFFF)];
        let result = LightnessPartition::new(3)
            .color_space(ColorSpace::Hsl)
            .fuzziness(1.0)
            .partition(&colors)
            .unwrap();
        let mid_appearances: usize = result.iter().filter(|b| b.contains(&mid)).count();
        assert!(mid_appearances >= 2);
    }

    #[test]
    fn lightness_fuzz_all_colors_still_present() {
        let colors: Vec<Color> = (0..15)
            .map(|i| {
                let v = (i as f32 / 14.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let result = LightnessPartition::new(5)
            .fuzziness(0.3)
            .partition(&colors)
            .unwrap();
        let all: Vec<Color> = result.iter().flatten().copied().collect();
        for c in &colors {
            assert!(all.contains(c));
        }
    }

    #[test]
    fn lightness_fuzz_cluster_increases_total() {
        let colors: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let no_fuzz = LightnessPartition::new(4)
            .color_space(ColorSpace::Hsl)
            .distribution(Distribution::Cluster)
            .partition(&colors)
            .unwrap();
        let with_fuzz = LightnessPartition::new(4)
            .color_space(ColorSpace::Hsl)
            .distribution(Distribution::Cluster)
            .fuzziness(0.5)
            .partition(&colors)
            .unwrap();
        let total_no: usize = no_fuzz.iter().map(|b| b.len()).sum();
        let total_with: usize = with_fuzz.iter().map(|b| b.len()).sum();
        assert!(total_with > total_no);
    }

    #[test]
    fn lightness_fuzz_symmetric_increases_total() {
        let colors: Vec<Color> = (0..20)
            .map(|i| {
                let v = (i as f32 / 19.0 * 255.0) as u8;
                Color::rgb8(v, v, v)
            })
            .collect();
        let no_fuzz = LightnessPartition::new(5)
            .color_space(ColorSpace::Hsl)
            .distribution(Distribution::Symmetric {
                concentration: 1.0,
                floor: 0.0,
            })
            .partition(&colors)
            .unwrap();
        let with_fuzz = LightnessPartition::new(5)
            .color_space(ColorSpace::Hsl)
            .distribution(Distribution::Symmetric {
                concentration: 1.0,
                floor: 0.0,
            })
            .fuzziness(0.5)
            .partition(&colors)
            .unwrap();
        let total_no: usize = no_fuzz.iter().map(|b| b.len()).sum();
        let total_with: usize = with_fuzz.iter().map(|b| b.len()).sum();
        assert!(total_with > total_no);
    }

    #[test]
    fn hue_fuzz_zero_no_duplicates() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
            Color::hex(0xFFFF00),
        ];
        let result = HuePartition::new(4)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        let total: usize = result.iter().map(|b| b.len()).sum();
        assert_eq!(total, 4);
    }

    #[test]
    fn hue_fuzz_increases_total_count() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0xFF3300),
            Color::hex(0x00FF00),
            Color::hex(0x00FF33),
            Color::hex(0x0000FF),
            Color::hex(0x0033FF),
        ];
        let no_fuzz = HuePartition::new(3)
            .color_space(ColorSpace::Hsl)
            .partition(&colors)
            .unwrap();
        let with_fuzz = HuePartition::new(3)
            .color_space(ColorSpace::Hsl)
            .fuzziness(0.8)
            .partition(&colors)
            .unwrap();
        let total_no: usize = no_fuzz.iter().map(|b| b.len()).sum();
        let total_with: usize = with_fuzz.iter().map(|b| b.len()).sum();
        assert!(total_with >= total_no);
    }

    #[test]
    fn hue_fuzz_wraps_around() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0xFF4400),
            Color::hex(0x00FF00),
            Color::hex(0x00FF44),
            Color::hex(0x0000FF),
            Color::hex(0x4400FF),
        ];
        let result = HuePartition::new(3)
            .color_space(ColorSpace::Hsl)
            .fuzziness(1.0)
            .partition(&colors)
            .unwrap();
        let total: usize = result[..3].iter().map(|b| b.len()).sum();
        assert!(total > 6);
    }

    #[test]
    fn hue_fuzz_all_colors_still_present() {
        let colors = vec![
            Color::hex(0xFF0000),
            Color::hex(0x00FF00),
            Color::hex(0x0000FF),
            Color::hex(0x808080),
            Color::hex(0xFFFF00),
        ];
        let result = HuePartition::new(4)
            .fuzziness(0.5)
            .partition(&colors)
            .unwrap();
        let all: Vec<Color> = result.iter().flatten().copied().collect();
        for c in &colors {
            assert!(all.contains(c));
        }
    }
}
