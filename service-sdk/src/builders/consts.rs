use std::net::IpAddr;

pub fn get_default_ip_address() -> IpAddr {
    IpAddr::from([0, 0, 0, 0])
}
