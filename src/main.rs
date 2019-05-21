/*
 * 早稲田大学理工学クソ実験用 CSV->TeXツール
 *
 */

use csv;
use std::io;

#[macro_use]
extern crate nom;
use nom::types::CompleteStr;

trait Flatten<T> {
    fn flatten(self) -> Option<T>;
}

impl<T, _E> Flatten<T> for Option<Result<T, _E>> {
    fn flatten(self) -> Option<T> {
        match self {
            Some(Ok(v)) => Some(v),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum KsjCsvEntry {
    Str(String),
    Expn(String, String),
}

named!(
    ksj_csv_entry_str<CompleteStr, KsjCsvEntry>,
    ws!(
        map!(take_while!(|_| true), |c| {
            KsjCsvEntry::Str(String::from(c.trim()))
        })
    )
);

named!(
    ksj_csv_entry_expn<CompleteStr, KsjCsvEntry>,
    ws!(
        do_parse!(
            s1: take_while!(|c: char| {
                ('0' <= c && c <= '9') || (c == '.') || (c == '-')
            }) >>
            tag!("E") >>
            s2: take_while!(|c: char| {
                ('0' <= c && c <= '9') || (c == '-')
            }) >>
            (
                KsjCsvEntry::Expn(String::from(s1.0), String::from(s2.0))
            )
        )
    )
);

named!(
    ksj_csv_entry<CompleteStr, KsjCsvEntry>,
    alt!(
        ksj_csv_entry_expn | ksj_csv_entry_str
    )
);

fn main() {
    // read CSV from stdin
    let mut rdr = csv::Reader::from_reader(io::stdin());

    let title = rdr.headers().expect("Failed to read CSV.");
    let title: Vec<_> = title.iter().collect();

    let mut table_fmt = Vec::<u8>::with_capacity(title.len());
    table_fmt.resize(title.len(), 'c' as u8);

    let table_fmt = std::str::from_utf8(&table_fmt[..]).unwrap();

    println!(
        "{}{}{}",
        r#"\begin{table}[!hb]
\begin{center}
\caption{タイトル}
\label{tab:XXX}
\begin{tabular}{"#,
        table_fmt,
        r#"} \hline"#
    );

    print!("{}", title.join("&"));
    println!(" \\\\ \\hline");

    let records = rdr.records();

    for res in records {
        if let Ok(res) = res {
            let mut v = Vec::<String>::new();
            for s in res.iter() {
                if let Ok((_, val)) = ksj_csv_entry(CompleteStr::from(s)) {
                    match val {
                        KsjCsvEntry::Str(s) => v.push(s),
                        KsjCsvEntry::Expn(s1, s2) => {
                            v.push(format!("${} \\times 10^{{{}}}$", s1, s2))
                        }
                        _ => {}
                    }
                }
            }
            print!("{}", v.join("&"));
            println!(" \\\\");
        }
    }

    println!(
        "{}",
        r#"\hline
\end{tabular}
\end{center}
\end{table}"#
    );
}
