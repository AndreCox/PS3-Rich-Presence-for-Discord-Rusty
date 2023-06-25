use std::time::SystemTime;

use regex::Regex;
use select::{document::Document, node::Node, predicate::Name};

use crate::{artifact_upload::ArtifactUploader, config, local_images::LocalImages};

pub struct Ps3Scraper<'a> {
    pub ip: String,
    html: Document,
    pub temp: [u32; 2],
    pub title_id: String,
    pub name: String,
    pub start_time: i64,
    pub image: String,
    local_images: LocalImages,
    image_uploader: ArtifactUploader,
    config: &'a config::Config,
}

impl<'a> Ps3Scraper<'a> {
    pub fn new(
        ip: &String,
        local_images: LocalImages,
        image_uploader: ArtifactUploader,
        config: &'a config::Config,
    ) -> Self {
        Self {
            image_uploader,
            local_images,
            ip: ip.clone(),
            html: Document::from(""),
            temp: [0, 0],
            title_id: "".to_string(),
            name: "".to_string(),
            image: "xmb".to_string(),
            start_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            config,
        }
    }

    pub fn fetch_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("http://{}/cpursx.ps3?/sman.ps3", self.ip);

        // create a reqwest client with a 5 second timeout
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        let resp = client.get(url).send();
        let html = match resp {
            Ok(html) => html.text().unwrap(),
            Err(e) => return Err(Box::new(e)),
        };
        self.html = Document::from(html.as_str());

        if self.config.show_temp {
            if self.get_thermals().is_err() {
                return Err("Failed to get thermals".into());
            }
        }
        if self.decide_game_type().is_err() {
            return Err("Failed to decide game type".into());
        }

