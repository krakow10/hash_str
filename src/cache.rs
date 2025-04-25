use crate::hash_str::{make_hash,anonymous};
use crate::hash_str::HashStr;
use hashbrown::HashTable;

pub struct StringHost(bumpalo::Bump);
impl StringHost{
	pub fn new()->Self{
		Self(bumpalo::Bump::new())
	}
}

pub struct StringCache<'str>{
	host:&'str StringHost,
	entries:HashTable<&'str HashStr>,
}

impl<'str> StringCache<'str>{
	#[inline]
	pub fn new(host:&'str StringHost)->Self{
		StringCache{
			host,
			entries:HashTable::new(),
		}
	}
	#[inline]
	pub fn get_str(&self,string:&str)->Option<&'str HashStr>{
		let hash=make_hash(string);
		self.get_with_hash(hash,string)
	}
	#[inline]
	pub fn get_hash_str(&self,hash_str:&HashStr)->Option<&'str HashStr>{
		self.get_with_hash(hash_str.precomputed_hash(),hash_str.as_str())
	}
	#[inline]
	fn get_with_hash(&self,hash:u64,string:&str)->Option<&'str HashStr>{
		self.entries.find(hash,|&s|s.as_str()==string).copied()
	}
	#[inline]
	pub fn intern_str(&mut self,string:&str)->&'str HashStr{
		// TODO: avoid allocation
		let hash_str=&*anonymous(string);
		self.intern_hash_str(hash_str)
	}
	#[inline]
	pub fn intern_hash_str(&mut self,hash_str:&HashStr)->&'str HashStr{
		// check exists
		if let Some(hash_str)=self.get_hash_str(hash_str){
			return hash_str;
		}

		// alloc new hash_str
		let new_hash_str_bytes=self.host.0.alloc_slice_copy(hash_str.as_hash_str_bytes());
		let new_hash_str=unsafe{core::mem::transmute(new_hash_str_bytes)};

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
	let lifetime_host=StringHost::new();
	let mut words=StringCache::new(&lifetime_host);

	// borrow Words mutably
	let a:&HashStr=words.intern_str("bruh");
	// drop mutable borrow and borrow immutably
	let b:&HashStr=words.get_str("bruh").unwrap();
	// compare both references; this is impossible when
	// the lifetimes of a and b are derived from
	// the borrows in .get and .intern
	// e.g.
	// fn    get<'a>(&'a     self,s:&str)->Option<&'a HashStr>{
	// fn intern<'a>(&'a mut self,s:&str)->       &'a HashStr {
	// instead of the lifetime of the underlying data 'str
	println!("{}",a==b);

	// with alloc owned by StringHost this is no longer UB
	drop(words);
	println!("{}",a==b);

	// dropping LifetimeHost gives the desired compile error
	// drop(lifetime_host);
	println!("{}",a==b);
}
