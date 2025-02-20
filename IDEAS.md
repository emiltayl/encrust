A list of things that could be done in no particular order:

* More efficient bytevecs? Possibly implemented with a custom type (EncrustedU8Vec or something)
* Implementations for additional data types, should be behind feature flags if they pull in additional dependencies
* Support for more advanced obfuscation, such as lenght, capacity, and pointers in vecs etc, will probably require at least a partial rewrite
* Support for arbitrary serializable data?
* Optional integrity checks?