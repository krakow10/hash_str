use parking_lot::Mutex;
use crate::ornaments::GetHash;
use crate::hash_str::HashStr;
use crate::cache::{HashStrHost,HashStrCache};

// Number of bins (shards) for map
const BIN_SHIFT: usize = 6;
const NUM_BINS: usize = 1 << BIN_SHIFT;
// Shift for top bits to determine bin a hash falls into
const TOP_SHIFT: usize =
    8 * std::mem::size_of::<usize>() - BIN_SHIFT;

/// The type used for the global string cache.
///
/// This is exposed to allow e.g. serialization of the data returned by the
/// [`cache()`] function.
#[repr(transparent)]
pub struct Bins<'str>([Mutex<HostCache<'str>>; NUM_BINS]);

struct HostCache<'str>{
	host:HashStrHost,
	cache:HashStrCache<'str>,
}

lazy_static::lazy_static!{
	static ref STRING_CACHE:Bins<'static> =
	Bins(core::array::from_fn(|_|Mutex::new(
		HostCache{
			host:HashStrHost::new(),
			cache:HashStrCache::new()
		}
	)));
}

#[inline]
pub fn get_cache()->&'static Bins<'static>{
	&STRING_CACHE
}

/// Don't clear the cache, all global interned strings
/// will become undefined behaviour to access. Used
/// for benchmarking only.
#[doc(hidden)]
pub unsafe fn _clear_cache(){
	for bin in &STRING_CACHE.0{
		let bin=&mut*bin.lock();
		bin.cache.clear();
		bin.host.clear();
	}
}

// Use the top bits of the hash to choose a bin
#[inline]
fn whichbin(hash: u64) -> usize {
    ((hash >> TOP_SHIFT as u64) % NUM_BINS as u64) as usize
}
impl<'str> Bins<'str>{
	/// Get a string from the global cache.
	#[inline]
	pub fn get(&self,index:impl GetHash+AsRef<str>+Copy)->Option<&'str HashStr>{
		let hash=index.get_hash();
	    self.0[whichbin(hash)].lock().cache.get_str_with_hash(hash,index.as_ref())
	}
	/// Intern a HashStr into the global cache.  The lifetime must be 'static.
	#[inline]
	pub fn intern(&self,hash_str:&'str HashStr)->&'str HashStr{
		let hash=hash_str.get_hash();
		self.0[whichbin(hash)].lock().cache.intern(hash_str)
	}
	/// Intern a string into the global cache.
	#[inline]
	pub fn intern_str(&self,str:&str)->&'str HashStr{
		let hash=str.get_hash();
		let HostCache{cache,host}=&mut*self.0[whichbin(hash)].lock();
		cache.intern_str_with_hash(||{
			// SAFETY: this pointer is created to be valid for the
			// duration of the .alloc borrow of host, but we know
			// that it is actually valid for the lifetime of
			// HostCache.host, which in this case is 'static
			let ptr=host.alloc_str_with_hash(hash,str) as *const HashStr;
			unsafe{&*ptr}
		},hash,str)
	}
}
