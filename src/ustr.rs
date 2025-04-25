#[derive(PartialEq,Eq)]
pub struct Ustr{
	hash:u64,
	ustr:str,
}

impl Ustr{
	#[inline]
	pub fn precomputed_hash(&self)->u64{
		self.hash
	}
	#[inline]
	pub fn as_str(&self)->&str{
		&self.ustr
	}
	#[inline]
	pub fn as_ustr_bytes(&self)->&[u8]{
		unsafe{core::mem::transmute(self)}
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

impl core::ops::Deref for Ustr{
	type Target=str;
	#[inline]
	fn deref(&self)->&Self::Target{
		self.as_str()
	}
}

impl core::fmt::Display for Ustr{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

// an anonymous Ustr that is not owned by a StringCache
pub(crate) fn anonymous(value: &str) -> Box<Ustr> {
	use std::hash::Hasher;
	let hash = {
		let mut hasher = ahash::AHasher::default();
		hasher.write(value.as_bytes());
		hasher.finish()
	};
	let mut bytes=Vec::with_capacity(value.len()+core::mem::size_of::<u64>());
	bytes.extend_from_slice(&hash.to_ne_bytes());
	bytes.extend_from_slice(value.as_bytes());
	let boxed=bytes.into_boxed_slice();
	// SAFETY: hold my beer
	unsafe{core::mem::transmute(boxed)}
}
