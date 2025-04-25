/// HashStr is a dynamically sized type so it is used similarly to &str.
/// A hash is stored at the beginning followed by a str.  The length is
/// known by the fat pointer when in the form &HashStr.
#[derive(Debug,PartialEq,Eq)]
pub struct HashStr{
	hash:u64,
	str:str,
}

impl HashStr{
	#[inline]
	pub const fn precomputed_hash(&self)->u64{
		self.hash
	}
	#[inline]
	pub const fn as_str(&self)->&str{
		&self.str
	}
	/// Struct bytes including hash prefix and trailing str
	#[inline]
	pub const fn as_hash_str_bytes<'a>(&'a self)->&'a [u8]{
		// SAFETY: HashStr is always valid as bytes
		unsafe{core::mem::transmute(self)}
	}
	/// Create a `&HashStr` from bytes.
	///
	/// SAFETY: `&bytes[8..]` must be valid UTF-8
	#[inline]
	pub const unsafe fn ref_from_bytes<'a>(bytes:&'a [u8])->&'a Self{
		unsafe{core::mem::transmute(bytes)}
	}
	/// An anonymous HashStr that is not owned by a StringCache
	#[inline]
	pub fn anonymous(value: &str) -> Box<HashStr> {
		let hash=make_hash(value);
		let mut bytes=Vec::with_capacity(value.len()+core::mem::size_of::<u64>());
		bytes.extend_from_slice(&hash.to_ne_bytes());
		bytes.extend_from_slice(value.as_bytes());
		let boxed=bytes.into_boxed_slice();
		// SAFETY: hold my beer
		unsafe{core::mem::transmute(boxed)}
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
			unsafe{HashStr::ref_from_bytes(BYTES.as_slice())}
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
