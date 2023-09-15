pub trait ServiceInfo {
    fn get_service_name(&self) -> String;
    fn get_service_version(&self) -> String;
}
