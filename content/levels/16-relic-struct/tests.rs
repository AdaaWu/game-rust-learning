const TABLET: &str = "螃蟹王的寶冠\n海鷗的羽毛\n古老的寶石";

#[test]
fn guide_points_to_original() {
    let text = String::from(TABLET);
    let guide = Guide::new(&text);
    ::std::assert_eq!(guide.text, TABLET, "卷軸記錄的內容和古碑不一樣");
    ::std::assert_eq!(guide.text.as_ptr(), text.as_ptr(), "卷軸應該指向古碑原文，而不是抄一份");
}

#[test]
fn counts_lines() {
    let text = String::from(TABLET);
    ::std::assert_eq!(Guide::new(&text).line_count(), 3, "古碑原文應該有 3 行");
}

#[test]
fn found_lines_outlive_the_guide() {
    let text = String::from(TABLET);
    let found;
    {
        let guide = Guide::new(&text);
        found = guide.find("寶");
    }
    ::std::assert_eq!(found, ::std::vec!["螃蟹王的寶冠", "古老的寶石"], "卷軸收起來之後，找到的句子應該還能讀");
}

#[test]
fn find_nothing() {
    let text = String::from(TABLET);
    ::std::assert!(Guide::new(&text).find("章魚").is_empty(), "古碑沒有提到章魚，不該找到任何一行");
}

#[test]
fn tour_reports_lines_and_matches() {
    let text = String::from(TABLET);
    let (lines, found) = tour(&text, "海鷗");
    ::std::assert_eq!(lines, 3, "導覽回報的行數不對");
    ::std::assert_eq!(found, ::std::vec!["海鷗的羽毛"], "導覽找到的句子不對");
    ::std::assert_eq!(found[0].as_ptr(), text[text.find("海鷗").unwrap()..].as_ptr(), "找到的句子應該是原文的切片");
}
