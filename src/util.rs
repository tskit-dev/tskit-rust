pub(crate) fn metadata_like_are_equal(a: &Option<Vec<u8>>, b: &Option<Vec<u8>>) -> bool {
    match a {
        Some(x) => match b {
            Some(y) => x == y,
            None => false,
        },
        None => match b.is_none() {
            true => true,
            false => false,
        },
    }
}

pub(crate) fn f64_partial_cmp_equal(a: &f64, b: &f64) -> bool {
    match a.partial_cmp(b) {
        Some(std::cmp::Ordering::Equal) => true,
        Some(std::cmp::Ordering::Less) => false,
        Some(std::cmp::Ordering::Greater) => false,
        None => false,
    }
}

#[cfg(test)]
mod test {
    use super::metadata_like_are_equal;

    #[test]
    fn compare_some_to_none() {
        let v: Vec<u8> = vec![1, 2, 3];
        assert!(!metadata_like_are_equal(&Some(v), &None));
    }

    #[test]
    fn compare_none_to_some() {
        let v: Vec<u8> = vec![1, 2, 3];
        assert!(!metadata_like_are_equal(&None, &Some(v)));
    }

    #[test]
    fn compare_none_to_none() {
        assert!(metadata_like_are_equal(&None, &None));
    }

    #[test]
    fn compare_some_to_some_are_equal() {
        let v: Vec<u8> = vec![1, 2, 3];
        let vc = v.clone();
        assert!(metadata_like_are_equal(&Some(v), &Some(vc)));
    }

    #[test]
    fn compare_some_to_some_are_not_equal() {
        let v: Vec<u8> = vec![1, 2, 3];
        let mut vc = v.clone();
        vc.push(11);
        assert!(!metadata_like_are_equal(&Some(v), &Some(vc)));
    }
}
