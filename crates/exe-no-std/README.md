
# Exe no-std

# Building Without Cargo

```
rustc src/main.rs `
    -C panic=abort `
    -o ./target/release/main.exe `
    -C opt-level=3
```

```
rustc src/main.rs `
    -C panic=abort `
    -C opt-level=3 `
    -C link-arg=/DEBUG:NONE `
    -C link-arg=/EMITPOGOPHASEINFO `
    -C link-arg=/OPT:REF `
    -C link-arg=/OPT:ICF `
    --out-dir ./target/release
```