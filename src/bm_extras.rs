pub fn goodsuffixsearch<'a>(pattern: &str, contents: &'a str) -> Vec<i32> {
    // Preprocessing

    let pl: i32 = pattern.len() as i32;
    let cl: i32 = contents.len() as i32;

    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = contents.chars().collect();

    // This is probably bad practice
    let shift: Vec<i32> = vec![0; (pl + 1) as usize];
    let bpos: Vec<i32> = vec![0; (pl + 1) as usize];

    let (shift, bpos) = strong_suffix_table(&pat, shift, bpos);
    let (shift, _bpos) = process_case_2(&pat, shift, bpos);

    let mut i = 0;
    let mut j: i32;

    let mut shifts_checked = 0;

    let mut locs: Vec<i32> = Vec::new();

    while i <= cl - pl {
        shifts_checked += 1;

        j = pl - 1;

        // If we're matching keep going
        while j >= 0 && pat[j as usize] == txt[(i + j) as usize] {
            j -= 1;
        }

        // If we matched here then j becomes -1
        if j < 0 {
            println!("Pattern occurs at shift {}", i);
            locs.push(i);

            i += shift[0];
        } else {
            i += shift[j as usize + 1];
        }
    }

    println!("Good suffix only checked {} shifts", shifts_checked);

    locs
}

pub fn badcharsearch<'a>(pattern: &str, contents: &'a str) -> Vec<i32> {
    // Preprocessing

    // Length variables
    let pl: i32 = pattern.len() as i32;
    let cl: i32 = contents.len() as i32;

    // Vectors of chars (much easier to work with than strings)
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = contents.chars().collect();

    let badchar = badchar_table(&pat);

    println!("\n\n\nPreprocessing done. Iterating.");

    // Looping
    // Only loop until the end of the pattern is at the end of the text, not beyond
    // cl - pl

    let mut locs: Vec<i32> = Vec::new();

    let mut shifts_checked = 0;

    // i is the pattern position in the text
    let mut i: i32 = 0;
    while i <= cl - pl {
        shifts_checked += 1;
        // j is the position in the pattern
        // if pl = 3
        // ABC
        // 012
        //   ^
        // j = 2

        let mut j: i32 = (pl - 1) as i32;

        // If the characters match, just dec j
        while j >= 0 && pat[j as usize] == txt[(i + j) as usize] {
            j -= 1;
        }

        // If everything matches, j will go to -1
        if j < 0 {
            println!("Pattern found at shift {}", i);

            locs.push(i);

            // Now move the pattern

            if i + pl < cl {
                i += pl - badchar[txt[(i + pl) as usize] as usize];
            } else {
                i += 1;
            }
        } else {
            // No match :(
            // Lets move the pattern according to our table

            i += cmp::max(1, j - badchar[txt[(i + j) as usize] as usize]);
        }
    }

    println!("Bad character only did {} shifts", shifts_checked);

    locs
}
