/// An edge ID
///
/// This is an integer referring to a row of an [``EdgeTable``](crate::EdgeTable).
///
/// The features for this type follow the same pattern as for [``NodeId``]
pub use crate::sys::newtypes::EdgeId;

/// An individual ID
///
/// This is an integer referring to a row of an [``IndividualTable``](crate::IndividualTable).
///
/// The features for this type follow the same pattern as for [``NodeId``]
pub use crate::sys::newtypes::IndividualId;

/// A mutation ID
///
/// This is an integer referring to a row of an [``MutationTable``](crate::MutationTable).
///
/// The features for this type follow the same pattern as for [``NodeId``]
pub use crate::sys::newtypes::MutationId;

/// A migration ID
///
/// This is an integer referring to a row of an [``MigrationTable``](crate::MigrationTable).
///
/// The features for this type follow the same pattern as for [``NodeId``]
pub use crate::sys::newtypes::MigrationId;

/// A node ID
///
/// This is an integer referring to a row of a [``NodeTable``](crate::NodeTable).
///
/// # Examples
///
/// These examples illustrate using this type as something "integer-like".
///
/// ```
/// use tskit::NodeId;
///
/// // The default value is null:
/// assert_eq!(tskit::NodeId::default(), tskit::NodeId::NULL);
///
/// let y: NodeId = NodeId::from(1);
/// assert_eq!(1, y);
/// assert_eq!(y, 1);
///
/// assert!(y < 2);
/// assert!(y <= 1);
/// assert!(2 > y);
/// assert!(1 + 1 >= y);
///
/// let z: NodeId = NodeId::from(1);
/// assert_eq!(y, z);
/// ```
///
/// It is also possible to write functions accepting both the `NodeId`
/// and `tsk_id_t`:
///
/// ```
/// use tskit::NodeId;
///
/// fn interesting<N: Into<NodeId>>(x: N) -> NodeId {
///     x.into()
/// }
///
/// let x = 1;
/// assert_eq!(interesting(x), x);
/// let x: NodeId = NodeId::from(0);
/// assert_eq!(interesting(x), x);
/// ```
///
/// The types also implement `Display`:
///
/// ```
/// use tskit::NodeId;
///
/// let n = NodeId::from(11);
/// assert_eq!(format!("{}", n), "11".to_string());
/// // Debug output contains type info
/// assert_eq!(format!("{:?}", n), "NodeId(11)".to_string());
/// let n = NodeId::from(NodeId::NULL);
/// assert_eq!(format!("{}", n), "NULL");
/// assert_eq!(format!("{:?}", n), "NodeId(-1)");
/// ```
///
pub use crate::sys::newtypes::NodeId;

/// A population ID
///
/// This is an integer referring to a row of an [``PopulationTable``](crate::PopulationTable).
///
/// The features for this type follow the same pattern as for [``NodeId``]
pub use crate::sys::newtypes::PopulationId;

/// A provenance ID
///
/// This is an integer referring to a row of a [``provenance::ProvenanceTable``].
///
/// The features for this type follow the same pattern as for [``NodeId``]
#[cfg(feature = "provenance")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "provenance")))]
pub use crate::sys::newtypes::ProvenanceId;

/// A site ID
///
/// This is an integer referring to a row of an [``SiteTable``](crate::SiteTable).
///
/// The features for this type follow the same pattern as for [``NodeId``]
pub use crate::sys::newtypes::SiteId;

/// Wraps `tsk_size_t`
///
/// This type plays the role of C's `size_t` in the `tskit` C library.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "bindings")]
/// # {
/// let s = tskit::SizeType::from(1);
/// let mut t: tskit::bindings::tsk_size_t = s.into();
/// assert!(t == s);
/// assert!(t == 1);
/// let u = tskit::SizeType::from(s);
/// assert!(u == s);
/// t += 1;
/// assert!(t > s);
/// assert!(s < t);
/// # }
/// ```
///
/// #[repr(transparent)]
pub use crate::sys::newtypes::SizeType;

/// A newtype for the concept of "genomic position".
/// A `Position` can represent either a locus or a
/// distance between loci.
///
/// Wraps [`f64`].
///
/// For examples, see [`Time`].
///
/// This type can be multiplied and divided by [`Time`].
pub use crate::sys::newtypes::Position;

/// A newtype for the concept of location.
/// A `Location` may represent a location or the
/// output of arithmetic involving locations.
///
/// Wraps [`f64`].
///
/// For examples, see [`Time`].
///
pub use crate::sys::newtypes::Location;

/// A newtype for the concept of time.
/// A `Time` value can represent either a point in time
/// or the output of arithmetic involving time.
///
/// Wraps [`f64`].
///
/// # Examples
///
/// ```
/// let t0 = tskit::Time::from(2.0);
/// let t1 = tskit::Time::from(10.0);
///
/// let mut sum = t0 + t1;
///
/// match sum.partial_cmp(&12.0) {
///    Some(std::cmp::Ordering::Equal) => (),
///    _ => assert!(false),
/// };
///
/// sum /= tskit::Time::from(2.0);
///
/// match sum.partial_cmp(&6.0) {
///    Some(std::cmp::Ordering::Equal) => (),
///    _ => assert!(false),
/// };
/// ```
///
/// # Notes
///
/// The current implementation of [`PartialOrd`] is based on
/// the underlying implementation for [`f64`].
///
/// A `Time` can be multiplied and divided by a [`Position`]
///
pub use crate::sys::newtypes::Time;
