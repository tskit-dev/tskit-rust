use tskit::*;

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_tskit_error_message() {
        let m = error::get_tskit_error_message(0);
        assert_eq!(m, "Normal exit condition. This is not an error!");
    }

    fn mock_error() -> TskReturnValue {
        handle_tsk_return_value!(-207);
    }

    fn mock_success() -> TskReturnValue {
        Ok(0)
    }

    #[test]
    fn test_error_formatting() {
        let x = mock_error();
        let mut s: String = "nope!".to_string();
        x.map_or_else(|e: TskitError| s = format!("{}", e), |_| ());
        assert_eq!(s, "Individual out of bounds");
    }

    #[test]
    fn test_extract_error_message() {
        let x = mock_error();
        match error::extract_error_message(x) {
            Some(s) => assert_eq!(s, "Individual out of bounds"),
            None => panic!(),
        }

        if error::extract_error_message(mock_success()).is_some() {
            panic!();
        }
    }
}

