use std::ops::Range;

#[derive(Clone, Debug)]
pub struct Random {
    state: u64,
}

impl Random {
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };

        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn next_f32(&mut self) -> f32 {
        let value = self.next_u64() >> 40;
        value as f32 / (1u64 << 24) as f32
    }

    pub fn coin(&mut self, probability: f32) -> bool {
        if probability <= 0.0 {
            return false;
        }

        if probability >= 1.0 {
            return true;
        }

        self.next_f32() <= probability
    }

    pub fn range_usize(&mut self, range: Range<usize>) -> usize {
        let width = range.end.saturating_sub(range.start);
        if width == 0 {
            return range.start;
        }

        range.start + (self.next_u64() as usize % width)
    }

    pub fn weighted_index(&mut self, weights: &[u32]) -> Option<usize> {
        let total = weights.iter().copied().sum::<u32>();
        if total == 0 {
            return None;
        }

        let mut cursor = (self.next_u64() % total as u64) as u32;
        for (index, weight) in weights.iter().enumerate() {
            if cursor < *weight {
                return Some(index);
            }
            cursor -= *weight;
        }

        None
    }
}
