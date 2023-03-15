use crate::NodeId;
use crate::Position;
use crate::TableCollection;
use crate::TskitError;

// Design considerations:
//
// We should be able to do better than
// the fwdpp implementation by taking a
// time-sorted list of alive nodes and inserting
// their edges.
// After insertion, we can truncate the input
// edge table, eliminating all edges corresponding
// to the set of alive nodes.
// This procedure would only be done AFTER
// simplification, such that the copied
// edges are guaranteed correct.
// We'd need to hash the existence of these alive nodes.
// Then, when going over the edge buffer, we can ask
// if an edge parent is in the hashed set.
// We would also keep track of the smallest
// edge id, and that (maybe minus 1?) is our truncation point.

fn swap_with_empty<T>(vec: &mut Vec<T>) {
    let mut t = vec![];
    std::mem::swap(&mut t, vec);
}

#[derive(Copy, Clone)]
struct AliveNodeTimes {
    min: f64,
    max: f64,
}

impl AliveNodeTimes {
    fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    fn non_overlapping(&self) -> bool {
        self.min == self.max
    }
}

#[derive(Debug)]
struct PreExistingEdge {
    first: usize,
    last: usize,
}

impl PreExistingEdge {
    fn new(first: usize, last: usize) -> Self {
        assert!(last > first);
        Self { first, last }
    }
}

#[derive(Debug)]
struct Segment {
    left: Position,
    right: Position,
}

type ChildSegments = std::collections::HashMap<NodeId, Vec<Segment>>;

#[derive(Default, Debug)]
struct BufferedBirths {
    children: Vec<NodeId>,
    segments: std::collections::HashMap<NodeId, ChildSegments>,
}

