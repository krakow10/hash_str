use crate::ustr::Ustr;
use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, Hasher},
};

/// A standard `HashMap` using `Ustr` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type UstrMap<'a,V> = HashMap<&'a Ustr, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `HashSet` using `Ustr` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type UstrSet<'a> = HashSet<&'a Ustr, BuildHasherDefault<IdentityHasher>>;

/// The worst hasher in the world -- the identity hasher.
#[doc(hidden)]
#[derive(Default)]
pub struct IdentityHasher {
    hash: u64,
}

impl Hasher for IdentityHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() == 8 {
            self.hash = u64::from_ne_bytes(bytes.try_into().unwrap());
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

#[test]
fn test_hashing() {
    use crate::ustr::Ustr;

    use std::hash::Hash;
	let u1:Box<Ustr> = {
		let string="the quick brown fox";
		let hash = {
			let mut hasher = ahash::AHasher::default();
			hasher.write(string.as_bytes());
			hasher.finish()
		};
		let mut bytes=Vec::new();
		bytes.extend_from_slice(&hash.to_ne_bytes());
		bytes.extend_from_slice(&string.len().to_ne_bytes());
		bytes.extend_from_slice(string.as_bytes());
		let boxed=bytes.into_boxed_slice();
		unsafe{core::mem::transmute(boxed)}
	};
	let u2:Box<Ustr> = {
		let string="jumped over the lazy dog";
		let hash = {
			let mut hasher = ahash::AHasher::default();
			hasher.write(string.as_bytes());
			hasher.finish()
		};
		let mut bytes=Vec::new();
		bytes.extend_from_slice(&hash.to_ne_bytes());
		bytes.extend_from_slice(&string.len().to_ne_bytes());
		bytes.extend_from_slice(string.as_bytes());
		let boxed=bytes.into_boxed_slice();
		unsafe{core::mem::transmute(boxed)}
	};

	let mut hasher = IdentityHasher::default();
	u1.hash(&mut hasher);
	assert_eq!(hasher.finish(), u1.precomputed_hash());

	let mut hasher = IdentityHasher::default();
	u2.hash(&mut hasher);
	assert_eq!(hasher.finish(), u2.precomputed_hash());

	let mut hm = UstrMap::<u32>::default();
	hm.insert(&u1, 17);
	hm.insert(&u2, 42);

	assert_eq!(hm.get(u1.as_ref()), Some(&17));
	assert_eq!(hm.get(u2.as_ref()), Some(&42));
}
