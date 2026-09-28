#[derive(Default, Clone)]
pub struct BarState {
    pub amplitude: f32,
    pub peak: f32,
    pub peak_hold: i32,
}

pub struct PhysicsConfig {
    pub gravity: f32,
    pub peak_gravity: f32,
    pub peak_hold_frames: i32,
    pub rise_smoothing: f32,
    pub fall_smoothing: f32,
}

pub fn update_physics(states: &mut [BarState], targets: &[f32], config: &PhysicsConfig) {
    for (state, &target) in states.iter_mut().zip(targets) {
        if target > state.amplitude {
            state.amplitude = lerp(state.amplitude, target, config.rise_smoothing);
        } else {
            state.amplitude = lerp(state.amplitude, target, config.fall_smoothing);
            state.amplitude = (state.amplitude - config.gravity).max(0.0);
        }
        state.amplitude = state.amplitude.clamp(0.0, 1.0);

        if state.amplitude >= state.peak {
            state.peak = state.amplitude;
            state.peak_hold = config.peak_hold_frames;
        } else if state.peak_hold > 0 {
            state.peak_hold -= 1;
        } else {
            state.peak = (state.peak - config.peak_gravity).max(0.0);
        }
        state.peak = state.peak.clamp(0.0, 1.0);
    }
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}
