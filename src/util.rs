pub(crate) fn partial_cmp_equal<T: PartialOrd>(lhs: &T, rhs: &T) -> bool {
    matches!(lhs.partial_cmp(rhs), Some(std::cmp::Ordering::Equal))
}

// If the system is not 16 or 32 bit, then u64 fits safely in the native
// pointer width, so we can suppress the clippy pedantic lints
//
// To test 32 bit builds:
//
// 1. sudo apt install gcc-multilib
// 2. rustup target install i686-unknown-linux-gnu
// 3. cargo clean
// 4. cargo test --target=i686-unknown-linux-gnu

#[inline]
#[cfg(not(any(target_pointer_width = "16", target_pointer_width = "32")))]
#[allow(clippy::cast_possible_truncation)]
/// Safely handle u64 -> usize on 16/32 vs 64 bit systems.
/// On 16/32-bit systems, panic! if the conversion cannot happen.
pub(crate) fn handle_u64_to_usize(v: u64) -> usize {
    v as usize
}

#[inline]
#[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
/// Safely handle u64 -> usize on 16/32 vs 64 bit systems.
/// On 16/32-bit systems, panic! if the conversion cannot happen.
pub(crate) fn handle_u64_to_usize(v: u64) -> usize {
    match usize::try_from(v) {
        Ok(u) => u,
        Err(_) => panic!("could not convert {} to usize", v),
    }
}
