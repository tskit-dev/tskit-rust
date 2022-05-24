pub(crate) fn partial_cmp_equal<T: PartialOrd>(lhs: &T, rhs: &T) -> bool {
    matches!(lhs.partial_cmp(rhs), Some(std::cmp::Ordering::Equal))
}
