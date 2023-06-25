// this file is the state machine for the program. It is responsible for what the program does at any given time.
// there are 4 conditions:
// 1. Discord Disconnected - the program is not connected to Discord
// 2. Discord Connected - the program is connected to Discord
// 3. Webman Found - the program has found a PS3 running webMAN
// 4. Webman Not Found - the program has not found a PS3 running webMAN

// The program should only run when Discord is connected and a PS3 running webMAN is found.

// There are 3 states:
// 1. Idle - the program is not connected to Discord or has not found a PS3 running webMAN yet, we sleep to save resources
// 2. Data Handling - the program is connected to Discord and has found a PS3 running webMAN, we fetch data from the PS3 and send it to Discord
// 3. Sleep - the program is connected to Discord and has found a PS3 running webMAN, we sleep to save resources and wait for the next data fetch

use crate::{ps3_scraper, update_discord, webman_discovery};

pub struct StateMachine {
    connected_discord: bool,
    found_webman: bool,
    idle_time: u64,  // idle time when we can't find a PS3 running webMAN
    sleep_time: u64, // sleep time between data fetches | min: 15

    // our objects that we use to do stuff
    ps3_scraper: ps3_scraper::Ps3Scraper,
    discord_rich_presence: update_discord::DiscordRichPresence,
}

impl StateMachine {
    pub fn new(
        idle_time: u64,
        sleep_time: u64,
        ps3_scraper: ps3_scraper::Ps3Scraper,
        discord_rich_presence: update_discord::DiscordRichPresence,
    ) -> Self {
        // check that idle_time is at least 15 seconds
        if sleep_time < 15 {
            println!("sleepTime must be at least 15 seconds");
            std::process::exit(2);
        }

        Self {
            connected_discord: false,
            found_webman: false,
            idle_time,
            sleep_time: idle_time,
            ps3_scraper: ps3_scraper,
            discord_rich_presence: discord_rich_presence,
        }
    }

    pub fn update(&mut self) {
        if self.connected_discord == false && self.found_webman == false {
            if self.discord_rich_presence.connect() {
                println!("Connected to Discord");
                self.connected_discord = true;
                return;
            }
        } else if self.connected_discord == false && self.found_webman == true {
            if self.discord_rich_presence.connect() {
                println!("Connected to Discord");
                self.connected_discord = true;
                return;
            }
            self.idle();
        } else if self.connected_discord == true && self.found_webman == false {
            self.discord_rich_presence.disconnect();
            println!("Searching for webMAN");
            if self.ps3_scraper.ip == "".to_string() {
                let host_network = crate::network_scanner::grab_host_network().unwrap();
                let hosts = crate::network_scanner::scan_network(host_network).unwrap();

                let webman_ip = webman_discovery::get_webman(hosts);
                if webman_ip == "".to_string() {
                    println!("webMAN not found");
                    self.idle();
                    return;
                } else {
                    println!("webMAN found at {}", webman_ip);
                    self.ps3_scraper.ip = webman_ip;
                }
            }
            if self.ps3_scraper.fetch_data().is_err() {
                self.found_webman = false;
                self.idle();
                return;
            }
            self.found_webman = true;
        } else if self.connected_discord == true && self.found_webman == true {
            self.data_handling();
            self.sleep();
        }
    }

    fn idle(&mut self) {
        println!("Idle");
        std::thread::sleep(std::time::Duration::from_secs(self.idle_time));
    }

    fn sleep(&mut self) {
        println!("Sleep");
        std::thread::sleep(std::time::Duration::from_secs(self.sleep_time));
    }

    fn data_handling(&mut self) {
        println!("Data Handling");
        let result = self.ps3_scraper.fetch_data();

        if result.is_err() {
            self.found_webman = false;
            return;
        }

        let result = self.discord_rich_presence.send_presence(&self.ps3_scraper);

        if result.is_err() {
            self.connected_discord = false;
        }
    }
}
