
# fixed_vec

# READ THIS:

Unfortunately (and somewhat fortunately), this crate was found to be unsound in a fundamental way. From [this reddit post](https://www.reddit.com/r/rust/comments/i0k1y6/fixed_vec_v010_avoiding_extra_bounds_checks_using/fzrumdj?utm_source=share&utm_medium=web2x):

```rust
fn unsound_fixedvec_example() {
    let mut v = vec![0];
    let mut idx_opt = None;
    
    loop {
        let v = std::mem::take(&mut v);
        let v = name!(v);
        let v = FixedVec::fix(v);
        if let Some(bad_idx) = idx_opt {
            println!("Bad: {}", v.get(bad_idx));
        } else {
            idx_opt = Some(v.check_index(0).unwrap());
        }
    }
}

fn unsound_fixedvec_example2(opt: Option<&dyn std::any::Any>) {
    let v = if opt.is_some() { Vec::new() } else { vec![666] };
    let v = name!(v);
    let v = FixedVec::fix(v);

    if let Some(idx_any) = opt {
        if let Some(bad_idx) = idx_any.downcast_ref() {
            println!("Bad: {}", v.get(*bad_idx));
        }
    } else {
        let idx = v.check_index(0).unwrap();
        unsound_fixedvec_example2(Some(&idx));
    }
}
```

both segfault with only safe rust :(. It was still a fun idea though! The problem is that the type of ``Name`` is only unique between macro invocations, and not between values. Dang.

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
