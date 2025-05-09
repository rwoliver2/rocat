use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process;

#[derive(Clone, Copy)]
enum NumberingMode {
    None,
    All,      // -n: number all lines
    NonBlank, // -b: number only non-blank lines
}

struct Options {
    numbering_mode: NumberingMode,
    show_ends: bool,        // Option -E/-e shows $ at the end of lines
    squeeze_blank: bool,    // Option -s squeeze multiple blank lines into one
    show_tabs: bool,        // Option -t/-T display TAB characters as ^I
    show_nonprinting: bool, // Option -v show non-printing characters
    #[allow(dead_code)]
    unbuffered: bool,       // Option -u ignored (historical compatibility)
}

fn print_help() {
    println!("Usage: rocat [OPTION]... [FILE]...
Concatenate FILE(s) to standard output.

With no FILE, or when FILE is -, read standard input.

Options:
    -b              number nonempty output lines, overrides -n
    -E, -e          display $ at end of each line
    -n              number all output lines
    -s              suppress repeated empty output lines
    -t, -T          display TAB characters as ^I
    -u              (ignored) for compatibility with GNU cat
    -v              use ^ and M- notation, except for LFD and TAB
    -h, -?, --help  display this help and exit

Examples:
    rocat f - g      Output f's contents, then standard input, then g's contents.
    rocat            Copy standard input to standard output.

Please report bugs to: https://github.com/rwoliver2/rocat/issues");
    process::exit(0);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Check for help flags before anything else
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help" || args[1] == "-?") {
        print_help();
    }
    
    // Parse supplied options
    let options = Options {
        // -n takes precedence over -b to match GNU's behavior
        numbering_mode: if args.iter().any(|arg| arg == "-n") {
            NumberingMode::All
        } else if args.iter().any(|arg| arg == "-b") {
            NumberingMode::NonBlank
        } else {
            NumberingMode::None
        },
        show_ends: args.iter().any(|arg| arg == "-e" || arg == "-E"),
        squeeze_blank: args.iter().any(|arg| arg == "-s"),
        show_tabs: args.iter().any(|arg| arg == "-t" || arg == "-T"),
        show_nonprinting: args.iter().any(|arg| arg == "-v"),
        unbuffered: args.iter().any(|arg| arg == "-u"),
    };
    
    // If no files are specified (excluding the flags), read from stdin
    let has_files = args.iter().skip(1).any(|arg| !is_flag(arg));
    if !has_files {
        return cat_stdin(&options);
    }

    // Process file(s) specified in the arguments, skipping flags
    let files: Vec<&String> = args[1..].iter()
        .filter(|arg| !is_flag(arg))
        .collect();

    for file_path in files {
        if let Err(err) = cat_file(file_path, &options) {
            eprintln!("Error reading {}: {}", file_path, err);
        }
    }

    Ok(())
}

fn is_flag(arg: &str) -> bool {
    matches!(arg, "-n" | "-b" | "-e" | "-E" | "-s" | "-t" | "-T" | "-u" | "-v" | 
                  "-h" | "--help" | "-?")
}

fn cat_stdin(options: &Options) -> io::Result<()> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    print_lines(reader, options)
}

fn cat_file(file_path: &str, options: &Options) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    print_lines(reader, options)
}

fn print_lines<R: BufRead>(reader: R, options: &Options) -> io::Result<()> {
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    print_lines_to_writer(reader, options, &mut writer)
}

