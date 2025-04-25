#[derive(PartialEq,Eq)]
pub struct HashStr{
	hash:u64,
	hash_str:str,
}

impl HashStr{
	#[inline]
	pub fn precomputed_hash(&self)->u64{
		self.hash
	}
	#[inline]
	pub fn as_str(&self)->&str{
		&self.hash_str
	}
	#[inline]
	pub fn as_hash_str_bytes(&self)->&[u8]{
		unsafe{core::mem::transmute(self)}
	}
}

// Just feed the precomputed hash into the Hasher. Note that this will of course
// be terrible unless the Hasher in question is expecting a precomputed hash.
impl std::hash::Hash for HashStr {
	#[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.precomputed_hash());
    }
}

pub(crate) fn make_hash(value:&str)->u64{
	use std::hash::Hasher;
	let mut hasher=ahash::AHasher::default();
	hasher.write(value.as_bytes());
	hasher.finish()
}

// an anonymous HashStr that is not owned by a StringCache
pub(crate) fn anonymous(value: &str) -> Box<HashStr> {
	let hash=make_hash(value);
	let mut bytes=Vec::with_capacity(value.len()+core::mem::size_of::<u64>());
	bytes.extend_from_slice(&hash.to_ne_bytes());
	bytes.extend_from_slice(value.as_bytes());
	let boxed=bytes.into_boxed_slice();
	// SAFETY: hold my beer
	unsafe{core::mem::transmute(boxed)}
}
