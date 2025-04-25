// === Ornaments ===
// Bells and whistles.
// Convenient impls which may obscure
// the readability of the core implementations.

use crate::hash_str::HashStr;

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
