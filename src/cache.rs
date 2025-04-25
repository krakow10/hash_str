use crate::hash_str::make_hash;
use crate::hash_str::{HashStr,SIZE_U64};
use hashbrown::HashTable;

/// "Host" backing storage for cached HashStrs.
/// Pass this to HashStrCache.intern_str to create new HashStrs.
pub struct HashStrHost(bumpalo::Bump);
impl HashStrHost{
	pub fn new()->Self{
		Self(bumpalo::Bump::new())
	}
}

/// Cache of existing entries in a HashStrHost.
/// Useful to deduplicate a finite set of unique strings,
/// minimizing the allocation of new strings.
pub struct HashStrCache<'str>{
	entries:HashTable<&'str HashStr>,
}

impl<'str> HashStrCache<'str>{
	#[inline]
	pub fn new()->HashStrCache<'str>{
		HashStrCache{
			entries:HashTable::new(),
		}
	}
	/// Calculate the hash of a &str and fetch an existing HashStr.
	#[inline]
	pub fn get_str(&self,str:&str)->Option<&'str HashStr>{
		let hash=make_hash(str);
		self.get_with_hash(hash,str)
	}
	/// Fetch an existing HashStr using the precalculated hash.
	#[inline]
	pub fn get_hash_str(&self,hash_str:&HashStr)->Option<&'str HashStr>{
		self.get_with_hash(hash_str.precomputed_hash(),hash_str.as_str())
	}
	#[inline]
	fn get_with_hash(&self,hash:u64,str:&str)->Option<&'str HashStr>{
		self.entries.find(hash,|&s|s.as_str()==str).copied()
	}
	/// Calculate the hash of a &str and intern it into the HashStrHost.
	/// Returns the newly allocated HashStr or an existing one if there was one.
	#[inline]
	pub fn intern_str(&mut self,host:&'str HashStrHost,str:&str)->&'str HashStr{
		let hash=make_hash(str);
		self.intern_with_hash(host,hash,str)
	}
	/// Intern the provided HashStr into the HashStrHost using the precalculated hash.
	/// Returns the newly allocated HashStr or an existing one if there was one.
	#[inline]
	pub fn intern_hash_str(&mut self,host:&'str HashStrHost,hash_str:&HashStr)->&'str HashStr{
		self.intern_with_hash(host,hash_str.precomputed_hash(),hash_str.as_str())
	}
	#[inline]
	fn intern_with_hash(&mut self,host:&'str HashStrHost,hash:u64,str:&str)->&'str HashStr{
		// check exists
		if let Some(hash_str)=self.get_with_hash(hash,str){
			return hash_str;
		}

		let hash_str_len=SIZE_U64+str.len();
		let layout=bumpalo::core_alloc::alloc::Layout::from_size_align(hash_str_len,SIZE_U64).unwrap();
		// alloc empty bytes for new HashStr
		let new_hash_str_bytes_ptr=host.0.alloc_layout(layout).as_ptr();
		// SAFETY: bumpalo panics if allocation fails
		// meaning ptr is always non-null
		let new_hash_str_bytes=unsafe{core::slice::from_raw_parts_mut(
			new_hash_str_bytes_ptr,
			hash_str_len
		)};
		new_hash_str_bytes[..SIZE_U64].copy_from_slice(&hash.to_ne_bytes());
		new_hash_str_bytes[SIZE_U64..].copy_from_slice(str.as_bytes());
		// SAFETY: A valid HashStr is constructed in new_hash_str_bytes
		let new_hash_str=unsafe{HashStr::ref_from_bytes(new_hash_str_bytes)};

		// insert into entries
		self.entries.insert_unique(
			hash,
			new_hash_str,
			|hash_str|hash_str.precomputed_hash()
		).get()
	}
}

#[test]
fn test_cache(){
	let lifetime_host=HashStrHost::new();
	let mut words=HashStrCache::new();

	// borrow Words mutably
	let a:&HashStr=words.intern_str(&lifetime_host,"bruh");
	// drop mutable borrow and borrow immutably
	let b:&HashStr=words.get_str("bruh").unwrap();
	// compare both references; this is impossible when
	// the lifetimes of a and b are derived from
	// the borrows in .get and .intern
	// e.g.
	// fn    get<'a>(&'a     self,s:&str)->Option<&'a HashStr>{
	// fn intern<'a>(&'a mut self,s:&str)->       &'a HashStr {
	// instead of the lifetime of the underlying data 'str
	assert_eq!(a,b);

	// with alloc owned by StringHost this is no longer UB
	drop(words);
	assert_eq!(a,b);

	// dropping LifetimeHost gives the desired compile error
	// drop(lifetime_host);
	assert_eq!(a,b);
}
