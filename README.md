# binstr

CLI to convert between:
- a string of binary digits (e.g. "1001010101100")
- UTF-8 strings (e.g. "hello")

## Usage

```
binstr

USAGE:
    binstr [OPTIONS]

OPTIONS:
    -d                Decode from digits to string
    -h, --help        Print help information
    -n                No trailing newline in output
        --no-strip    Don't strip trailing newline from input
```

## Exampleo

Encode a string to binary

```
~
$ echo "this is a test" | binstr

0111010001101000011010010111001100100000011010010111001100100000011000010010000001110100011001010111001101110100
```

Decode binary to a human-readable string

```
$ echo "0111010001101000011010010111001100100000011010010111001100100000011000010010000001110100011001010111001101110100" | binstr -d

this is a test
```
