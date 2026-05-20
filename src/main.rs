// Merge pdfs using command line arguments

use std::process::Command;
use std::fs;
use std::path::Path;
use inquire::{Text, Autocomplete, validator::Validation, CustomUserError};

#[derive(Clone, Default)]
struct LivePathCompleter;

impl Autocomplete for LivePathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let input_path = Path::new(input);

        let (dir_scan, filename_prefix) = if input.ends_with('/') || input.is_empty() {
            (input_path, "")
        } else {
            (
                input_path.parent().unwrap_or_else(|| Path::new(".")),
                input_path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            )
        };

        // Fallback to current dir if the string is empty. Therefore, look in the current directory.
        let mut search_dir = dir_scan;
        if search_dir.as_os_str().is_empty() {
            search_dir = Path::new(".");
        }

        // If the folder path is not found, return an Error. A list of empty suggestions will be returned 
        let entries = match fs::read_dir(search_dir) {
            Ok(e) => e,
            Err(_) => return Ok(vec![]),
        };

        let mut suggestions = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let mut path_str = path.to_string_lossy().into_owned();
            let name_str = entry.file_name().to_string_lossy().into_owned();

            if !name_str.starts_with(filename_prefix) {
                continue;
            }

            if path_str.starts_with("./") && !input.starts_with("./") {
                path_str = path_str.replacen("./", "", 1);
            }

            if path.is_dir() && !path_str.ends_with("/") {
                path_str.push_str("/");
            }

            suggestions.push(path_str);
        }
        suggestions.sort();
        Ok(suggestions)
    }

    fn get_completion(&mut self, 
        _input: &str,
        highlighted_suggestion: Option<String>
        ) -> Result<inquire::autocompletion::Replacement, CustomUserError> {
        match highlighted_suggestion {
            Some(suggestion) => Ok(inquire::autocompletion::Replacement::Some(suggestion)),
            None => Ok(inquire::autocompletion::Replacement::None),
        }
    }
} 

fn main() {
    
    println!("\n-------------------------");
    println!("----- Merge PDF App -----");
    println!("-------------------------\n");
    println!("Select your PDFs. Type 'c' and hit 'Enter' to confirm when you are done collecting.\n");
    let mut file_paths: Vec<String> = Vec::new();
    let mut file_counter = 1;
    loop {
        let prompt_msg = format!("Select PDF file #{}: ", file_counter);
        let path_input = Text::new(&prompt_msg)
            .with_placeholder("Type file path or press 'c' to confirm list ...")
            .with_autocomplete(LivePathCompleter)
            .with_validator(|input: &str| {
                let trimmed = input.trim();
                
                // Checking if the string is empty
                if trimmed.is_empty() {
                    return Ok(Validation::Invalid("Filepath cannot be an empty string!".into()))
                }

                if trimmed.eq_ignore_ascii_case("c") {
                    return Ok(Validation::Valid)
                }

                // Checking if the file name ends with .pdf
                if !trimmed.ends_with(".pdf") {
                    return Ok(Validation::Invalid("Filename must end with '.pdf'".into()))
                }

                if !Path::new(trimmed).exists() {
                    return Ok(Validation::Invalid("This file does not exist in your system.".into()))
                }

                Ok(Validation::Valid)
            })
            .prompt();

        match path_input {
            Ok(confirmed_path) => {
                let trimmed = confirmed_path.trim();

                if trimmed.eq_ignore_ascii_case("c") {
                    if file_paths.len() >= 2 {
                        println!("->  Files confirmed! Moving to merge step.\n");
                        break;
                    } else {
                        println!("WARNING: You need to select atleast 2 files before the merge!\n");
                        continue;
                    }
                }

                file_paths.push(trimmed.to_string());
                println!("Staged files: {} \n", trimmed);

                file_counter += 1;
            }

            Err(_) => {
                println!("Prompt cancelled. Exiting ...");
                return;
            }
        }
    }

    /* 
    Get name of the output file from the user prompt
    */
    let mut final_output_name = String::new();
    let merge_name = Text::new("Enter the output PDF name:")
        .with_default("merged.pdf")
        .with_placeholder("merged.pdf")
        .with_validator(|input: &str| {
            let trimmed = input.trim();

            // Checking if filename is empty
            if trimmed.is_empty() {
                return Ok(Validation::Invalid("Filename cannot be empty.".into()))
            } 

            // Check the file extension
            if !trimmed.to_lowercase().ends_with(".pdf") {
                return Ok(Validation::Invalid("Filename must end with '.pdf'".into()))
            }

            Ok(Validation::Valid)
        })
        .prompt();
        
    match merge_name {
        Ok(name) => {
            println!("Output filename is set to: {}\n", name);
            final_output_name = name;
        },
        Err(_) => println!("Prompt cancelled.\n"),
    }

    let merge_prompt = Text::new("Type 'm' and press 'Enter' to perform the merge:")
        .with_placeholder("m")
        .prompt();

    match merge_prompt {
        Ok(input) if input.trim().eq_ignore_ascii_case("m") => {
            println!("Executing PDF merge ...");

            let merge_cmd = Command::new("gs")
                .args(["-dBATCH", "-dNOPAUSE", "-q", "-sDEVICE=pdfwrite"])
                .arg(format!("-sOutputFile={}", final_output_name))
                .args(&file_paths)
                .output()
                .expect("ERROR: Failed to execute");

            // Command to format output from the executed command
            let _stdout = String::from_utf8_lossy(&merge_cmd.stdout);
            let stderr = String::from_utf8_lossy(&merge_cmd.stderr);
            
            if merge_cmd.status.success() {
                println!("Created the merged PDF : {}", final_output_name);
            } else {
                println!("ERROR: {}", stderr);
            }
        }

        Err(_) => {
            println!("Merge cancelled. Abort!\n");
        }
     
        _ => {
            println!("Invalid input. Merge aborted.\n");
        }
    }
}
    /* To-Do:
- Get home directory
- Compile the package for apt repositories
- MIT licence
- Create screen recording (mock pdfs)
- README instructions
- Have systemwise access using the terminal CLI
 */
