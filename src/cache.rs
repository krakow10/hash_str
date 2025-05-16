use crate::ornaments::{GetHash,HashedStr};
use crate::hash_str::{HashStr,SIZE_HASH};
use hashbrown::HashTable;

/// "Host" backing storage for cached HashStrs.
/// Pass this to HashStrCache.intern_with to do string interning with deduplication.
#[derive(Debug)]
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
	pub fn alloc<'a>(&self,index:impl GetHash+Into<&'a str>)->&HashStr{
		self.alloc_str_with_hash(index.get_hash(),index.into())
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
		unsafe{HashStr::ref_from_bytes_unchecked(new_hash_str_bytes)}
	}
}

/// Cache of existing entries in a HashStrHost.
/// Useful to deduplicate a finite set of unique strings,
/// minimizing the allocation of new strings.
#[derive(Debug)]
pub struct HashStrCache<'host>{
	entries:HashTable<&'host HashStr>,
}

fn get_precomputed_hash(&hash_str:&&HashStr)->u64{
	hash_str.precomputed_hash()
}

impl<'host> HashStrCache<'host>{
	#[inline]
	pub fn new()->HashStrCache<'host>{
		HashStrCache{
			entries:HashTable::new(),
		}
	}
	#[inline]
	pub fn with_capacity(capacity:usize)->HashStrCache<'host>{
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
	pub fn get<'a>(&self,index:impl GetHash+Into<&'a str>)->Option<&'host HashStr>{
		self.presence(index).get()
	}
	/// Finds an existing HashStr if it is present.  Can be chained to
	/// spill missing items into another cache, reusing the hash.
	/// The lifetimes of the chained caches must be ordered with equivalent or narrowing
	/// scope because the returned type has the lifetime of the last cache in the chain.
	///
	/// ```rust
	/// use hash_str::{HashStrHost,HashStrCache};
	///
	/// let host=HashStrHost::new();
	/// let cache1=HashStrCache::new();
	/// let cache2=HashStrCache::new();
	/// let mut cache3=HashStrCache::new();
	///
	/// let hs=cache1.presence("str").or_present_in(&cache2).or_intern_with(&host,&mut cache3);
	/// ```
	#[inline]
	pub fn presence<'a>(&self,index:impl GetHash+Into<&'a str>)->Presence<&'host HashStr,HashedStr<'a>>{
		self.presence_str_with_hash(index.get_hash(),index.into())
	}
	#[inline]
	pub(crate) fn presence_str_with_hash<'a>(&self,hash:u64,str:&'a str)->Presence<&'host HashStr,HashedStr<'a>>{
		match self.entries.find(hash,|&s|s.as_str()==str){
			Some(entry)=>Presence::Present(entry),
			None=>Presence::Absent(HashedStr{hash,str})
		}
	}
	/// Cache the provided HashStr, utilizing the precalculated hash.
	/// This will reuse an existing HashStr if one exists.
	/// The lifetime of the provided HashStr must outlive the HashStrCache.
	/// Allocates no new HashStrs.
	#[inline]
	pub fn cache(&mut self,hash_str:&'host HashStr)->&'host HashStr{
		let (hash,str)=(hash_str.precomputed_hash(),hash_str.as_str());
		self.intern_str_with_hash(||hash_str,hash,str)
	}
	/// Intern the provided string, utilizing the precalculated hash if possible.
	/// This will return an existing HashStr if one exists, or allocate
	/// a new one on the provided HashStrHost.
	#[inline]
	pub fn intern_with(&mut self,host:&'host HashStrHost,index:impl GetHash+AsRef<str>)->&'host HashStr{
		let (hash,str)=(index.get_hash(),index.as_ref());
		self.intern_str_with_hash(||host.alloc_str_with_hash(hash,str),hash,str)
	}
	#[inline]
	pub(crate) fn intern_str_with_hash(&mut self,with:impl FnOnce()->&'host HashStr,hash:u64,str:&str)->&'host HashStr{
		self.entries.entry(
			hash,
			|&s|s.as_str()==str,
			get_precomputed_hash,
		).or_insert_with(with).get()
	}
	#[inline]
	pub fn iter<'a>(&'a self)->impl Iterator<Item=&'host HashStr>+'a{
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

impl<'host,'a> IntoIterator for &'a HashStrCache<'host>{
	type Item=&'host HashStr;
	type IntoIter=core::iter::Copied<hashbrown::hash_table::Iter<'a,&'host HashStr>>;
	#[inline]
	fn into_iter(self)->Self::IntoIter{
		self.entries.iter().copied()
	}
}

// Idea:
// string cache chaining
// cache1.presence("str").or_present_in(cache2).or_intern_with(host,cache3)
// the point is that the hash is passed along with the entry

/// Represents the presence of a HashStr in a HashStrCache.  Call `.get()`
/// to grab the present value.  Holds the computed hash of the index str
/// to be reused when checking additional caches.
#[derive(Debug)]
pub enum Presence<P,A>{
	Present(P),
	Absent(A),
}

impl<P,A> Presence<P,A>{
	#[inline]
	pub fn get(self)->Option<P>{
		match self{
			Presence::Present(entry)=>Some(entry),
			Presence::Absent(_)=>None,
		}
	}
}
impl<'a,'host> Presence<&'host HashStr,HashedStr<'a>>{
	/// If the HashStr was not present, check if it is present in the specified cache.
	/// Note that this requires the lifetime of items from the previous caches
	/// to cover the lifetime of the specified cache to make the return types match.
	#[inline]
	pub fn or_present_in<'new>(self,cache:&'a HashStrCache<'new>)->Presence<&'new HashStr,HashedStr<'a>> where 'host:'new{
		match self{
			Presence::Present(entry)=>Presence::Present(entry),
			Presence::Absent(HashedStr{hash,str})=>cache.presence_str_with_hash(hash,str),
		}
	}
	/// If the HashStr was not present, intern the string using the specified host storage
	/// into the specified cache.
	/// Note that this requires the lifetime of items from the previous caches
	/// to cover the lifetime of the specified cache to make the return types match.
	#[inline]
	pub fn or_intern_with<'new>(self,host:&'new HashStrHost,cache:&mut HashStrCache<'new>)->&'new HashStr where 'host:'new{
		match self{
			Presence::Present(entry)=>entry,
			Presence::Absent(HashedStr{hash,str})=>cache.intern_str_with_hash(||host.alloc_str_with_hash(hash,str),hash,str),
		}
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
	// instead of the lifetime of the underlying data 'host
	assert_eq!(a,b);
	assert!(core::ptr::addr_eq(a,b));

	// it also works with a HashStr as the index
	let a2:&HashStr=words.cache(a);
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

// test the readme
#[test]
fn readme(){
	use crate::hstr;
	use crate::hash::HashStrMap;
	use crate::ornaments::UnhashedStr;
	// string with hash calculated at compile time
	let hstr_static:&HashStr=hstr!("bruh");
	// string with hash calculated at run time
	// anonymous means it does not belong to any HashStrCache
	let hstr_runtime:&HashStr=&HashStr::anonymous("bruh".to_owned());

	// string internment cache
	let lifetime_host=HashStrHost::new();
	let mut cache=HashStrCache::new();

	// Intern string into deduplication cache
	// Does not allocate unless "bruh" is a new string
	let hstr_interned:&HashStr=cache.intern_with(&lifetime_host,"bruh");

	// Intern HashStr into deduplication cache, utilizing existing hash
	// The HashStr lifetime does not matter because a new one is allocated if needed.
	let hstr_interned1:&HashStr=cache.intern_with(&lifetime_host,hstr_static);

	// Cache a HashStr stored somewhere else.
	// Provided HashStr must outlive the cache, enforced at compile time.
	// Does not allocate a new HashStr.
	let hstr_interned2:&HashStr=cache.cache(hstr_runtime);
	let hstr_interned3:&HashStr=cache.cache(hstr_interned);

	// all pointers point to the first hstr that was interned
	assert!(core::ptr::addr_eq(hstr_interned,hstr_interned1));
	assert!(core::ptr::addr_eq(hstr_interned,hstr_interned2));
	assert!(core::ptr::addr_eq(hstr_interned,hstr_interned3));

	let mut map=HashStrMap::default();
	map.insert(hstr_static,1);

	assert_eq!(map.get(hstr_static),Some(&1));
	assert_eq!(map.get(hstr_runtime),Some(&1));
	assert_eq!(map.get(hstr_interned),Some(&1));
	// The trait bound `&HashStr : Borrow<UnhashedStr>` allows UnhashedStr
	// to index HashMap without needing to allocate a temporary HashStr.
	// However, it does not contain a precomputed hash, so it is hashed
	// every time it is used.
	assert_eq!(map.get(UnhashedStr::from_ref("bruh")),Some(&1));

	// free cache memory of interned strings
	// does not affect static or anonymous HashStrs
	drop(cache);
	drop(lifetime_host);

	// hstr_runtime is dropped after cache
}
