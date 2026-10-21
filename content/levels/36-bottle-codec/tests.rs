const TOKENS: [&str; 5] = ["R", "L", "U", "D", "*"];
const STEPS: [Step; 5] = [Step::Right, Step::Left, Step::Up, Step::Down, Step::Pick];
const TEXTS: [&str; 5] = ["MOVE RIGHT", "MOVE LEFT", "MOVE UP", "MOVE DOWN", "PICK"];

// 測試用的簡易亂數（線性同餘）
fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed >> 33
}

#[test]
fn parse_every_token() {
    for i in 0..5 {
        let got = TOKENS[i].parse::<Step>();
        ::std::assert!(
            ::std::matches!(&got, Ok(step) if std::mem::discriminant(step) == std::mem::discriminant(&STEPS[i])),
            "記號 {} 應該解析成 {:?}，實際 {:?}",
            TOKENS[i],
            STEPS[i],
            got
        );
    }
}

#[test]
fn parse_rejects_unknown_tokens() {
    for bad in ["X", "r", "RR", "", "右", "**"] {
        let got = bad.parse::<Step>();
        ::std::assert!(
            ::std::matches!(&got, Err(StepError::BadToken(token)) if token == bad),
            "看不懂的記號 {:?} 應該回報 BadToken，不能亂猜，實際 {:?}",
            bad,
            got
        );
    }
}

#[test]
fn display_every_step() {
    for i in 0..5 {
        ::std::assert_eq!(::std::format!("{}", STEPS[i]), TEXTS[i], "{:?} 應該印成 {}", STEPS[i], TEXTS[i]);
        ::std::assert_eq!(STEPS[i].to_string(), TEXTS[i], "實作 Display 之後，to_string() 也要得到 {}", TEXTS[i]);
    }
}

#[test]
fn random_letters_round_trip() {
    for round in 0..20u64 {
        let mut seed = round * 41 + 9;
        let picks: Vec<usize> = (0..12).map(|_| (next_rand(&mut seed) % 5) as usize).collect();
        let letter: Vec<&str> = picks.iter().map(|i| TOKENS[*i]).collect();
        let got: Vec<String> = letter.iter().map(|t| t.parse::<Step>().unwrap().to_string()).collect();
        let want: Vec<String> = picks.iter().map(|i| TEXTS[*i].to_string()).collect();
        ::std::assert_eq!(got, want, "瓶中信 {:?} 翻譯錯了", letter.join(" "));
    }
}

#[test]
fn route_follows_the_letter() {
    assert_cleared(&level_grid(), &route());
}
