use crate::ustr::anonymous;
use crate::ustr::Ustr;
use crate::hash::UstrSet;

pub struct StringHost(bumpalo::Bump);
impl StringHost{
	pub fn new()->Self{
		Self(bumpalo::Bump::new())
	}
}

pub struct StringCache<'str>{
	host:&'str StringHost,
	entries:UstrSet<'str>,
}

impl<'str> StringCache<'str>{
	#[inline]
	pub fn new(host:&'str StringHost)->Self{
		StringCache{
			host,
			entries:UstrSet::default(),
		}
	}
	#[inline]
	pub fn get(&self,string:&str)->Option<&'str Ustr>{
		// TODO: avoid allocation
		let ustr=&*anonymous(string);
		self.get_ustr(ustr)
	}
	#[inline]
	pub fn get_ustr(&self,ustr:&Ustr)->Option<&'str Ustr>{
		self.entries.get(ustr).copied()
	}
	#[inline]
	pub fn intern(&mut self,string:&str)->&'str Ustr{
		// TODO: avoid allocation
		let ustr=&*anonymous(string);
		self.intern_ustr(ustr)
	}
	#[inline]
	pub fn intern_ustr(&mut self,ustr:&Ustr)->&'str Ustr{
		if let Some(ustr)=self.get(ustr){
			return ustr;
		}
		// alloc new ustr
		let new_ustr_bytes=self.host.0.alloc_slice_copy(ustr.as_ustr_bytes());
		let new_ustr=unsafe{core::mem::transmute(new_ustr_bytes)};
		// insert into entries
		self.entries.insert(new_ustr);
		new_ustr
	}
}

#[test]
fn test_cache(){
	let lifetime_host=StringHost::new();
	let mut words=StringCache::new(&lifetime_host);

	// borrow Words mutably
	let a=words.intern("bruh");
	// drop mutable borrow and borrow immutably
	let b=words.get("bruh").unwrap();
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
