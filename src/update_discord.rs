use discord_rich_presence::{
    activity::{self, Timestamps},
    DiscordIpc, DiscordIpcClient,
};

use crate::config;
use crate::ps3_scraper;

pub struct DiscordRichPresence<'a> {
    client: DiscordIpcClient,
    config: &'a config::Config,
}

impl<'a> DiscordRichPresence<'a> {
    pub fn new(client_id: String, config: &'a config::Config) -> Self {
        Self {
            client: DiscordIpcClient::new(&client_id).unwrap(),
            config: config,
        }
    }

    pub fn connect(&mut self) -> bool {
        if self.client.connect().is_err() {
            println!("Failed to connect to Discord IPC");
            return false;
        }
        return true;
    }

    pub fn send_presence<'b>(
        &mut self,
        ps3_scraper: &'b ps3_scraper::Ps3Scraper,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.show_temp {
            let state_binding = format!(
                "CPU Temp: {} | RSX Temp: {}",
                ps3_scraper.temp[0], ps3_scraper.temp[1]
            );
            println!("{}", ps3_scraper.image);
            let presence = activity::Activity::new()
                .state(&state_binding)
                .details(&ps3_scraper.name)
                .assets(
                    activity::Assets::new()
                        .large_image(&ps3_scraper.image)
                        .large_text(&ps3_scraper.title_id),
                )
                .timestamps(Timestamps::new().start(ps3_scraper.start_time));

            if self.client.set_activity(presence.clone()).is_err() {
                println!("Failed to set activity");
                return Err("Failed to set activity".into());
            }
        } else {
            let presence = activity::Activity::new()
                .details(&ps3_scraper.name)
                .assets(
                    activity::Assets::new()
                        .large_image(&ps3_scraper.image)
                        .large_text(&ps3_scraper.title_id),
                )
                .timestamps(Timestamps::new().start(ps3_scraper.start_time));

            if self.client.set_activity(presence.clone()).is_err() {
                println!("Failed to set activity");
                return Err("Failed to set activity".into());
            }
        }

        Ok(())
    }
    pub fn disconnect(&mut self) {
        if self.client.close().is_err() {
            println!("Failed to disconnect from Discord IPC");
        }
    }
}
