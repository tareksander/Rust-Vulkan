# Rust


For the Rust backend, a .rs file is generated, with all entrypoints and public types marked as public. The file must compile on the latest stable Rust compiler (at the time of release of the RSL compiler), but no guarantees about older versions are made. If the resulting code does not compile please create a GitHub issue with the minimal source required to reproduce the failure.

When a struct layout was unspecified, Rust layout is used. Scalar block layout is translated to C layout.

Each entrypoint receives a struct as its first parameter containing the uniform and storage buffer data used.









