#[cfg(test)]
mod tests {
    use crate::ui::visible_offset;

    #[test]
    fn test_offset_keeps_selection_visible() {
        assert_eq!(visible_offset(0, 0, 3, 10), 0);
        assert_eq!(visible_offset(2, 0, 3, 10), 0);
        assert_eq!(visible_offset(3, 0, 3, 10), 1);
        assert_eq!(visible_offset(5, 0, 3, 10), 3);
        assert_eq!(visible_offset(9, 0, 3, 10), 7);
    }

    #[test]
    fn test_offset_scrolls_back_up() {
        assert_eq!(visible_offset(4, 7, 3, 10), 4);
        assert_eq!(visible_offset(0, 7, 3, 10), 0);
    }

    #[test]
    fn test_offset_clamps_to_list_end() {
        assert_eq!(visible_offset(9, 9, 3, 10), 7);
        assert_eq!(visible_offset(1, 0, 5, 2), 0);
    }

    #[test]
    fn test_offset_for_empty_list() {
        assert_eq!(visible_offset(0, 0, 5, 0), 0);
    }

    #[test]
    fn test_offset_with_zero_height() {
        assert_eq!(visible_offset(5, 2, 0, 10), 0);
    }
}
