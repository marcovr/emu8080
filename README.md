# emu8080

My first experiment using Rust.

An emulator for the [Intel 8080](https://en.wikipedia.org/wiki/Intel_8080) processor capable of running
the original [Space Invaders](https://en.wikipedia.org/wiki/Space_Invaders) game from 1978.

<br>

# Build instructions
**Requirements:**
- [Rust & cargo](https://rustup.rs/)
- [OpenAL and libsndfile](https://crates.io/crates/ears#before-you-start) for the ears package to work.

<br>

**Build:**

```cargo build```

<br>

**Dependencies:** (cargo will take care of them) 
* [piston_window](https://crates.io/crates/piston_window)
* [image](https://crates.io/crates/image)
* [ears](https://crates.io/crates/ears)

