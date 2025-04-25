use crate::ustr::{make_hash,anonymous};
use crate::ustr::Ustr;
use hashbrown::HashTable;

pub struct StringHost(bumpalo::Bump);
impl StringHost{
	pub fn new()->Self{
		Self(bumpalo::Bump::new())
	}
}

pub struct StringCache<'str>{
	host:&'str StringHost,
	entries:HashTable<&'str Ustr>,
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
	pub fn get_str(&self,string:&str)->Option<&'str Ustr>{
		let hash=make_hash(string);
		self.get_with_hash(hash,string)
	}
	#[inline]
	pub fn get_ustr(&self,ustr:&Ustr)->Option<&'str Ustr>{
		self.get_with_hash(ustr.precomputed_hash(),ustr.as_str())
	}
	#[inline]
	fn get_with_hash(&self,hash:u64,string:&str)->Option<&'str Ustr>{
		self.entries.find(hash,|&s|s.as_str()==string).copied()
	}
	#[inline]
	pub fn intern_str(&mut self,string:&str)->&'str Ustr{
		// TODO: avoid allocation
		let ustr=&*anonymous(string);
		self.intern_ustr(ustr)
	}
	#[inline]
	pub fn intern_ustr(&mut self,ustr:&Ustr)->&'str Ustr{
		// check exists
		if let Some(ustr)=self.get_ustr(ustr){
			return ustr;
		}

		// alloc new ustr
		let new_ustr_bytes=self.host.0.alloc_slice_copy(ustr.as_ustr_bytes());
		let new_ustr=unsafe{core::mem::transmute(new_ustr_bytes)};

		// insert into entries
		self.entries.insert_unique(
			ustr.precomputed_hash(),
			new_ustr,
			|ustr|ustr.precomputed_hash()
		).get()
	}
}

#[test]
fn test_cache(){
	let lifetime_host=StringHost::new();
	let mut words=StringCache::new(&lifetime_host);

	// borrow Words mutably
	let a=words.intern_str("bruh");
	// drop mutable borrow and borrow immutably
	let b=words.get_str("bruh").unwrap();
	// compare both references; this is impossible when
	// the lifetimes of a and b are derived from
	// the borrows in .get and .intern
	// e.g.
	// fn    get<'a>(&'a     self,s:&str)->Option<&'a str>{
	// fn intern<'a>(&'a mut self,s:&str)->       &'a str {
	// instead of the lifetime of the underlying data 'str
	println!("{}",a==b);

	// with alloc owned by StringHost this is no longer UB
	drop(words);
	println!("{}",a==b);

	// dropping LifetimeHost gives the desired compile error
	// drop(lifetime_host);
	println!("{}",a==b);
}
