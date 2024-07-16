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
