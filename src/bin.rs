use clap::Parser;
use vol_path::render;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  /// Name of the person to greet
  #[arg(short, long)]
  out_path: String,
}


fn main() {
  let args = Args::parse();
  println!("Rendering image, saving to {}...", args.out_path);

  render(&args.out_path);
}
