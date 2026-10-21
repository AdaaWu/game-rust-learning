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
        self.text.chars().count()
    }

    // 從第 start 個字開始，取 len 個字；超過結尾就取到結尾為止
    fn excerpt(&self, start: usize, len: usize) -> &'a str {
        let end = (start + len).min(self.char_count());
        &self.text[self.byte_at(start)..self.byte_at(end)]
    }

    // 第 idx 個字從原文的第幾個 byte 開始；到結尾就回傳總長度
    fn byte_at(&self, idx: usize) -> usize {
        for (count, (byte, _)) in self.text.char_indices().enumerate() {
            if count == idx {
                return byte;
            }
        }
        self.text.len()
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
