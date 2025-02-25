use tskit::{NodeFlags, RawFlags, SimplificationOptions};

fn example_node_flags() {
    let f: RawFlags = 1000000;

    // Node flags allow user-specified values,
    // so ::from accepts input as-is.
    let mut node_flags = NodeFlags::from(f);

    assert_eq!(node_flags.bits(), f);

    // The first bit is not set...
    assert!(!node_flags.contains(NodeFlags::IS_SAMPLE));

    // Set the first bit, making the flag indicate "sample"
    node_flags.toggle(NodeFlags::IS_SAMPLE);
    assert_ne!(node_flags.bits(), f);

    // Remove the sample status
    node_flags.remove(NodeFlags::IS_SAMPLE);
    assert_eq!(node_flags.bits(), f);
}

#[test]
fn test_example_node_flags() {
    example_node_flags();
}

#[test]
fn test_bit_ops() {
    let options = SimplificationOptions::default();
    assert!(!options.contains(SimplificationOptions::KEEP_INPUT_ROOTS));
    assert!((options & SimplificationOptions::KEEP_INPUT_ROOTS) == 0.into());
    let options = options | SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY;
    assert!(!options.contains(SimplificationOptions::KEEP_INPUT_ROOTS));
    assert!(options.contains(SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY));
    let options = options ^ SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY;
    assert!(!options.contains(SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY));
    let options = options ^ SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY;
    assert!(options.contains(SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY));

    let mut options = SimplificationOptions::default();
    assert!(!options.contains(SimplificationOptions::KEEP_INPUT_ROOTS));
    options |= SimplificationOptions::KEEP_INPUT_ROOTS;
    assert!(options.contains(SimplificationOptions::KEEP_INPUT_ROOTS));
    options &= SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY;
    assert_eq!(options, SimplificationOptions::default());
    options |= SimplificationOptions::REDUCE_TO_SITE_TOPOLOGY;
    assert_ne!(options, SimplificationOptions::default());
}
