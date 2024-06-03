use std::net::Ipv4Addr;

pub fn prompt_user_for_service_ip_addr(service_name: &str) -> Ipv4Addr {
    loop {
        println!(
            "enter {} IPv4 address (keep blank for {}):",
            service_name,
            Ipv4Addr::LOCALHOST
        );
        let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt).unwrap();
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return Ipv4Addr::LOCALHOST;
        }
        match prompt.parse() {
            Ok(ip_addr) => {
                return ip_addr;
            }
            Err(e) => println!("could not parse ip addr: {}", e),
        }
    }
}
