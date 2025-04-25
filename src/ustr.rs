
#[derive(Clone,Copy,PartialEq,Eq)]
pub(crate) struct Header{
	hash:u64,
	len:usize,
}

#[derive(PartialEq,Eq)]
pub struct Ustr{
	header:Header,
	ustr:str,
}

impl Ustr{
	#[inline]
	pub fn precomputed_hash(&self)->u64{
		self.header.hash
	}
}

// Just feed the precomputed hash into the Hasher. Note that this will of course
// be terrible unless the Hasher in question is expecting a precomputed hash.
impl std::hash::Hash for Ustr {
	#[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.precomputed_hash().hash(state);
    }
}
