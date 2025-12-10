use ratatui::{layout::Rect, style::Color, Frame};
use tachyonfx::{Duration, EffectManager, Interpolation, Motion, fx};

/// A wrapper for the effect manager to handle animations.
pub struct FxManager {
    effects: EffectManager<()>,
}

impl FxManager {
    pub fn new() -> Self {
        FxManager {
            effects: EffectManager::default(),
        }
    }

    /// Process and render effects on the frame buffer
    pub fn render(&mut self, frame: &mut Frame, area: Rect, elapsed: Duration) {
        self.effects
            .process_effects(elapsed, frame.buffer_mut(), area);
    }

    /// Trigger the startup slide-in animation
    /// Exactly replicates statui's approach
    pub fn trigger_startup(&mut self) {
        // A nice slide_in animation from tachyonfx-ftl
        // https://junkdog.github.io/tachyonfx-ftl/?example=slide_in
        let c = Color::Reset;
        let timer = (300, Interpolation::Linear);
        let fx = fx::slide_in(Motion::UpToDown, 10, 0, c, timer);
        self.effects.add_effect(fx);
    }
}


