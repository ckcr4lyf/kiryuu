#[cfg(test)]
use crate::req_log;

#[test]
fn is_legit(){
    assert_eq!("ip_addr,infohash", req_log::generate_csv("ip_addr", "infohash"));
    assert_eq!("1.1.1.1,abcd", req_log::generate_csv("1.1.1.1", "abcd"));
}