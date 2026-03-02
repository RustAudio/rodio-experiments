use crate::effects::pure_effect;

pure_effect! {
    supports_dynamic_source
    struct WithData<D> {
        data: D,
    }

    fn next(&mut self) -> Option<Sample> {
        self.inner.next()
    }

    fn new(source: S, data: D) -> WithData<Self> {
        Self {
            inner: source,
            data,
        }
    }

    pub fn data_mut(&mut self) -> &mut D {
        &mut self.data
    }

    pub fn data(&self) -> &D {
        &self.data
    }
}
