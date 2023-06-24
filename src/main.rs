#![windows_subsystem = "windows"]

mod network_scanner;
mod ps3_scraper;
mod states;
mod update_discord;
mod webman_discovery;

fn main() {
    let mut discord_rich_presence =
        update_discord::DiscordRichPresence::new("780389261870235650".to_string());
    discord_rich_presence.connect();

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
        ps3_scraper::Ps3Scraper::new(webman_ip),
        discord_rich_presence,
    );

    loop {
        state_machine.update();
    }
}
