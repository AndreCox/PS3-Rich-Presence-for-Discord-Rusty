use netscan::host::HostInfo;
use reqwest;
use select::document::Document;
use select::predicate::Name;
use std::sync::mpsc;
use std::thread;

pub fn get_webman(hosts: Vec<HostInfo>) -> String {
    let (tx, rx) = mpsc::channel();

    for host in hosts {
        let tx = tx.clone();
        thread::spawn(move || tx.send(webman_discovery(host)));
    }

    // now we wait until we get a response from one of the threads that isn't an empty string
    let mut webman_ip = "".to_string();

    while webman_ip == "".to_string() {
        webman_ip = rx.recv().unwrap();
    }

    return webman_ip;
}

fn webman_discovery(ip: HostInfo) -> String {
    let resp = reqwest::blocking::get(&format!("http://{}", ip.ip_addr));

    let html = match resp {
        Ok(html) => html.text(),
        Err(_) => return "".to_string(),
    };

    let mut titles: Vec<String> = Vec::new();

    Document::from(html.unwrap().as_str())
        .find(Name("title"))
        .for_each(|n| titles.push(n.text()));

    if titles.len() == 0 {
        return "".to_string();
    }
    if titles[0].contains("wMAN") || titles[0].contains("webMAN") {
        println!("{} is running webMAN", ip.ip_addr);
        return ip.ip_addr.to_string();
    }

    return "".to_string();
}
