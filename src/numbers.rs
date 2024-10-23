pub trait ClampNumType: PartialOrd + Copy {}
impl ClampNumType for usize {}
impl ClampNumType for u16 {}
impl ClampNumType for u32 {}
impl ClampNumType for u64 {}
impl ClampNumType for i32 {}
impl ClampNumType for i64 {}
impl ClampNumType for f32 {}
impl ClampNumType for f64 {}

pub trait ClampNumExt<T> {
    fn clamp_min(&self, min: T) -> T;
    fn clamp_max(&self, max: T) -> T;
}

impl<T: ClampNumType> ClampNumExt<T> for T {
    fn clamp_min(&self, min: T) -> T {
        if *self < min {
            min
        } else {
            *self
        }
    }

    fn clamp_max(&self, max: T) -> T {
        if *self > max {
            max
        } else {
            *self
        }
    }
}

#[allow(dead_code)]
pub trait MyIntExt<T> {
    fn move_rotating(&self, delta: i32, max: T) -> T;
    fn move_bound(&self, delta: i32, max: T) -> T;
    fn add_cast(&self, delta: i32) -> i32;
    fn fraction(&self, multiplier: f64) -> T;
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

    fn add_cast(&self, delta: i32) -> i32 {
        *self as i32 + delta
    }

    fn fraction(&self, multiplier: f64) -> usize {
        let multiplied = (*self as f64) * multiplier;
        multiplied as usize
    }
}

impl MyIntExt<u16> for u16 {
    fn move_rotating(&self, delta: i32, max: u16) -> u16 {
        let mut new_cursor: i32 = *self as i32 + delta;
        while new_cursor < 0 {
            new_cursor += max as i32;
        }
        (new_cursor % max as i32) as u16
    }

    fn move_bound(&self, delta: i32, max: u16) -> u16 {
        (*self as i32 + delta)
            .clamp_max(max as i32 - 1)
            .clamp_min(0) as u16
    }

    fn add_cast(&self, delta: i32) -> i32 {
        *self as i32 + delta
    }

    fn fraction(&self, multiplier: f64) -> u16 {
        let multiplied = (*self as f64) * multiplier;
        multiplied as u16
    }
}
