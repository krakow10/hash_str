use crate::hash::make_hash;

/// HashStr is a dynamically sized type so it is used similarly to &str.
/// A hash is stored at the beginning followed by a str.  The length is
/// known by the fat pointer when in the form &HashStr.
#[repr(C)]
#[derive(Debug,PartialEq,Eq)]
pub struct HashStr{
	hash:u64,
	str:str,
}

pub(crate) const SIZE_HASH:usize=core::mem::size_of::<u64>();

impl HashStr{
	#[inline]
	pub const fn precomputed_hash(&self)->u64{
		self.hash
	}
	#[inline]
	pub const fn as_str(&self)->&str{
		// This code does not resize the fat pointer,
		// so it must be hacked to the correct size ahead of time.
		&self.str
	}
	/// Struct bytes including hash prefix and trailing str
	#[inline]
	pub const fn as_hash_str_bytes<'a>(&'a self)->&'a [u8]{
		// SAFETY: HashStr is always valid as bytes,
		// but the fat pointer must be widened to undo the hack
		unsafe{core::slice::from_raw_parts(
			self as *const Self as *const u8,
			SIZE_HASH+self.as_str().len()
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
		let dst_len_hacked=bytes.len() - SIZE_HASH;
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
	pub fn anonymous(value: String) -> Box<HashStr> {
		let hash=make_hash(&value);
		let mut bytes=value.into_bytes();
		// prefix bytes with hash
		bytes.reserve_exact(SIZE_HASH);
		insert_bytes(&mut bytes,&hash.to_ne_bytes());

		let boxed=bytes.into_boxed_slice();
		// SAFETY: leak the box to avoid calling its destructor
		let href=unsafe{Self::ref_from_bytes(Box::leak(boxed))};
		// SAFETY: we know that this is a unique reference because we just created it
		unsafe{Box::from_raw(href as *const Self as *mut Self)}
	}
}

// copied from std String
// why doesn't this function doesn't exist on std Vec?
fn insert_bytes(vec:&mut Vec<u8>, bytes: &[u8]) {
    let len = vec.len();
    let amt = bytes.len();
    vec.reserve_exact(amt);

    unsafe {
        core::ptr::copy(vec.as_ptr(), vec.as_mut_ptr().add(amt), len);
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), vec.as_mut_ptr(), amt);
        vec.set_len(len + amt);
    }
}
