use rand::{thread_rng, Rng};

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789)(*&^%$#@!~";
const CHARSET2: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789)(*&^%$#@!~";

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


pub fn int() -> i64 {
    let mut rng = thread_rng();
    rng.gen::<i64>()
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