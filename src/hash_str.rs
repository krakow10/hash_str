/// HashStr is a dynamically sized type so it is used similarly to &str.
/// A hash is stored at the beginning followed by a str.  The length is
/// known by the fat pointer when in the form &HashStr.
#[derive(PartialEq,Eq)]
pub struct HashStr{
	hash:u64,
	str:str,
}

impl HashStr{
	#[inline]
	pub const fn precomputed_hash(&self)->u64{
		self.hash
	}
	#[inline]
	pub const fn as_str(&self)->&str{
		&self.str
	}
	/// Struct bytes including hash prefix and trailing str
	#[inline]
	pub const fn as_hash_str_bytes<'a>(&'a self)->&'a [u8]{
		// SAFETY: HashStr is always valid as bytes
		unsafe{core::mem::transmute(self)}
	}
	/// Create a `&HashStr` from bytes.
	///
	/// SAFETY: `&bytes[8..]` must be valid UTF-8
	#[inline]
	pub const unsafe fn ref_from_bytes<'a>(bytes:&'a [u8])->&'a Self{
		unsafe{core::mem::transmute(bytes)}
	}
	/// An anonymous HashStr that is not owned by a StringCache
	#[inline]
	pub fn anonymous(value: &str) -> Box<HashStr> {
		let hash=make_hash(value);
		let mut bytes=Vec::with_capacity(value.len()+core::mem::size_of::<u64>());
		bytes.extend_from_slice(&hash.to_ne_bytes());
		bytes.extend_from_slice(value.as_bytes());
		let boxed=bytes.into_boxed_slice();
		// SAFETY: hold my beer
		unsafe{core::mem::transmute(boxed)}
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
