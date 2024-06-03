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

pub fn prompt_user_for_eps() -> f64 {
    const DEFAULT_EPS: f64 = 0.01;
    loop {
        println!("enter epsilon value (keep blank for {DEFAULT_EPS}):");
        let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt).unwrap();
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return DEFAULT_EPS;
        }
        match prompt.parse() {
            Ok(eps) => {
                return eps;
            }
            Err(e) => println!("could not parse epsilon value: {}", e),
        }
    }
}
