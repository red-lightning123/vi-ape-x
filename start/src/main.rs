use coordinator_client::CoordinatorClient;
use prompt::prompt_user_for_service_ip_addr;

fn main() {
    let coordinator_ip_addr = prompt_user_for_service_ip_addr("coordinator");
    println!("coordinator ip addr set to {}...", coordinator_ip_addr);
    let coordinator_addr = (coordinator_ip_addr, ports::COORDINATOR).into();
    let coordinator_client = CoordinatorClient::new(coordinator_addr);
    coordinator_client.start();
    println!("start message sent");
}
