use crate::ustr::Ustr;
use crate::hash::UstrSet;

pub struct StringCache<'str>{
	alloc:bumpalo::Bump,
	entries:UstrSet<'str>,
}

impl<'str> StringCache<'str>{
	pub fn new()->Self{
		StringCache{
			alloc:bumpalo::Bump::new(),
			entries:UstrSet::default(),
		}
	}
	pub fn get(&self,ustr:&Ustr)->Option<&'str Ustr>{
		self.entries.get(ustr).copied()
	}
	pub fn intern(&mut self,ustr:&Ustr)->&'str Ustr{
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
