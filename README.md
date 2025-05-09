# Robert's Rust Cat Implementation

A Rust implementation of the Unix/Linux `cat` utility that concatenates and displays file contents with various formatting options.

## Features

- File concatenation and display
- Standard input processing
- Line numbering options
- Special character display
- Multiple blank line handling
- Comprehensive test coverage

### Supported Options

- `-b` - Number non-empty output lines
- `-E`, `-e` - Display `$` at the end of each line
- `-n` - Number all output lines
- `-s` - Suppress repeated empty output lines
- `-t`, `-T` - Display TAB characters as `^I`
- `-v` - Display non-printing characters using `^` and `M-` notation
- `-u` - (Ignored) Included for GNU cat compatibility
- `-h`, `-?`, `--help` - Display help information
