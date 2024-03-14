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
use std::io::{Read, Write, Error, ErrorKind};
use console::{Key, Term};
use rand::{seq::SliceRandom, Rng, prelude::ThreadRng};
use std::result::Result;
use std::time::{SystemTime, UNIX_EPOCH, Instant, Duration};
use std::process::{Command, Output};
use std::thread::sleep;
use tempfile::NamedTempFile;

fn main() {
    // Return error message if the program is run on macOS
    #[cfg(target_os = "macos")] {
        println!("This program is not currently supported on macOS.");
        return;
    }
    
    // Get the command line arguments
    let args: Vec<String> = env::args().collect();
    let data_dir: &str = "data";
    let term: Term = Term::stdout();

    if args.len() == 1 {
        if !is_admin() {
            #[cfg(target_os = "windows")] {
                println!("Admin privileges not detected.");
            }
            #[cfg(target_os = "linux")] {
                println!("Root privileges not detected.");
            }
            
            println!("File creation time will default to the current time.");
            println!("Would you like to continue? (Y/n)");

            let mut input: char = ' ';
            let start_time: Instant = Instant::now();
            let max_time: Duration = Duration::from_secs(10);
            loop {
                if start_time.elapsed() > max_time {
                    input = 'y';
                    break;
                }

                if let Ok(key) = term.read_key() {
                    input = match key {
                        Key::Char('y') | Key::Char('Y') | Key::Enter => 'y',
                        Key::Char('n') | Key::Char('N') => 'n',
                        _ => input,
                    };
                }

                if input == 'n' || input == 'y' {
                    break;
                }
                sleep(Duration::from_millis(5));
            }
            if input == 'n' {
                println!("Exiting...");
                return;
            }

            println!("Continuing...");
        }
        // Shuffle the data once
        shuffle_data(data_dir).expect("Failed to shuffle data");
    } else {
        // Parse the command line arguments
        match args[1].as_str() {
            "--no-warning" => {
                // Shuffle the data once
                shuffle_data(data_dir).expect("Failed to shuffle data");
            }
            "-l" | "--loop" => {
                // Run shuffle_data() every 30 seconds
                // Repeat the number of times specified by the user or indefinitely if the user does not provide a number
                let mut count: i32 = 100;
                if args.len() > 2 {
                    count = args[2].parse().expect("Failed to parse count");
                }
                while count > 0 {
                    count -= 1;
                    shuffle_data("data").expect("Failed to shuffle data");
                    if count > 0 {
                        println!("Shuffling data in 30 seconds...");
                        sleep(Duration::from_secs(30));
                    }
                }
            }
            "-s" | "--schedule" => {
                // Schedule the data shuffle to run weekly with the system's scheduler
                // Schedule the program to run weekly on Sunday at 12:00 AM if the user does not provide a time or day
                let mut time: String = "00:00".to_string();
                let mut day: String = "Sun".to_string();
                if args.len() > 2 {
                    for mut i in 2..(args.len() - 1) {
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
                println!("Scheduling the data shuffle to run every {} at {}...", day, time);
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
                println!("  -l, --loop [<COUNT>]               Shuffle the data every 30 seconds and repeat the specified number");
                println!("                                     of times or until Esc is pressed");
                println!("  -s, --schedule [<DAY>] [at <TIME>]");
                println!("                                     Schedule the data shuffle to run every week with the system's scheduler");
                println!("                                     Defaults to Sunday at 12:00 AM if no time or day is provided");
                println!("  -c, --cancel                       Cancel the scheduled data shuffle");
                println!("  --no-warning                       Shuffle the data once without warning message if not run as admin or root");
                println!("  -h, --help                         Print the help message");
            }
            _ => {
                println!("Invalid option");
            }
        }
    }
}

fn schedule(day: &str, time: &str) {
    let cwd: String = env::current_dir().expect("Failed to get current directory").to_str().expect("Failed to convert path to string").to_string();
    let exe: String = env::current_exe().expect("Failed to get current executable").to_str().expect("Failed to convert path to string").to_string();
    let command: String = if cfg!(target_os = "windows") {
        format!("schtasks /create /tn DataShuffler /tr \"cmd /c cd /d {} && {} --no-warning\" /sc weekly /d {} /st {}", cwd, exe, day, time)
    } else {
        let hrs: &str = time.split(":").collect::<Vec<&str>>()[0];
        let mins: &str = time.split(":").collect::<Vec<&str>>()[1];
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
    let exe: String = env::current_exe().expect("Failed to get current executable").to_str().expect("Failed to convert path to string").to_string();
    let command: String = if cfg!(target_os = "windows") {
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
    let time: Vec<&str> = time.split(":").collect::<Vec<&str>>();
    if time.len() != 2 {
        return false;
    }
    let hour: u32 = time[0].parse::<u32>().unwrap();
    let minute: u32 = time[1].parse::<u32>().unwrap();
    if hour > 23 || minute > 59 {
        return false;
    }
    true
}

fn is_admin() -> bool {
    let output: Output = if cfg!(target_os = "windows") {
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
        let uid: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stdout);
        uid.trim() == "0"
    }
}

fn shuffle_data(dir: &str) -> Result<(), Error> {
    // Check if data directory exists
    if !Path::new(dir).exists() {
        return Err(Error::new(ErrorKind::Other, format!("Data directory {:?} does not exist", dir)));
    }

    // Generate a random unix time within the last 10 days
    let now: u64 = SystemTime::now().duration_since(UNIX_EPOCH).expect("Failed to get system time").as_secs();
    let ten_days: u64 = 10 * 24 * 60 * 60;
    let random_time: u64 = now - rand::thread_rng().gen_range(0..ten_days);

    // Set the system time to the generated unix time
    if is_admin() {
        change_time(random_time);
    }

    // For each directory in data directory
    for entry in fs::read_dir(dir)? {
        let entry: fs::DirEntry = entry?;
        let path: PathBuf = entry.path();
        if path.is_dir() {
            // Consolidate files
            println!("Consolidating files in {}...", path.to_str().expect("Failed to convert path to str"));
            consolidate(path.to_str().expect("Failed to convert path to str"))?;

            // Anonymize data
            anonymize_data(path.to_str().expect("Failed to convert path to str"))?;
        }
    }

    // Restore the system time to the current time
    if is_admin() {
        resync_time();
    }

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
        Command::new("date")
            .arg(format!("-s @{}", timestamp))
            .output()
            .expect("Failed to execute command");

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
        Command::new("sudo")
            .arg("ntpdate")
            .arg("pool.ntp.org")
            .output()
            .expect("Failed to execute command");
    }
}

fn consolidate(dir: &str) -> Result<(), Error> {
    let path: &Path = Path::new(dir);
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::Other, format!("Consolidation Error: {:?} is not a directory", dir)));
    }

    // For each subdirectory in the directory
    for entry in fs::read_dir(path)? {
        let entry: fs::DirEntry = entry?;
        let path: PathBuf = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Move all files to the parent directory
        for file in fs::read_dir(path.clone())? {
            let file: fs::DirEntry = file?;
            let file_path: PathBuf = file.path();
            let file_name: &std::ffi::OsStr = file_path.file_name().ok_or_else(|| Error::new(ErrorKind::Other, "No filename"))?;
            println!("Moving {:?} out of {}...", file_name, path.to_str().expect("Failed to convert path to str"));
            let dest_path: PathBuf = PathBuf::from(dir).join(file_name);
            fs::rename(file_path, dest_path)?;
        }
        // Delete the subdirectory
        println!("Deleting {}...", path.to_str().expect("Failed to convert path to str"));
        fs::remove_dir(path)?;
    }

    Ok(())
}

