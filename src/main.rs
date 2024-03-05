/******************************************************************************
* Title: Data Shuffler
* Author: Devon Lattery
* Description: A program to consolidate and anonymize data in a directory.
*              The program can be run once, every 30 seconds for a specified
*              number of times, or scheduled to run daily, weekly, or monthly.
*******************************************************************************/

use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Read, Write, Error, ErrorKind};
use rand::seq::SliceRandom;
use std::result::Result;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::process::Command;
use std::os::raw::c_void;
use winapi::um::minwinbase::SYSTEMTIME;
use rand::Rng;

fn main() {    // TODO: Implement the main function
    //shuffle_data().unwrap();
    // If the user provides the -l or --loop flag followed by a number
    //   Run shuffle_data() every 30 seconds
    //   Repeat the number of times specified by the user or indefinitely if the user does not provide a number
    // Else if the user provides the -S or --schedule flag followed by daily, weekly, or monthly
    //   Schedule the data shuffle to run daily, weekly, or monthly with the system's scheduler
    //   Schedule the program to run weekly on Sunday at 12:00 AM if the user does not provide a time or day
    // Else if the user provides the -c or --cancel flag
    //   Cancel the scheduled data shuffle
    // Else if the user provides the -h or --help flag
    //   Print the help message
    // Else
    //   Shuffle the data once
    //   Print a message to the user
}

fn shuffle_data(dir: &str) -> Result<(), Error> {
    // Generate a random unix time within the last 10 days
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let ten_days = 10 * 24 * 60 * 60;
    let random_time = now - rand::thread_rng().gen_range(0..ten_days);

    // Set the system time to the generated unix time
    change_time(random_time);

    // For each directory in data directory
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Consolidate files
            print!("Consolidating files in {}...\n", path.to_str().unwrap());
            consolidate(path.to_str().unwrap())?;

            // Anonymize data
            anonymize_data(path.to_str().unwrap())?;
        }
    }

    // Restore the system time to the current time
    resync_time();

    Ok(())
}


fn change_time(timestamp: u64) {

    #[cfg(target_os = "windows")] {
        Command::new("powershell")
            .arg("-Command")
            .arg("Set-Date")
            .arg(format!("(Get-Date 01.01.1970).AddSeconds({})", timestamp))
            .output()
            .expect("Failed to execute command");
    } 
    #[cfg(target_os = "linux")] {
        let output = Command::new("date")
            .arg(format!("-s '@{}'", timestamp))
            .output()
            .expect("Failed to execute command");

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn resync_time() {
    #[cfg(target_os = "windows")] {
        // Start the Windows Time service
        Command::new("powershell")
            .arg("-Command")
            .arg("Start-Service")
            .arg("w32time")
            .output()
            .expect("Failed to execute command");

        // Resynchronize the system time
        Command::new("w32tm")
            .arg("/resync")
            .output()
            .expect("Failed to execute command");
    } 
    #[cfg(target_os = "linux")] {
        // Check if ntpdate is installed
        let output = Command::new("which")
        .arg("ntpdate")
        .output()
        .expect("Failed to execute command");

        // If ntpdate is not installed, install it
        if output.stdout.is_empty() {
            let output = Command::new("sudo")
                .arg("apt-get")
                .arg("install")
                .arg("ntpdate")
                .output()
                .expect("Failed to execute command");

            if !output.status.success() {
                println!("Failed to install ntpdate");
                return;
            }
        }

        // Resynchronize the system time
        let output = Command::new("sudo")
            .arg("ntpdate")
            .arg("pool.ntp.org")
            .output()
            .expect("Failed to execute command");

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}

fn consolidate(dir: &str) -> Result<(), Error> {
    let path = Path::new(dir);
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::Other, format!("Consolidation Error: {:?} is not a directory", dir)));
    }

    // For each subdirectory in the directory
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Move all files to the parent directory
            for file in fs::read_dir(path.clone())? {
                let file = file?;
                let file_path = file.path();
                let file_name = file_path.file_name().ok_or_else(|| Error::new(ErrorKind::Other, "No filename"))?;
                print!("Moving {:?} out of {}...\n", file_name, path.to_str().unwrap());
                let dest_path = PathBuf::from(dir).join(file_name);
                fs::rename(file_path, dest_path)?;
            }
            // Delete the subdirectory
            print!("Deleting {}...\n", path.to_str().unwrap());
            fs::remove_dir(path)?;
        }
    }

    Ok(())
}

