use crate::ustr::Ustr;
use crate::hash::UstrSet;

struct A{
	alloc:LeakyBumpAlloc,
	old_alloc:Vec<LeakyBumpAlloc>,
}

struct StringCache<'str,A>{
	alloc:A,
	entries:UstrSet<'str>,
}

impl<'str,A> StringCache<'str,A>{
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
