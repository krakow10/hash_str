use crate::ornaments::GetHash;
use crate::hash_str::{HashStr,SIZE_HASH};
use hashbrown::HashTable;

/// "Host" backing storage for cached HashStrs.
/// Pass this to HashStrCache.intern_str to create new HashStrs.
pub struct HashStrHost(bumpalo::Bump);
impl HashStrHost{
	pub fn new()->Self{
		Self(bumpalo::Bump::new())
	}
	fn alloc(&self,hash:u64,str:&str)->&HashStr{
		let hash_str_len=SIZE_HASH+str.len();
		let layout=bumpalo::core_alloc::alloc::Layout::from_size_align(hash_str_len,SIZE_HASH).unwrap();
		// alloc empty bytes for new HashStr
		let new_hash_str_bytes_ptr=self.0.alloc_layout(layout).as_ptr();
		// SAFETY: bumpalo panics if allocation fails
		// meaning ptr is always non-null
		let new_hash_str_bytes=unsafe{core::slice::from_raw_parts_mut(
			new_hash_str_bytes_ptr,
			hash_str_len
		)};
		new_hash_str_bytes[..SIZE_HASH].copy_from_slice(&hash.to_ne_bytes());
		new_hash_str_bytes[SIZE_HASH..].copy_from_slice(str.as_bytes());
		// SAFETY: A valid HashStr is constructed in new_hash_str_bytes
		let new_hash_str=unsafe{HashStr::ref_from_bytes(new_hash_str_bytes)};

		new_hash_str
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
	/// Fetch an existing HashStr, utilizing the precalculated hash if possible.
	#[inline]
	pub fn get(&self,index:impl GetHash+AsRef<str>+Copy)->Option<&'str HashStr>{
		self.get_with_hash(index.get_hash(),index.as_ref())
	}
	#[inline]
	fn get_with_hash(&self,hash:u64,str:&str)->Option<&'str HashStr>{
		self.entries.find(hash,|&s|s.as_str()==str).copied()
	}
	/// Intern the provided string, utilizing the precalculated hash if possible.
	/// This will reuse an existing HashStr without allocating if one exists.
	#[inline]
	pub fn intern(&mut self,host:&'str HashStrHost,index:impl GetHash+AsRef<str>+Copy)->&'str HashStr{
		self.intern_with_hash(host,index.get_hash(),index.as_ref())
	}
	#[inline]
	fn intern_with_hash(&mut self,host:&'str HashStrHost,hash:u64,str:&str)->&'str HashStr{
		// check exists
		if let Some(hash_str)=self.get_with_hash(hash,str){
			return hash_str;
		}

		// create new
		let new_hash_str=host.alloc(hash,str);

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
	let a:&HashStr=words.intern(&lifetime_host,"bruh");
	// drop mutable borrow and borrow immutably
	let b:&HashStr=words.get("bruh").unwrap();
	// compare both references; this is impossible when
	// the lifetimes of a and b are derived from
	// the borrows in .get and .intern
	// e.g.
	// fn    get<'a>(&'a     self,s:&str)->Option<&'a HashStr>{
	// fn intern<'a>(&'a mut self,s:&str)->       &'a HashStr {
	// instead of the lifetime of the underlying data 'str
	assert_eq!(a,b);
	assert!(core::ptr::addr_eq(a,b));

	// it also works with a HashStr as the index
	let a2:&HashStr=words.intern(&lifetime_host,a);
	let b2:&HashStr=words.get(b).unwrap();
	assert_eq!(a,a2);
	assert_eq!(b,b2);
	assert!(core::ptr::addr_eq(a,a2));
	assert!(core::ptr::addr_eq(b,b2));

	// with alloc owned by StringHost this is no longer UB
	drop(words);
	assert_eq!(a,b);

	// dropping LifetimeHost gives the desired compile error
	// drop(lifetime_host);
	assert_eq!(a,b);
}
