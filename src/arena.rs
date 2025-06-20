#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjRef(pub u32);

#[derive(Debug)]
pub struct Arena<T>(Vec<T>);

impl<T> Arena<T> {
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self(vec)
    }
    /// Create an empty pool.
    pub(crate) fn default() -> Self {
        Self(Vec::new())
    }

    /// Dereference an object reference, obtaining the underlying `ParseTreeNode`.
    pub fn get(&self, obj_ref: ObjRef) -> &T {
        &self.0[obj_ref.0 as usize]
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Add an object to the pool and get a reference to it.
    pub fn add(&mut self, obj: T) -> ObjRef {
        let idx = self.0.len();
        self.0.push(obj);
        ObjRef(idx.try_into().expect("too many exprs in the pool"))
    }
}