impl BufferedBirths {
    fn initialize(&mut self, parents: &[NodeId], children: &[NodeId]) -> Result<(), TskitError> {
        self.children = children.to_vec();
        self.children.sort();
        self.children.dedup();
        self.segments.clear();
        // FIXME: don't do this work if the parent already exists
        for p in parents {
            let mut segments = ChildSegments::default();
            for c in &self.children {
                if segments.insert(*c, vec![]).is_some() {
                    return Err(TskitError::LibraryError("redundant child ids".to_owned()));
                }
            }
            self.segments.insert(*p, segments);
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct EdgeBuffer {
    left: Vec<Position>,
    right: Vec<Position>,
    child: Vec<NodeId>,
    // TODO: this should be
    // an option so that we can use take.
    buffered_births: BufferedBirths,
    // NOTE: these vectors are wasteful:
    // 1. usize is more than we need,
    //    but it is more convenient.
    // 2. Worse, these vectors will
    //    contain N elements, where
    //    N is the total number of nodes,
    //    but likely many fewer nodes than that
    //    have actually had offspring.
    //    It is hard to fix this -- we cannot
    //    guarantee that parents are entered
    //    in any specific order.
    // 3. Performance IMPROVES MEASURABLY
    //    if we use u32 here. But tsk_size_t
    //    is u64.
    head: Vec<usize>,
    tail: Vec<usize>,
    next: Vec<usize>,
}

impl EdgeBuffer {
    fn insert_new_parent(&mut self, parent: usize, child: NodeId, left: Position, right: Position) {
        self.left.push(left);
        self.right.push(right);
        self.child.push(child);
        self.head[parent] = self.left.len() - 1;
        self.tail[parent] = self.head[parent];
        self.next.push(usize::MAX);
    }

    fn extend_parent(&mut self, parent: usize, child: NodeId, left: Position, right: Position) {
        self.left.push(left);
        self.right.push(right);
        self.child.push(child);
        let t = self.tail[parent];
        self.tail[parent] = self.left.len() - 1;
        self.next[t] = self.left.len() - 1;
        self.next.push(usize::MAX);
    }

    fn clear(&mut self) {
        self.left.clear();
        self.right.clear();
        self.child.clear();
        self.head.clear();
        self.tail.clear();
        self.next.clear();
    }

    fn release_memory(&mut self) {
        swap_with_empty(&mut self.head);
        swap_with_empty(&mut self.next);
        swap_with_empty(&mut self.left);
        swap_with_empty(&mut self.right);
        swap_with_empty(&mut self.child);
        swap_with_empty(&mut self.tail);
    }

    fn extract_buffered_births(&mut self) -> BufferedBirths {
        let mut b = BufferedBirths::default();
        std::mem::swap(&mut self.buffered_births, &mut b);
        b
    }

    // Should Err if prents/children not unique
    pub fn setup_births(
        &mut self,
        parents: &[NodeId],
        children: &[NodeId],
    ) -> Result<(), TskitError> {
        self.buffered_births.initialize(parents, children)
    }

    pub fn finalize_births(&mut self) {
        let buffered_births = self.extract_buffered_births();
        for (p, children) in buffered_births.segments.iter() {
            for c in buffered_births.children.iter() {
                if let Some(segs) = children.get(c) {
                    for s in segs {
                        self.buffer_birth(*p, *c, s.left, s.right).unwrap();
                    }
                } else {
                    // should be error
                    panic!();
                }
            }
        }
    }

    pub fn record_birth<P, C, L, R>(
        &mut self,
        parent: P,
        child: C,
        left: L,
        right: R,
    ) -> Result<(), TskitError>
    where
        P: Into<NodeId>,
        C: Into<NodeId>,
        L: Into<Position>,
        R: Into<Position>,
    {
        let parent = parent.into();

        let child = child.into();
        if let Some(parent_buffer) = self.buffered_births.segments.get_mut(&parent) {
            if let Some(v) = parent_buffer.get_mut(&child) {
                let left = left.into();
                let right = right.into();
                v.push(Segment { left, right });
            } else {
                // should be an error
                panic!();
            }
        } else {
            // should be an error
            panic!();
        }

        Ok(())
    }

    // NOTE: tskit is overly strict during simplification,
    // enforcing sorting requirements on the edge table
    // that are not strictly necessary.
    pub fn buffer_birth<P, C, L, R>(
        &mut self,
        parent: P,
        child: C,
        left: L,
        right: R,
    ) -> Result<(), TskitError>
    where
        P: Into<NodeId>,
        C: Into<NodeId>,
        L: Into<Position>,
        R: Into<Position>,
    {
        let parent = parent.into();
        if parent < 0 {
            return Err(TskitError::IndexError);
        }

        let parent = parent.as_usize();

        if parent >= self.head.len() {
            self.head.resize(parent + 1, usize::MAX);
            self.tail.resize(parent + 1, usize::MAX);
        }

        if self.head[parent] == usize::MAX {
            self.insert_new_parent(parent, child.into(), left.into(), right.into());
        } else {
            self.extend_parent(parent, child.into(), left.into(), right.into());
        }
        Ok(())
    }

    // NOTE: we can probably have this function not error:
    // the head array is populated by i32 converted to usize,
    // so if things are getting out of range, we should be
    // in trouble before this point.
    // NOTE: we need a bitflags here for other options, like sorting the head
    // contents based on birth time.
    pub fn pre_simplification(&mut self, tables: &mut TableCollection) -> Result<(), TskitError> {
        let num_input_edges = tables.edges().num_rows().as_usize();
        let mut head_index: Vec<usize> = self
            .head
            .iter()
            .enumerate()
            .filter(|(_, j)| **j != usize::MAX)
            .map(|(i, _)| i)
            .collect();

        let node_time = tables.nodes().time_slice();
        head_index.sort_by(|a, b| node_time[*a].partial_cmp(&node_time[*b]).unwrap());
        //for (i, h) in self.head.iter().rev().enumerate() {
        for h in head_index.into_iter() {
            let parent = match i32::try_from(h) {
                Ok(value) => value,
                Err(_) => {
                    return Err(TskitError::RangeError(
                        "usize to i32 conversion failed".to_owned(),
                    ))
                }
            };
            tables.add_edge(
                self.left[self.head[h]],
                self.right[self.head[h]],
                parent,
                self.child[self.head[h]],
            )?;

            let mut next = self.next[self.head[h]];
            while next != usize::MAX {
                tables.add_edge(self.left[next], self.right[next], parent, self.child[next])?;
                next = self.next[next];
            }
        }

        self.release_memory();

        // This assert is redundant b/c TableCollection
        // works via MBox/NonNull.
        assert!(!tables.as_ptr().is_null());
        // SAFETY: table collection pointer is not null and num_edges
        // is the right length.
        let num_edges = tables.edges().num_rows().as_usize();
        let edge_left =
            unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.left, num_edges) };
        let edge_right = unsafe {
            std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.right, num_edges)
        };
        let edge_parent = unsafe {
            std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.parent, num_edges)
        };
        let edge_child = unsafe {
            std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.child, num_edges)
        };
        edge_left.rotate_left(num_input_edges);
        edge_right.rotate_left(num_input_edges);
        edge_parent.rotate_left(num_input_edges);
        edge_child.rotate_left(num_input_edges);
        Ok(())
    }

    fn alive_node_times(&self, alive: &[NodeId], tables: &mut TableCollection) -> AliveNodeTimes {
        let node_times = tables.nodes().time_slice_raw();
        let mut max_alive_node_time = 0.0;
        let mut min_alive_node_time = f64::MAX;

        for a in alive {
            let time = node_times[a.as_usize()];
            max_alive_node_time = if time > max_alive_node_time {
                time
            } else {
                max_alive_node_time
            };
            min_alive_node_time = if time < min_alive_node_time {
                time
            } else {
                min_alive_node_time
            };
        }
        AliveNodeTimes::new(min_alive_node_time, max_alive_node_time)
    }

    // The method here ends up creating a problem:
    // we are buffering nodes with increasing node id
    // that are also more ancient. This is the opposite
    // order from what happens during a forward-time simulation.
    // NOTE: the mechanics of this fn differ if we use
    // "regular" simplification or streaming!
    // For the former case, we have to do the setup/finalize
    // business. For the latter, WE DO NOT.
    // This differences suggests there are actually two types/impls
    // being discussed here.
    fn buffer_existing_edges(
        &mut self,
        pre_existing_edges: Vec<PreExistingEdge>,
        tables: &mut TableCollection,
    ) -> Result<usize, TskitError> {
        let parent = tables.edges().parent_slice();
        let child = tables.edges().child_slice();
        let left = tables.edges().left_slice();
        let right = tables.edges().right_slice();
        let mut rv = 0;
        for pre in pre_existing_edges.iter() {
            self.setup_births(&[parent[pre.first]], &child[pre.first..pre.last])?;
            for e in pre.first..pre.last {
                assert_eq!(parent[e], parent[pre.first]);
                self.record_birth(parent[e], child[e], left[e], right[e])?;
                rv += 1;
            }
            self.finalize_births();
        }

        Ok(rv)
    }

    // FIXME: clean up commented-out code
    // if we decide we don't need it.
    fn collect_pre_existing_edges(
        &self,
        alive_node_times: AliveNodeTimes,
        tables: &mut TableCollection,
    ) -> Vec<PreExistingEdge> {
        let mut edges = vec![];
        let mut i = 0;
        let parent = tables.edges().parent_slice();
        //let child = tables.edges().child_slice();
        let node_time = tables.nodes().time_slice();
        while i < parent.len() {
            let p = parent[i];
            // let c = child[i];
            if node_time[p.as_usize()] <= alive_node_times.max
            //|| (node_time[c.as_usize()] < alive_node_times.max
            //    && node_time[p.as_usize()] > alive_node_times.max)
            {
                let mut j = 0_usize;
                while i + j < parent.len() && parent[i + j] == p {
                    j += 1;
                }
                edges.push(PreExistingEdge::new(i, i + j));
                i += j;
            } else {
                break;
            }
        }
        edges
    }

    // FIXME:
    //
    // 1. If min/max parent alive times are equal, return.
    //    DONE
    // 2. Else, we need to do a rotation at min_edge
    //    before truncation.
    //    DONE
    // 3. However, we also have to respect our API
    //    and process each parent carefully,
    //    setting up the birth/death epochs.
    //    We need to use setup_births and finalize_births
    //    to get this right.
    //    DONE
    // 4. We are doing this in the wrong temporal order.
    //    We need to pre-process all existing edge intervals,
    //    cache them, then go backwards through them,
    //    so that we buffer them present-to-past.
    //    DONE
    // 5. This step should be EARLY in a recording epoch,
    //    so that we avoid the gotcha of stealing edges
    //    from the last generation of a simulation.
    pub fn post_simplification(
        &mut self,
        alive: &[NodeId],
        tables: &mut TableCollection,
    ) -> Result<(), TskitError> {
        self.clear();

        let alive_node_times = self.alive_node_times(alive, tables);
        if alive_node_times.non_overlapping() {
            // There can be no overlap between current
            // edges and births that are about to happen,
            // so we get out.
            return Ok(());
        }

        let pre_existing_edges = self.collect_pre_existing_edges(alive_node_times, tables);
        let min_edge = self.buffer_existing_edges(pre_existing_edges, tables)?;
        let num_edges = tables.edges().num_rows().as_usize();
        let edge_left =
            unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.left, num_edges) };
        let edge_right = unsafe {
            std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.right, num_edges)
        };
        let edge_parent = unsafe {
            std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.parent, num_edges)
        };
        let edge_child = unsafe {
            std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.child, num_edges)
        };
        edge_left.rotate_left(min_edge);
        edge_right.rotate_left(min_edge);
        edge_parent.rotate_left(min_edge);
        edge_child.rotate_left(min_edge);
        // SAFETY: ?????
        let rv = unsafe {
            crate::bindings::tsk_edge_table_truncate(
                &mut (*tables.as_mut_ptr()).edges,
                (num_edges - min_edge) as crate::bindings::tsk_size_t,
            )
        };
        handle_tsk_return_value!(rv, ())
    }
}

