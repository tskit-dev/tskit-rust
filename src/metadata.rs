use crate::bindings::{tsk_id_t, tsk_size_t};
use thiserror::Error;

pub trait MetadataRoundtrip {
    fn encode(&self) -> Result<Vec<u8>, MetadataError>;
    fn decode(md: &[u8]) -> Result<Self, MetadataError>
    where
        Self: Sized;
}

pub(crate) struct EncodedMetadata {
    encoded: Option<Vec<u8>>,
}

impl EncodedMetadata {
    pub(crate) fn new(md: Option<&dyn MetadataRoundtrip>) -> Result<Self, MetadataError> {
        match md {
            Some(x) => {
                let e = x.encode()?;
                Ok(Self { encoded: Some(e) })
            }
            None => Ok(Self { encoded: None }),
        }
    }

    pub(crate) fn as_ptr(&self) -> *const libc::c_char {
        match &self.encoded {
            Some(x) => x.as_ptr() as *const libc::c_char,
            None => std::ptr::null(),
        }
    }

    pub(crate) fn len(&self) -> tsk_size_t {
        match &self.encoded {
            Some(x) => x.len() as tsk_size_t,
            None => 0,
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum MetadataError {
    /// Error related to types implementing
    /// [``MetadataRoundtrip``]
    #[error("{}", *msg)]
    RoundtripError { msg: String },
}

pub(crate) fn char_column_to_vector(
    column: *const libc::c_char,
    column_offset: *const tsk_size_t,
    row: tsk_id_t,
    num_rows: tsk_size_t,
    column_length: tsk_size_t,
) -> Result<Option<Vec<u8>>, crate::TskitError> {
    if row < 0 || (row as tsk_size_t) >= num_rows {
        return Err(crate::TskitError::IndexError {});
    }
    if column_length == 0 {
        return Ok(None);
    }
    let start = unsafe { *column_offset.offset(row as isize) };
    let stop = if (row as tsk_size_t) < num_rows {
        unsafe { *column_offset.offset((row + 1) as isize) }
    } else {
        column_length
    };
    if start >= stop {
        return Ok(None);
    }
    if column_length == 0 {
        return Ok(None);
    }
    let mut buffer = vec![];
    for i in start..stop {
        buffer.push(unsafe { *column.offset(i as isize) } as u8);
    }
    Ok(Some(buffer))
}

#[cfg(test)]
mod test {
    use super::*;

    struct F {
        x: i32,
        y: u32,
    }

    impl MetadataRoundtrip for F {
        fn encode(&self) -> Result<Vec<u8>, MetadataError> {
            let mut rv = vec![];
            rv.extend(self.x.to_le_bytes().iter().copied());
            rv.extend(self.y.to_le_bytes().iter().copied());
            Ok(rv)
        }
        fn decode(md: &[u8]) -> Result<Self, MetadataError> {
            use std::convert::TryInto;
            let (x_int_bytes, rest) = md.split_at(std::mem::size_of::<i32>());
            let (y_int_bytes, _) = rest.split_at(std::mem::size_of::<u32>());
            Ok(Self {
                x: i32::from_le_bytes(x_int_bytes.try_into().unwrap()),
                y: u32::from_le_bytes(y_int_bytes.try_into().unwrap()),
            })
        }
    }

    #[test]
    fn test_metadata_round_trip() {
        let f = F { x: -3, y: 42 };
        let v = f.encode().unwrap();
        let c = v.as_ptr() as *const libc::c_char;
        let mut d = vec![];
        for i in 0..v.len() {
            d.push(unsafe { *c.add(i as usize) as u8 });
        }
        let df = F::decode(&d).unwrap();
        assert_eq!(f.x, df.x);
        assert_eq!(f.y, df.y);
    }

    #[test]
    fn test_encoded_metadata_roundtrip() {
        let f = F { x: -3, y: 42 };
        let enc = EncodedMetadata::new(Some(&f)).unwrap();
        let p = enc.as_ptr();
        let mut d = vec![];
        for i in 0..enc.len() {
            d.push(unsafe { *p.add(i as usize) as u8 });
        }
        let df = F::decode(&d).unwrap();
        assert_eq!(f.x, df.x);
        assert_eq!(f.y, df.y);
    }
}
