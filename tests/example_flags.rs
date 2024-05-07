use tskit::{NodeFlags, RawFlags, SimplificationOptions};

fn clip_invalid_flags() {
    // This value contains bits set to 1
    // that are not valid values for
    // tskit::SimplificationFlags
    let f: RawFlags = 1000000;

    // Creating flags from this value will unset invalid
    // bits, meaning the final value != the input value.
    let simplification_flags = SimplificationOptions::from(f);

    assert_ne!(f, simplification_flags.bits());
    assert!(simplification_flags.is_valid());

    // You can skip the unsetting of invalid bits...
    let simplification_flags = SimplificationOptions::from_bits_retain(f);

    // ... and use this function to check.
    assert!(!simplification_flags.is_valid());
}

fn example_node_flags() {
    let f: RawFlags = 1000000;

    // Node flags allow user-specified values,
    // so ::from accepts input as-is.
    let mut node_flags = NodeFlags::from(f);

    assert_eq!(node_flags.bits(), f);
    assert!(node_flags.is_valid());

    // The first bit is not set...
    assert!(!node_flags.contains(NodeFlags::IS_SAMPLE));

    // Set the first bit, making the flag indicate "sample"
    node_flags.toggle(NodeFlags::IS_SAMPLE);
    assert!(node_flags.is_valid());
    assert_ne!(node_flags.bits(), f);

    // Remove the sample status
    node_flags.remove(NodeFlags::IS_SAMPLE);
    assert!(node_flags.is_valid());
    assert_eq!(node_flags.bits(), f);
}

#[test]
fn test_clip_invalid_flags() {
    clip_invalid_flags();
}

#[test]
fn test_example_node_flags() {
    example_node_flags();
}
