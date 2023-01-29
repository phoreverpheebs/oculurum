# oculurum
`oculurum` allows you to visualise data in interesting ways.
Convert a binary data stream into an RGB image and enjoy an interesting
representation of the data you use daily.

# Usage
`oculurum [options] <file or directory> [options]`

`file or directory` is either one input file or a directory to recurse through.

`options` are listed below.

`oculurum --help` is at your disposal whenever, however I will give an
overview of each option here:

## Compression levels
| Value | Level | Description |
| ----- | ----- | ----------- |
|   0   | `Default` | The default PNG compression level, that is probably most common |
|   1   | `Fast` | The fastest compression level, sacrifices resulting size for the speed of compression |
|   2   | `Best` | The opposite of `Fast`, sacrifices speed for output size |
|   3   | `Huffman` | Deprecated |
|   4   | `Rle` | Deprecated |

## Colour types
| Value | Type | Description |
| ----- | ---- | ----------- |
|   0   | `Bitwise` | Uses the inner `Grayscale` colour type, but tells oculurum to parse each byte into bits and make a 0 bit a black (off) pixel and a 1 bit a white (on) pixel |
|   1   | `Grayscale` | A pixel is defined by one byte ranging from 0 to 255 |
|   2   | `RGB` | A pixel is defined by 3 bytes, determining the amount  of red, green and blue of the pixel |
|   4   | `Grayscale Alpha` | A pixel is defined by 2 bytes; the first byte details the darkness factor, whilst the second is the transparency of the pixel |
|   5   | `RGB Alpha` | A pixel is defined by 4 bytes; the first 3 pertaining to the `RGB` colour type and the 4 byte being the transparency factor |
| Unimplemented | `Indexed` | Not implemented |

# Limitations
I've tried running the entire source code of the linux kernel through this, but
hit an integer overflow, I will be exploring whether this is a limitation of PNG or 
a limitation of `oculurum`'s implementation.

Nevertheless, images can get very large with this program, so have fun <3

# Disclaimer
Be careful sharing images created by `oculurum` as they are an accurate
representation of the base data, therefore the input data is reversible.
Don't share images formed from by any personal files.

On the other hand this could be an interesting way of sharing files.