struct StreamingSimplifier {
    simplifier: crate::bindings::tsk_streaming_simplifier_t,
}

impl StreamingSimplifier {
    fn new<O: Into<crate::SimplificationOptions>>(
        samples: &[NodeId],
        options: O,
        tables: &mut TableCollection,
    ) -> Result<Self, TskitError> {
        let mut simplifier =
            std::mem::MaybeUninit::<crate::bindings::tsk_streaming_simplifier_t>::uninit();
        let num_samples = samples.len() as crate::bindings::tsk_size_t;
        match unsafe {
            crate::bindings::tsk_streaming_simplifier_init(
                simplifier.as_mut_ptr(),
                tables.as_mut_ptr(),
                samples.as_ptr().cast::<crate::bindings::tsk_id_t>(),
                num_samples,
                options.into().bits(),
            )
        } {
            code if code < 0 => Err(TskitError::ErrorCode { code }),
            _ => Ok(Self {
                simplifier: unsafe { simplifier.assume_init() },
            }),
        }
    }

    fn add_edge(
        &mut self,
        left: Position,
        right: Position,
        parent: NodeId, // FIXME: shouldn't be here
        child: NodeId,
    ) -> Result<(), TskitError> {
        let code = unsafe {
            crate::bindings::tsk_streaming_simplifier_add_edge(
                &mut self.simplifier,
                left.into(),
                right.into(),
                parent.into(),
                child.into(),
            )
        };
        handle_tsk_return_value!(code, ())
    }

