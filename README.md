hash_str
========

## Strings with Precomputed Hash

A simple library for strings with a precomputed hash.

Features:
- Create HashStr with precomputed hash
- Create HashStrMap utilizing HashStr's precomputed hash
- Intern strings into an explicit cache
- Create HashStr at compile time with a macro, deduplicated

Wishlist:
- Intern strings into a global cache like ustr

Non-Goals:
- Dynamic string type like std String

Example:
```rust
use hash_str::hstr;
use hash_str::{HashStr,UnhashedStr};
// requires cache feature
use hash_str::{HashStrHost,HashStrCache};

fn main(){
	let lifetime_host=HashStrHost::new();
	let mut cache=HashStrCache::new();

	let hstr_static=hstr!("bruh");
	let hstr_runtime=&*HashStr::anonymous("bruh".to_owned());

	let hstr_interned=cache.intern(&lifetime_host,"bruh");

	let mut map=hash_str::HashStrMap::default();
	map.insert(hstr_static,1);

	assert_eq!(map.get(hstr_static),Some(&1));
	assert_eq!(map.get(hstr_runtime),Some(&1));
	assert_eq!(map.get(hstr_interned),Some(&1));
	assert_eq!(map.get(UnhashedStr::from_ref("bruh")),Some(&1));
}
```