// print_lines function that accepts a generic writer
fn print_lines_to_writer<R: BufRead, W: Write>(
    reader: R,
    options: &Options,
    writer: &mut W,
) -> io::Result<()> {
    let mut line_number = 1;
    let mut last_was_blank = false;

    for line in reader.lines() {
        let line = line?;
        let is_blank = line.trim().is_empty();

        if options.squeeze_blank && is_blank && last_was_blank {
            continue;
        }
        last_was_blank = is_blank;

        match options.numbering_mode {
            NumberingMode::All => {
                write!(writer, "{:6}\t", line_number)?;
                line_number += 1;
            }
            NumberingMode::NonBlank => {
                if !is_blank {
                    write!(writer, "{:6}\t", line_number)?;
                    line_number += 1;
                }
            }
            NumberingMode::None => {}
        }

        let mut output = String::new();
        for c in line.chars() {
            match c {
                '\t' if options.show_tabs => output.push_str("^I"),
                c if options.show_nonprinting && !c.is_ascii_graphic() && !c.is_ascii_whitespace() => {
                    output.push('^');
                    output.push((c as u8 + 64) as char);
                }
                c => output.push(c),
            }
        }

        write!(writer, "{}", output)?;
        
        if options.show_ends {
            write!(writer, "$")?;
        }
        writeln!(writer)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Helper function to process a string
    fn process_string(input: &str, options: &Options) -> io::Result<String> {
        let cursor = Cursor::new(input);
        let mut output = Vec::new();
        {
            // Temporarily replace stdout with vector
            let mut custom_writer = Cursor::new(&mut output);
            print_lines_to_writer(cursor, options, &mut custom_writer)?;
        }
        Ok(String::from_utf8(output).unwrap())
    }

    fn default_options() -> Options {
        Options {
            numbering_mode: NumberingMode::None,
            show_ends: false,
            squeeze_blank: false,
            show_tabs: false,
            show_nonprinting: false,
            unbuffered: false,
        }
    }

    #[test]
    fn test_basic_output() -> io::Result<()> {
        let input = "Hello\nWorld";
        let options = default_options();
        let output = process_string(input, &options)?;
        assert_eq!(output, "Hello\nWorld\n");
        Ok(())
    }

    #[test]
    fn test_number_all_lines() -> io::Result<()> {
        let input = "Hello\nWorld";
        let mut options = default_options();
        options.numbering_mode = NumberingMode::All;
        let output = process_string(input, &options)?;
        assert_eq!(output, "     1\tHello\n     2\tWorld\n");
        Ok(())
    }

    #[test]
    fn test_number_nonblank_lines() -> io::Result<()> {
        let input = "Hello\n\nWorld";
        let mut options = default_options();
        options.numbering_mode = NumberingMode::NonBlank;
        let output = process_string(input, &options)?;
        assert_eq!(output, "     1\tHello\n\n     2\tWorld\n");
        Ok(())
    }

    #[test]
    fn test_show_ends() -> io::Result<()> {
        let input = "Hello\nWorld";
        let mut options = default_options();
        options.show_ends = true;
        let output = process_string(input, &options)?;
        assert_eq!(output, "Hello$\nWorld$\n");
        Ok(())
    }

    #[test]
    fn test_squeeze_blank() -> io::Result<()> {
        let input = "Hello\n\n\n\nWorld";
        let mut options = default_options();
        options.squeeze_blank = true;
        let output = process_string(input, &options)?;
        assert_eq!(output, "Hello\n\nWorld\n");
        Ok(())
    }

    #[test]
    fn test_show_tabs() -> io::Result<()> {
        let input = "Hello\tWorld";
        let mut options = default_options();
        options.show_tabs = true;
        let output = process_string(input, &options)?;
        assert_eq!(output, "Hello^IWorld\n");
        Ok(())
    }

    #[test]
    fn test_show_nonprinting() -> io::Result<()> {
        let input = "Hello\u{0001}World";
        let mut options = default_options();
        options.show_nonprinting = true;
        let output = process_string(input, &options)?;
        assert_eq!(output, "Hello^AWorld\n");
        Ok(())
    }

    #[test]
    fn test_multiple_options() -> io::Result<()> {
        let input = "Hello\n\n\tWorld";
        let mut options = default_options();
        options.show_ends = true;
        options.show_tabs = true;
        options.squeeze_blank = true;
        let output = process_string(input, &options)?;
        assert_eq!(output, "Hello$\n$\n^IWorld$\n");
        Ok(())
    }
}