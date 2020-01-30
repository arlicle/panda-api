use rand::{thread_rng, Rng};
use std::time::{Duration, SystemTime};
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789)(*&^%$#@!~";
const CHARSET2: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789)(*&^%$#@!~";


enum Type {
    String(String),
    Int(i32),
}

/// 随机生成一个bool
pub fn bool() -> bool {
    let mut rng = thread_rng();
    let n = rng.gen_range(1, 10);

    if n % 2 == 0 {
        true
    } else {
        false
    }
}

#[macro_export]
macro_rules! int {
    ($min_value:expr, $max_value:expr) => {
    {
        let mut rng = thread_rng();
        rng.gen_range($min_value, $max_value)
    }
    };

    ($min_value:expr) => {
    {
        let mut rng = thread_rng();
        rng.gen_range($min_value, i32::max_value())
    }
    };

    () => {
    {
        let mut rng = thread_rng();
        rng.gen::<i32>()
    }
    };
}


pub fn round(n: f64, precision: u32) -> f64 {
    (n * 10_u32.pow(precision) as f64).round() / 10_i32.pow(precision) as f64
}

#[macro_export]
macro_rules! float {
    ($min_value:expr, $max_value:expr, $min_decimal_places:expr, $max_decimal_places:expr) => {
    {
        let mut rng = thread_rng();
        let n = rng.gen_range($min_value as f64, $max_value as f64);
        let l = rng.gen_range($min_decimal_places as u32, $max_decimal_places as u32);
        (n * 10_u64.pow(l) as f64).round() / 10_i64.pow(l) as f64
    }
    };

    ($min_value:expr, $max_value:expr, $min_decimal_places:expr) => {
    {
        let mut rng = thread_rng();
        let n = rng.gen_range($min_value as f64, $max_value as f64);
        (n * 10_u64.pow($min_decimal_places) as f64).round() / 10_i64.pow($min_decimal_places) as f64
    }
    };
}



#[macro_export]
macro_rules! timestamp {
    ($min_value:expr, $max_value:expr) => {
    {
        let s = SystemTime::now();
        let mut min_value = $min_value;
        let mut max_value = $max_value;
        if min_value == 0 {
            // 两年前：当前时间-两年
            let s2 = s.checked_sub(Duration::from_secs(63072000)).unwrap();
            let s2 = s2.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            min_value = s2.as_secs();
        }

        if max_value == 0 {
            // 两年后：当前时间+两年
            let s2 = s.checked_add(Duration::from_secs(63072000)).unwrap();
            let s2 = s2.duration_since(SystemTime::UNIX_EPOCH).unwrap();
            max_value = s2.as_secs();
        }

        int!(min_value, max_value)
    }
    };

    () => {
    {
        timestamp!(0,0)
    }
    };
}



fn datetime_str_to_timestamp(datetime_str: &str) -> u64 {
    let re = Regex::new(r"\d+").unwrap();

    let mut v = Vec::with_capacity(6);

    for cap in re.captures_iter(datetime_str) {
        v.push((&cap[0]).parse::<u32>().unwrap());
    }

    for i in 0..v.len() {
        v.push(0);
    }

    let dt = Utc.ymd(v[0] as i32, v[1], v[2]).and_hms(v[3], v[4], v[5]);
    dt.timestamp() as u64
}



pub fn datetime(min_value: &str, max_value: &str, format: &str) -> String {
    let mut timestamp_min_value = 0;
    let mut timestamp_max_value = 0;
    let mut format = format;
    let re = Regex::new(r"\d+").unwrap();
    if min_value != "" {
        timestamp_min_value = datetime_str_to_timestamp(min_value);
    }
    if max_value != "" {
        timestamp_max_value = datetime_str_to_timestamp(max_value);
    }

    if format.trim() == "" {
        format = "%Y-%m-%d %H:%M:%S";
    }

    let t = timestamp!(timestamp_min_value, timestamp_max_value);

    let dt = Utc.timestamp(t as i64, 0);
    dt.format(&format).to_string()
}


pub fn float() -> f64 {
    let mut rng = thread_rng();
    rng.gen::<f64>()
}


pub fn char() -> char {
    let mut rng = thread_rng();
    let idx = rng.gen_range(0, CHARSET.len());
    CHARSET[idx] as char
}


/// 随机生成英文+符号的字符串
pub fn string(mut length: usize) -> String {
    let mut s = String::new();
    let mut rng = thread_rng();

    if length == 0 {
        length = rng.gen_range(7, 20);
    }

    let l = CHARSET.len();
    while length > 0 {
//        let n: usize = rng.gen_range(0, l);
//        let a1 = &CHARSET[n];
        s.push(char());
        length -= 1;
    }

    s
}