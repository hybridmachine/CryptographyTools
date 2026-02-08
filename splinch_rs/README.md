# splinch_rs

A CLI tool that splits a file into two XOR-complementary parts for secure transport.

Neither output file alone reveals any information about the original. XOR-ing the two files together reconstructs the original exactly (one-time pad splitting).

## Installation

### From .deb package (Ubuntu/Debian)

```bash
sudo dpkg -i splinch-rs_0.1.0-1_amd64.deb
```

### From source

```bash
cargo build --release
cp target/release/splinch_rs /usr/local/bin/splinch-rs
```

## Usage

Split a file:

```bash
splinch-rs -i secret.pdf
# Creates: secret.pdf.xor1, secret.pdf.xor2
```

Split and verify:

```bash
splinch-rs -i secret.pdf -v
```

Combine two parts back into the original:

```bash
splinch-rs -i secret.pdf.xor1 -c
# Restores: secret.pdf
```

## Security

For secure transport, send the `.xor1` and `.xor2` files over **separate, independent channels**. Sending both over the same channel defeats the security guarantee.

Each output file is statistically indistinguishable from random data. The splitting uses a cryptographically secure random number generator.

## License

MIT
