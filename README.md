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

To directly run the program, substitute `build` with `run`.

<br>

## With audio support
**Requirements:**
- [Rust & cargo](https://rustup.rs/)
- [OpenAL and libsndfile](https://crates.io/crates/ears#before-you-start) for the ears package to work.
- Space Invaders audio files
  1. Download the files (They can be easily found online)
  2. Move the files into the `sound` directory
  3. Make sure they are named correctly: `0.wav, ... , 8.wav`

<br>

**Build:**

```cargo build --features audio```

To directly run the program, substitute `build` with `run`.

<br>

## Dependencies (cargo takes care of them)
* [piston_window](https://crates.io/crates/piston_window)
* [image](https://crates.io/crates/image)
* [ears](https://crates.io/crates/ears) (optional, for audio support)

