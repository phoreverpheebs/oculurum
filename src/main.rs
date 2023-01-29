use std::{
    env,
    process,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use png;
use bitflags::bitflags;

bitflags!{
    #[derive(Default)]
    struct Flags: u32 {
        const C_DEFAULT     = 1 << 2;
        const C_FAST        = 1 << 3;
        const C_BEST        = 1 << 4;
        const C_HUFFMAN     = 1 << 5;
        const C_RLE         = 1 << 6;
        const COMPRESSION   = 0b1111100;
        const T_BIT         = 1 << 8;
        const T_GRAY        = 1 << 9;
        const T_RGBS        = 1 << 10;
        // const T_INDX        = 1 << 11;
        const T_GRAL        = 1 << 12;
        const T_RGBA        = 1 << 13;
        const COLORTYPE     = 0b11111100000000;
    }
}

#[derive(Debug)]
struct ParseFlagError;

impl Flags {
    #[allow(deprecated)]
    fn compression(&self) -> png::Compression {
        let  n = *self & Flags::COMPRESSION;
        match n {
            Flags::C_FAST => png::Compression::Fast,
            Flags::C_BEST => png::Compression::Best,
            Flags::C_HUFFMAN => png::Compression::Huffman,
            Flags::C_RLE => png::Compression::Rle,
            _ => png::Compression::Default,
        }
    }

    fn compression_from_str(s: &str) ->  Result<Self, ParseFlagError> {
        s.parse::<u32>().map_or(Err(ParseFlagError), |n| match n {
            0 => Ok(Flags::C_DEFAULT),
            1 => Ok(Flags::C_FAST),
            2 => Ok(Flags::C_BEST),
            3 => Ok(Flags::C_HUFFMAN),
            4 => Ok(Flags::C_RLE),
            _ => Err(ParseFlagError),
        })
    }

    fn color_type(&self) -> png::ColorType {
        let n = *self & Flags::COLORTYPE;
        match n {
            Flags::T_RGBS => png::ColorType::Rgb,
            // Flags::T_INDX => png::ColorType::Indexed,
            Flags::T_GRAL => png::ColorType::GrayscaleAlpha,
            Flags::T_RGBA => png::ColorType::Rgba,
            _ => png::ColorType::Grayscale,
        }
    }

    fn type_from_str(s: &str) -> Result<Self, ParseFlagError> {
        s.parse::<u32>().map_or(Err(ParseFlagError), |n| match n {
            0 => Ok(Flags::T_BIT),
            1 => Ok(Flags::T_GRAY),
            2 => Ok(Flags::T_RGBS),
            // 3 => Ok(Flags::T_INDX),
            4 => Ok(Flags::T_GRAL),
            5 => Ok(Flags::T_RGBA),
            _ => Err(ParseFlagError),
        })
    }
}

fn main() {
    let mut flags: Flags = Default::default();
    let mut input_file: String = String::with_capacity(255);
    let mut args: Vec<String> = env::args().skip(1).rev().collect();

    while let Some(arg) = args.pop() {
        if arg.starts_with("--") {
            match arg.as_str() {
                "--compression" => if let Some(value) = args.pop() {
                    flags |= Flags::compression_from_str(&value)
                        .unwrap_or_else(|_| {
                            eprintln!("Invalid argument to `--compression`.\nUsing default.");
                            Flags::C_DEFAULT
                        });
                } else {
                    exit_with_help(1);
                },
                "--type" => if let Some(value) = args.pop() {
                    flags |= Flags::type_from_str(&value)
                        .unwrap_or_else(|_| {
                            eprintln!("Invalid argument to `--type`.\nUsing default.");
                            Flags::T_GRAY
                        });
                } else {
                    exit_with_help(1);
                },
                "--help" => exit_with_help(0),
                _ => exit_with_help(1),
            }
        } else if arg.starts_with("-") {
            match arg.as_str() {
                "-c" => if let Some(value) = args.pop() {
                    flags |= Flags::compression_from_str(&value)
                        .unwrap_or_else(|_| {
                            eprintln!("Invalid argument to `-c`.\nUsing default.");
                            Flags::C_DEFAULT
                        });
                } else {
                    exit_with_help(1);
                },
                "-t" => if let Some(value) = args.pop() {
                    flags |= Flags::type_from_str(&value)
                        .unwrap_or_else(|_| {
                            eprintln!("Invalid argument to `-t`.\nUsing default.");
                            Flags::T_GRAY
                        });
                } else {
                    exit_with_help(1);
                },
                "-h" => exit_with_help(0),
                _ => exit_with_help(1),
            }
        } else {
            if !input_file.is_empty() {
                eprintln!("Multiple input files.");
                process::exit(2);
            }

            input_file = arg;
        }
    }

    if input_file.is_empty() {
        exit_with_help(3);
    }

    match run(input_file, flags) {
        Ok(_) => eprintln!("\nSuccess!"),
        Err(e) => eprintln!("\nError occured: {e}"),
    }
}

const BLACK_PX: u8 = 0x00;
const WHITE_PX: u8 = 0xff;

#[inline]
fn handle_bitwise(data: Vec<u8>) -> Vec<u8> {
    data
        .into_iter()
        .map(|b| (0..8)
            .map(|shift| if b & 1 << shift == 0 {
                BLACK_PX
            } else {
                WHITE_PX
            })
            .collect::<Vec<u8>>()
        )
        .flatten()
        .collect::<Vec<u8>>()
}

#[inline(always)]
fn handle_others(data: Vec<u8>) -> Vec<u8> {
    data
}

fn run<P: AsRef<Path>>(path: P, flags: Flags) -> Result<(), png::EncodingError> {
    let path = path.as_ref();

    let dimension: u32;
    let mut filenames: Vec<PathBuf>;

    // not sure how i feel about introducing floating points here
    // it will make the initial calculation slower, but it is just ONE fp calculation
    // compiler might optimise it without using the `div` instruction
    let (bytes_per_pixel, multiplier): (u32, f64) = match flags & Flags::COLORTYPE {
        Flags::T_BIT => (1, 8.0),
        Flags::T_RGBS => (3, 0.3),
        Flags::T_GRAL => (2, 0.5),
        Flags::T_RGBA => (4, 0.25),
        _ => (1, 1.0),
    };

    if path.is_dir() {
        filenames = Vec::new();
        dimension = f64::sqrt(multiplier * calculate_dimensions(&path, &mut filenames)
            .expect("Error in calculating  directory size.") as f64) as u32 + 1;
    } else {
        dimension = f64::sqrt(multiplier * path
            .metadata()
            .expect("Couldn't get file metadata.")
            .len() as f64) as u32 + 1;
        filenames = Vec::with_capacity(1);
        filenames.push(path.to_path_buf());
    }

    let mut output_path = path.to_str().unwrap().to_string();
    output_path.push_str(".png");
    let file = fs::File::create(output_path).unwrap();

    let ref mut w = io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, dimension, dimension);

    encoder.set_color(flags.color_type());
    encoder.set_compression(flags.compression());

    let handler = if flags & Flags::COLORTYPE == Flags::T_BIT {
        handle_bitwise
    } else {
        handle_others
    };

    let mut binding = encoder.write_header().unwrap();
    let mut writer = binding.stream_writer().unwrap();

    let mut buffer: [u8; 4096] = [0; 4096];
    let mut written = 0usize;

    filenames
        .into_iter()
        .for_each(|filename| {
            eprintln!("Writing data from {}", &filename.display());
            let ref mut f = fs::File::open(filename).unwrap();
            let mut reader = io::BufReader::new(f);

            let mut nread = reader.read(&mut buffer).unwrap();

            let mut chunk_counter = 0;

            while nread > 0 {
                eprint!("Writing chunk {chunk_counter}\r");

                writer.write_all(&handler(buffer[..nread].to_vec())).unwrap();
                written += nread;

                nread = reader.read(&mut buffer).unwrap();
                chunk_counter += 1;
            }
        });

    dbg!(written);
    dbg!(dimension*dimension);
    let padding = vec![0;
        (((bytes_per_pixel << 2).checked_mul(dimension.checked_pow(2).unwrap()).unwrap()) as usize)
            .checked_sub(written)
            .unwrap_or(0)
    ];
    writer.write_all(&padding[..]).unwrap();
    Ok(())
}

fn calculate_dimensions<P: AsRef<Path>>(dir: P, filenames: &mut Vec<PathBuf>) -> io::Result<u64> {
    let dir = dir.as_ref();
    let mut size = 0u64;
    if dir.is_dir() {
        for entry in dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                size += calculate_dimensions(&path, filenames)?;
            } else {
                size += path.metadata()?.len();
                filenames.push(path);
            }
        }
    }
    Ok(size)
}

fn exit_with_help(code: u8) {
    println!("Usage: oculurum [options] <file or directory>

Options:
        -h | --help
            Prints this help message.

        -c | --compression <value>
            The PNG compression level.
            Values: 
                0 => Default
                1 => Fast
                2 => Best
            Deprecated Values:
                3 => Huffman
                4 => Rle

        -t | --type <value>
            The PNG colour type.
            Values:
                0 => Bitwise
                1 => Grayscale (Default)
                2 => RGB
                4 => Grayscale Alpha
                5 => RGB Alpha
            Unimplemented:
                _ => Indexed 
");
    process::exit(code as i32);
}
