use crate::ustr::Ustr;
use crate::hash::UstrSet;

struct NeverDrop<A>{
	alloc:A,
	old_alloc:Vec<A>,
}

pub struct StringCache<'str>{
	alloc:NeverDrop<LeakyBumpAlloc>,
	entries:UstrSet<'str>,
}

impl<'str> StringCache<'str>{
	pub fn get(&self,ustr:&Ustr)->Option<&'str Ustr>{
		self.entries.get(ustr).map(|&ustr|ustr)
	}
	pub fn intern(&mut self,ustr:&Ustr)->&'str Ustr{
		if let Some(ustr)=self.get(ustr){
			return ustr;
		}
		// alloc new ustr
		// insert into entries
		self.entries.insert(new_ustr);
		new_ustr
	}
}
