pub mod http; // re-export for main, and to silence dead_code
pub mod release;

#[cfg(test)]
mod tests {
    use crate::release::get_release;
    use crate::release::Release;

    #[test]
    fn test_release() {
        // https://www.discogs.com/release/8196883
        let rel: Release = get_release(8196883).unwrap();
        assert_eq!(rel.year, 1998);
        assert_eq!(rel.id, 8196883);
    }

    #[test]
    fn test_nonexistent_release() {
        assert_eq!(get_release(0), None);
    }
}
