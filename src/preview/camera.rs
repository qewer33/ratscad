const DRAG_ROTATION_GAIN: f32 = 4.0;
const SCROLL_SCALE_STEP: f32 = 0.1;
const MIN_SCALE: f32 = 0.1;
const MAX_SCALE: f32 = 2.0;

pub fn drag_to_rotation(dx: i16, dy: i16) -> [f32; 2] {
    [f32::from(dy) * DRAG_ROTATION_GAIN, f32::from(dx) * DRAG_ROTATION_GAIN]
}

pub fn scroll_to_scale(current: f32, direction: i8) -> f32 {
    let next = current * (1.0 + f32::from(direction) * SCROLL_SCALE_STEP);
    next.clamp(MIN_SCALE, MAX_SCALE)
}
