use std::env::args;

mod args;
mod e2e_yaml;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = args::parse_args(args().collect())?;
    let e2e_yaml = e2e_yaml::load_e2e_yaml_from_file(args.e2e_yaml())?;
    dbg!(e2e_yaml);

    Ok(())
}
