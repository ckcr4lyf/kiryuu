mod tests;

pub fn generate_csv(ip_addr: &str, infohash: &str) -> String {
    let mut csv: String = ip_addr.to_string();
    csv.push_str(",");
    csv.push_str(infohash);
    return csv;
}