fn anonymize_data(dir: &str) -> Result<(), Error> {
    let paths: Vec<_> = fs::read_dir(dir)?.map(|res: Result<fs::DirEntry, Error>| res.map(|e: fs::DirEntry| e.path())).collect::<Result<Vec<_>, Error>>()?;
    let mut temp_paths: Vec<_> = Vec::new();
    let mut rng: ThreadRng = rand::thread_rng();

    // Generate a random number for each file
    let mut numbers: Vec<_> = (1..=paths.len()).collect();
    numbers.shuffle(&mut rng);

    // Rename each file to a random number with a .csv extension
    for path in paths {

        if path.is_dir() {
            continue;
        } 

        println!("Anonymizing {}...", path.to_str().expect("Failed to convert path to str"));

        let path_extension: &str = path.extension().expect("Failed to get file extension").to_str().expect("Failed to convert OsStr to str");
        let mut new_name: String = String::new();

        if path_extension == "txt" {
            // Rename with a .csv extension
            new_name = format!("{}/{}.csv", dir, numbers.pop().expect("Failed to pop number"));
        } else {
            // Rename without changing the file extension
            new_name = format!("{}/{}.{}", dir, numbers.pop().expect("Failed to pop number"), path_extension);
        };

        // Open the file
        let mut file: fs::File = fs::File::open(&path)?;
        
        // Read the file contents
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end (&mut contents)?;

        // Write the file contents to a temporary file
        let mut temp_file = NamedTempFile::new_in(dir)?;
        temp_file.write_all(&contents)?;

        // Push the temporary file and new name to the temp_paths vector
        temp_paths.push((temp_file, new_name));

        // Delete the original file
        fs::remove_file(&path)?;

    }


    for (temp_file, new_name) in temp_paths {
        temp_file.persist(new_name)?;
    }

    Ok(())
}