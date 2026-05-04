pub struct SiteRefIterator<'ts> {
    pub sites: &'ts [super::bindings::tsk_site_t],
}

impl<'ts> Iterator for SiteRefIterator<'ts> {
    type Item = super::SiteRef<'ts>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((l, r)) = self.sites.split_first() {
            self.sites = r;
            Some(super::new_site_ref(l))
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.sites = if n < self.sites.len() {
            &self.sites[n..]
        } else {
            &[]
        };
        self.next()
    }
}
