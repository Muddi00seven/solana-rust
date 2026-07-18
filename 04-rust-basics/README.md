# Rust Basics 🦀

Rust ke saare zaroori basics — har concept ka **alag file**, poori tarah **comments** ke saath (Roman Urdu + English). Har file ek independent, chalne wali example hai.

## Kaise run karein

Rust install hona chahiye (`rustc`, `cargo`). Phir is folder ke andar:

```bash
# Kisi ek example ko run karo:
cargo run --bin 01_variables

# Sab kuch compile/check karo (bina run kiye):
cargo build
```

## Files (seekhne ka order)

| # | File | Topic |
|---|------|-------|
| 01 | `src/bin/01_variables.rs` | Variables, mutability, constants, shadowing |
| 02 | `src/bin/02_data_types.rs` | Scalar & compound types (int, float, bool, char, tuple, array) |
| 03 | `src/bin/03_functions.rs` | Functions, parameters, return values, expressions |
| 04 | `src/bin/04_control_flow.rs` | if/else, loop, while, for, ranges |
| 05 | `src/bin/05_ownership.rs` | Ownership, move, clone, copy |
| 06 | `src/bin/06_borrowing.rs` | References & borrowing (`&`, `&mut`), borrow rules |
| 07 | `src/bin/07_slices.rs` | String & array slices |
| 08 | `src/bin/08_structs.rs` | Structs, methods, `impl`, associated functions |
| 09 | `src/bin/09_enums.rs` | Enums, `match`, `Option`, `if let` |
| 10 | `src/bin/10_collections.rs` | `Vec`, `String`, `HashMap` |
| 11 | `src/bin/11_error_handling.rs` | `Result`, `Option`, `panic!`, `?` operator |
| 12 | `src/bin/12_generics_traits.rs` | Generics & traits |
| 13 | `src/bin/13_closures_iterators.rs` | Closures & iterators (map/filter/sum) |

## Tip

Har file ko upar se neeche parho — comments step by step samjhate hain ke kya ho raha hai aur kyun.
