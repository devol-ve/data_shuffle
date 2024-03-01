// ------------------------------------------------------------------------------
// Title: Data Shuffler
// Author: Devon Lattery
// Description: A program to consolidate and anonymize data in a directory.
//   The program can be run once, every 30 seconds for a specified number of times, or scheduled to run daily, weekly, or monthly.
// ------------------------------------------------------------------------------

use std::io;

fn main() {    // TODO: Implement the main function
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

// TODO: Implement this function
fn shuffle_data() {
    // Generate a random unix time within the last 10 days
    //   Generate a random number between 0 and 864000
    //   Subtract the random number from the current unix time
    
    // Set the system time to the generated unix time

    // For each directory in data directory
    //   Consolidate files
    //   Anonymize data
    
    // Restore the system time to the current time
} 

// TODO: Implement this function
fn consolidate(dir: &str) -> Result<(), io::Error> {
    // For each subdirectory in the directory
    //   Move all files to the parent directory
    //   Delete the subdirectory
    Ok(())
}

// TODO: Implement this function
fn anonymize_data(dir: &str) -> Result<(), io::Error> {
    // Generate a random list of unique integers from 1 to the number of files in the directory

    // For each file in the directory
    //   Convert the file to a .csv file
    //   Copy contents of the file to a new .csv file with a random number as the name
    //   Delete the original file
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

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

        // Check that the number of files is unchanged
        assert_eq!(fs::read_dir("test_dir").unwrap().count(), 2);

        // Check that files are renamed to .csv
        assert!(Path::new("test_dir/1.csv").exists());
        assert!(Path::new("test_dir/2.csv").exists());

        // Check that the contents of the files are unchanged
        assert_eq!(fs::read_to_string("test_dir/1.csv").unwrap(), "Hello, World!");
        assert_eq!(fs::read_to_string("test_dir/2.csv").unwrap(), "Hello, Rust!");

        // Teardown: Clean up the test directory
        fs::remove_dir_all("test_dir").unwrap();
    }


}
