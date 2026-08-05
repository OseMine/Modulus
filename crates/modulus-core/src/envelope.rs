/// Linear ADSR envelope, ported from `Am-Synth` with a corrected release
/// stage (the original decayed from the sustain level instead of the
/// current value).
pub struct Adsr {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    stage: AdsrStage,
    value: f32,
    sample_rate: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AdsrStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl Adsr {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.5,
            release: 0.1,
            stage: AdsrStage::Idle,
            value: 0.0,
            sample_rate,
        }
    }

    pub fn set_params(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.attack = attack.max(0.001);
        self.decay = decay.max(0.001);
        self.sustain = sustain.clamp(0.0, 1.0);
        self.release = release.max(0.001);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn trigger(&mut self) {
        self.stage = AdsrStage::Attack;
        self.value = 0.0;
    }

    pub fn release(&mut self) {
        if self.stage != AdsrStage::Idle {
            self.stage = AdsrStage::Release;
        }
    }

    pub fn is_idle(&self) -> bool {
        self.stage == AdsrStage::Idle
    }

    pub fn process(&mut self) -> f32 {
        match self.stage {
            AdsrStage::Idle => {}
            AdsrStage::Attack => {
                self.value += 1.0 / (self.attack * self.sample_rate);
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = AdsrStage::Decay;
                }
            }
            AdsrStage::Decay => {
                self.value -= (1.0 - self.sustain) / (self.decay * self.sample_rate);
                if self.value <= self.sustain {
                    self.value = self.sustain;
                    self.stage = AdsrStage::Sustain;
                }
            }
            AdsrStage::Sustain => {}
            AdsrStage::Release => {
                self.value -= self.value / (self.release * self.sample_rate);
                if self.value <= 0.0001 {
                    self.value = 0.0;
                    self.stage = AdsrStage::Idle;
                }
            }
        }
        self.value
    }
}
