use chrono::Local;
use rand::Rng;

/// 生成 ID（格式: 前缀_时间戳毫秒_6位随机十六进制）
pub fn generate_id(prefix: &str) -> String {
    let ts = Local::now().timestamp_millis();
    let random: u32 = rand::thread_rng().gen_range(0..0xFFFFFF);
    format!("{}_{}_{:06x}", prefix, ts, random)
}
