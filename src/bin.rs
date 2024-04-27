use std::path::PathBuf;

use clap::Parser;
use image::io::Reader as ImageReader;
use seam_carving::SeamCarver;

/// Seam carving
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path to image
    path: String,
        
    /// Where to save the output image
    #[arg(short, long)]
    output: Option<String>,

    /// The ratio of output/input width
    #[arg(short, long, default_value_t = 0.9)]
    width_ratio: f32,

    /// The ratio of output/input height
    #[arg(short='l', long, default_value_t = 0.9)]
    height_ratio: f32,

}

fn main() {
    let args = Args::parse();
    let src_path = &args.path;
    let img = ImageReader::open(src_path).unwrap().decode().unwrap();
    let width = img.width() as usize;
    let height = img.height() as usize;
    let new_width = (width as f32 * args.width_ratio) as usize;
    let new_height = (height as f32 * args.height_ratio) as usize;

    let new_img = SeamCarver::new(img, new_width, new_height).apply();
    let fname = match args.output{
        Some(out) => out,
        None => {
            let path_buf = PathBuf::from(args.path);
            let dir = path_buf.parent().unwrap().to_str().unwrap();
            let fname = path_buf.file_stem().unwrap().to_str().unwrap();
            format!("{dir}/{fname}_seamed.png")
        }
    };
    new_img.save(fname).unwrap();

}