# Error handling <img align="right" width="73" height="45" src="https://raw.githubusercontent.com/tskit-dev/administrative/main/logos/svg/tskit-rust/Tskit_rust_logo.eps.svg">

The error type is [`tskit::error::TskitError`](https://docs.rs/tskit/latest/tskit/error/enum.TskitError.html#).

This type implements `Display` and may thus be printed via:

```rust, noplayground, ignore
let x = match operation() {
    Ok(y) => y,
    Err(e) => panic("{}", e);
};
```

The enum variant `TskitError::ErrorCode` represents integer return values from `tskit-c`.
When printed via `Display`, the detailed error message is fetched.
When printed via `Debug`, the numerical error code is shown.

