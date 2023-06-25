// lets use a hashmap here as we can have O(1) lookup time for images
use std::collections::HashMap;
use std::fs;

pub struct LocalImages {
    images: HashMap<String, String>,
}

impl LocalImages {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    pub fn load_images(&mut self) {
        // check if images folder exists
        if !std::path::Path::new("./images").exists() {
            return;
        }

        let paths = fs::read_dir("./images").unwrap();

        let mut num_images = 0;

        for path in paths {
            let path = path.unwrap().path();
            let mut file_name = path
                .file_name()
                .unwrap()
                .to_ascii_uppercase()
                .to_str()
                .unwrap()
                .to_string();

            // remove the file extension
            file_name = file_name.replace(".PNG", "");
            file_name = file_name.replace(".JPG", "");
            file_name = file_name.replace(".JPEG", "");

            let file_path = path.to_str().unwrap().to_string();
            println!("Loaded image: {} from {}", file_name, file_path);

            self.images.insert(file_name, file_path);
            num_images += 1;
        }

        println!("Loaded {} local images", num_images);
    }

    pub fn get_image(&self, image_name: &String) -> Option<String> {
        self.images.get(image_name).cloned()
    }
}
