use crate::ornaments::GetHash;
use crate::hash_str::{HashStr,SIZE_HASH};
use hashbrown::HashTable;

/// "Host" backing storage for cached HashStrs.
/// Pass this to HashStrCache.intern_with to do string interning with deduplication.
pub struct HashStrHost(bumpalo::Bump);
impl HashStrHost{
	#[inline]
	pub fn new()->Self{
		Self(bumpalo::Bump::new())
	}

	#[doc(hidden)]
	pub unsafe fn clear(&mut self){
		self.0.reset();
	}
	/// Allocate a new HashStr, regardless of duplicates.
	#[inline]
	pub fn alloc(&self,str:&str)->&HashStr{
		self.alloc_with_hash(str.get_hash(),str)
	}
	#[inline]
	pub(crate) fn alloc_with_hash(&self,hash:u64,str:&str)->&HashStr{
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
		unsafe{HashStr::ref_from_bytes(new_hash_str_bytes)}
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
	#[inline]
	pub fn clear(&mut self){
		self.entries.clear();
	}
	/// Fetch an existing HashStr, utilizing the precalculated hash if possible.
	#[inline]
	pub fn get(&self,index:impl GetHash+AsRef<str>+Copy)->Option<&'str HashStr>{
		self.get_with_hash(index.get_hash(),index.as_ref())
	}
	#[inline]
	pub(crate) fn get_with_hash(&self,hash:u64,str:&str)->Option<&'str HashStr>{
		self.entries.find(hash,|&s|s.as_str()==str).copied()
	}
	/// Intern the provided HashStr, utilizing the precalculated hash.
	/// This will reuse an existing HashStr if one exists.
	/// The lifetime of the provided HashStr must outlive the HashStrCache.
	/// Allocates no new HashStrs.
	#[inline]
	pub fn intern(&mut self,hash_str:&'str HashStr)->&'str HashStr{
		let hash=hash_str.precomputed_hash();
		let str=hash_str.as_str();
		self.entries.entry(hash,|&s|s.as_str()==str,|hash_str|hash_str.precomputed_hash()).or_insert(hash_str).get()
	}
	/// Intern the provided string.  This will return an existing HashStr if one exists,
	/// or allocate a new one on the provided HashStrHost.
	#[inline]
	pub fn intern_with(&mut self,host:&'str HashStrHost,str:&str)->&'str HashStr{
		let hash=str.get_hash();
		self.intern_with_hash(||host.alloc_with_hash(hash,str),hash,str)
	}
	#[inline]
	pub(crate) fn intern_with_hash(&mut self,with:impl FnOnce()->&'str HashStr,hash:u64,str:&str)->&'str HashStr{
		self.entries.entry(
			hash,
			|&s|s.as_str()==str,
			|hash_str|hash_str.precomputed_hash(),
		).or_insert_with(with).get()
	}
}

#[test]
fn test_cache(){
	let lifetime_host=HashStrHost::new();
	let mut words=HashStrCache::new();

	// borrow Words mutably
	let a:&HashStr=words.intern_with(&lifetime_host,"bruh");
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
	let a2:&HashStr=words.intern(a);
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
