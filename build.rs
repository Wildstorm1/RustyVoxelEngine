/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use copy_to_output::copy_to_output;
use std::process::Command;
use std::env;

fn main() {
    println!("cargo:warning=Running shader compile step...");

    let dir = env::current_dir().unwrap();
    let shaders_compiler = dir.join("shaders.bat");

    Command::new("cmd")
        .args(["/c", shaders_compiler.to_str().unwrap()])
        .status()
        .unwrap();

    let profile = env::var("PROFILE").unwrap();
    copy_to_output("./shaders", &profile).expect("Could not copy files to target");
}