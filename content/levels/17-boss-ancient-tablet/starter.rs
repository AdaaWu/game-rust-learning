// 古碑解讀器：只借用古碑原文，不複製
struct Tablet<'a> {
    text: &'a str,
}

impl<'a> Tablet<'a> {
    fn new(text: &'a str) -> Tablet<'a> {
        Tablet { text }
    }

    // 古碑上有幾個字
    fn char_count(&self) -> usize {
        self.text.len()
    }

    // 從第 start 個字開始，取 len 個字；超過結尾就取到結尾為止
    fn excerpt(&self, start: usize, len: usize) -> &'a str {
        let end = (start + len).min(self.char_count());
        &self.text[start..end]
    }
}

// 解讀：每塊古碑取開頭 n 個字，用「·」串起來
fn decode(stones: &[&str], n: usize) -> String {
    let mut parts = Vec::new();
    for stone in stones {
        parts.push(Tablet::new(stone).excerpt(0, n));
    }
    parts.join("·")
}

fn main() {
    let stones = ["蟹王之墓", "海鷗守門"];
    println!("{}", decode(&stones, 2));
}
