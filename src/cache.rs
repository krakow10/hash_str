use crate::ustr::anonymous;
use crate::ustr::Ustr;
use crate::hash::UstrSet;

pub struct StringCache<'str>{
	alloc:bumpalo::Bump,
	entries:UstrSet<'str>,
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
		// alloc new ustr
		let new_ustr_bytes=self.alloc.alloc_slice_copy(ustr.as_ustr_bytes());
		let new_ustr=unsafe{core::mem::transmute(new_ustr_bytes)};
		// insert into entries
		self.entries.insert(new_ustr);
		new_ustr
	}
}
