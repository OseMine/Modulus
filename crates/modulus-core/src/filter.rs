use std::f32::consts::PI;

pub const FILTER_MIN_CUTOFF: f32 = 20.0;
pub const FILTER_MAX_CUTOFF: f32 = 20_000.0;

/// The four filter models consolidated from `variable-filter`.
///
/// `Moog`, `Roland` and `Le13700` share the 4-pole ladder topology and only
/// differ in their resonance scaling factor; `Arp4075` is a 4-stage
/// one-pole cascade.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FilterType {
    Moog,
    Roland,
    Le13700,
    Arp4075,
}

impl FilterType {
    pub const ALL: [FilterType; 4] = [
        FilterType::Moog,
        FilterType::Roland,
        FilterType::Le13700,
        FilterType::Arp4075,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            FilterType::Moog => "Moog",
            FilterType::Roland => "Roland",
            FilterType::Le13700 => "LE13700",
            FilterType::Arp4075 => "ARP 4075",
        }
    }

    pub fn from_index(index: usize) -> Self {
        FilterType::ALL[index.min(FilterType::ALL.len() - 1)]
    }
}

/// One-pole low-pass smoother for cutoff/resonance automation.
///
/// Replaces the `static mut` globals from `variable-filter`, which were
/// neither thread-safe nor real-time safe.
pub struct OnePoleSmoother {
    current: f32,
    coeff: f32,
}

impl OnePoleSmoother {
    pub fn new() -> Self {
        Self {
            current: 0.0,
            coeff: 0.0,
        }
    }

    /// `0.0` means the target is applied instantly.
    pub fn set_coeff(&mut self, coeff: f32) {
        self.coeff = coeff.clamp(0.0, 1.0);
    }

    pub fn reset(&mut self, value: f32) {
        self.current = value;
    }

    pub fn next(&mut self, target: f32) -> f32 {
        self.current = self.current * self.coeff + target * (1.0 - self.coeff);
        self.current
    }
}

impl Default for OnePoleSmoother {
    fn default() -> Self {
        Self::new()
    }
}

pub struct VariableFilter {
    filter_type: FilterType,
    cutoff_target: f32,
    resonance_target: f32,
    cutoff_smoother: OnePoleSmoother,
    resonance_smoother: OnePoleSmoother,
    y1: f32,
    y2: f32,
    y3: f32,
    y4: f32,
    oldx: f32,
    oldy1: f32,
    oldy2: f32,
    oldy3: f32,
    arp_state: [f32; 4],
}

impl VariableFilter {
    pub fn new() -> Self {
        let mut filter = Self {
            filter_type: FilterType::Moog,
            cutoff_target: 1000.0,
            resonance_target: 0.0,
            cutoff_smoother: OnePoleSmoother::new(),
            resonance_smoother: OnePoleSmoother::new(),
            y1: 0.0,
            y2: 0.0,
            y3: 0.0,
            y4: 0.0,
            oldx: 0.0,
            oldy1: 0.0,
            oldy2: 0.0,
            oldy3: 0.0,
            arp_state: [0.0; 4],
        };
        filter.cutoff_smoother.reset(1000.0);
        filter.resonance_smoother.reset(0.0);
        filter
    }

    pub fn set_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
    }

    pub fn set_smoothing(&mut self, coeff: f32) {
        self.cutoff_smoother.set_coeff(coeff);
        self.resonance_smoother.set_coeff(coeff);
    }

    pub fn set_params(&mut self, cutoff: f32, resonance: f32) {
        self.cutoff_target = cutoff.clamp(FILTER_MIN_CUTOFF, FILTER_MAX_CUTOFF);
        self.resonance_target = resonance.clamp(0.0, 1.0);
    }

    pub fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
        self.y3 = 0.0;
        self.y4 = 0.0;
        self.oldx = 0.0;
        self.oldy1 = 0.0;
        self.oldy2 = 0.0;
        self.oldy3 = 0.0;
        self.arp_state = [0.0; 4];
        self.cutoff_smoother.reset(self.cutoff_target);
        self.resonance_smoother.reset(self.resonance_target);
    }

    pub fn process(&mut self, input: f32, sample_rate: f32) -> f32 {
        let cutoff = self.cutoff_smoother.next(self.cutoff_target);
        let resonance = self.resonance_smoother.next(self.resonance_target);

        match self.filter_type {
            FilterType::Moog => self.process_ladder(input, sample_rate, cutoff, resonance, 1.8),
            FilterType::Roland => self.process_ladder(input, sample_rate, cutoff, resonance, 1.0),
            FilterType::Le13700 => self.process_ladder(input, sample_rate, cutoff, resonance, 1.0),
            FilterType::Arp4075 => self.process_arp(input, sample_rate, cutoff, resonance),
        }
    }

    fn process_ladder(
        &mut self,
        input: f32,
        sample_rate: f32,
        cutoff: f32,
        resonance: f32,
        scale_factor: f32,
    ) -> f32 {
        let f = 2.0 * cutoff / sample_rate;
        let k = 3.6 * f - 1.6 * f * f - 1.0;
        let p = (k + 1.0) * 0.5;
        let scale = (scale_factor - p) * 1.386249;
        let r = resonance * scale;

        let x = input - r * self.y4;

        self.y1 = x * p + self.oldx * p - k * self.y1;
        self.y2 = self.y1 * p + self.oldy1 * p - k * self.y2;
        self.y3 = self.y2 * p + self.oldy2 * p - k * self.y3;
        self.y4 = self.y3 * p + self.oldy3 * p - k * self.y4;
        self.y4 = self.y4.clamp(-1.0, 1.0);

        self.oldx = x;
        self.oldy1 = self.y1;
        self.oldy2 = self.y2;
        self.oldy3 = self.y3;

        self.y4
    }

    fn process_arp(&mut self, input: f32, sample_rate: f32, cutoff: f32, resonance: f32) -> f32 {
        let fc = (2.0 * PI * cutoff / sample_rate).sin();
        let res = resonance;

        let mut output = input;
        for stage in &mut self.arp_state {
            *stage += fc * (output - *stage + res * (*stage - output));
            output = *stage;
        }

        output
    }
}

impl Default for VariableFilter {
    fn default() -> Self {
        Self::new()
    }
}
