pub struct SiteRefIterator<'ts> {
    pub sites: &'ts [super::bindings::tsk_site_t],
    pub current: usize,
}

impl<'ts> Iterator for SiteRefIterator<'ts> {
    type Item = super::SiteRef<'ts>;
    fn next(&mut self) -> Option<Self::Item> {
        let n = self.sites.get(self.current).map(|s| super::new_site_ref(s));
        self.current += 1;
        n
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.current = n + 1;
        self.sites.get(n).map(|s| super::new_site_ref(s))
    }
}
