/******************************************************************************
* Title: Data Shuffler
* Author: Devon Lattery
* Description: A program to consolidate and anonymize data in a directory.
*              The program can be run once, every 30 seconds for a specified
*              number of times, or scheduled to run weekly.
*******************************************************************************/

use std::fs;
use std::env;
use std::path::{Path, PathBuf};
use std::io::{Read, Error, ErrorKind};
use rand::seq::SliceRandom;
use std::result::Result;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::process::Command;
use rand::Rng;
use std::thread::sleep;
use console::Term;

fn main() {
    // Get the command line arguments
    let args: Vec<String> = env::args().collect();
    let data_dir = "data";
    let term = Term::stdout();

    if args.len() == 1 {
        if !is_admin() {
            if cfg!(target_os = "windows") {
                println!("Admin privileges not detected.");
            } else {
                println!("Root privileges not detected.");
            }
            
            println!("File creation time will default to the current time.");
            println!("Would you like to continue? (Y/n)");

            let mut input = 'y';
            loop {
                if let Ok(key) = term.read_key() {
                    input = match key {
                        console::Key::Char('y') | console::Key::Char('Y') | console::Key::Enter => 'y',
                        console::Key::Char('n') | console::Key::Char('N') => 'n',
                        _ => input,
                    };
                }
                if input == 'n' {
                    break;
                }
                sleep(Duration::from_millis(5));
            }
            if input == 'n' {
                println!("Exiting...");
                return;
            } else {
                println!("Continuing...");
            }
        }
        // Shuffle the data once
        shuffle_data(data_dir).unwrap();
    } else {
        // Parse the command line arguments
        match args[1].as_str() {
            "--no-warning" => {
                // Shuffle the data once
                shuffle_data(data_dir).unwrap();
            }
            "-l" | "--loop" => {
                // Run shuffle_data() every 30 seconds
                // Repeat the number of times specified by the user or indefinitely if the user does not provide a number
                let mut count = 100;
                if args.len() > 2 {
                    count = args[2].parse().unwrap();
                }
                loop {
                    shuffle_data("data").unwrap();
                    count -= 1;
                    // If count is 0 or the user presses Esc, break the loop
                    if term.read_key().unwrap() == console::Key::Escape || count == 0 {
                        break;
                    }
                    sleep(Duration::from_secs(30));
                }
            }
            "-S" | "--schedule" => {
                // Schedule the data shuffle to run weekly with the system's scheduler
                // Schedule the program to run weekly on Sunday at 12:00 AM if the user does not provide a time or day
                let mut time = "00:00".to_string();
                let mut day = "Sun".to_string();
                if args.len() > 2 {
                    for mut i in 2..args.len() {
                        match args[i].to_lowercase().as_str() {
                            "su" | "sun" | "sunday" => {
                                // Schedule for Sunday
                                day = "Sun".to_string();
                            }
                            "m" | "mon" | "monday" => {
                                // Schedule for Monday
                                day = "Mon".to_string();
                            }
                            "t" | "tu" | "tue" | "tues" | "tuesday" => {
                                // Schedule for Tuesday
                                day = "Tue".to_string();
                            }
                            "w" | "wed" | "wednesday" => {
                                // Schedule for Wednesday
                                day = "Wed".to_string();
                            }
                            "h" | "th" | "thu" | "thurs" | "thursday" => {
                                // Schedule for Thursday
                                day = "Thu".to_string();
                            }
                            "f" | "fri" | "friday" => {
                                // Schedule for Friday
                                day = "Fri".to_string();
                            }
                            "s" | "sa" | "sat" | "saturday" => {
                                // Schedule for Saturday
                                day = "Sat".to_string();
                            }
                            "at" => {
                                // Increment the index to get the time
                                i += 1;

                                // Validate the time
                                if !is_valid_time(&args[i]) {
                                    println!("Invalid time. Please provide a valid time in the format HH:MM.\nFor example, 13:30 for 1:30 PM.");
                                    println!("Exiting...");
                                    return;
                                }
                                // Schedule the data shuffle to run at a specific time
                                if args.len() > i {
                                    time = args[i].to_string();
                                }
                            }
                            _ => {
                                println!("Invalid schedule option");
                                return;
                            }
                        }
                    }
                }
                // Schedule the program to run weekly on Sunday at 12:00 AM if the user does not provide a time or day
                println!("Scheduling the data shuffle to run {} on {} at {}...", args[2], day, time);
                schedule(&day, &time);
            }
            "-c" | "--cancel" => {
                // Cancel the scheduled data shuffle
                println!("Cancelling the scheduled data shuffle...");
                cancel()
            }
            "-h" | "--help" => {
                // Print the help message
                println!("Usage: data_shuffler [OPTION]");
                println!("Shuffle the data in the data directory");
                println!("");
                println!("Options:");
                println!("  -l, --loop [COUNT]                 Run shuffle_data() every 30 seconds and repeat the number of times");
                println!("                                     specified by the user or 100 times if the user does not provide a number");
                println!("  -S, --schedule [DAY] [at TIME]");
                println!("                                     Schedule the data shuffle to run every week with the system's scheduler");
                println!("                                     Defaults to Sunday at 12:00 AM if the user does not provide a time or day");
                println!("  -c, --cancel                       Cancel the scheduled data shuffle");
                println!("  -h, --help                         Print the help message");
            }
            _ => {
                println!("Invalid option");
            }
        }
    }
}

