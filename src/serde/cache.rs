use serde::de::{Error,Visitor};
use crate::cache::{HashStrCache,HashStrHost};
use crate::ornaments::GetHash;
use crate::hash_str::HashStr;
use super::HashStrVisitorZeroCopy;

/// Read hash value and str and intern into specified cache.
pub struct HashStrVisitorHostedFromHashStr<'host>{
	host:&'host HashStrHost,
	cache:&'host mut HashStrCache<'host>,
}

impl<'host> Visitor<'_> for HashStrVisitorHostedFromHashStr<'host>{
	type Value=&'host HashStr;
	fn expecting(&self,formatter:&mut std::fmt::Formatter)->std::fmt::Result{
		write!(formatter,"Hash Str")
	}
	fn visit_borrowed_bytes<E:Error>(self,v:&[u8])->Result<Self::Value,E>{
		let h=HashStrVisitorZeroCopy.visit_bytes(v)?;
		let (hash,str)=(h.get_hash(),h.as_str());
		Ok(self.cache.intern_str_with_hash(||self.host.alloc_str_with_hash(hash,str),hash,str))
	}
}

/// Read str and intern into specified cache, calculates hash on the fly.
pub struct HashStrVisitorHostedFromStr<'host>{
	host:&'host HashStrHost,
	cache:&'host mut HashStrCache<'host>,
}

impl<'host> Visitor<'_> for HashStrVisitorHostedFromStr<'host>{
	type Value=&'host HashStr;
	fn expecting(&self,formatter:&mut std::fmt::Formatter)->std::fmt::Result{
		write!(formatter,"Hash Str")
	}
	fn visit_str<E:Error>(self,v:&str)->Result<Self::Value,E>{
		Ok(self.cache.intern_with(&self.host,v))
	}
}
