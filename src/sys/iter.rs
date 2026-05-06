use crate::sys::new_site_ref;

#[derive(Debug)]
#[repr(transparent)]
pub struct SiteRefIterator<'ts>(&'ts [super::bindings::tsk_site_t]);

impl<'ts> SiteRefIterator<'ts> {
    pub(crate) fn new(data: &'ts [super::bindings::tsk_site_t]) -> Self {
        Self(data)
    }
}

impl<'ts> Iterator for SiteRefIterator<'ts> {
    type Item = super::SiteRef<'ts>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((l, r)) = self.0.split_first() {
            self.0 = r;
            Some(super::new_site_ref(l))
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0 = if n < self.0.len() { &self.0[n..] } else { &[] };
        self.next()
    }
}

impl<'ts> DoubleEndedIterator for SiteRefIterator<'ts> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some((l, r)) = self.0.split_last() {
            self.0 = r;
            Some(new_site_ref(l))
        } else {
            None
        }
    }
}
