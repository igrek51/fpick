#[cfg(test)]
mod tests {
    use crate::numbers::{ClampNumExt, MyIntExt};

    #[test]
    fn test_clamp() {
        assert_eq!(5u16.clamp_min(10), 10);
        assert_eq!(15u16.clamp_min(10), 15);
        assert_eq!(5u16.clamp_max(10), 5);
        assert_eq!(15u16.clamp_max(10), 10);
    }

    #[test]
    fn test_move_rotating() {
        assert_eq!(5usize.move_rotating(2, 10), 7);
        assert_eq!(8usize.move_rotating(5, 10), 3);
        assert_eq!(0usize.move_rotating(-1, 10), 9);
        assert_eq!(1usize.move_rotating(-22, 10), 9);
    }

    #[test]
    fn test_move_bound() {
        assert_eq!(5usize.move_bound(2, 10), 7);
        assert_eq!(8usize.move_bound(50, 10), 9);
        assert_eq!(0usize.move_bound(-1, 10), 0);
        assert_eq!(1usize.move_bound(-22, 10), 0);
    }
}