    fn merge_ancestors(&mut self, parent: NodeId) -> Result<(), TskitError> {
        let code = unsafe {
            crate::bindings::tsk_streaming_simplifier_merge_ancestors(
                &mut self.simplifier,
                parent.into(),
            )
        };
        handle_tsk_return_value!(code, ())
    }

    // FIXME: need to be able to validate that node_map is correct length!
    fn finalise(&mut self, node_map: Option<&mut [NodeId]>) -> Result<(), TskitError> {
        let n = match node_map {
            Some(x) => x.as_mut_ptr().cast::<crate::bindings::tsk_id_t>(),
            None => std::ptr::null_mut(),
        };
        let code =
            unsafe { crate::bindings::tsk_streaming_simplifier_finalise(&mut self.simplifier, n) };
        handle_tsk_return_value!(code, ())
    }

    fn input_num_edges(&self) -> usize {
        unsafe {
            crate::bindings::tsk_streaming_simplifier_get_num_input_edges(&self.simplifier) as usize
        }
    }

    fn input_left(&self) -> &[Position] {
        unsafe {
            std::slice::from_raw_parts(
                crate::bindings::tsk_streaming_simplifier_get_input_left(&self.simplifier)
                    .cast::<Position>(),
                self.input_num_edges(),
            )
        }
    }

