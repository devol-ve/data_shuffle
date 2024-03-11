# Data Shuffler

Data Shuffler is a Rust program designed to consolidate and anonymize data in a directory. The program provides various options for running the data shuffling process, including a one-time run, a scheduled run, or a loop that runs the process multiple times.

## Installation

Clone the GitHub repository.

```bash
git clone https://github.com/devol-ve/data_shuffle.git
```

Install the program using Cargo.

```bash
cargo install --path ./data_shuffle
```

## Usage

```bash
data_shuffle [OPTION]
```

### Options

-  `-l`, `--loop` `[<COUNT>]` Shuffle the data every 30 seconds and repeat the specified number of times or until Esc is pressed.
-  `-s`, `--schedule` `[<DAY>]` `[at <TIME>]` Schedule the data shuffle to run every week with the system's scheduler. Defaults to Sunday at 12:00 AM if no time or day is provided.
-  `-c`, `--cancel` Cancel the scheduled data shuffle.
-  `--no-warning` Shuffle the data once without warning message if not run as admin or root.
-  `-h`, `--help` Print the help message.

## Requirements

- Rust programming language.
- The targeted directory `./data/` must be located within the current working directory upon execution.
- Admin/root privileges are required to randomize creation time. (Optional)

## Limitations

Currently only compatible with Debian-based Linux distributions and Windows 10 or newer. No support for macOS at this time.

## License

[MIT](https://choosealicense.com/licenses/mit/)
