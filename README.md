hash_str
========

## Strings with Precomputed Hash

A simple library to precompute string hashes.

Features:
- Create HashStr with procomputed hash
- Create HashStrMap using precomputed hash

Feature Wishlist:
- Intern strings into an explicit cache (not working)
- Create HashStr at compile time with a macro, deduplicated
- Intern strings into a global cache like ustr

Non-Goals:
- Dynamic string type like std String
