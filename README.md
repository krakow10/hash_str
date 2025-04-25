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
