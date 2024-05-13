use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
 
pub fn generate_and_compile_rcfile(include_paths:Vec<PathBuf>) {
    let includeargs= include_paths.iter().map(|include_path| {
        format!(
            "/I {}",
            include_path
                .to_str()
                .expect("Non Unicode paths are not supported")
        )
    });

    let (companyname, copyright, productname) = get_packagemetadatadetails();
    let (productversion, description, fileversion) = get_packagedetails();
    getandset_rcfile(companyname, copyright, productname, productversion ,description, fileversion, includeargs);
}
fn getandset_rcfile(s1: String, s2: String, s3: String, s4:String, s5:String, s6:String, s7:impl Iterator<Item = String>) {
    println!("Set and create rc file... ");
    let rcfile_path = "resources.rc";
    if fs::metadata(&rcfile_path).is_ok() {
        // File exists, so let's remove it
        if let Err(err) = fs::remove_file(&rcfile_path) {
            eprintln!("Error deleting file: {}", err);
        } else {
            println!("File deleted successfully!");
        }
    } else {
        println!("File does not exist.");
    }

    let ver_filetype = "VFT_DRV";
    let ver_filesubtype = "VFT2_DRV_SYSTEM";
    let ver_internalname = "SurfaceButton.sys";
    let ver_originalfilename = "VER_INTERNALNAME_STR";

    // Create the RC file content
    let rc_content = format!(
        r#"#include <windows.h>
#include <ntverp.h>
#define	VER_FILETYPE	            {file_type}
#define	VER_FILESUBTYPE	            {file_subtype}
#define VER_INTERNALNAME_STR        {internal_name}
#define VER_ORIGINALFILENAME_STR    {original_filename}

#undef VER_FILEDESCRIPTION_STR     
#define VER_FILEDESCRIPTION_STR "{s5}"

#undef  VER_PRODUCTNAME_STR
#define VER_PRODUCTNAME_STR    VER_FILEDESCRIPTION_STR

#define VER_FILEVERSION        {s6},0
#define VER_FILEVERSION_STR    "{s4}.0"

#undef  VER_PRODUCTVERSION
#define VER_PRODUCTVERSION          VER_FILEVERSION

#undef  VER_PRODUCTVERSION_STR
#define VER_PRODUCTVERSION_STR      VER_FILEVERSION_STR

#define VER_LEGALCOPYRIGHT_STR      "{s2}"
#ifdef  VER_COMPANYNAME_STR

#undef  VER_COMPANYNAME_STR
#define VER_COMPANYNAME_STR         "{s1}"
#endif

#undef  VER_PRODUCTNAME_STR
#define VER_PRODUCTNAME_STR    "{s3}"

#include "common.ver""#,
        file_type = ver_filetype,
        file_subtype = ver_filesubtype,
        internal_name = ver_internalname,
        original_filename = ver_originalfilename
    );

    // Print the RC file content
    //println!("{}", env!("CARGO_PKG_VERSION"));
    //println!("{}", env!("CARGO_PKG_METADATA.WDK"));
   
    std::fs::write("resources.rc", rc_content).expect("Unable to write RC file");
    invoke_rc(s7);
}
fn invoke_rc(s7:impl Iterator<Item = String>) {
    // Replace with the actual path to rc.exe
    let rc_path = env::var("PATH_TO_RC").unwrap_or_else(|_| {
        // Default path if environment variable is not set
        r#"D:\EWDK\content\Program Files\Windows Kits\10\bin\10.0.22621.0\x86\rc.exe"#.to_string()
    });

    println!("Using rc.exe path: {}", rc_path);

    // Replace "resource.rc" with the name of your resource script file
    let resource_script = "resources.rc";

    let status = Command::new(&rc_path).args(s7).arg(resource_script).status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                println!("Resource compilation successful!");
            } else {
                println!("Resource compilation failed.");
                std::process::exit(1); 
            }
        }
        Err(err) => {
            eprintln!("Error running rc.exe: {}", err);
            std::process::exit(1);
        }
    }
}
fn get_packagemetadatadetails() -> (String, String, String) {
    // Run the 'cargo metadata' command and capture its output
    println!("get rc file metadata... ");
    let output = Command::new("cargo")
        .arg("metadata")
        .output()
        .expect("Failed to execute 'cargo metadata'");

    // Parse the JSON output
    let metadata_str = String::from_utf8_lossy(&output.stdout);
    let metadata: serde_json::Value =
        serde_json::from_str(&metadata_str).expect("Failed to parse JSON");
    // Extract the values
    let mut companyname = String::new();
    let mut copyrightname = String::new();
    let mut productname = String::new();

    // Extract the version
    if let Some(package) = metadata["packages"].get(1) {
        if let Some(company) = package["metadata"]["wdk"]["companyname"].as_str() {
            companyname = company.to_string();
        } else {
            println!("CompanyName not found in metadata.");
        }
        if let Some(copyright) = package["metadata"]["wdk"]["copyright"].as_str() {
            copyrightname = copyright.to_string();
        } else {
            println!("Copyright not found in metadata.");
        }
        if let Some(product) = package["metadata"]["wdk"]["productname"].as_str() {
            productname = product.to_string();
        } else {
            println!("ProductName not found in metadata.");
        }
    } else {
        println!("No packages found in metadata.");
    }
    (companyname, copyrightname, productname)
}

fn get_packagedetails() -> (String, String, String) {
    let mut fileversion = String::new();
    let mut description = String::new();
    let mut productversion = String::new();

    match fs::read_to_string("Cargo.toml") {
        Ok(text1) => {
            for line in text1.lines() {
                if line.starts_with("version") {
                    let start = line.find('"').unwrap_or(0) + 1;
                    let end = line.rfind('"').unwrap_or(0);
                    productversion = line[start..end].to_string();
                    let versionparts: Vec<&str> = productversion.split('.').collect();
                    fileversion = versionparts.join(",");
                }
                if line.starts_with("description") {
                    let start = line.find('"').unwrap_or(0) + 1;
                    let end = line.rfind('"').unwrap_or(0);
                    description = line[start..end].to_string();
                }
            }
        }
        Err(_) => {
            eprintln!("Error reading Cargo.toml");
        }
    }
    (productversion, description, fileversion)
}