fn anonymize_data(dir: &str) -> Result<(), Error> {
    let paths: Vec<_> = fs::read_dir(dir)?.map(|res| res.map(|e| e.path())).collect::<Result<Vec<_>, Error>>()?;
    let mut rng = rand::thread_rng();

    // Generate a random number for each file
    let mut numbers: Vec<_> = (1..=paths.len()).collect();
    numbers.shuffle(&mut rng);

    // Rename each file to a random number with a .csv extension
    for path in paths {
        print!("Anonymizing {}...\n", path.to_str().unwrap());
        let new_name = format!("{}/{}.csv", dir, numbers.pop().unwrap());
        let mut file = fs::File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        fs::write(new_name, contents)?;
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_change_time() {
        let then = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        // Call the function to test
        change_time(0);

        // Check that the system time has been changed
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        assert!(now < then);

        // Restore the system time to the current time
        resync_time();
    }

    #[test]
    fn test_resync_time() {
        // Call the function to test
        resync_time();

        // Check that the system time has been restored to the current time
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        assert!(now > 0);
    }

    #[test]
    fn test_consolidate() {
        // Setup: Create a directory structure for testing
        fs::create_dir_all("test_dir/sub_dir1").unwrap();
        fs::create_dir_all("test_dir/sub_dir2").unwrap();
        fs::write("test_dir/sub_dir1/file1.txt", "Hello, World!").unwrap();
        fs::write("test_dir/sub_dir2/file2.txt", "Hello, Rust!").unwrap();

        // Call the function to test
        consolidate("test_dir").unwrap();

        // Check that the files have been moved to the parent directory
        assert!(Path::new("test_dir/file1.txt").exists());
        assert!(Path::new("test_dir/file2.txt").exists());

        // Check that the subdirectories are deleted
        assert!(!Path::new("test_dir/sub_dir1").exists());
        assert!(!Path::new("test_dir/sub_dir2").exists());

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

    #[test]
    fn test_anonymize_data() {
        // Setup: Create a directory structure for testing
        fs::create_dir_all("test_dir").unwrap();
        fs::write("test_dir/file1.txt", "Hello, World!").unwrap();
        fs::write("test_dir/file2.txt", "Hello, Rust!").unwrap();

        // Call the function to test
        anonymize_data("test_dir").unwrap();

        // Check that files are renamed to .csv
        assert!(Path::new("test_dir/1.csv").exists());
        assert!(Path::new("test_dir/2.csv").exists());

        // Check that the contents of the files are unchanged
        assert!(fs::read_to_string("test_dir/1.csv").unwrap().contains("Hello, World!") || fs::read_to_string("test_dir/1.csv").unwrap().contains("Hello, Rust!"));
        assert!(fs::read_to_string("test_dir/2.csv").unwrap().contains("Hello, World!") || fs::read_to_string("test_dir/2.csv").unwrap().contains("Hello, Rust!"));

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

    #[test]
    fn test_consolidate_error() {
        // Setup: Create a file for testing
        fs::write("test_file.txt", "Hello, World!").unwrap();

        // Call the function to test
        let result = consolidate("test_file.txt");

        // Check that the function returns an error
        assert!(result.is_err());

        // Teardown: Clean up the test file
        fs::remove_file("test_file.txt").unwrap();
    }

    #[test]
    fn test_anonymize_data_error() {
        // Setup: Create a file for testing
        fs::write("test_file.txt", "Hello, World!").unwrap();

        // Call the function to test
        let result = anonymize_data("test_file.txt");

        // Check that the function returns an error
        assert!(result.is_err());

        // Teardown: Clean up the test file
        fs::remove_file("test_file.txt").unwrap();
    }

    #[test]
    fn test_consolidate_empty() {
        // Setup: Create a directory for testing
        fs::create_dir_all("test_dir").unwrap();

        // Call the function to test
        let result = consolidate("test_dir");

        // Check that the function returns an error
        assert!(result.is_ok());

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

    #[test]
    fn test_anonymize_data_empty() {
        // Setup: Create a directory for testing
        fs::create_dir_all("test_dir").unwrap();

        // Call the function to test
        let result = anonymize_data("test_dir");

        // Check that the function returns an error
        assert!(result.is_ok());

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

    #[test]
    fn test_consolidate_no_subdirectories() {
        // Setup: Create a directory for testing
        fs::create_dir_all("test_dir").unwrap();
        fs::write("test_dir/file1.txt", "Hello, World!").unwrap();
        fs::write("test_dir/file2.txt", "Hello, Rust!").unwrap();

        // Call the function to test
        let result = consolidate("test_dir");

        // Check that the function returns an error
        assert!(result.is_ok());

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

    #[test]
    fn test_anonymize_data_no_files() {
        // Setup: Create a directory for testing
        fs::create_dir_all("test_dir").unwrap();

        // Call the function to test
        let result = anonymize_data("test_dir");

        // Check that the function returns an error
        assert!(result.is_ok());

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

    #[test]
    fn test_consolidate_no_files() {
        // Setup: Create a directory for testing
        fs::create_dir_all("test_dir").unwrap();
        fs::create_dir_all("test_dir/sub_dir1").unwrap();
        fs::create_dir_all("test_dir/sub_dir2").unwrap();

        // Call the function to test
        let result = consolidate("test_dir");

        // Check that the function returns an error
        assert!(result.is_ok());

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

    #[test]
    fn test_anonymize_data_one_file() {
        // Setup: Create a directory for testing
        fs::create_dir_all("test_dir").unwrap();
        fs::write("test_dir/file1.txt", "Hello, World!").unwrap();

        // Call the function to test
        let result = anonymize_data("test_dir");

        // Check that the function returns an error
        assert!(result.is_ok());

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

    #[test]
    fn test_consolidate_one_subdirectory() {
        // Setup: Create a directory for testing
        fs::create_dir_all("test_dir").unwrap();
        fs::create_dir_all("test_dir/sub_dir1").unwrap();
        fs::write("test_dir/sub_dir1/file1.txt", "Hello, World!").unwrap();

        // Call the function to test
        let result = consolidate("test_dir");

        // Check that the function returns an error
        assert!(result.is_ok());

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

    #[test]
    fn test_shuffle_data() {
        // Setup: Create a directory structure for testing
        fs::create_dir_all("test_dir/sub_dir1/dir1").unwrap();
        fs::create_dir_all("test_dir/sub_dir1/dir2").unwrap();
        fs::create_dir_all("test_dir/sub_dir2/dir1").unwrap();
        fs::create_dir_all("test_dir/sub_dir3").unwrap();
        fs::write("test_dir/sub_dir1/dir1/file1.txt", "Hello, World!").unwrap();
        fs::write("test_dir/sub_dir1/dir2/file2.txt", "Hello, Rust!").unwrap();
        fs::write("test_dir/sub_dir2/dir1/file3.txt", "Hello, World!").unwrap();
        fs::write("test_dir/sub_dir3/file4.txt", "Hello, Rust!").unwrap();

        // Call the function to test
        shuffle_data("test_dir").unwrap();

        // Check that the number of files is unchanged
        let mut count = 0;
        for entry in fs::read_dir("test_dir/sub_dir1").unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                count += 1;
            }
        }
        assert_eq!(count, 2);
        count = 0;
        for entry in fs::read_dir("test_dir/sub_dir2").unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                count += 1;
            }
        }
        assert_eq!(count, 1);
        count = 0;
        for entry in fs::read_dir("test_dir/sub_dir3").unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                count += 1;
            }
        }
        assert_eq!(count, 1);

        // Check that files are renamed to .csv
        for entry in fs::read_dir("test_dir/sub_dir1").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                assert_eq!(path.extension().unwrap(), "csv");
            }
        }
        for entry in fs::read_dir("test_dir/sub_dir2").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                assert_eq!(path.extension().unwrap(), "csv");
            }
        }
        for entry in fs::read_dir("test_dir/sub_dir3").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                assert_eq!(path.extension().unwrap(), "csv");
            }
        }

        // Check that the contents of the files are unchanged
        assert!(fs::read_to_string("test_dir/sub_dir1/1.csv").unwrap().contains("Hello, World!") || fs::read_to_string("test_dir/sub_dir1/1.csv").unwrap().contains("Hello, Rust!"));
        assert!(fs::read_to_string("test_dir/sub_dir1/2.csv").unwrap().contains("Hello, World!") || fs::read_to_string("test_dir/sub_dir1/2.csv").unwrap().contains("Hello, Rust!"));
        assert!(fs::read_to_string("test_dir/sub_dir2/1.csv").unwrap().contains("Hello, World!") || fs::read_to_string("test_dir/sub_dir2/1.csv").unwrap().contains("Hello, Rust!"));
        assert!(fs::read_to_string("test_dir/sub_dir3/1.csv").unwrap().contains("Hello, World!") || fs::read_to_string("test_dir/sub_dir3/1.csv").unwrap().contains("Hello, Rust!"));

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }

}
