use crate::ustr::anonymous;
use crate::ustr::Ustr;
use crate::hash::UstrSet;

pub struct StringCache<'str>{
	alloc:bumpalo::Bump,
	entries:UstrSet<'str>,
}

// SAFETY: caller must ensure that 'short lifetime is actualy valid for 'long
unsafe fn extend_lifetime<'short,'long,T:?Sized>(short:&'short T)->&'long T{
	unsafe{std::mem::transmute(short)}
}

impl<'str> StringCache<'str>{
	#[inline]
	pub fn new()->Self{
		StringCache{
			alloc:bumpalo::Bump::new(),
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
		println!("ustr={ustr}");
		// alloc new ustr
		let bytes=ustr.as_ustr_bytes();
		let round_trip=Ustr::ref_from_bytes(bytes);
		println!("what the ustr doing {}",round_trip);
		let new_ustr_bytes=self.alloc.alloc_slice_copy(bytes);
		let new_ustr=Ustr::ref_from_bytes(new_ustr_bytes);
		println!("new_ustr={new_ustr}");
		let ustr_longer=unsafe{extend_lifetime(new_ustr)};
		// insert into entries
		self.entries.insert(ustr_longer);
		println!("ustr_longer={ustr_longer}");
		ustr_longer
	}
}

#[test]
fn test_cache(){
	let mut words=StringCache::new();

	// borrow Words mutably
	let a=words.intern("bruh");
	println!("a={}",a.as_str());
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

	// with a correct implementation,
	// dropping words here should introduce a compile error
	drop(words);
	println!("{}",a==b);

	// dropping LifetimeHost gives the desired compile error
	// drop(lifetime_host);
	println!("{}",a==b);
}
