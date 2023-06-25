//#![windows_subsystem = "windows"]

mod artifact_upload;
mod config;
mod local_images;
mod network_scanner;
mod ps3_scraper;
mod states;
mod update_discord;
mod webman_discovery;

fn main() {
    let config = config::Config::new();

    let mut local_images = local_images::LocalImages::new();
    local_images.load_images();

    let image_uploader = artifact_upload::ArtifactUploader::new("https://discord.com/api/webhooks/1122287939938959380/33Zr7YlrN-DM6imzeV-fwZ4C7mMxb1bzquGlkmKMCfxg7UcPNi6JNTLXiap-1V_THwuC".to_string());

    let discord_rich_presence =
        update_discord::DiscordRichPresence::new("780389261870235650".to_string(), &config);

    let mut state_machine = states::StateMachine::new(
        &config.idle_time,
        &config.refresh_time,
        ps3_scraper::Ps3Scraper::new(&config.ps3_ip, local_images, image_uploader, &config),
        discord_rich_presence,
        &config,
    );

    loop {
        state_machine.update();
    }
}
