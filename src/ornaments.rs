// === Ornaments ===
// Bells and whistles.
// Convenient impls which may obscure
// the readability of the core implementations.

use crate::hash::make_hash;
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
	#[inline]
	fn fmt(&self,f:&mut core::fmt::Formatter<'_>)->core::fmt::Result{
		f.write_str(self.as_str())
	}
}

impl PartialEq for HashStr{
	#[inline]
	fn eq(&self,other:&Self)->bool{
		self.precomputed_hash()==other.precomputed_hash()&&self.as_str().eq(other.as_str())
	}
}
impl Eq for HashStr{}
// TODO: more PartialOrd impls e.g. PartialOrd<str>
impl PartialOrd for HashStr{
	#[inline]
	fn partial_cmp(&self,other:&Self)->Option<core::cmp::Ordering>{
		self.as_str().partial_cmp(other.as_str())
	}
}
impl Ord for HashStr{
	#[inline]
	fn cmp(&self,other:&Self)->std::cmp::Ordering{
		self.as_str().cmp(other.as_str())
	}
}

/// Helper type for indexing a HashMap without allocation
/// Unhashed str is hashed on the fly instead of using a precalculated hash.
/// Useful for indexing a HashMap without needing to allocate a Box<HashStr>
#[repr(transparent)]
#[derive(Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct UnhashedStr(str);
impl UnhashedStr{
	#[inline]
	pub const fn from_ref<'a>(str:&'a str)->&'a Self{
		// SAFETY: UnhashedStr is #[repr(transparent)]
		let ptr=str as *const str as *const Self;
		unsafe{&*ptr}
	}
	#[inline]
	pub const fn as_str<'a>(&'a self)->&'a str{
		// SAFETY: UnhashedStr is #[repr(transparent)]
		let ptr=self as *const Self as *const str;
		unsafe{&*ptr}
	}
}
impl core::hash::Hash for UnhashedStr{
	#[inline]
	fn hash<H:std::hash::Hasher>(&self,state:&mut H){
		let hash=make_hash(self.into());
		state.write_u64(hash);
	}
}
impl<'a> From<&'a str> for &'a UnhashedStr{
	#[inline]
	fn from(value:&'a str)->Self{
		UnhashedStr::from_ref(value)
	}
}
impl<'a> From<&'a UnhashedStr> for &'a str{
	#[inline]
	fn from(value:&'a UnhashedStr)->Self{
		value.as_str()
	}
}

impl<'a> core::borrow::Borrow<UnhashedStr> for &HashStr{
	#[inline]
	fn borrow(&self)->&UnhashedStr{
		self.as_str().into()
	}
}

pub trait GetHash{
	fn get_hash(self)->u64;
}
impl GetHash for &str{
	#[inline]
	fn get_hash(self)->u64{
		make_hash(self)
	}
}
impl GetHash for &HashStr{
	#[inline]
	fn get_hash(self)->u64{
		self.precomputed_hash()
	}
}

macro_rules! partial_eq_lhs_as_str{
	($lhs:ty,$rhs:ty)=>{
		impl PartialEq<$rhs> for $lhs {
			#[inline]
			fn eq(&self, other: &$rhs) -> bool {
				self.as_str() == other
			}
		}
		impl PartialEq<$lhs> for $rhs {
			#[inline]
			fn eq(&self, other: &$lhs) -> bool {
				self == other.as_str()
			}
		}
	};
}
macro_rules! partial_eq_lhs_as_str_rhs_deref{
	($lhs:ty,$rhs:ty)=>{
		impl PartialEq<$rhs> for $lhs {
			#[inline]
			fn eq(&self, &other: &$rhs) -> bool {
				self.as_str() == other
			}
		}
		impl PartialEq<$lhs> for $rhs {
			#[inline]
			fn eq(&self, other: &$lhs) -> bool {
				*self == other.as_str()
			}
		}
	};
}
macro_rules! partial_eq_lhs_as_str_rhs_as_ref{
	($lhs:ty,$rhs:ty)=>{
		impl PartialEq<$rhs> for $lhs {
			#[inline]
			fn eq(&self, other: &$rhs) -> bool {
				self.as_str() == other.as_ref()
			}
		}
		impl PartialEq<$lhs> for $rhs {
			#[inline]
			fn eq(&self, other: &$lhs) -> bool {
				self.as_ref() == other.as_str()
			}
		}
	};
}
partial_eq_lhs_as_str!(HashStr,str);
partial_eq_lhs_as_str!(&HashStr,str);
partial_eq_lhs_as_str!(HashStr,String);
partial_eq_lhs_as_str!(&HashStr,String);
partial_eq_lhs_as_str_rhs_deref!(HashStr,&str);
partial_eq_lhs_as_str_rhs_deref!(HashStr,&String);
partial_eq_lhs_as_str_rhs_as_ref!(HashStr,Box<str>);
partial_eq_lhs_as_str_rhs_as_ref!(&HashStr,Box<str>);
partial_eq_lhs_as_str_rhs_as_ref!(HashStr,&Box<str>);
use std::borrow::Cow;
partial_eq_lhs_as_str_rhs_as_ref!(HashStr,Cow<'_,str>);
partial_eq_lhs_as_str_rhs_as_ref!(&HashStr,Cow<'_,str>);
partial_eq_lhs_as_str_rhs_as_ref!(HashStr,&Cow<'_,str>);

// TODO:
// Path and OsStr requre CStr
// use std::path::Path;
// use std::ffi::OsStr;
