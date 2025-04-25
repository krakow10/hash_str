use crate::ustr::Ustr;
use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, Hasher},
};

/// A standard `HashMap` using `&Ustr` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type UstrMap<'a,V> = HashMap<&'a Ustr, V, BuildHasherDefault<IdentityHasher>>;

/// A standard `HashSet` using `&Ustr` as the key type with a custom `Hasher`
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
    fn write_u64(&mut self, value: u64) {
        self.hash = value;
    }
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.hash = u64::from_ne_bytes(bytes.try_into().unwrap());
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

#[test]
fn test_hashing() {
	use crate::ustr::anonymous;
	use std::hash::Hash;

	let u1=&*anonymous("the quick brown fox");
	let u2=&*anonymous("jumped over the lazy dog");

	let mut hasher = IdentityHasher::default();
	u1.hash(&mut hasher);
	assert_eq!(hasher.finish(), u1.precomputed_hash());

	let mut hasher = IdentityHasher::default();
	u2.hash(&mut hasher);
	assert_eq!(hasher.finish(), u2.precomputed_hash());

	let mut hm = UstrMap::<u32>::default();
	hm.insert(u1, 17);
	hm.insert(u2, 42);

	assert_eq!(hm.get(u1), Some(&17));
	assert_eq!(hm.get(u2), Some(&42));
}
