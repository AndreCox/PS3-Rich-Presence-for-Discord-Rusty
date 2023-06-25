#![windows_subsystem = "windows"]

mod artifact_upload;
mod local_images;
mod network_scanner;
mod ps3_scraper;
mod states;
mod update_discord;
mod webman_discovery;

fn main() {
    let mut local_images = local_images::LocalImages::new();
    local_images.load_images();

    let mut image_uploader = artifact_upload::ArtifactUploader::new("https://discord.com/api/webhooks/1122287939938959380/33Zr7YlrN-DM6imzeV-fwZ4C7mMxb1bzquGlkmKMCfxg7UcPNi6JNTLXiap-1V_THwuC".to_string());

    let discord_rich_presence =
        update_discord::DiscordRichPresence::new("780389261870235650".to_string());

    let host_network = network_scanner::grab_host_network().unwrap();
    let hosts = network_scanner::scan_network(host_network).unwrap();

    let webman_ip = webman_discovery::get_webman(hosts);
    if webman_ip == "".to_string() {
        println!("webMAN not found");
    } else {
        println!("webMAN found at {}", webman_ip);
    }

    let mut state_machine = states::StateMachine::new(
        15,
        15,
        ps3_scraper::Ps3Scraper::new(webman_ip, local_images, image_uploader),
        discord_rich_presence,
    );

    loop {
        state_machine.update();
    }
}
