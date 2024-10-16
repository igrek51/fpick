pub trait ClampNumExt<T> {
    fn clamp_min(&self, min: T) -> T;
    fn clamp_max(&self, max: T) -> T;
}

impl ClampNumExt<f32> for f32 {
    fn clamp_min(&self, min: f32) -> f32 {
        if *self < min {
            min
        } else {
            *self
        }
    }

    fn clamp_max(&self, max: f32) -> f32 {
        if *self > max {
            max
        } else {
            *self
        }
    }
}

impl ClampNumExt<i32> for i32 {
    fn clamp_min(&self, min: i32) -> i32 {
        if *self < min {
            min
        } else {
            *self
        }
    }

    fn clamp_max(&self, max: i32) -> i32 {
        if *self > max {
            max
        } else {
            *self
        }
    }
}

#[allow(dead_code)]
pub trait MyIntExt<T> {
    fn move_rotating(&self, delta: i32, max: usize) -> T;
    fn move_bound(&self, delta: i32, max: usize) -> T;
}

impl MyIntExt<usize> for usize {
    fn move_rotating(&self, delta: i32, max: usize) -> usize {
        let mut new_cursor: i32 = *self as i32 + delta;
        while new_cursor < 0 {
            new_cursor += max as i32;
        }
        (new_cursor % max as i32) as usize
    }

    fn move_bound(&self, delta: i32, max: usize) -> usize {
        (*self as i32 + delta)
            .clamp_max(max as i32 - 1)
            .clamp_min(0) as usize
    }
}
