// === Ornaments ===
// Bells and whistles.
// Convenient impls which may obscure
// the readability of the core implementations.

use crate::hash_str::make_hash;
use crate::hash_str::HashStr;

impl AsRef<str> for HashStr{
	#[inline]
	fn as_ref(&self)->&str{
		self.as_str()
	}
}
impl core::ops::Deref for HashStr{
	type Target=str;
	#[inline]
	fn deref(&self)->&Self::Target{
		self.as_str()
	}
}

impl core::fmt::Display for HashStr{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}

pub trait GetHash{
	fn get_hash(self)->u64;
}
impl GetHash for &str{
	fn get_hash(self)->u64{
		make_hash(self)
	}
}
impl GetHash for &HashStr{
	fn get_hash(self)->u64{
		self.precomputed_hash()
	}
}

macro_rules! partial_eq_lhs_as_str{
	($lhs:ty,$rhs:ty)=>{
		impl PartialEq<$rhs> for $lhs {
			fn eq(&self, other: &$rhs) -> bool {
				self.as_str() == other
			}
		}
		impl PartialEq<$lhs> for $rhs {
			fn eq(&self, other: &$lhs) -> bool {
				self == other.as_str()
			}
		}
	};
}
macro_rules! partial_eq_lhs_as_str_rhs_deref{
	($lhs:ty,$rhs:ty)=>{
		impl PartialEq<$rhs> for $lhs {
			fn eq(&self, &other: &$rhs) -> bool {
				self.as_str() == other
			}
		}
		impl PartialEq<$lhs> for $rhs {
			fn eq(&self, other: &$lhs) -> bool {
				*self == other.as_str()
			}
		}
	};
}
macro_rules! partial_eq_lhs_as_str_rhs_as_ref{
	($lhs:ty,$rhs:ty)=>{
		impl PartialEq<$rhs> for $lhs {
			fn eq(&self, other: &$rhs) -> bool {
				self.as_str() == other.as_ref()
			}
		}
		impl PartialEq<$lhs> for $rhs {
			fn eq(&self, other: &$lhs) -> bool {
				self.as_ref() == other.as_str()
			}
		}
	};
}
partial_eq_lhs_as_str!(HashStr,str);
partial_eq_lhs_as_str!(HashStr,String);
partial_eq_lhs_as_str_rhs_deref!(HashStr,&str);
partial_eq_lhs_as_str_rhs_deref!(HashStr,&String);
partial_eq_lhs_as_str_rhs_as_ref!(HashStr,Box<str>);
partial_eq_lhs_as_str_rhs_as_ref!(HashStr,&Box<str>);
use std::borrow::Cow;
partial_eq_lhs_as_str_rhs_as_ref!(HashStr,Cow<'_,str>);
partial_eq_lhs_as_str_rhs_as_ref!(HashStr,&Cow<'_,str>);

// TODO:
// Path and OsStr requre CStr
// use std::path::Path;
// use std::ffi::OsStr;
