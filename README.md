# emu8080

My first experiment using Rust.

An emulator for the [Intel 8080](https://en.wikipedia.org/wiki/Intel_8080) processor capable of running
the original [Space Invaders](https://en.wikipedia.org/wiki/Space_Invaders) game from 1978.

<br>

# Build instructions
## Without audio support
**Requirements:**
- [Rust & cargo](https://rustup.rs/)

<br>

**Build:**

```cargo build```

<br>

## With audio support
**Requirements:**
- [Rust & cargo](https://rustup.rs/)
- [OpenAL and libsndfile](https://crates.io/crates/ears#before-you-start) for the ears package to work.
- Space Invaders audio files `0.wav, ... , 8.wav`
  1. Download the files (They can be easily found online)
  2. Create directory `sound`
  3. Move the files into the directory

<br>

**Build:**

```cargo build --features audio```

<br>

## Dependencies (cargo takes care of them)
* [piston_window](https://crates.io/crates/piston_window)
* [image](https://crates.io/crates/image)
* [ears](https://crates.io/crates/ears) (optional, for audio support)

