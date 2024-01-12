use std::cmp;
// use std::env;
use std::error::Error;
use std::fs;
use std::usize;

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub slow_mode: bool,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // Build a list of flags that we can have. Very concise code right here
        let mut flags: Vec<Flag> = Vec::new();
        flags.push(Flag::new(
            "-s",
            "--slow",
            "Uses rust iterators to search for the string",
        ));
        flags.push(Flag::new(
            "-i",
            "--ignore-case",
            "Matches all occurences regardless of if the case matches",
        ));

        args.next(); // Skip the bin location

        let query = match args.next() {
            Some(arg) => {
                if arg == "-h" || arg == "--help" {
                    println!("crep - Custom Regex search and Print");
                    println!("Usage: crep query file/path");
                    println!();
                    println!("Available flags:");
                    println!("-h --help          Display this help text");

                    for flag in flags {
                        println!("{}", flag.help());
                    }

                    return Err("Help page");
                }
                arg
            }
            None => return Err("Didn't get a query string"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };

        // Get the remaining arguments back into a vec
        let remaining: Vec<String> = args.collect();

        // See what flags we have
        // Because I couldn't be bothered doing a hashmap they're just indexes
        // I might change that later, but for now its easiest to just add a new flag at the end of
        // this section and the creation of the flags

        let slow_mode = flags[0].found_in(&remaining);

        let ignore_case = flags[1].found_in(&remaining);

        // Environment variables
        // I want to deprecate this eventually
        // let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
            slow_mode,
        })
    }
}

pub struct Flag {
    pub short: String,
    pub long: String,
    pub description: String,
}

impl Flag {
    fn new(sh: &str, lo: &str, desc: &str) -> Flag {
        let short = sh.to_string();
        let long = lo.to_string();
        let description = desc.to_string();

        Flag {
            short,
            long,
            description,
        }
    }

    fn found_in(&self, args: &Vec<String>) -> bool {
        if args.contains(&self.short) || args.contains(&self.long) {
            true
        } else {
            false
        }
    }

    fn help(&self) -> String {
        format!("{} {:015} {}", self.short, self.long, self.description)
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    let results = if config.ignore_case {
        if config.slow_mode {
            println!("Slow mode case insensitive\n");
            search_case_insensitive(&config.query, &contents)
        } else {
            boyer_moore_lines_case_insensitive(&config.query, &contents)
        }
    } else {
        if config.slow_mode {
            println!("Slow mode case sensitive\n");
            search(&config.query, &contents)
        } else {
            boyer_moore_lines(&config.query, &contents)
        }
    };

    for line in results {
        println!("{line}");
    }

    Ok(()) // We finished with no errors :)
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();

    contents
        .lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}

/*
 *  Boyer-Moore implementation
 *
 *  First, preprocess for the Bad Character and Good Suffix Heuristics.
 *  Then run the search algorithm to get the index of each occurence of the pattern.
 *  Finally, convert those indices to lines we can print out.
 *
 */

// ✓ This has been refactored :)
fn badchar_table(pattern: &Vec<char>) -> Vec<i32> {
    let size = pattern.len();
    let mut badchar: Vec<i32> = vec![-1; 256]; // 256 is the number of chars we're dealing with

    for i in 0..size {
        badchar[pattern[i] as usize] = i as i32;
    }

    badchar
}

// ✓ This one is done too!
fn strong_suffix_table(
    pattern: &Vec<char>,
    mut shift: Vec<i32>,
    mut bpos: Vec<i32>,
) -> (Vec<i32>, Vec<i32>) {
    let plen = pattern.len();

    let mut i = plen;
    let mut j = plen + 1;

    bpos[i] = j as i32;

    // This loop is looking for the border (i.e. a suffix which is also a prefix)
    while i > 0 {
        while j <= plen && pattern[i - 1] != pattern[j - 1] {
            if shift[j] == 0 {
                shift[j] = (j - i) as i32;
            }

            j = bpos[j] as usize;
        }

        i -= 1;
        j -= 1;
        bpos[i] = j as i32;
    }

    (shift, bpos)
}

// ✓ Preprocessing refactored!
pub fn process_case_2(
    pattern: &Vec<char>,
    mut shift: Vec<i32>,
    bpos: Vec<i32>,
) -> (Vec<i32>, Vec<i32>) {
    let pl = pattern.len();

    let mut j = bpos[0];

    for i in 0..(pl + 1) {
        if shift[i] == 0 {
            shift[i] = j;
        }

        if i == j as usize {
            j = bpos[j as usize];
        }
    }

    (shift, bpos)
}

pub fn boyer_moore_search(pattern: &str, contents: &str) -> Vec<i32> {
    // Preprocessing

    // length vars
    let pl: i32 = pattern.len() as i32;
    let cl: i32 = contents.len() as i32;

    // Vectors to index into
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = contents.chars().collect();

    // Preprocessing functions
    let badchar = badchar_table(&pat);

    let shift = vec![0; (pl + 1) as usize];
    let bpos = vec![0; (pl + 1) as usize];
    let (shift, bpos) = strong_suffix_table(&pat, shift, bpos);
    let (shift, _bpos) = process_case_2(&pat, shift, bpos);

    // Searching

    let mut locs: Vec<i32> = Vec::new(); //Store where we found the pattern

    let mut i: i32 = 0; // The position of the pattern
    let mut j: i32; // The position we're looking at in the pattern
    while i <= cl - pl {
        j = (pl - 1) as i32;

        // If we're matching:
        while j >= 0 && pat[j as usize] == txt[(i + j) as usize] {
            j -= 1;
        }

        if j < 0 {
            // i.e. everything matches
            // println!("Pattern found at shift {}", i);

            locs.push(i);

            // Move the pattern

            // Bad char options:
            let bc = if i + pl < cl {
                pl - badchar[txt[(i + pl) as usize] as usize]
            } else {
                1
            };

            // Good suffix options:
            let gs = shift[0];

            let inc = cmp::max(bc, gs);

            i += inc;
        } else {
            // No match :(

            // Bad character options:
            let bc = cmp::max(1, j - badchar[txt[(i + j) as usize] as usize]);

            // Good suffix options:
            let gs = shift[j as usize + 1];

            i += cmp::max(bc, gs);
        }
    }

    locs
}

pub fn indices_to_lines<'a>(indices: Vec<i32>, contents: &'a str) -> Vec<&'a str> {
    let mut l_nums = Vec::new();

    for index in indices {
        let line_no = contents[..index as usize]
            .chars()
            .filter(|x| *x == '\n')
            .count();

        l_nums.push(line_no);
    }

    let lines: Vec<&str> = contents.lines().collect();

    let out: Vec<&str> = l_nums.into_iter().map(|l_num| lines[l_num]).collect();

    out
}

pub fn boyer_moore_lines<'a>(pattern: &str, contents: &'a str) -> Vec<&'a str> {
    let indices = boyer_moore_search(pattern, contents);
    let lines = indices_to_lines(indices, contents);

    lines
}

pub fn boyer_moore_lines_case_insensitive<'a>(pattern: &str, contents: &'a str) -> Vec<&'a str> {
    let indices = boyer_moore_search(&pattern.to_lowercase(), &contents.to_lowercase());
    let lines = indices_to_lines(indices, contents);

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }

    #[test]
    fn boyer_moore() {
        // let query = "ABC";
        // let contents = "ABAAABCDABC";

        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(
            vec!["safe, fast, productive."],
            boyer_moore_lines(query, contents)
        );
    }

    #[test]
    fn boyer_moore_case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            boyer_moore_lines_case_insensitive(query, contents)
        );
    }
}
