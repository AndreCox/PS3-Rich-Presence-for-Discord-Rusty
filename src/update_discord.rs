use discord_rich_presence::{
    activity::{self, Timestamps},
    DiscordIpc, DiscordIpcClient,
};

use crate::ps3_scraper;

pub struct DiscordRichPresence {
    client: DiscordIpcClient,
}

impl DiscordRichPresence {
    pub fn new(client_id: String) -> Self {
        Self {
            client: DiscordIpcClient::new(&client_id).unwrap(),
        }
    }

    pub fn connect(&mut self) -> bool {
        if self.client.connect().is_err() {
            println!("Failed to connect to Discord IPC");
            return false;
        }
        return true;
    }

    pub fn send_presence<'a>(
        &mut self,
        ps3_scraper: &'a ps3_scraper::Ps3Scraper,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let state_binding = format!(
            "CPU Temp: {} | RSX Temp: {}",
            ps3_scraper.temp[0], ps3_scraper.temp[1]
        );
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

        Ok(())
    }
    pub fn disconnect(&mut self) {
        if self.client.close().is_err() {
            println!("Failed to disconnect from Discord IPC");
        }
    }
}
