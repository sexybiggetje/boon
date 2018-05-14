extern crate zip;

use APP_INFO;
use app_dirs::*;

use ::{Platform, Bitness};
use download::LoveVersion;

use std::io::prelude::*;
use std::iter::Iterator;
use std::io::{Write, Seek};

use std::path::Path;
use std::fs::File;

use zip::result::ZipError;
use zip::write::FileOptions;
use std::process::Command;
use walkdir::{WalkDir, DirEntry};

const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);

// @TODO: Return an Option instead
pub fn get_love_filename<'a>(version: &LoveVersion, platform: &Platform, bitness: &Bitness) -> &'a str {
    match (version, platform, bitness) {
        (&LoveVersion::V11_1,   &Platform::Windows, &Bitness::X64) => "love-11.1.0-win64",
        (&LoveVersion::V11_1,   &Platform::Windows, &Bitness::X86) => "love-11.1.0-win32",
        (&LoveVersion::V11_1,   &Platform::MacOs,   &Bitness::X64) => "love-11.1.0-macos",
        (&LoveVersion::V0_10_2, &Platform::Windows, &Bitness::X64) => "love-0.10.2-win64",
        (&LoveVersion::V0_10_2, &Platform::Windows, &Bitness::X86) => "love-0.10.2-win32",
        (&LoveVersion::V0_10_2, &Platform::MacOs,   &Bitness::X64) => "love-0.10.2-macos",
        _ => ""
    }
}

pub fn get_output_filename<'a>(name: String, platform: &Platform, bitness: &Bitness) -> String {
    match (platform, bitness) {
        (&Platform::Windows, &Bitness::X64) => format!("{}-win64.exe", name),
        (&Platform::Windows, &Bitness::X86) => format!("{}-win32.exe", name),
        (&Platform::MacOs,   &Bitness::X64) => format!("{}-macos.app", name),
        _ => {
            panic!("Unsupported platform {:?}-{:?}");
        }
    }
}

pub fn build_love(directory: String) {
    let method = METHOD_DEFLATED;

    let src_dir = &directory;
    let dst_file: &str = "test.love";

    match zip_directory(src_dir, dst_file, method.unwrap()) {
        Ok(_) => {
            println!("done: {} written to {}", src_dir, dst_file);
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

pub fn build_windows(directory: String, version: &LoveVersion, bitness: &Bitness) {
    build_love(directory);

    let filename = get_love_filename(version, &Platform::Windows, bitness);
    let path = &format!("/home/camchenry/.local/share/love-kit/{}/love.exe", filename);
    let love_exe_path = Path::new(path);
    let output_file_name = get_output_filename(String::from("game"), &Platform::Windows, bitness);
    let output_path = Path::new(output_file_name.as_str());

    println!("Copying love from {}", love_exe_path.display());

    let output = if cfg!(target_os = "windows") {
        let result: &str = &format!("{}+{}", &love_exe_path.to_str().unwrap(), "test.love");
        println!("Building for windows.. {}", result);
        Command::new("cmd")
                .args(&["copy", "/b", result, "game-win64.exe"])
                .output()
                .expect("failed to execute process")
    } else {
        Command::new("cat")
            .args(&[love_exe_path.to_str().unwrap(), "test.love"])
            .output()
            .expect("failed to execute process")
    };

    let mut file = match File::create(&output_path) {
        Ok(file) => file,
        Err(why) => {
            panic!("Unable to create file `{}`: {}", output_path.display(), why);
        }
    };

    match file.write_all(&output.stdout) {
        Ok(_) => {},
        Err(why) => {
            panic!("{}", why);
        }
    }
}

fn zip_dir<T>(it: &mut Iterator<Item=DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod)
              -> zip::result::ZipResult<()>
    where T: Write+Seek
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o644);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix))
            .unwrap()
            .to_str()
            .unwrap();

        if path.is_file() {
            println!("adding {:?} as {:?} ...", path, name);
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        }
    }
    zip.finish()?;
    Result::Ok(())
}

fn zip_directory(src_dir: &str, dst_file: &str, method: zip::CompressionMethod) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(src_dir.to_string());
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}