# General Predicate Testing

## Profiling

Profiling on MacOS can be done with [cargo-instruments](https://github.com/cmyr/cargo-instruments). Refer to its installation documentation.

Example profiling command:
```sh
cargo instruments -t time run --no-show ../examples/gpt-lang/complex.gpt
```