        Ok(())
    }

    pub fn get_thermals(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // search for a tag that has href="/cpursx.ps3?up"
        // first convert to a Document

        let mut up_tags: Vec<String> = Vec::new();

        self.html.find(Name("a")).for_each(|n| {
            // don't use unwrap() here, as it will panic if the attribute doesn't exist
            // instead, use if let to check if the attribute exists
            if let Some(href) = n.attr("href") {
                if href.contains("cpursx.ps3?up") {
                    up_tags.push(n.text());
                }
            }
        });

        let temp_raw = up_tags[0].clone();
        let cpu = Regex::new(r"CPU: (.+?)°C").unwrap();
        let rsx = Regex::new(r"RSX: (.+?)°C").unwrap();

        let binding = cpu.captures(&temp_raw).unwrap();
        let cpu_temp = binding[1].trim();
        let binding = rsx.captures(&temp_raw).unwrap();
        let rsx_temp = binding[1].trim();

        self.temp = [cpu_temp.parse().unwrap(), rsx_temp.parse().unwrap()];

        Ok(())
    }

    pub fn decide_game_type(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut target_blank_tags: Vec<String> = Vec::new();

        // PS3ISO, JB Folder Format, and PS3 PKG games will display this field in wman
        self.html.find(Name("a")).for_each(|n| {
            // don't use unwrap() here, as it will panic if the attribute doesn't exist
            // instead, use if let to check if the attribute exists
            if let Some(target) = n.attr("target") {
                if target.contains("_blank") {
                    target_blank_tags.push(n.text());
                }
            }
        });

        // right now I can't seem to get PS2ISO or PSXISO to work so this won't work for now
        // do a check to see if we can find an a tag with the href that matches this regex
        // /(dev_hdd0|dev_usb00[0-9])/(PSXISO|PS2ISO)
        let mut ps2_ps1_tags: Vec<String> = Vec::new();
        self.html.find(Name("a")).for_each(|n| {
            // don't use unwrap() here, as it will panic if the attribute doesn't exist
            // instead, use if let to check if the attribute exists
            if let Some(href) = n.attr("href") {
                if href.contains("dev_hdd0") || href.contains("dev_usb00")
                // we leave out the [0-9] because we don't know how many USB devices the user has and it saves us from having to do a loop
                {
                    if href.contains("PSXISO") || href.contains("PS2ISO") {
                        ps2_ps1_tags.push(n.text());
                    }
                }
            }
        });

        // if there are no target="_blank" tags, then we could be dealing with a PS2, PS1 game or on the XMB
        if target_blank_tags.len() > 0 {
            println!("PS3 Game or Homebrew");
            if self.get_ps3_details().is_err() {
                return Err("Failed to get PS3 details".into());
            }
        } else if ps2_ps1_tags.len() > 0 {
            println!("PS2 or PS1 Game");
        } else {
            println!("XMB");
            if self.dif_check(&"XMB".to_string(), &"PS3".to_string()) {
                self.start_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
            }
            self.name = "XMB".to_string();
            self.title_id = "PS3".to_string();
            self.image = "xmb".to_string();
        }

        Ok(())
    }

    fn get_ps3_details(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // get the a tag with target _blank that text is the titleID
        // the next sibling is the game name

        let mut target_blank_tags: Vec<String> = Vec::new();

        // PS3ISO, JB Folder Format, and PS3 PKG games will display this field in wman
        self.html.find(Name("a")).for_each(|n| {
            // don't use unwrap() here, as it will panic if the attribute doesn't exist
            // instead, use if let to check if the attribute exists
            if let Some(target) = n.attr("target") {
                if target.contains("_blank") {
                    target_blank_tags.push(n.text());
                }
            }
        });

        // now we have the titleID, we can find the next sibling this contains the game name
        let target_blank = target_blank_tags[0].clone();

        let mut target_blank_siblings: Vec<Node> = Vec::new();

        // this is messy and can probably be done better
        self.html.find(Name("a")).for_each(|n| {
            // don't use unwrap() here, as it will panic if the attribute doesn't exist
            // instead, use if let to check if the attribute exists
            if let Some(target) = n.attr("target") {
                if target.contains("_blank") {
                    // now we have the target blank tag, we want to find the next a tag that is next to it
                    // we can do this by getting the parent that contains both of them then get the next sibling
                    let parent = n.parent().unwrap();
                    // get all a tags that are children of the parent
                    let children = parent.find(Name("a"));
                    // the name will be the second child
                    let mut children_vec: Vec<Node> = Vec::new();
                    children.for_each(|n| {
                        children_vec.push(n);
                    });
                    // now we have the children, we can get the second one
                    let sibling = children_vec[1].clone();
                    target_blank_siblings.push(sibling);
                }
            }
        });

        let mut game_name = target_blank_siblings[0].clone().text();

        // remove the version number from the game name if it exists
        let version_regex = Regex::new(r"(.+)[0-9]{2}.[0-9]{2}").unwrap();
        let binding = version_regex.captures(&game_name).unwrap();
        game_name = binding[1].trim().to_string();

        if self.dif_check(&game_name, &target_blank) {
            println!("Game name or titleID are different");
            println!("Game name: {}", game_name);
            println!("TitleID: {}", target_blank);

            self.title_id = target_blank;
            self.name = game_name;

            self.start_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // now lets get the game icon
            self.image_fetcher();
        }

        Ok(())
    }

    fn image_fetcher(&mut self) {
        // we can use the website gametdb.com to get the game icon
        // the url is https://art.gametdb.com/ps3/cover/{Region}/{titleid}.jpg

        // we need to get the region from the titleID
        // this is the third character in the titleID string
        // read more about it here https://www.psdevwiki.com/ps3/TITLE_ID

        // first we'll do a quick lookup to see if we have a local copy of the image matching the titleID
        // if we do then we want to prioritize that over the gametdb.com image
        let local_image = self.local_images.get_image(&self.title_id);
        // if we have a local image, then we can use that
        if local_image.is_some() {
            // now we can't directly use the image we need to upload it to discord first
            let result = self.image_uploader.upload_artifact(local_image.unwrap());
            if result.is_ok() {
                self.image = result.unwrap();
                println!("Image Url: {}", self.image);
                return;
            } else {
                println!("Failed to upload local image: {}", result.unwrap_err());
                println!("Using gametdb.com image")
            }
        }

        let region: char = self.title_id.chars().nth(2).unwrap();

        // now we match the region to the correct region code
        let region_code = match region {
            'A' => "AS",
            'E' => "EU",
            'H' => "HK",
            'J' => "JA",
            'U' => "US",
            _ => "US",
        };

        // now we can build the url
        let url = format!(
            "https://art.gametdb.com/ps3/cover/{}/{}.jpg",
            region_code, self.title_id
        );

        println!("URL: {}", url);

        // now we test if the url resolves to a 200
        let client = reqwest::blocking::Client::new();
        let res = client.get(&url).send();

        // if the response is a 200, then we can use the image else we will use xmb
        if res.unwrap().status().is_success() {
            self.image = url;
        } else {
            self.image = "xmb".to_string();
        }

        // if the response is a 200, then we
    }

    // I can't get this to work right now
    //fn get_retro_details(&mut self) {
    //    self.name = "Retro".to_string();
    //
    //    // get the a tag with href that maches this regex /(dev_hdd0|dev_usb00[0-9])/(PSXISO|PS2ISO)
    //    let mut ps2_ps1_tags: Vec<String> = Vec::new();
    //    self.html.find(Name("a")).for_each(|n| {
    //        // don't use unwrap() here, as it will panic if the attribute doesn't exist
    //        // instead, use if let to check if the attribute exists
    //        if let Some(href) = n.attr("href") {
    //            if href.contains("dev_hdd0") || href.contains("dev_usb00")
    //            // we leave out the [0-9] because we don't know how many USB devices the user has and it saves us from having to do a loop
    //            {
    //                if href.contains("PSXISO") || href.contains("PS2ISO") {
    //                    ps2_ps1_tags.push(n.text());
    //                }
    //            }
    //        }
    //    });
    //}
    // check if the current state is different from the previous state
    fn dif_check(&self, name: &String, title_id: &String) -> bool {
        if &self.name != name || &self.title_id != title_id {
            return true;
        } else {
            return false;
        }
    }
}
