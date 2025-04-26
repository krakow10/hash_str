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
	#[inline]
	pub fn with_capacity(capacity:usize)->Self{
		Self(bumpalo::Bump::with_capacity(capacity))
	}

	#[doc(hidden)]
	pub unsafe fn clear(&mut self){
		self.0.reset();
	}
	/// Allocate a new HashStr, regardless of duplicates.
	#[inline]
	pub fn alloc(&self,str:&str)->&HashStr{
		self.alloc_str_with_hash(str.get_hash(),str)
	}
	#[inline]
	pub(crate) fn alloc_str_with_hash(&self,hash:u64,str:&str)->&HashStr{
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

fn get_precomputed_hash(&hash_str:&&HashStr)->u64{
	hash_str.precomputed_hash()
}

impl<'str> HashStrCache<'str>{
	#[inline]
	pub fn new()->HashStrCache<'str>{
		HashStrCache{
			entries:HashTable::new(),
		}
	}
	#[inline]
	pub fn with_capacity(capacity:usize)->HashStrCache<'str>{
		HashStrCache{
			entries:HashTable::with_capacity(capacity),
		}
	}
	#[inline]
	pub fn clear(&mut self){
		self.entries.clear();
	}
	/// Fetch an existing HashStr, utilizing the precalculated hash if possible.
	#[inline]
	pub fn get<I>(&self,index:&I)->Option<&'str HashStr>
	where I:GetHash+AsRef<str>+?Sized{
		self.presence(index).get()
	}
	/// Finds an existing HashStr if it is present.  Can be chained to
	/// spill missing items into another cache, reusing the hash.
	/// The lifetimes of the chained caches must be in shrinking order
	/// so that the returned type has the lifetime of the shortest cache.
	///
	/// ```rust
	/// cache1.presence("str").or_present_in(cache2).or_intern_with(host,cache3);
	/// ```
	#[inline]
	pub fn presence<'a,I>(&self,index:&'a I)->Presence<'a,&'str HashStr>
	where I:GetHash+AsRef<str>+?Sized{
		self.presence_str_with_hash(index.get_hash(),index.as_ref())
	}
	#[inline]
	pub(crate) fn presence_str_with_hash<'a>(&self,hash:u64,str:&'a str)->Presence<'a,&'str HashStr>{
		match self.entries.find(hash,|&s|s.as_str()==str){
			Some(entry)=>Presence::Present(entry),
			None=>Presence::NotPresent(HashedStr{hash,str})
		}
	}
	/// Intern the provided HashStr, utilizing the precalculated hash.
	/// This will reuse an existing HashStr if one exists.
	/// The lifetime of the provided HashStr must outlive the HashStrCache.
	/// Allocates no new HashStrs.
	#[inline]
	pub fn intern(&mut self,hash_str:&'str HashStr)->&'str HashStr{
		let hash=hash_str.precomputed_hash();
		let str=hash_str.as_str();
		self.intern_str_with_hash(||hash_str,hash,str)
	}
	/// Intern the provided string.  This will return an existing HashStr if one exists,
	/// or allocate a new one on the provided HashStrHost.
	#[inline]
	pub fn intern_str_with(&mut self,host:&'str HashStrHost,str:&str)->&'str HashStr{
		let hash=str.get_hash();
		self.intern_str_with_hash(||host.alloc_str_with_hash(hash,str),hash,str)
	}
	#[inline]
	pub(crate) fn intern_str_with_hash(&mut self,with:impl FnOnce()->&'str HashStr,hash:u64,str:&str)->&'str HashStr{
		self.entries.entry(
			hash,
			|&s|s.as_str()==str,
			get_precomputed_hash,
		).or_insert_with(with).get()
	}
	#[inline]
	pub fn iter<'a>(&'a self)->impl Iterator<Item=&'str HashStr>+'a{
		self.into_iter()
	}
	#[inline]
	pub fn len(&self)->usize{
		self.entries.len()
	}
	#[inline]
	pub fn capacity(&self)->usize{
		self.entries.capacity()
	}
	#[inline]
	pub fn reserve(&mut self,additional:usize){
		self.entries.reserve(additional,get_precomputed_hash)
	}
}

impl<'str,'a> IntoIterator for &'a HashStrCache<'str>{
	type Item=&'str HashStr;
	type IntoIter=core::iter::Copied<hashbrown::hash_table::Iter<'a,&'str HashStr>>;
	#[inline]
	fn into_iter(self)->Self::IntoIter{
		self.entries.iter().copied()
	}
}

// Idea:
// string cache chaining
// cache1.presence("str").or_present_in(cache2).or_intern_with(host,cache3)
// the point is that the hash is passed along with the entry

/// Represents the presence of a HashStr in a HashStrCache.  Call .get()
/// to grab the present value.  Holds the computed hash of the index str
/// to be reused when checking additional caches.
pub enum Presence<'a,T>{
	Present(T),
	NotPresent(HashedStr<'a>),
}

/// A type used internally to reuse the same hash while checking multiple caches.
/// See HashStr for the more important data type.
pub struct HashedStr<'a>{
	hash:u64,
	str:&'a str,
}

impl<'a,T> Presence<'a,T>{
	#[inline]
	pub fn get(self)->Option<T>{
		match self{
			Presence::Present(entry)=>Some(entry),
			Presence::NotPresent(_)=>None,
		}
	}
}
impl<'a,'str> Presence<'a,&'str HashStr>{
	/// If the HashStr was not present, check if it is present in the specified cache.
	/// Note that this requires the lifetime of items from the previous caches
	/// to be extendable to the lifetime of the specified cache to make the return types match.
	#[inline]
	pub fn or_present_in<'new>(self,cache:&'a HashStrCache<'new>)->Presence<'a,&'new HashStr> where 'str:'new{
		match self{
			Presence::Present(entry)=>Presence::Present(entry),
			Presence::NotPresent(HashedStr{hash,str})=>cache.presence_str_with_hash(hash,str),
		}
	}
	/// If the HashStr was not present, intern the string using the specified host storage
	/// into the specified cache.
	/// Note that this requires the lifetime of items from the previous caches
	/// to be extendable to the lifetime of the specified cache to make the return types match.
	#[inline]
	pub fn or_intern_with<'new>(self,host:&'new HashStrHost,cache:&mut HashStrCache<'new>)->&'new HashStr where 'str:'new{
		match self{
			Presence::Present(entry)=>entry,
			Presence::NotPresent(HashedStr{hash,str})=>cache.intern_str_with_hash(||host.alloc_str_with_hash(hash,str),hash,str),
		}
	}
}

#[test]
fn test_cache(){
	let lifetime_host=HashStrHost::new();
	let mut words=HashStrCache::new();

	// borrow Words mutably
	let a:&HashStr=words.intern_str_with(&lifetime_host,"bruh");
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
