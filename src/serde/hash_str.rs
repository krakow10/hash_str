use crate::hash_str::{HashStr,RefFromBytesError};
use serde::{Serialize,Serializer,Deserialize,Deserializer};
use serde::de::{Error,Unexpected,Visitor};

impl Serialize for HashStr{
	fn serialize<S:Serializer>(&self,serializer:S)->Result<S::Ok,S::Error>{
		serializer.serialize_bytes(self.as_hash_str_bytes())
	}
}

pub struct HashStrVisitorZeroCopy;

impl<'de> Visitor<'de> for HashStrVisitorZeroCopy{
	type Value=&'de HashStr;

	fn expecting(&self,formatter:&mut std::fmt::Formatter)->std::fmt::Result{
		write!(formatter,"Hash Str")
	}

	fn visit_borrowed_bytes<E:Error>(self,v:&'de [u8])->Result<Self::Value,E>{
		match HashStr::ref_from_bytes(v){
			Ok(h)=>Ok(h),
			Err(RefFromBytesError::TooShort)=>Err(E::invalid_length(v.len(),&"8 or longer")),
			Err(RefFromBytesError::UTF8(_))=>Err(E::invalid_value(Unexpected::Bytes(v),&"valid utf8 after position 8"))
		}
	}
}

impl<'a,'de:'a> Deserialize<'de> for &'a HashStr{
	fn deserialize<D:Deserializer<'de>>(deserializer:D)->Result<Self,D::Error>{
		deserializer.deserialize_bytes(HashStrVisitorZeroCopy)
	}
}