fn schedule(day: &str, time: &str) {
    let cwd = env::current_dir().expect("Failed to get current directory").to_str().expect("Failed to convert path to string").to_string();
    let exe = env::current_exe().expect("Failed to get current executable").to_str().expect("Failed to convert path to string").to_string();
    let command = if cfg!(target_os = "windows") {
        format!("schtasks /create /tn DataShuffler /tr \"cmd /c cd /d {} && {} --no-warning\" /sc weekly /d {} /st {}", cwd, exe, day, time)
    } else {
        let hrs = time.split(":").collect::<Vec<&str>>()[0];
        let mins = time.split(":").collect::<Vec<&str>>()[1];
        format!("( crontab -l; echo \"{} {} * * {} cd {} && {} --no-warning\"; ) | crontab -", mins, hrs, day, cwd, exe)
    };
    if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(&["-Command", &command])
            .output()
            .expect("Failed to execute command")
    } else {
        Command::new("sh")
            .args(&["-c", &command])
            .output()
            .expect("Failed to execute command")
    };
}

fn cancel() {
    let exe = env::current_exe().expect("Failed to get current executable").to_str().expect("Failed to convert path to string").to_string();
    let command = if cfg!(target_os = "windows") {
        "schtasks /delete /tn DataShuffler /f".to_string()
    } else {
        format!("crontab -l | grep -v \" {} \" | crontab -", exe)
    };
    
    if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(&["-Command", &command])
            .output()
            .expect("Failed to execute command")
    } else {
        Command::new("sh")
            .args(&["-c", &command])
            .output()
            .expect("Failed to execute command")
    };
}

fn is_valid_time(time: &str) -> bool {
    let time = time.split(":").collect::<Vec<&str>>();
    if time.len() != 2 {
        return false;
    }
    let hour = time[0].parse::<u32>().unwrap();
    let minute = time[1].parse::<u32>().unwrap();
    if hour > 23 || minute > 59 {
        return false;
    }
    true
}

fn is_admin() -> bool {
    let output = if cfg!(target_os = "windows") {
        Command::new("net")
            .arg("session")
            .output()
            .expect("Failed to execute command")
    } else {
        Command::new("id")
            .arg("-u")
            .output()
            .expect("Failed to execute command")
    };

    if cfg!(target_os = "windows") {
        output.status.success()
    } else {
        let uid = String::from_utf8_lossy(&output.stdout);
        uid.trim() == "0"
    }
}

fn shuffle_data(dir: &str) -> Result<(), Error> {
    // Check if data directory exists
    if !Path::new(dir).exists() {
        return Err(Error::new(ErrorKind::Other, format!("Data directory {:?} does not exist", dir)));
    }

    // Generate a random unix time within the last 10 days
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let ten_days = 10 * 24 * 60 * 60;
    let random_time = now - rand::thread_rng().gen_range(0..ten_days);

    // Set the system time to the generated unix time
    if is_admin() {
        change_time(random_time);
    }

    // For each directory in data directory
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Consolidate files
            println!("Consolidating files in {}...", path.to_str().unwrap());
            consolidate(path.to_str().unwrap())?;

            // Anonymize data
            anonymize_data(path.to_str().unwrap())?;
        }
    }

    // Restore the system time to the current time
    if is_admin() {
        resync_time();
    }

    Ok(())
}


fn change_time(timestamp: u64) {

    if cfg!(target_os = "windows") {
        Command::new("powershell")
            .arg("-Command")
            .arg("Set-Date")
            .arg(format!("(Get-Date 01.01.1970).AddSeconds({})", timestamp))
            .output()
            .expect("Failed to execute command");
    } else {
        let output = Command::new("date")
            .arg(format!("-s @{}", timestamp))
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
                println!("Moving {:?} out of {}...", file_name, path.to_str().unwrap());
                let dest_path = PathBuf::from(dir).join(file_name);
                fs::rename(file_path, dest_path)?;
            }
            // Delete the subdirectory
            println!("Deleting {}...", path.to_str().unwrap());
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
        println!("Anonymizing {}...", path.to_str().unwrap());

        let path_extension = path.extension().unwrap().to_str().unwrap();
        if path.is_dir() {
            continue;
        } else if path_extension != "csv" | path_extension != "txt" {
            // Rename without changing the file extension
            let new_name = format!("{}/{}.{}", dir, numbers.pop().unwrap(), path_extension);
            continue;
        }
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
    fn test_schedule() {
        schedule("Sun", "00:00");
    }

    #[test]
    fn test_cancel() {
        cancel();
    }

    #[test]
    fn test_is_valid_time() {
        let result = is_valid_time("00:00");
        assert_eq!(result, true);
    }

    #[test]
    fn test_is_invalid_time() {
        let result = is_valid_time("24:00");
        assert_eq!(result, false);
    }

    #[test]
    fn test_is_admin() {
        let result = is_admin();
        assert_eq!(result, true);
    }

    #[test]
    fn test_is_not_admin() {
        let result = is_admin();
        assert_eq!(result, false);
    }
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
