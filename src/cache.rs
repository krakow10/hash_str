use crate::hash_str::make_hash;
use crate::hash_str::HashStr;
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
	pub fn get_str(&self,string:&str)->Option<&'str HashStr>{
		let hash=make_hash(string);
		self.get_with_hash(hash,string)
	}
	/// Fetch an existing HashStr using the precalculated hash.
	#[inline]
	pub fn get_hash_str(&self,hash_str:&HashStr)->Option<&'str HashStr>{
		self.get_with_hash(hash_str.precomputed_hash(),hash_str.as_str())
	}
	#[inline]
	fn get_with_hash(&self,hash:u64,string:&str)->Option<&'str HashStr>{
		self.entries.find(hash,|&s|s.as_str()==string).copied()
	}
	/// Calculate the hash of a &str and intern it into the HashStrHost.
	/// Returns the newly allocated HashStr or an existing one if there was one.
	#[inline]
	pub fn intern_str(&mut self,host:&'str HashStrHost,str:&str)->&'str HashStr{
		// TODO: avoid allocation
		let mut string=String::with_capacity(str.len()+crate::hash_str::SIZE_U64);
		string.push_str(str);
		let hash_str=&*HashStr::anonymous(string);
		self.intern_hash_str(host,hash_str)
	}
	/// Intern the provided HashStr into the HashStrHost using the precalculated hash.
	/// Returns the newly allocated HashStr or an existing one if there was one.
	#[inline]
	pub fn intern_hash_str(&mut self,host:&'str HashStrHost,hash_str:&HashStr)->&'str HashStr{
		// check exists
		if let Some(hash_str)=self.get_hash_str(hash_str){
			return hash_str;
		}

		// alloc new hash_str
		let new_hash_str_bytes=host.0.alloc_slice_copy(hash_str.as_hash_str_bytes());
		// SAFETY: the bytes returned from the alloc
		// are copied from the bytes fed into the alloc
		let new_hash_str=unsafe{HashStr::ref_from_bytes(new_hash_str_bytes)};

		// insert into entries
		self.entries.insert_unique(
			hash_str.precomputed_hash(),
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
