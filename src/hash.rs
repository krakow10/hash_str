use crate::hash_str::HashStr;
use std::collections::{HashMap,HashSet};
use core::hash::{BuildHasher,Hash,Hasher};

pub(crate) fn make_hash(value:&str)->u64{
	let not_random_state=ahash::RandomState::with_seeds(0,0,0,0);
	let mut hasher=not_random_state.build_hasher();
	hasher.write(value.as_bytes());
	hasher.finish()
}

// Just feed the precomputed hash into the Hasher. Note that this will of course
// be terrible unless the Hasher in question is expecting a precomputed hash.
impl Hash for HashStr{
	#[inline]
    fn hash<H:Hasher>(&self,state:&mut H){
        state.write_u64(self.precomputed_hash());
    }
}

/// A standard `HashMap` using `&HashStr` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type HashStrMap<'a,V>=HashMap<&'a HashStr,V,RandomState>;

/// A standard `HashSet` using `&HashStr` as the key type with a custom `Hasher`
/// that just uses the precomputed hash for speed instead of calculating it.
pub type HashStrSet<'a>=HashSet<&'a HashStr,RandomState>;

// generate a random u64 for each HashMap to avoid quadratic behaviour
#[doc(hidden)]
#[derive(Copy,Clone,Debug)]
pub struct RandomState{
	random_state:u64,
}
impl Default for RandomState{
	fn default()->Self{
		// hashbrown does this
		let mut seed=0;
		let ptr=core::ptr::addr_of!(seed);
		let mut hasher=ahash::AHasher::default();
		hasher.write_usize(ptr as usize);
		seed=hasher.finish();
		Self{
			random_state:seed,
		}
	}
}

impl BuildHasher for RandomState{
	type Hasher=XORHasher;
	fn build_hasher(&self)->Self::Hasher{
		XORHasher{
			random_state:self.random_state,
			hash:0,
		}
	}
}

/// XOR the provided precomputed hash with the random state
/// so creating a hash table from another doesn't have
/// accidental quadratic time complexity.
#[doc(hidden)]
pub struct XORHasher{
	random_state:u64,
    hash:u64,
}

impl Hasher for XORHasher{
	#[inline]
    fn write_u64(&mut self,value:u64){
        self.hash=value^self.random_state;
    }
    #[inline]
    fn write(&mut self,_bytes:&[u8]){
    	unreachable!();
    }
    #[inline]
    fn finish(&self)->u64{
        self.hash
    }
}

impl Default for XORHasher{
	fn default()->Self{
		RandomState::default().build_hasher()
	}
}

#[test]
fn test_hashing(){
	let u1=&*HashStr::anonymous("the quick brown fox".to_owned());
	let u2=&*HashStr::anonymous("jumps over the lazy dog".to_owned());

	let mut hasher=XORHasher::default();
	u1.hash(&mut hasher);
	assert_eq!(hasher.finish(),u1.precomputed_hash());

	let mut hasher=XORHasher::default();
	u2.hash(&mut hasher);
	assert_eq!(hasher.finish(),u2.precomputed_hash());

	let mut hm=HashStrMap::<u32>::default();
	hm.insert(u1,17);
	hm.insert(u2,42);

	assert_eq!(hm.get(u1), Some(&17));
	assert_eq!(hm.get(u2), Some(&42));
}
