#[derive(Debug, Default)]
pub struct RawFileLoaderSource;

impl amethyst::assets::Source for RawFileLoaderSource {
    fn modified(&self, _path: &str) -> Result<u64, amethyst::Error> {
        Ok(0)
    }
    fn load(&self, path: &str) -> Result<Vec<u8>, amethyst::Error> {
        use crate::initialize_paths;
        use std::{fs::File, io::Read};

        let (_, _, asset_dir) = initialize_paths().expect("Could not initialize paths");
        let path = asset_dir.join(path);

        let content = {
            let mut file = File::open(path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            buffer
        };

        Ok(content)
    }
}
