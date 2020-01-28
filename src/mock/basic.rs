use rand::{thread_rng, Rng};



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




