pub fn generate_csv(ip_addr: &str, infohash: &str) -> String {
    let mut csv: String = ip_addr.to_string();
    csv.push_str(",");
    csv.push_str(infohash);
    return csv;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_legit() {
        assert_eq!("ip_addr,infohash", generate_csv("ip_addr", "infohash"));
        assert_eq!("1.1.1.1,abcd", generate_csv("1.1.1.1", "abcd"));
    }
}
