use serde::de::{Error,Visitor};
use crate::global::get_cache;
use crate::ornaments::GetHash;
use crate::hash_str::HashStr;
use super::HashStrVisitorZeroCopy;

/// Read hash value and str and intern into global cache.
pub struct HashStrVisitorGlobalFromHashStr;

impl Visitor<'_> for HashStrVisitorGlobalFromHashStr{
	type Value=&'static HashStr;
	fn expecting(&self,formatter:&mut std::fmt::Formatter)->std::fmt::Result{
		write!(formatter,"Hash Str")
	}
	fn visit_borrowed_bytes<E:Error>(self,v:&[u8])->Result<Self::Value,E>{
		let h=HashStrVisitorZeroCopy.visit_bytes(v)?;
		Ok(get_cache().intern_str_with_hash(h.get_hash(),h.as_str()))
	}
}

/// Read str and intern into global cache, calculates hash on the fly.
pub struct HashStrVisitorGlobalFromStr;

impl Visitor<'_> for HashStrVisitorGlobalFromStr{
	type Value=&'static HashStr;
	fn expecting(&self,formatter:&mut std::fmt::Formatter)->std::fmt::Result{
		write!(formatter,"Hash Str")
	}
	fn visit_str<E:Error>(self,v:&str)->Result<Self::Value,E>{
		Ok(get_cache().intern(v))
	}
}
