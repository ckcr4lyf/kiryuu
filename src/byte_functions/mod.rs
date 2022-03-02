mod tests;

use hex;

pub fn do_nothing() {
    println!("XD");
}

pub fn url_encoded_to_hex(urlenc: &str) -> String {
    let mut hex_str = String::new();

    for i in 0..urlenc.chars().count() {

        let the_char = urlenc.chars().nth(i).expect("Shits fucked up yo");

        

        if the_char == '%' {
            hex_str.push_str(&urlenc[i+1..i+3]);
        } else {
            hex_str.push_str(&hex::encode(the_char.to_string()));
        }
    }

    return hex_str;
}
