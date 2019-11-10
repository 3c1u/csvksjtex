/*
 * 早稲田大学理工学クソ実験用 CSV->TeXツール
 *
 */

use clap::{App, Arg};
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
                ('0' <= c && c <= '9') || (c == '.') || (c == '-') || (c == '+')
            }) >>
            tag!("E") >>
            s2: take_while!(|c: char| {
                ('0' <= c && c <= '9') || (c == '-') || (c == '+')
            }) >>
            (
                KsjCsvEntry::Expn(String::from(s1.0), String::from(format!("{}",s2.0.parse::<i32>().unwrap())))
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
    let authors = &*(env!("CARGO_PKG_AUTHORS")
        .split(':')
        .collect::<Vec<_>>()
        .join(", "));

    let version_message = &*format!(
        "version {}
Copyright (c) 2019 Hikaru Terazono. All rights reserved.",
        env!("CARGO_PKG_VERSION"),
    );

    let app = App::new("CSV to KSJ format")
        .version_short("v")
        .version(version_message)
        .help_message(
            "Prints helps information. \n\
             Use --help for more detailed information.",
        )
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(authors)
        .arg(Arg::with_name("INPUT").help("Sets the input file to process"))
        .arg(
            Arg::with_name("OUTPUT")
                .short("o")
                .long("output")
                .value_name("TEX")
                .help("Sets a custom output file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("TITLE")
                .short("t")
                .long("title")
                .value_name("TILTE")
                .help("Sets a custom title")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("LABEL")
                .short("l")
                .long("label")
                .value_name("LABEL")
                .help("Sets a custom label")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("DENSEI")
                .short("d")
                .long("densei")
                .help("Outputs with Densei Jikken format."),
        )
        .get_matches();

    // read CSV from stdin
    let mut rdr = csv::Reader::from_reader(
        app.value_of("INPUT")
            .map(std::fs::File::open)
            .and_then(Result::<_, _>::ok)
            .map(|b| Box::new(b) as Box<dyn std::io::Read>)
            .unwrap_or_else(|| Box::new(io::stdin())),
    );

    let mut wrt = app
        .value_of("OUTPUT")
        .map(std::fs::File::create)
        .and_then(Result::<_, _>::ok)
        .map(|b| Box::new(b) as Box<dyn std::io::Write>)
        .unwrap_or_else(|| Box::new(io::stdout()));

    let title = rdr.headers().expect("Failed to read CSV.");
    let title: Vec<_> = title.iter().collect();

    let mut table_fmt = Vec::<char>::with_capacity(title.len());
    if app.is_present("DENSEI") {
        let last = title.len() - 1;

        for i in 0..title.len() {
            table_fmt.push('c');
            if i != last {
                table_fmt.push('|');
            }
        }
    } else {
        table_fmt.resize(title.len(), 'c');
    }

    let table_fmt: String = table_fmt.into_iter().collect();

    writeln!(
        wrt,
        "{}{}{}{}{}{}{}",
        r#"\begin{table}[!hb]
\begin{center}
\caption{"#,
        app.value_of("TITLE").unwrap_or("タイトル"),
        r#"}
\label{tab:"#,
        app.value_of("LABEL").unwrap_or("XXX"),
        r#"}
\begin{tabular}{"#,
        table_fmt,
        r#"} \hline"#
    )
    .unwrap();

    writeln!(wrt, "{}", title.join("&")).unwrap();
    writeln!(wrt, " \\\\ \\hline").unwrap();

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
            write!(wrt, "{}", v.join("&")).unwrap();
            writeln!(wrt, " \\\\").unwrap();
        }
    }

    writeln!(
        wrt,
        "{}",
        r#"\hline
\end{tabular}
\end{center}
\end{table}"#
    )
    .unwrap();
}
