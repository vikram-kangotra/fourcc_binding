use std::{fs::File, path::{Path, PathBuf}};
use std::io::{Result, BufWriter, Write};

fn main() {

    println!("cargo:rerun-if-changed=wrapper.h");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let header_file = Path::new("./wrapper.h");
    let temp_header = out_path.join("vlc_fourcc.h");
    
    // Preprocess the header file to change macro definitions to variable assignments
    preprocess_header(header_file, &temp_header).expect("cannot preprocess header");

    // Ask bindgen to Preprocess the temporary header file
    // and dump the preprocessed input to __bindgen.i
    bindgen::Builder::default()
        .header(temp_header.to_str().unwrap())
        .dump_preprocessed_input()
        .expect("Unable to generate bindings");

    // Generate fourcc.rs from __bindgen.i
    generate_fourcc(Path::new("./__bindgen.i"), out_path.join("fourcc.rs")).expect("cannot generate fourcc.rs");
}

// Changes macro Definitions to variable assignments
// to force bindgen to process the macro definitions
//
// Example:
// #define VLC_CODEC_4XM  VLC_FOURCC('4','X','M',' ')
// to
// VLC_CODEC_4XM = VLC_FOURCC('4','X','M',' ')
//
fn preprocess_header(from: &Path, to: &PathBuf) -> Result<()> {
    let mut output_file = BufWriter::new(File::create(&to).expect("cannot open vlc_fourcc.h"));
    let header_contents = std::fs::read_to_string(from).unwrap();

    for line in header_contents.lines() {
        if line.starts_with("#define VLC_CODEC") {
            let (name, fourcc) = line
                .strip_prefix("#define ")
                .unwrap()
                .split_once(" ")
                .unwrap();

            writeln!(
                output_file,
                "{} = {}",
                name, fourcc
            )?;
        } else {
            writeln!(output_file, "{}", line)?;
        }
    }

    Ok(())
}

fn generate_fourcc(from: &Path, to: PathBuf) -> Result<()> {
    let mut output_file = BufWriter::new(File::create(to).expect("cannot open vlc_fourcc.h"));
    let fourcc_include_contents = std::fs::read_to_string(from).unwrap();

    writeln!(output_file, "{}", "#[macro_export] macro_rules! fourcc_consts { () => { ")?;

    for line in fourcc_include_contents.lines() {
        if line.starts_with("VLC_CODEC") {

            let (name, fourcc) = line.split_once(" = ").unwrap();

            let fourcc = if fourcc.starts_with("VLC_FOURCC") {
                // ('u','n','d','f')
                let fourcc = fourcc.strip_prefix("VLC_FOURCC").unwrap();
                format!("fourcc!{}", fourcc)
            } else {
                format!("Self::{}", fourcc)
            };

            writeln!(
                output_file,
                "    pub const {}: FourCC = {};",
                name, fourcc
            )
            .unwrap();
        }
    }
    
    writeln!(output_file, "{}", "} }")?;

    Ok(())
}
