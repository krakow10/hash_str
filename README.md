hash_str
========

[![Latest version](https://img.shields.io/crates/v/hash_str.svg)](https://crates.io/crates/hash_str)
![License](https://img.shields.io/crates/l/hash_str.svg)

## Strings with Precomputed Hash

A simple library for strings with a precomputed hash.

Features:
- Create HashStr with precomputed hash
- Create HashStrMap utilizing HashStr's precomputed hash
- Index HashStrMap using UnhashedStr or HashStr
- Intern strings into an explicit cache
- Create HashStr at compile time with a macro, deduplicated
- Intern strings into a global cache like ustr
  - ustr is faster if this is your main use case
  - Convenient for migrating to explicit caches piecemeal

Wishlist:
- Create compile-time deduplicated cache of all compile-time HashStrs

Non-Goals:
- Dynamic string type like std String

Example:
```rust
use hash_str::hstr;
use hash_str::{HashStr,UnhashedStr};
use hash_str::HashStrMap;
// requires cache feature
use hash_str::{HashStrHost,HashStrCache};

// string with hash calculated at compile time
let hstr_static:&HashStr=hstr!("bruh");
// string with hash calculated at run time
// anonymous means it does not belong to any HashStrCache
let hstr_runtime:&HashStr=&HashStr::anonymous("bruh".to_owned());

// string internment cache
let lifetime_host=HashStrHost::new();
let mut cache=HashStrCache::new();

// Intern string into deduplication cache
// Does not allocate unless "bruh" is a new string
let hstr_interned:&HashStr=cache.intern_str_with(&lifetime_host,"bruh");

// Intern HashStr into deduplication cache
// Provided HashStr must outlive the cache, enforced at compile time
// Does not allocate a new HashStr.
let hstr_interned1:&HashStr=cache.intern(hstr_static);
let hstr_interned2:&HashStr=cache.intern(hstr_runtime);
let hstr_interned3:&HashStr=cache.intern(hstr_interned);

// all pointers point to the first hstr that was interned
assert!(core::ptr::addr_eq(hstr_interned,hstr_interned1));
assert!(core::ptr::addr_eq(hstr_interned,hstr_interned2));
assert!(core::ptr::addr_eq(hstr_interned,hstr_interned3));

let mut map=HashStrMap::default();
map.insert(hstr_static,1);

assert_eq!(map.get(hstr_static),Some(&1));
assert_eq!(map.get(hstr_runtime),Some(&1));
assert_eq!(map.get(hstr_interned),Some(&1));
// The trait bound `Borrow<UnhashedStr> : &HashStr` allows UnhashedStr
// to index HashMap without needing to allocate a temporary HashStr.
// However, it does not contain a precomputed hash, so it is hashed
// every time it is used.
assert_eq!(map.get(UnhashedStr::from_ref("bruh")),Some(&1));

// free cache memory of interned strings
// does not affect static or anonymous HashStrs
drop(cache);
drop(lifetime_host);

// hstr_runtime is dropped after cache
```
