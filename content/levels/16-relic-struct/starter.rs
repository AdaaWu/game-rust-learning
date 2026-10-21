// 導遊卷軸：只指向古碑原文，不複製
struct Guide<'a> {
    text: &'a str,
}

impl<'a> Guide<'a> {
    // 建立卷軸
    fn new(text: &'a str) -> Guide<'a> {
        todo!("把 text 存進卷軸")
    }

    // 原文有幾行
    fn line_count(&self) -> usize {
        todo!("用 self.text.lines() 數行數")
    }

    // 找出含有關鍵字的每一行；找到的句子屬於原文，不屬於卷軸
    fn find(&self, keyword: &str) -> Vec<&'a str> {
        let mut found = Vec::new();
        for line in self.text.lines() {
            todo!("line 含有 keyword 就放進 found")
        }
        found
    }
}

// 帶隊導覽：卷軸用完就丟，找到的句子要能帶走
fn tour<'a>(text: &'a str, keyword: &str) -> (usize, Vec<&'a str>) {
    let guide = Guide::new(text);
    (guide.line_count(), guide.find(keyword))
}

fn main() {
    let text = String::from("螃蟹王的寶冠\n海鷗的羽毛\n古老的寶石");
    let (lines, found) = tour(&text, "寶");
    println!("古碑共 {} 行，提到寶物的有：{:?}", lines, found);
}
