# Properly implementing "ghosts of departed proofs"

In this article, we'll be looking at a clever way to remove redundent bounds checks on array indexing using techniques discussed in the legendary paper [Ghosts of Departed Proofs](https://kataskeue.com/gdp.pdf).

Let's get one thing out of the way off the bat: if you don't want to have to deal with bounds checks, most of the time you want to use iterators instead of loops. [This page](https://www.cs.brandeis.edu/~cs146a/rust/doc-02-21-2015/book/iterators.html) provides a nice introduction to iterators and their adapters if you've never used them before. Suffice it to say, using ``map``'s and ``filter``'s will compile into code without bounds checks, and fits what you were probably trying to do anyway.

Besides this, there are sometimes situations where writing for loops is either the only way, or the only way in readable code. Consider the following example:

```rust
let mut v = vec![0; 10];

// ...

let index_a = todo!("Some unknown-at-compile-time index");
let index_b = todo!("These could be user supplied!");

for _ in 0..100 {
    v[index_a] += 5;
    v[index_b] += 10;
}

// ...
```

An example implementation might look like [this](https://godbolt.org/z/8786Gc). Now, there is bounds checking going on (see line 75 of the assembly). In an ideal world, the bounds checks would happen once before the loop, since the indices aren't changing, and the vec isn't shrinking in size. Is the optimized code only doing the bounds check once? Well, to find out, we'll have to put on our assembly hats.

Below is the assembly generated for my example:

```assembly
example::compute:
        push    r15
        push    r14
        push    rbx
        sub     rsp, 32
        mov     r15, rsi
        mov     r14, rdi
        mov     qword ptr [rsp + 8], 80
        mov     qword ptr [rsp + 16], 8
        mov     edi, 80
        mov     esi, 8
        call    qword ptr [rip + __rust_alloc_zeroed@GOTPCREL]
        test    rax, rax
        je      .LBB2_10
        mov     qword ptr [rsp + 8], rax
        mov     qword ptr [rsp + 16], 10
        mov     qword ptr [rsp + 24], 10
        cmp     r14, 9
        ja      .LBB2_5
        mov     ecx, 100
.LBB2_3:
        add     qword ptr [rax + 8*r14], 5
        cmp     r15, 10
        jae     .LBB2_4
        add     qword ptr [rax + 8*r15], 10
        add     qword ptr [rax + 8*r14], 5
        mov     rbx, qword ptr [rax + 8*r15]
        add     rbx, 10
        mov     qword ptr [rax + 8*r15], rbx
        add     ecx, -2
        jne     .LBB2_3
        add     rbx, qword ptr [rax + 8*r14]
        mov     esi, 80
        mov     edx, 8
        mov     rdi, rax
        call    qword ptr [rip + __rust_dealloc@GOTPCREL]
        mov     rax, rbx
        add     rsp, 32
        pop     rbx
        pop     r14
        pop     r15
        ret
.LBB2_4:
        lea     rdx, [rip + .L__unnamed_1]
        mov     esi, 10
        mov     rdi, r15
        call    qword ptr [rip + core::panicking::panic_bounds_check@GOTPCREL]
.LBB2_6:
        ud2
.LBB2_10:
        lea     rdi, [rsp + 8]
        call    alloc::raw_vec::RawVec<T,A>::allocate_in::{{closure}}
        ud2
.LBB2_5:
        lea     rdx, [rip + .L__unnamed_2]
        mov     esi, 10
        mov     rdi, r14
        call    qword ptr [rip + core::panicking::panic_bounds_check@GOTPCREL]
        jmp     .LBB2_6
        mov     rbx, rax
        lea     rdi, [rsp + 8]
        call    core::ptr::drop_in_place
        mov     rdi, rbx
        call    _Unwind_Resume@PLT
        ud2
```

We can see that ``LLB2_5`` and ``LLB2_4`` are the panicking IOOB branch. Where do we jump to them from? Ignoring whatever ``LLB2_5`` is, ``LLB2_4`` can be jumped to right from the beginning of ``LLB2_3``, the hot part of the loop! So, in essence, the written code is doing 100 bounds checks when it could be doing 1. Let's try and fix that.

The essence of "Ghosts of departed proofs" is that we should pass around at compile time some "proof" that some invariant holds, so that you do not need to check that invariant. For us, that will mean checking that an index is in bounds. Following the example of "tagging" from the paper, we want our final interface for the user to look something like this:

```rust
let v = vec![0u32; 10];
let v = name!(v);
let mut v = fix_size(v);

// Perform the two index checks here:
let index_a = v.check_index(...).unwrap();
let index_b = v.check_index(...).unwrap();

for _ in 0..100 {
    // These do *not* perform bounds checks!
    *v.get_mut(index_a) += 5;
    *v.get_mut(index_b) += 10;
}

let v = v.unname();

// continue using v...
```

So what's happening here? In the first 3 lines, we're wrapping the Vec in a FixedVec, so that it's size won't change (we could let it expand, but that's not important). Because of ownership, since we're passing ownership of v, this is fine. We also give v a name, just like in the "ghosts of departed proofs" paper.

In essence, what's important is that ``index_a`` has type ``Index<Name>``, and ``v`` has type ``FixedVec<u32, Name>``, and that ``Name`` matches between them. ``Name`` is a type created anonymously in the ``name!()`` macro. Since these types match, we must have created ``index_a`` from ``check_index`` on ``v``, and only on ``v``, and since ``v`` has a fixed size, the index must be in bounds.
