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
pub trait ConvertibleIntExt<T>: PartialOrd + Copy + PartialEq + Eq {
    fn into_intermediary(&self) -> i32;
    fn from_intermediary(intermediary: i32) -> T;
}

impl ConvertibleIntExt<u16> for u16 {
    fn into_intermediary(&self) -> i32 {
        *self as i32
    }
    fn from_intermediary(intermediary: i32) -> u16 {
        intermediary as u16
    }
}

impl ConvertibleIntExt<usize> for usize {
    fn into_intermediary(&self) -> i32 {
        *self as i32
    }
    fn from_intermediary(intermediary: i32) -> usize {
        intermediary as usize
    }
}

#[allow(dead_code)]
pub trait MyIntExt<T> {
    fn move_rotating(&self, delta: i32, max: T) -> T;
    fn move_bound(&self, delta: i32, max: T) -> T;
    fn add_casting(&self, delta: i32) -> i32;
    fn fraction(&self, multiplier: f64) -> T;
}

impl<T: ConvertibleIntExt<T>> MyIntExt<T> for T {
    fn move_rotating(&self, delta: i32, max: T) -> T {
        let max_i32: i32 = max.into_intermediary();
        if max_i32 == 0 {
            return T::from_intermediary(0);
        }
        let self_i32: i32 = (*self).into_intermediary();
        let mut new_cursor: i32 = self_i32 + delta;
        while new_cursor < 0 {
            new_cursor += max_i32;
        }
        T::from_intermediary(new_cursor % max_i32)
    }

    fn move_bound(&self, delta: i32, max: T) -> T {
        let max_i32: i32 = max.into_intermediary();
        let self_i32: i32 = (*self).into_intermediary();
        T::from_intermediary((self_i32 + delta).clamp_max(max_i32 - 1).clamp_min(0))
    }

    fn add_casting(&self, delta: i32) -> i32 {
        let self_i32: i32 = (*self).into_intermediary();
        self_i32 + delta
    }

    fn fraction(&self, multiplier: f64) -> T {
        let self_i32: i32 = (*self).into_intermediary();
        let multiplied = (self_i32 as f64) * multiplier;
        T::from_intermediary(multiplied as i32)
    }
}
