
# fixed_vec

Bounds check indices only once, instead of over and over if the indices will be re-used:

```rust
use fixed_vec::{name, FixedVec};

let v = vec![0u32; 10];
let v = name!(v);
let mut v = FixedVec::fix(v);

// Perform the two index checks here:
let index_a = v.check_index(...).unwrap();
let index_b = v.check_index(...).unwrap();

for _ in 0..100 {
    // These do *not* perform bounds checks!
    // At compile time, v and index_a must match
    *v.get_mut(index_a) += 5;
    *v.get_mut(index_b) += 10;
}

let v = v.unfix();

// continue using v...
```

See the [concept post](https://github.com/Torrencem/fixed_vec/blob/master/post.md) for more information.
