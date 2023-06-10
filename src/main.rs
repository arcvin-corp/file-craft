use colored::Colorize;
use fake::faker::company::en::CompanyName;
use fake::faker::filesystem::en::FileName;
use fake::faker::lorem::en::Paragraph;
use fake::Fake;
use rand::Rng;
use std::env;
use std::fs;
use std::io::stdout;
use std::io::Result;
use std::io::Write;
use std::path::PathBuf;

fn get_working_directory() -> Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();
    Ok(dir)
}

fn path_to_string(path: &PathBuf) -> String {
    return path.clone().into_os_string().into_string().unwrap();
}

fn generate_file_size() -> usize {
    // Generate a random file size between 2KB and 16KB
    let mut rng = rand::thread_rng();
    let file_size = rng.gen_range(2048..=16384);
    file_size
}

fn create_file(
    file_path: &str,
    file_size: usize,
    files_count: &mut u64,
    bytes_written: &mut usize,
    disk_size: &usize,
) -> usize {
    let paragraph_range = file_size / (Paragraph(3..5).fake::<String>().into_bytes().len());

    let paragraphs: Vec<String> = (0..paragraph_range)
        .map(|_| Paragraph(3..5).fake())
        .collect();
    let file_content = paragraphs.join("\n");

    fs::write(file_path, file_content).expect("Failed to create file");

    let new_file_size: u64 = fs::metadata(&file_path)
        .expect("Error in file metadata")
        .len();

    *files_count += 1;

    *bytes_written += new_file_size as usize;

    print!(
        "\r{} [ {} {} ] | [ {}({}) {} {} ]",
        "Stats for this run:".bold().yellow(),
        "Files created:".bold().blue(),
        *files_count,
        "Disk Size".bold().blue(),
        *disk_size,
        "Bytes Written:".bold().blue(),
        *bytes_written
    );
    stdout().flush().unwrap();

    new_file_size as usize
}

fn calculate_num_files(num_folders: usize, disk_size: &usize) -> (usize, usize) {
    // Calculate the number of files based on the number of folders and disk size
    let avg_folder_size = *disk_size / num_folders;
    let max_file_size = avg_folder_size / 16384;
    let min_file_size = avg_folder_size / 2048;
    let num_files = avg_folder_size / ((max_file_size + min_file_size) / 2);
    (avg_folder_size, num_files)
}

fn create_files_in_folders(
    root_folder: &str,
    num_folders: usize,
    disk_size: &usize,
    files_count: &mut u64,
) {
    // Create the calculated number of files in each folder
    let (avg_folder_size, num_files_per_folder) = calculate_num_files(num_folders, &disk_size);
    let mut complete_folder_size = 0;

    let data_dir = get_working_directory()
        .expect("Folder not found")
        .join(&root_folder);

    println!(
        "\nData will be generated in the following folder: [ {} ] \n{}\n",
        path_to_string(&data_dir).bold().green(),
        "=".repeat(125)
    );

    let mut bytes_written: usize = 0;

    for _ in 0..num_folders {
        if complete_folder_size > *disk_size {
            // println!("Breaking out of folder creation loop {} | {}", complete_folder_size, disk_size);
            break;
        }

        let mut current_folder_size: usize = 0;

        // let folder_name = format!("Folder{}", folder_index);
        let folder_name: String = String::from(CompanyName().fake::<String>()).replace(" ", "_");
        let folder_path = format!("{}/{}", root_folder, folder_name);
        fs::create_dir_all(&folder_path).expect("Failed to create folder");

        for file_index in 0..num_files_per_folder {
            if current_folder_size >= avg_folder_size {
                // println!("Breaking out of file creation loop {} | {}", current_folder_size, avg_folder_size);
                break;
            }

            let file_name: String = format!("{}-{}", file_index, FileName().fake::<String>());
            let file_path = format!("{}/{}", folder_path, file_name);

            let file_size = generate_file_size();
            let new_file_size = create_file(
                &file_path,
                file_size,
                files_count,
                &mut bytes_written,
                &disk_size,
            );
            current_folder_size += new_file_size;
        }
        complete_folder_size += current_folder_size;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!(
            "{} {} {} {} {} \n\n{}",
            "Usage:".bold(),
            "./file-craft".blue().bold(),
            "<folder_count>".green().italic(),
            "<disk_size_bytes>".green().italic(),
            "<root_folder_name>".green().italic(),
            "Example: ./file-generator 100 204800 data_repo"
        );
        return;
    }

    let num_folders: usize = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid number of folders");
            return;
        }
    };

    let disk_size: usize = match args[2].parse() {
        Ok(size) => size,
        Err(_) => {
            println!("Invalid disk size");
            return;
        }
    };

    let root_folder: String = match args.get(3) {
        Some(name) => name.to_owned(),
        None => {
            println!("Error: Invalid root folder");
            return;
        }
    };

    let mut files_count: u64 = 0;

    create_files_in_folders(&root_folder, num_folders, &disk_size, &mut files_count);
}