    fn input_right(&self) -> &[Position] {
        unsafe {
            std::slice::from_raw_parts(
                crate::bindings::tsk_streaming_simplifier_get_input_right(&self.simplifier)
                    .cast::<Position>(),
                self.input_num_edges(),
            )
        }
    }

    fn input_parent(&self) -> &[NodeId] {
        unsafe {
            std::slice::from_raw_parts(
                crate::bindings::tsk_streaming_simplifier_get_input_parent(&self.simplifier)
                    .cast::<NodeId>(),
                self.input_num_edges(),
            )
        }
    }

    fn input_child(&self) -> &[NodeId] {
        unsafe {
            std::slice::from_raw_parts(
                crate::bindings::tsk_streaming_simplifier_get_input_child(&self.simplifier)
                    .cast::<NodeId>(),
                self.input_num_edges(),
            )
        }
    }

    fn get_input_parent(&self, u: usize) -> NodeId {
        assert!(u < self.input_num_edges());
        self.input_parent()[u]
    }
    fn get_input_child(&self, u: usize) -> NodeId {
        assert!(u < self.input_num_edges());
        self.input_child()[u]
    }
    fn get_input_left(&self, u: usize) -> Position {
        assert!(u < self.input_num_edges());
        self.input_left()[u]
    }
    fn get_input_right(&self, u: usize) -> Position {
        assert!(u < self.input_num_edges());
        self.input_right()[u]
    }
}

impl Drop for StreamingSimplifier {
    fn drop(&mut self) {
        let code = unsafe { crate::bindings::tsk_streaming_simplifier_free(&mut self.simplifier) };
        assert_eq!(code, 0);
    }
}

// TODO:
// 1. The edge buffer API is wrong here.
//    We need to encapsulate the existing type,
//    and make one whose public API does what we need.
// 2. If this works out, it means we need to extract
//    the core buffer ops out to a private type
//    and make public newtypes using it.
// FIXME: this function is unsafe b/c of how tskit-c
//        messes w/pointers behind the scenes.
//        Solution is to take ownership of the tables?
pub fn simplfify_from_buffer<O: Into<crate::SimplificationOptions>>(
    samples: &[NodeId],
    options: O,
    tables: &mut TableCollection,
    buffer: &mut EdgeBuffer,
    node_map: Option<&mut [NodeId]>,
) -> Result<(), TskitError> {
    // have to take copies of the current members of
    // the edge table.
    let mut last_parent_time = -1.0;
    let mut head_index: Vec<usize> = buffer
        .head
        .iter()
        .enumerate()
        .filter(|(_, j)| **j != usize::MAX)
        .map(|(i, _)| i)
        .collect();

    let node_time = tables.nodes().time_slice();
    head_index.sort_by(|a, b| node_time[*a].partial_cmp(&node_time[*b]).unwrap());
    let mut simplifier = StreamingSimplifier::new(samples, options, tables)?;
    // Simplify the most recent births
    //for (i, h) in buffer.head.iter().rev().enumerate() {
    for h in head_index.into_iter() {
        let parent = i32::try_from(h).unwrap();
        simplifier.add_edge(
            buffer.left[buffer.head[h]],
            buffer.right[buffer.head[h]],
            parent.into(),
            buffer.child[buffer.head[h]],
        )?;
        let mut next = buffer.next[buffer.head[h]];
        assert!(parent >= 0);
        while next != usize::MAX {
            assert!(next < buffer.left.len());
            simplifier.add_edge(
                buffer.left[next],
                buffer.right[next],
                parent.into(),
                buffer.child[next],
            )?;
            next = buffer.next[next];
        }
        simplifier.merge_ancestors(parent.into())?;

        // major stress-test -- delete later
        //{
        //    let l = tables.edges().left_slice();
        //    let p = tables.edges().parent_slice();
        //    let c = tables.edges().child_slice();
        //    let mut i = 0;
        //    while i < l.len() {
        //        let pi = p[i];
        //        while i < l.len() && p[i] == pi {
        //            if i > 0 && c[i] == c[i - 1] {
        //                assert_ne!(
        //                    l[i],
        //                    l[i - 1],
        //                    "{:?},{:?} | {:?},{:?} | {:?},{:?} => {:?}",
        //                    p[i],
        //                    p[i - 1],
        //                    c[i],
        //                    c[i - 1],
        //                    l[i],
        //                    l[i - 1],
        //                    edge_check
        //                );
        //            }
        //            i += 1;
        //        }
        //    }
        //}
    }
    buffer.release_memory();

    // Simplify pre-existing edges.
    //let mut i = 0;
    //let num_input_edges = simplifier.input_num_edges();
    //while i < num_input_edges {
    //    let p = simplifier.get_input_parent(i);
    //    //let mut edge_check: Vec<(NodeId, Position)> = vec![];
    //    while i < num_input_edges && simplifier.get_input_parent(i) == p {
    //        //assert!(!edge_check.iter().any(|x| *x == (child[i], left[i])));
    //        simplifier.add_edge(
    //            simplifier.get_input_left(i),
    //            simplifier.get_input_right(i),
    //            simplifier.get_input_parent(i),
    //            simplifier.get_input_child(i),
    //        )?;
    //        //edge_check.push((child[i], left[i]));
    //        i += 1;
    //    }
    //    simplifier.merge_ancestors(p)?;
    //    // major stress-test -- delete later
    //    //{
    //    //    let l = tables.edges().left_slice();
    //    //    let p = tables.edges().parent_slice();
    //    //    let c = tables.edges().child_slice();
    //    //    let mut i = 0;
    //    //    while i < l.len() {
    //    //        let pi = p[i];
    //    //        while i < l.len() && p[i] == pi {
    //    //            if i > 0 && c[i] == c[i - 1] {
    //    //                assert_ne!(
    //    //                    l[i],
    //    //                    l[i - 1],
    //    //                    "{:?},{:?} | {:?},{:?} | {:?},{:?} => {:?}",
    //    //                    p[i],
    //    //                    p[i - 1],
    //    //                    c[i],
    //    //                    c[i - 1],
    //    //                    l[i],
    //    //                    l[i - 1],
    //    //                    edge_check
    //    //                );
    //    //            }
    //    //            i += 1;
    //    //        }
    //    //    }
    //    //}
    //}

    simplifier.finalise(node_map)?;
    Ok(())
}

