/// Deterministic xorshift32 PRNG.
///
/// Replaces `rand::thread_rng()` from the original repos, which is not
/// real-time safe: it lazily allocates and locks on first use.
pub struct FastRng {
    state: u32,
}

impl FastRng {
    pub fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Uniform value in `[0, 1)`.
    pub fn f32_unit(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * (1.0 / 16_777_216.0)
    }
}
