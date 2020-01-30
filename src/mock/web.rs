use rand::{thread_rng, Rng};
use crate::mock;

const DOMAIN_SUFFIX:[&str;10] = ["com", "net", "org", "edu", "gov", "cc", "cn", "com.cn", "name", "mobi"];

pub fn ip() -> String {
    let mut rng = thread_rng();
    format!("{}.{}.{}.{}", rng.gen_range(0, 255), rng.gen_range(0, 255), rng.gen_range(0, 255), rng.gen_range(0, 255))
}


pub fn domain_suffix() -> String {
    let mut rng = thread_rng();
    let n = rng.gen_range(0, DOMAIN_SUFFIX.len());
    DOMAIN_SUFFIX[n].to_string()
}


pub fn domain(is_use_www: bool) -> String {
    if !is_use_www {
        format!("{}.{}", mock::text::word(0), domain_suffix())
    } else {
        format!("www.{}.{}", mock::text::word(0), domain_suffix())
    }
}

pub fn email() -> String {
    format!("{}@{}", mock::text::word(0), domain(false))
}

pub fn url() -> String {
    let mut rng = thread_rng();
    let n = rng.gen_range(1, 10);
    let http = if n % 2 == 0 {
        "http"
    } else {
        "https"
    };
    format!("{}://{}.{}/{}/", http, mock::text::word(0), domain(false), mock::text::word(0))
}