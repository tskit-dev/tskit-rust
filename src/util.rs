pub(crate) fn f64_partial_cmp_equal(a: &f64, b: &f64) -> bool {
    match a.partial_cmp(b) {
        Some(std::cmp::Ordering::Equal) => true,
        Some(std::cmp::Ordering::Less) => false,
        Some(std::cmp::Ordering::Greater) => false,
        None => false,
    }
}
