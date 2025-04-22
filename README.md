# LaBRADOR, implemented in lattirust

This repository contains a Rust implementation of the LaBRADOR lattice-based argument [[BS23]](#BS23), using the [lattirust](https://github.com/lattirust) library.
Currently, only the core LaBRADOR protocol is implemented, with implementations of reductions from (binary and ring) R1CS in progress. 

## Usage 
### Tests
```
cargo test
```

## References
<a id="BS23">[BS23]</a>: W. Beullens and G. Seiler, “LaBRADOR: Compact Proofs for R1CS from Module-SIS”, CRYPTO 2023. Available at https://eprint.iacr.org/2022/1341. 
