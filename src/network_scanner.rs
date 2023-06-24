use ipnet::Ipv4Net;
use local_ip_address::{local_ip, Error};
use netscan::blocking::HostScanner;
use netscan::host::HostInfo;
use netscan::setting::ScanType;
use pbr::ProgressBar;
use std::{
    net::{IpAddr, Ipv4Addr},
    thread,
    time::Duration,
};

pub fn scan_network(host_network: Vec<u8>) -> Result<Vec<HostInfo>, Error> {
    let interface = default_net::get_default_interface().unwrap();
    let mut host_scanner = match HostScanner::new(IpAddr::V4(interface.ipv4[0].addr)) {
        Ok(scanner) => scanner,
        Err(e) => panic!("Error creating scanner: {}", e),
    };
    let net: Ipv4Net = Ipv4Net::new(
        Ipv4Addr::new(
            host_network[0],
            host_network[1],
            host_network[2],
            host_network[3],
        ),
        host_network[4],
    )
    .unwrap();

    let nw_addr = Ipv4Net::new(net.network(), host_network[4]).unwrap();
    let hosts: Vec<Ipv4Addr> = nw_addr.hosts().collect();
    let count = hosts.len() as u64;
    for host in hosts {
        let dst: HostInfo = HostInfo::new_with_ip_addr(IpAddr::V4(host));
        host_scanner.add_target(dst);
    }
    // Set options
    host_scanner.set_scan_type(ScanType::IcmpPingScan);
    host_scanner.set_timeout(Duration::from_millis(10000));
    host_scanner.set_wait_time(Duration::from_millis(500));

    let rx = host_scanner.get_progress_receiver();
    // Run scan
    let handle = thread::spawn(move || host_scanner.scan());
    // Print progress
    let mut pb = ProgressBar::new(count);
    while let Ok(_socket_addr) = rx.lock().unwrap().recv() {
        pb.inc();
    }
    pb.finish_print("Finished");
    let result = handle.join().unwrap();
    println!("\nStatus: {:?}", result.scan_status);
    println!("Up hosts:");
    for host in &result.hosts {
        println!("  {}", host.ip_addr);
    }
    println!("Scan Time: {:?}", result.scan_time);

    Ok(result.hosts)
}

pub fn grab_host_network() -> Result<Vec<u8>, Error> {
    let device_ip_result = local_ip();
    let device_ip = match device_ip_result {
        Ok(device_ip) => device_ip,
        Err(e) => return Err(e),
    };
    println!("Device IP: {}", device_ip);
    // convert the host_network to a x.x.x.0/24 format
    let host_network = device_ip.to_string();
    let mut host_network: Vec<&str> = host_network.split(".").collect();
    host_network.pop();
    host_network.push("0");
    host_network.push("24");

    // convert from Vec<&str> to Vec<u8>
    let host_network: Vec<u8> = host_network
        .iter()
        .map(|&s| s.parse::<u8>().unwrap())
        .collect();

    println!(
        "Host Network: {}.{}.{}.{}/{}",
        host_network[0], host_network[1], host_network[2], host_network[3], host_network[4]
    );

    Ok(host_network)
}
