use crate::hash_str::{HashStr,SIZE_HASH};

/// Construct a &'static HashStr at compile time.  These are presumably deduplicated by the compiler.
#[macro_export]
macro_rules! hstr{
	($str:literal)=>{
		{
			const SIZE:usize=SIZE_HASH+$str.len();
			const BYTES:[u8;SIZE]={
				let mut bytes=[0;SIZE];
				let hash=ahash_macro::hash_literal!($str);
				let hash_bytes=hash.to_ne_bytes();
				let mut i=0;
				while i<SIZE_HASH{
					bytes[i]=hash_bytes[i];
					i+=1;
				}
				let str_bytes=$str.as_bytes();
				while i<SIZE{
					bytes[i]=str_bytes[i-SIZE_HASH];
					i+=1;
				}
				bytes
			};
			unsafe{HashStr::ref_from_bytes(&BYTES)}
		}
	};
}

#[cfg(test)]
mod test{
	use crate::hash::make_hash;
	use crate::hash_str::{HashStr,SIZE_HASH};
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
		let h1=&*HashStr::anonymous("hey".to_owned());
		let h2=hstr!("hey");
		assert_eq!(h1,h2);
		assert_eq!(h1.as_str(),"hey");
		assert_eq!(h2.as_str(),"hey");
		assert_eq!(h1.as_str().len(),3);
		assert_eq!(h2.as_str().len(),3);
		assert_eq!(h1.as_hash_str_bytes().len(),3+SIZE_HASH);
		assert_eq!(h2.as_hash_str_bytes().len(),3+SIZE_HASH);
		assert_eq!(hash,h1.precomputed_hash(),"make_hash does not equal runtime hash");
		assert_eq!(hash,h2.precomputed_hash(),"make_hash does not equal const hash");
	}
}
