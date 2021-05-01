//  * This file is part of the uutils coreutils package.
//  *
//  * (c) Michael Gehring <mg@ebfe.org>
//  * (c) kwantam <kwantam@gmail.com>
//  *     * 2015-04-28 ~ created `expand` module to eliminate most allocs during setup
//  * (c) Sergey "Shnatsel" Davidoff <shnatsel@gmail.com>
//  *
//  * For the full copyright and license information, please view the LICENSE
//  * file that was distributed with this source code.

// spell-checker:ignore (ToDO) allocs bset dflag cflag sflag tflag

#[macro_use]
extern crate uucore;

mod expand;

use bit_set::BitSet;
use clap::{App, Arg};
use fnv::FnvHashMap;
use std::io::{stdin, stdout, BufRead, BufWriter, Write};

use crate::expand::ExpandSet;
use uucore::InvalidEncodingHandling;

static NAME: &str = "tr";
static VERSION: &str = env!("CARGO_PKG_VERSION");
static ABOUT: &str = "translate or delete characters";
static LONG_HELP: &str = "Translate,  squeeze, and/or delete characters from standard input,
writing to standard output.";
const BUFFER_LEN: usize = 1024;

mod options {
    pub const COMPLEMENT: &str = "complement";
    pub const DELETE: &str = "delete";
    pub const SQUEEZE: &str = "squeeze-repeats";
    pub const TRUNCATE: &str = "truncate";
    pub const SETS: &str = "sets";
}

fn get_usage() -> String {
    format!("{} [OPTION]... SET1 [SET2]", executable!())
}


fn delete<'a>(set1: &'a str, complement: bool, s: &'a str) -> String {
    let set1_ = ExpandSet::new(set1.as_ref());
    let bset: BitSet = set1_.map(|c| c as usize).collect();
    let delete = |c: &char| (complement == bset.contains(*c as usize));
    s.chars().filter(delete).collect()
}


fn squeeze<'a>(set1: &'a str, complement: bool, s: &'a str) -> String {

    let set1_ = ExpandSet::new(set1.as_ref());
    let squeeze_set: BitSet = set1_.map(|c| c as usize).collect();

    // Define a closure that computes the squeeze operation.
    //
    // We keep track of the previously seen character on
    // each call to `squeeze()`, but we need to reset the
    // `prev_c` variable at the beginning of each line of
    // the input. That's why we define the closure inside
    // the `while` loop.
    let mut prev_c = 0 as char;
    let squeeze = |c| {
        let result = if prev_c == c && complement != squeeze_set.contains(c as usize) {
            None
        } else {
            Some(c)
        };
        prev_c = c;
        result
    };

    // First translate, then squeeze each character of the input line.
    s.chars().filter_map(squeeze).collect()
}


fn translate<'a>(set1: &'a str, set2: &'a str, truncate: bool, s: &'a str) -> String {

    let set1_ = ExpandSet::new(set1.as_ref());
    let mut set2_ = ExpandSet::new(set2.as_ref());

    let mut map = FnvHashMap::default();
    let mut s2_prev = '_';
    for i in set1_ {
        let s2_next = set2_.next();

        if s2_next.is_none() && truncate {
            map.insert(i as usize, i);
        } else {
            s2_prev = s2_next.unwrap_or(s2_prev);
            map.insert(i as usize, s2_prev);
        }
    }

    let f = |c| *map.get(&(c as usize)).unwrap_or(&c);

    s.chars().map(f).collect()
}

pub fn uumain(args: impl uucore::Args) -> i32 {
    let usage = get_usage();
    let args = args
        .collect_str(InvalidEncodingHandling::ConvertLossy)
        .accept_any();

    let matches = App::new(executable!())
        .version(VERSION)
        .about(ABOUT)
        .usage(&usage[..])
        .after_help(LONG_HELP)
        .arg(
            Arg::with_name(options::COMPLEMENT)
                .short("C")
                .short("c")
                .long(options::COMPLEMENT)
                .help("use the complement of SET1"),
        )
        .arg(
            Arg::with_name(options::DELETE)
                .short("d")
                .long(options::DELETE)
                .help("delete characters in SET1, do not translate"),
        )
        .arg(
            Arg::with_name(options::SQUEEZE)
                .long(options::SQUEEZE)
                .short("s")
                .help(
                    "replace each sequence  of  a  repeated  character  that  is
            listed  in the last specified SET, with a single occurrence
            of that character",
                ),
        )
        .arg(
            Arg::with_name(options::TRUNCATE)
                .long(options::TRUNCATE)
                .short("t")
                .help("first truncate SET1 to length of SET2"),
        )
        .arg(Arg::with_name(options::SETS).multiple(true))
        .get_matches_from(args);

    let delete_flag = matches.is_present(options::DELETE);
    let complement_flag = matches.is_present(options::COMPLEMENT);
    let squeeze_flag = matches.is_present(options::SQUEEZE);
    let truncate_flag = matches.is_present(options::TRUNCATE);

    let sets: Vec<String> = match matches.values_of(options::SETS) {
        Some(v) => v.map(|v| v.to_string()).collect(),
        None => vec![],
    };

    if sets.is_empty() {
        show_error!(
            "missing operand\nTry `{} --help` for more information.",
            NAME
        );
        return 1;
    }

    if !(delete_flag || squeeze_flag) && sets.len() < 2 {
        show_error!(
            "missing operand after ‘{}’\nTry `{} --help` for more information.",
            sets[0],
            NAME
        );
        return 1;
    }

    if complement_flag && !delete_flag && !squeeze_flag {
        show_error!("-c is only supported with -d or -s");
        return 1;
    }

    let stdin = stdin();
    let mut locked_stdin = stdin.lock();
    let stdout = stdout();
    let locked_stdout = stdout.lock();
    let mut buffered_stdout = BufWriter::new(locked_stdout);

    let f = |s: &str| {
        if delete_flag {
            if squeeze_flag {
                squeeze(&sets[1], complement_flag, &delete(&sets[0], complement_flag, s))
            } else {
                delete(&sets[0], complement_flag, s)
            }
        } else if squeeze_flag {
            if sets.len() < 2 {
                squeeze(&sets[0], complement_flag, s)
            } else {
                squeeze(&sets[1], complement_flag, &translate(&sets[0], &sets[1], truncate_flag, s))
            }
        } else {
            translate(&sets[0], &sets[1], truncate_flag, s)
        }
    };

    // Prepare some memory to read each line of the input (`buf`).
    let mut buf = String::with_capacity(BUFFER_LEN + 4);

    // Loop over each line of stdin.
    while let Ok(length) = locked_stdin.read_line(&mut buf) {
        if length == 0 {
            break;
        }

        let filtered = f(&buf);
        buf.clear();
        buffered_stdout.write_all(filtered.as_bytes()).unwrap();
    }

    0
}
