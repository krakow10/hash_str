#[repr(C)]
#[derive(Debug,PartialEq,Eq)]
pub(crate) struct HashDST<T:?Sized>{
	hash:u64,
	dst:T,
}

/// HashStr is a dynamically sized type so it is used similarly to &str.
/// A hash is stored at the beginning followed by a str.  The length is
/// known by the fat pointer when in the form &HashStr.
#[repr(transparent)]
#[derive(Debug,PartialEq,Eq)]
pub struct HashStr(HashDST<str>);

impl HashStr{
	#[inline]
	pub const fn precomputed_hash(&self)->u64{
		self.0.hash
	}
	#[inline]
	pub const fn as_str(&self)->&str{
		// This code does not resize the fat pointer,
		// so it must be hacked to the correct size ahead of time.
		&self.0.dst
	}
	/// Struct bytes including hash prefix and trailing str
	#[inline]
	pub const fn as_hash_str_bytes<'a>(&'a self)->&'a [u8]{
		// SAFETY: HashStr is always valid as bytes
		unsafe{core::slice::from_raw_parts(
			self as *const Self as *const u8,
			SIZE_U64+self.as_str().len()
		)}
	}
	/// Create a `&HashStr` from bytes.
	///
	/// SAFETY:
	/// - `bytes.len()` must be at least 8
	/// - `&bytes[8..]` must be valid UTF-8
	#[inline]
	pub const unsafe fn ref_from_bytes<'a>(bytes:&'a [u8])->&'a Self{
		// adapted from https://github.com/jonhoo/codecrafters-bittorrent-rust/blob/9dc424d4699febed87fefe8eef94509ab5392b56/src/peer.rs#L350-L359
		let dst_ptr=bytes as *const [u8] as *const u8;
		// fat pointer hack: set size to the dst portion without the hash
		let dst_len_hacked=bytes.len() - SIZE_U64;
		let dst_bytes_hacked=unsafe{core::slice::from_raw_parts(
			dst_ptr,
			dst_len_hacked
		)};
		let h = dst_bytes_hacked as *const [u8] as *const Self;
		// SAFETY: above pointer is non-null
		unsafe{&*h}
	}
	/// An anonymous HashStr that is not owned by a StringCache
	#[inline]
	pub fn anonymous(value: &str) -> Box<HashStr> {
		let hash=make_hash(value);
		let mut bytes=Vec::with_capacity(value.len()+SIZE_U64);
		bytes.extend_from_slice(&hash.to_ne_bytes());
		bytes.extend_from_slice(value.as_bytes());
		let boxed=bytes.into_boxed_slice();
		// SAFETY: leak the box to avoid calling its destructor
		let href=unsafe{Self::ref_from_bytes(Box::leak(boxed))};
		// SAFETY: we know that this is a unique reference because we just created it
		unsafe{Box::from_raw(href as *const Self as *mut Self)}
	}
}

// Just feed the precomputed hash into the Hasher. Note that this will of course
// be terrible unless the Hasher in question is expecting a precomputed hash.
impl std::hash::Hash for HashStr {
	#[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.precomputed_hash());
    }
}

pub(crate) fn make_hash(value:&str)->u64{
	use std::hash::Hasher;
	let mut hasher=ahash::AHasher::default();
	hasher.write(value.as_bytes());
	hasher.finish()
}

const SIZE_U64:usize=core::mem::size_of::<u64>();
#[macro_export]
macro_rules! hstr{
	($str:literal)=>{
		{
			const SIZE:usize=SIZE_U64+$str.len();
			const BYTES:[u8;SIZE]={
				let mut bytes=[0;SIZE];
				let hash=ahash_macro::hash_literal!($str);
				let hash_bytes=hash.to_ne_bytes();
				let mut i=0;
				while i<SIZE_U64{
					bytes[i]=hash_bytes[i];
					i+=1;
				}
				let str_bytes=$str.as_bytes();
				while i<SIZE{
					bytes[i]=str_bytes[i-SIZE_U64];
					i+=1;
				}
				bytes
			};
			unsafe{HashStr::ref_from_bytes(&BYTES)}
		}
	};
}


#[test]
fn ahash_macro(){
	let hash_macro=ahash_macro::hash_literal!("hey");
	let hash_runtime=make_hash("hey");
	assert_eq!(hash_macro,hash_runtime);
}
#[test]
fn dedup(){
	let h1=hstr!("hey");
	let h2=hstr!("hey");
	assert!(core::ptr::addr_eq(h1,h2));
}
#[test]
fn macro_vs_constructor(){
	let hash=make_hash("hey");
	let h1=&*HashStr::anonymous("hey");
	let h2=hstr!("hey");
	println!("h1={}",h1.as_str());
	println!("h2={}",h2.as_str());
	assert_eq!(hash,h1.precomputed_hash(),"make_hash does not equal runtime hash");
	assert_eq!(hash,h2.precomputed_hash(),"make_hash does not equal const hash");
	assert_eq!(h1,h2);
}
