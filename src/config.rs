// this file deals with configuration

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub webhook_url: String,
    pub ps3_ip: String,
    pub idle_time: u64,
    pub refresh_time: u64,
    pub show_temp: bool,
}

impl Config {
    pub fn new() -> Self {
        let mut config = Config {
            webhook_url: "".to_string(),
            ps3_ip: "".to_string(),
            idle_time: 30,
            refresh_time: 15,
            show_temp: false,
        };

        config.load_config();

        config
    }

    fn load_config(&mut self) {
        // check if config file exists
        if !std::path::Path::new("./config.json").exists() {
            println!("config.json not found");
            std::fs::File::create("./config.json").unwrap();
            let json = serde_json::to_string_pretty(&self).unwrap();
            std::fs::write("./config.json", json).unwrap();
            return;
        } else {
            let file = std::fs::File::open("./config.json").unwrap();
            let reader = std::io::BufReader::new(file);
            let config: Config = serde_json::from_reader(reader).unwrap();

            self.webhook_url = config.webhook_url;
            self.ps3_ip = config.ps3_ip;
            self.idle_time = config.idle_time;
            self.refresh_time = config.refresh_time;
            self.show_temp = config.show_temp;
        }
    }
}
