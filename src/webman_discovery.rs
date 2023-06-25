use netscan::host::HostInfo;
use reqwest;
use select::document::Document;
use select::predicate::Name;
use std::sync::mpsc;
use std::thread;

pub fn get_webman(hosts: Vec<HostInfo>) -> String {
    let (tx, rx) = mpsc::channel();

    let hosts_len = hosts.len();
    for host in hosts {
        let tx = tx.clone();
        let handle = thread::spawn(move || tx.send(webman_discovery(host)));
    }

    // now we wait until we get a response from one of the threads that isn't an empty string
    let mut webman_ip = "".to_string();
    let mut returned_threads = 0;
    while webman_ip == "".to_string() {
        webman_ip = rx.recv().unwrap();

        // if we've gone through all the hosts and still haven't found webman, return an empty string
        returned_threads += 1;
        if returned_threads == hosts_len {
            return "".to_string();
        }
    }

    return webman_ip;
}

fn webman_discovery(ip: HostInfo) -> String {
    // create a custom reqwest client with a timeout of 5 seconds
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let resp = client.get(&format!("http://{}", ip.ip_addr)).send();

    let html_result = match resp {
        Ok(html) => html.text(),
        Err(_) => return "".to_string(),
    };

    let html = match html_result {
        Ok(html) => html,
        Err(_) => return "".to_string(),
    };

    let mut titles: Vec<String> = Vec::new();

    Document::from(html.as_str())
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
