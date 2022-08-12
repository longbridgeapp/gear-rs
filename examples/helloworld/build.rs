use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    gear_codegen::build()
}