#[test]
fn test_pre_simplification() {
    let mut tables = TableCollection::new(10.).unwrap();
    let mut buffer = EdgeBuffer::default();
    let p0 = tables.add_node(0, 1.0, -1, -1).unwrap();
    let p1 = tables.add_node(0, 1.0, -1, -1).unwrap();
    let c0 = tables.add_node(0, 0.0, -1, -1).unwrap();
    let c1 = tables.add_node(0, 0.0, -1, -1).unwrap();
    buffer.setup_births(&[p0, p1], &[c0, c1]).unwrap();

    // Record data in a way that intentionally
    // breaks what tskit wants:
    // * children are not sorted in increading order
    //   of id.
    buffer.record_birth(0, 3, 5.0, 10.0).unwrap();
    buffer.record_birth(0, 2, 0.0, 5.0).unwrap();
    buffer.record_birth(1, 3, 0.0, 5.0).unwrap();
    buffer.record_birth(1, 2, 5.0, 10.0).unwrap();
    buffer.finalize_births();
    buffer.pre_simplification(&mut tables).unwrap();
    assert_eq!(tables.edges().num_rows(), 4);
    tables.simplify(&[2, 3], 0, false).unwrap();
    assert_eq!(tables.edges().num_rows(), 0);
}

#[test]
fn test_post_simplification() {
    let mut tables = TableCollection::new(10.).unwrap();
    let p0 = tables.add_node(0, 1.0, -1, -1).unwrap();
    let p1 = tables.add_node(0, 1.0, -1, -1).unwrap();
    let c0 = tables.add_node(0, 0.0, -1, -1).unwrap();
    let c1 = tables.add_node(0, 0.0, -1, -1).unwrap();
    let _e0 = tables.add_edge(0.0, 10.0, p0, c0).unwrap();
    let _e1 = tables.add_edge(0.0, 10.0, p1, c1).unwrap();
    assert_eq!(tables.edges().num_rows(), 2);
    let alive = vec![c0, c1]; // the children have replaced the parents
    let mut buffer = EdgeBuffer::default();
    buffer.post_simplification(&alive, &mut tables).unwrap();
    assert_eq!(tables.edges().num_rows(), 2);
}
