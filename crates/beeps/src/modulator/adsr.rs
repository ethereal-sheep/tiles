use super::{ModTarget, Modulator};

#[derive(Clone, Copy, PartialEq, Eq)]
enum AdsrStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone)]
pub struct Adsr {
    target_param: ModTarget,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    stage: AdsrStage,
    level: f32,
    time: f32,
}

impl Adsr {
    pub fn new(target: ModTarget, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            target_param: target,
            attack: attack.max(0.001),
            decay: decay.max(0.001),
            sustain: sustain.clamp(0.0, 1.0),
            release: release.max(0.001),
            stage: AdsrStage::Idle,
            level: 0.0,
            time: 0.0,
        }
    }
}

impl Modulator for Adsr {
    fn target(&self) -> ModTarget {
        self.target_param
    }

    fn value(&self) -> f32 {
        self.level
    }

    fn advance(&mut self, sample_rate: u32) {
        let dt = 1.0 / sample_rate as f32;
        self.time += dt;

        match self.stage {
            AdsrStage::Idle => {}
            AdsrStage::Attack => {
                self.level = (self.time / self.attack).min(1.0);
                if self.time >= self.attack {
                    self.stage = AdsrStage::Decay;
                    self.time = 0.0;
                }
            }
            AdsrStage::Decay => {
                let progress = (self.time / self.decay).min(1.0);
                self.level = 1.0 + (self.sustain - 1.0) * progress;
                if self.time >= self.decay {
                    self.stage = AdsrStage::Sustain;
                    self.time = 0.0;
                }
            }
            AdsrStage::Sustain => {
                self.level = self.sustain;
            }
            AdsrStage::Release => {
                let progress = (self.time / self.release).min(1.0);
                self.level = self.sustain * (1.0 - progress);
                if self.time >= self.release {
                    self.level = 0.0;
                    self.stage = AdsrStage::Idle;
                }
            }
        }
    }

    fn note_on(&mut self) {
        self.stage = AdsrStage::Attack;
        self.time = 0.0;
    }

    fn note_off(&mut self) {
        if self.stage != AdsrStage::Idle {
            self.stage = AdsrStage::Release;
            self.time = 0.0;
        }
    }
}
