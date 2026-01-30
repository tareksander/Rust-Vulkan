# Hello Compute

````Rust
use ::globalInvocationId;

#[compute(1, 1, 1)]
fn add(
    // Parameters to compute shader functions are converted to push constants
    // "uni" means that the values in the buffer are uniform, that is the same for every invocation.
    a: *const phy uni u32,
    b: *const phy uni u32,
    // "nuni" means that the values are different for each invocation. While every invocation could observe the
    // same value in the same index of the output buffer after it is written, in principle every invocation computes a distinct value.
    c: *mut phy nuni u32,
) {
    // The type of i is actually not just "u32", but "nuni u32", since each invocation has its own id.
    let i = globalInvocationId.x;
    // An unsafe block is needed for pointer interactions, since there are no references yet.
    unsafe {
        // Indexing the buffers with a non-uniform index "destroys" the uniformity of the values,
        // which is why the buffer is declared with nuni in the parameters.
        a[i] = b[i] + c[i];
    }
}
````



