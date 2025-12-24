//
// fwtype --- fix width typing for TeX picture-env.
//
// this code does not support combinatin of UTF-8.
// you should use 'nkf' or simular programs:
//      $ nkf --ic=utf8-mac --oc=utf-8 in.txt |thisprogram > out.tex
//

use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use unicode_segmentation::UnicodeSegmentation;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[inline]
fn git_hash() -> &'static str {
    option_env!("GIT_HASH").unwrap_or("")
}

#[inline]
fn build_date() -> &'static str {
    option_env!("BUILD_DATE").unwrap_or("")
}

fn long_version() -> String {
    let h = git_hash();
    let d = build_date();
    match (h.is_empty(), d.is_empty()) {
        (true, true) => VERSION.to_string(),
        (false, true) => format!("{VERSION} ({h})"),
        (true, false) => format!("{VERSION} [{d}]"),
        (false, false) => format!("{VERSION} ({h}) [{d}]"),
    }
}

pub fn long_version_static() -> &'static str {
    Box::leak(long_version().into_boxed_str())
}

pub const DIME_AUTO: usize = 99999;
/*
const CSIZE_DEFAULT: &str = "10x5";
*/
/*
const WMIN_DEFAULT: &str = "0";
*/
const WMAX_DEFAULT: usize = 53;
const LMAX_DEFAULT: usize = 9999;

fn dime_auto_str() -> &'static str {
    Box::leak(DIME_AUTO.to_string().into_boxed_str())
}

type MyResult<T> = Result<T, Box<dyn Error>>;

pub fn last_number<T: std::str::FromStr>(matches: &clap::ArgMatches, name: &str, default: T) -> T {
    matches
        .values_of(name)
        .and_then(|mut v| v.next_back())
        .and_then(|s| s.parse::<T>().ok())
        .unwrap_or(default)
}

pub fn last_string<'a>(matches: &'a clap::ArgMatches, name: &str, default: &'a str) -> String {
    matches
        .values_of(name)
        .and_then(|mut v| v.next_back())
        .unwrap_or(default)
        .to_string()
}

#[derive(Debug, Clone)]
struct CharSize {
    width: usize,
    height: usize,
}

impl PartialEq for CharSize {
    fn eq(&self, other: &Self) -> bool {
        self.height == other.height && self.width == other.width
    }
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    _Text(String),
    Escape(String),
    Ascii(String),
    Misc(String),
    Hole(String),
    Skip,
    _Nop,
}

#[derive(Debug, Clone)]
pub struct Token {
    kind: TokenKind,
    width: isize,
}

impl TokenKind {
    pub fn fmt(&self) -> String {
        match &self {
            TokenKind::_Text(s) => format!("t{:?}", s),
            TokenKind::Escape(s) => format!("e{:?}", s),
            TokenKind::Ascii(s) => format!("a{:?}", s),
            TokenKind::Misc(s) => format!("m{:?}", s),
            TokenKind::Hole(s) => format!("h{:?}", s),
            TokenKind::Skip => "skip".to_string(),
            TokenKind::_Nop => "nop".to_string(),
        }
    }
}

/*
*/
impl Token {
    pub fn fmt(&self) -> String {
        let k = &self.kind.fmt();
        format!("{}{}", k, &self.width)
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    lineno: isize,
    width: isize,
    setret: bool,
    tokens: Vec<Token>,
}

impl Row {
    pub fn clear(&mut self) {
        self.lineno = -1;
        self.width = -1;
        self.setret = false;
        self.tokens.clear();
    }
    pub fn fmt(self) -> String {
        let pre = format!("{:5}:{:<3}", self.lineno, self.width);
        let mark = if self.setret { "*" } else { "." };
        let tks: Vec<String> = self.tokens.iter().map(|t| t.fmt()).collect::<Vec<_>>();
        let joined = tks.join(" ");
        format!("{} {} {}", pre, mark, joined)
    }
    fn calcwidth(&mut self) {
        let mut sum: isize = 0;
        for tk in &self.tokens {
            sum += tk.width;
        }
        self.width = sum;
    }
}

type RowChunk = Vec<Row>;

/*
fn fmt_row(row: &Row) -> String {
    row.tokens.iter()
        .map(|t| t.fmt())
        .collect::<Vec<_>>()
        .join(",")
}
*/

/*
fn eprint_row(prefix: &str, row: &Row) {
    if prefix != "" {
        eprint!("{} ", prefix);
    }
    let wstr = format!("<{}>", row.width);
    eprint!("{:>5} ", wstr);
    let rmb = row.tokens.iter()
                .map(|t| t.fmt())
                .collect::<Vec<_>>()
                .join(",");
    eprintln!("|{}|", rmb);
}

fn eprint_chunk(prefix: &str, chunk: &RowChunk) {
    for (i, row) in chunk.iter().enumerate() {
        if prefix != "" {
            eprint!("{} ", prefix);
        }
        let pre = format!("{:>3}", i);
        eprint_row(&pre, row);
    }
}
*/

/*
fn fmt_rowchunk(chunk: &RowChunk) -> String {
    chunk.iter()
        .enumerate()
        .map(|(i, row)| format!("Row {}: {}", i, fmt_row(row)))
        .collect::<Vec<_>>()
        .join("\n")
}
*/

/*
fn view_chunk(prefix: &str, chunk: &RowChunk) {
    let mut i = 0;
    for r in chunk {
        let dmy = r.clone().fmt();
        eprintln!("{} {} |{}|", prefix, i, dmy);
        i += 1;
    }
}
*/

fn parse_line(rawstr: &str, tabstop: isize, wcolumn: isize) -> RowChunk {
    let mut iter = rawstr.graphemes(true).peekable();
    let mut c = 0;
    let mut x = 0;
    let mut _y = 0;
    let mut tk: Token;
    let mut rchk: RowChunk = Vec::new();
    let mut currow: Row = Row {
        lineno: -1,
        width: -1,
        setret: false,
        tokens: Vec::new(),
    };
    loop {
        let Some(q) = iter.next() else { break };

        if q.is_ascii() {
            if q == "\t" {
                let nx = ((x) / tabstop) * tabstop + tabstop;
                tk = Token {
                    kind: TokenKind::Skip,
                    width: nx - x,
                };
            } else if q == "\x1b" {
                let mut seq = String::from(q);
                while let Some(&next) = iter.peek() {
                    seq.push_str(next);
                    iter.next();
                    if next.ends_with('m') {
                        break;
                    }
                }
                tk = Token {
                    kind: TokenKind::Escape(seq),
                    width: 1,
                };
            } else if q == "\x1d" {
                let Some(q2) = iter.next() else { break };
                tk = Token {
                    kind: TokenKind::Hole(q2.to_string()),
                    width: 4,
                };
            } else {
                tk = Token {
                    kind: TokenKind::Ascii(q.to_string()),
                    width: 1,
                };
            }
        } else {
            tk = Token {
                kind: TokenKind::Misc(q.to_string()),
                width: 2,
            };
        }

        if x + tk.width > wcolumn {
            currow.setret = true;
            currow.calcwidth();
            rchk.push(currow.clone());

            currow.tokens.clear();
            currow.clear();
            x = 0;

            _y += 1;
        }
        x += tk.width;
        currow.tokens.push(tk);

        c += 1;
    }

    if (!currow.tokens.is_empty()) || c == 0 {
        currow.calcwidth();
        rchk.push(currow.clone());

        currow.tokens.clear();

        _y += 1;
    }

    rchk
}

#[derive(Debug)]
pub struct Param {
    font: String,
    csize: CharSize,
    numcsize: CharSize,
    lheight: usize,
    inmargin: usize,
    sepmargin: usize,
    frames: usize,
    tabstop: usize,
    wmin: usize,
    wmax: usize,
    lmin: usize,
    lmax: usize,
    braise: isize,
    gridpitch: String,
    ghpitch: usize,
    gvpitch: usize,
    raise: String,
    outmargin: String,
    abovegap: String,
    belowgap: String,
    leftgap: String,
    rightgap: String,
    lnooffset: usize,
    lnowidth: usize,
    //
    spcmarking: bool,
    grid: bool,
    numbering: bool,
    standalone: bool,
    pagebreaking: bool,
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    params: Param,
}

fn arg_usize(
    name: &'static str,
    short: &'static str,
    long: &'static str,
    desc: &'static str,
    default: usize,
) -> Arg<'static, 'static> {
    let def_str: &'static str = Box::leak(default.to_string().into_boxed_str());
    //    let help: &'static str = Box::leak(format!("{desc} [default: {default}]").into_boxed_str());

    Arg::with_name(name)
        .short(short)
        .long(long)
        .takes_value(true)
        .default_value(def_str)
        .help(desc)
}

// --------------------------------------------------
pub fn get_args() -> MyResult<Config> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(long_version_static())
        .author("Ken-ichi Chinen https://github.com/k-chinen/fwtype")
        .about("generate fixed-width printing for LaTeX from plain-text")
        .after_help(
            r#"EXAMPLES:
    % fwtype input.txt
    % fwtype -n -u input.txt
    % fwtype -w 80 input.txt
    % fwtype -l 50 -p -n input.txt          # make picture by 50 lines per page
    % fwtype -g -G 4 src/*.txt
    % fwtype -M 12pt intput.txt
    % fwtype -U3\\lh input.txt              # raise 3 lines height
    % fwtype -S input.txt > output.tex"#,
        )
        .arg(
            Arg::with_name("lnooffset")
                .short("N")
                .long("lnooffset")
                .takes_value(true)
                .help("linenumber offset")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("lnowidth")
                .long("lnowidth")
                .takes_value(true)
                .help("linenumber width")
                .default_value(dime_auto_str()),
        )
        .arg(
            Arg::with_name("outmargin")
                .short("M")
                .long("outmargin")
                .takes_value(true)
                .help("out margin by dimen; all/t,b/l,t,r,b (replace _ into -)")
                .multiple(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("raise")
                .short("U")
                .long("raise")
                .takes_value(true)
                .help("raise picture by dimen; e.g., 12pt or 1cm")
                .multiple(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("tabstop")
                .short("t")
                .long("tabstop")
                .takes_value(true)
                .help("tabstop")
                .default_value("8"),
        )
        .arg(
            Arg::with_name("gridpitch")
                .short("G")
                .long("gridpitch")
                .takes_value(true)
                .help("grid pitch by pt; h,v")
                .multiple(true)
                .number_of_values(1),
        )
        /*
                .arg(
                    Arg::with_name("wmin")
                        .short("K")
                        .long("wmin")
                        .takes_value(true)
                        .help("width mininum")
                        .default_value("0"),
                )
                .arg(
                    Arg::with_name("lmin")
                        .short("k")
                        .long("lmin")
                        .takes_value(true)
                        .help("line minimum")
                        .default_value("0"),
                )
        */
        .arg(
            Arg::with_name("wmin")
                .short("K")
                .long("wmin")
                .takes_value(true)
                .help("width min per picture")
                .multiple(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("lmin")
                .short("k")
                .long("lmin")
                .takes_value(true)
                .help("line min per picture")
                .multiple(true)
                .number_of_values(1),
        )
        /*
        .arg(
            Arg::with_name("lmax")
                .short("l")
                .long("lmax")
                .takes_value(true)
                .help("line limit per picture")
                .default_value("9999"),
        )
        */
        .arg(arg_usize(
            "lmax",
            "l",
            "lmax",
            "line max; line per picture",
            LMAX_DEFAULT,
        ))
        .arg(arg_usize(
            "wmax",
            "w",
            "wmax",
            "width max; column per line",
            WMAX_DEFAULT,
        ))
        .arg(
            Arg::with_name("braise")
                .short("b")
                .long("braise")
                .takes_value(true)
                .help("baseline raise for ASCII")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("inmargin")
                .short("m")
                .long("inmargin")
                .takes_value(true)
                .help("inner margin width and height by pt")
                .multiple(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("sepmargin")
                .short("s")
                .long("sepmargin")
                .takes_value(true)
                .help("sep margin width")
                .default_value("2"),
        )
        .arg(
            Arg::with_name("lheight")
                .short("H")
                .long("lheight")
                .takes_value(true)
                .help("lheight; if not specified csize *1.2")
                .default_value(dime_auto_str()),
        )
        .arg(
            Arg::with_name("csize")
                .short("c")
                .long("csize")
                .takes_value(true)
                .help("character size by pt; e.g., 17 or 20x10")
                .default_value("10x5"),
        )
        .arg(
            Arg::with_name("font")
                .short("F")
                .long("font")
                .takes_value(true)
                .help("base font")
                .default_value("\\ttfamily\\gtfamily"),
        )
        .arg(
            Arg::with_name("numcsize")
                /*
                                .short("C")
                */
                .long("numcsize")
                .takes_value(true)
                .help("character size of line numbers by pt; e.g., 12x6")
                .default_value("6x3"),
        )
        .arg(
            Arg::with_name("grid")
                .short("g")
                .long("grid")
                .takes_value(false)
                .help("Enable grid. See -G"),
        )
        .arg(
            Arg::with_name("spcmarking")
                .short("u")
                .long("spcmarking")
                .takes_value(false)
                .help("Space marking by triangle"),
        )
        .arg(
            Arg::with_name("numbering")
                .short("n")
                .long("numbering")
                .takes_value(false)
                .help("Line numbering"),
        )
        .arg(
            Arg::with_name("pagebreaking")
                .short("p")
                .long("pagebreaking")
                .takes_value(false)
                .help("Insert a page break after each picture. See -l"),
        )
        .arg(
            Arg::with_name("standalone")
                .short("S")
                .long("standalone")
                .takes_value(false)
                .help("Insert preamble and begin/end document in first"),
        )
        .arg(
            Arg::with_name("frames")
                .short("f")
                .long("frames")
                .takes_value(true)
                .help("set of frames <default 15>")
                .multiple(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .get_matches();

    if matches.is_present("version") {
        // clap が自動で処理します
    }

    let csize = matches
        .value_of("csize")
        .map(parse_2d_int)
        .transpose()
        .map_err(|e| format!("illegal csize -- {}", e))?;

    let numcsize = matches
        .value_of("numcsize")
        .map(parse_2d_int)
        .transpose()
        .map_err(|e| format!("illegal numcsize -- {}", e))?;

    let mut lheight = matches
        .value_of("lheight")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal lheight -- {}", e))?;

    let lnooffset = matches
        .value_of("lnooffset")
        .map(parse_uint)
        .transpose()
        .map_err(|e| format!("illegal lnooffset -- {}", e))?;

    let lnowidth = matches
        .value_of("lnowidth")
        .map(parse_uint)
        .transpose()
        .map_err(|e| format!("illegal lnowidth -- {}", e))?;

    let tabstop = matches
        .value_of("tabstop")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal tabstop -- {}", e))?;

    /*
        let wmin = Some(0);
        let lmin = Some(0);
    */
    /*
        let wmin = matches
            .value_of("wmin")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal wmin -- {}", e))?;

        let lmin = matches
            .value_of("lmin")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal lmin -- {}", e))?;
    */

    let lmax = matches
        .value_of("lmax")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal lmax -- {}", e))?;

    /*
        let wmax = matches
            .value_of("wmax")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal wmax -- {}", e))?;
    */

    let braise = matches
        .value_of("braise")
        .map(parse_int)
        .transpose()
        .map_err(|e| format!("illegal braise  -- {}", e))?;

    /*
        let inmargin = matches
            .value_of("inmargin")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal inmargin width -- {}", e))?;
    */

    let sepmargin = matches
        .value_of("sepmargin")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal sepmargin width -- {}", e))?;

    /*
        let gvpitch = matches
            .value_of("gvpitch")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal grid v pitch -- {}", e))?;

        let ghpitch = matches
            .value_of("ghpitch")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal grid h pitch -- {}", e))?;
    */

    /*
        let frames = matches
            .value_of("frames")
            .map(parse_positive_int)
            .transpose()
            .map_err(|e| format!("illegal frame -- {}", e))?;
    */

    // not specified, set automatically csize.height * 1.2
    if lheight == Some(DIME_AUTO) {
        lheight = Some((csize.clone().unwrap().height * 12) / 10);
    }
    if lnowidth == Some(DIME_AUTO) {
        eprintln!("auto lnowidth");
        //        lnowidth = Some((csize.clone().unwrap().height*12)/10);
    }

    let mut param = Param {
        font: matches.value_of("font").unwrap().to_string(),
        csize: csize.unwrap(),
        numcsize: numcsize.unwrap(),
        lheight: lheight.unwrap(),
        //        inmargin: inmargin.unwrap(),
        inmargin: last_number(&matches, "inmargin", 5usize),
        sepmargin: sepmargin.unwrap(),
        tabstop: tabstop.unwrap(),
        raise: last_string(&matches, "raise", ""),
        //        wmax: wmax.unwrap(),
        wmax: last_number(&matches, "wmax", WMAX_DEFAULT),
        //      wmin: wmin.unwrap(),
        wmin: last_number(&matches, "wmin", 0usize),
        lmax: lmax.unwrap(),
        lmin: last_number(&matches, "lmin", 0usize),
        //      lmin: 0,
        braise: braise.unwrap() as isize,
        frames: last_number(&matches, "frames", 15usize),
        gridpitch: last_string(&matches, "gridpitch", ""),
        ghpitch: 0,
        gvpitch: 0,
        outmargin: last_string(&matches, "outmargin", ""),
        leftgap: "".to_string(),
        abovegap: "".to_string(),
        rightgap: "".to_string(),
        belowgap: "".to_string(),
        /*
                leftgap: last_string(&matches, "leftgap", ""),
                abovegap: last_string(&matches, "abovegap", ""),
                rightgap: last_string(&matches, "rightgap", ""),
                belowgap: last_string(&matches, "belowgap", ""),
        */
        lnooffset: lnooffset.unwrap(),
        lnowidth: lnowidth.unwrap(),
        grid: matches.is_present("grid"),
        spcmarking: matches.is_present("spcmarking"),
        numbering: matches.is_present("numbering"),
        standalone: matches.is_present("standalone"),
        pagebreaking: matches.is_present("pagebreaking"),
    };

    if !param.gridpitch.is_empty() {
        let parts: Vec<_> = param.gridpitch.split(",").collect();
        eprintln!("gpitch before {:?}", (&param.ghpitch, &param.gvpitch));
        match parts.len() {
            2 => {
                param.ghpitch = parts[0].parse()?;
                param.gvpitch = parts[1].parse()?;
            }
            1 => {
                param.ghpitch = parts[0].parse()?;
                param.gvpitch = parts[0].parse()?;
            }
            _ => {}
        }
        eprintln!("gpitch after  {:?}", (&param.ghpitch, &param.gvpitch));
    }

    if !param.outmargin.is_empty() {
        let cooked: String = param.outmargin.replace("_", "-");
        let parts: Vec<_> = cooked.split(",").collect();
        eprintln!("parts {:?}", parts);
        eprintln!(
            "gaps before {:?}",
            (
                &param.leftgap,
                &param.abovegap,
                &param.rightgap,
                &param.belowgap
            )
        );
        match parts.len() {
            4 => {
                param.leftgap = parts[0].to_string();
                param.abovegap = parts[1].to_string();
                param.rightgap = parts[2].to_string();
                param.belowgap = parts[3].to_string();
            }
            2 => {
                param.abovegap = parts[0].to_string();
                param.belowgap = parts[1].to_string();
            }
            1 => {
                param.leftgap = parts[0].to_string();
                param.abovegap = parts[0].to_string();
                param.rightgap = parts[0].to_string();
                param.belowgap = parts[0].to_string();
            }
            _ => {}
        }
        eprintln!(
            "gaps after  {:?}",
            (
                &param.leftgap,
                &param.abovegap,
                &param.rightgap,
                &param.belowgap
            )
        );
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        params: param,
    })
}

#[derive(Debug, Clone)]
struct Geo {
    nchars: isize,
    ndigits: isize,
    txoffset: isize,
    txwidth: isize,
    txwmin: isize,
    cvwidth: isize,
    cvheight: isize,
    cvhmin: isize,
}

fn print_picture(chunk: RowChunk, lnooffset: usize, _crow: isize, parent_geo: &Geo, param: &Param) {
    let cmdchars = r"#$%&^_{}\\~";
    let cvheight: isize = (chunk.len() * param.lheight + param.inmargin * 2) as isize;

    let mut geo: Geo = parent_geo.clone();
    geo.cvheight = cvheight; /* overwrite by current picture's height */
    let gheight = if geo.cvhmin > cvheight {
        geo.cvhmin
    } else {
        cvheight
    };
    eprintln!("gheight {}", gheight);

    eprintln!("lnooffset {}", lnooffset);

    println!();

    println!("%% you should use \\usepackage[T1]{{fontenc}}");
    println!("{{%");

    println!("{}%", param.font);
    //  println!("\\fboxsep=-.5pt%");
    println!("\\setlength{{\\unitlength}}{{1pt}}%");
    println!(
        "% csize w,h={}, {}; lheight {}",
        param.csize.width, param.csize.height, param.lheight
    );
    //  println!("\\def\\lh{{{}pt}}", param.lheight);
    println!("\\newdimen\\lh\\lh=12pt");
    println!(
        "\\fontsize{{{}pt}}{{{}pt}}\\selectfont%",
        param.csize.height, param.csize.height
    );
    println!(
        "\\def\\numfont{{\\fontsize{{{}pt}}{{{}pt}}\\selectfont}}%",
        param.numcsize.height, param.numcsize.height
    );
    /*
        println!("\\def\\spcmark{{\\fontsize{{{}pt}}{{{}pt}}\\selectfont$\\diamond$}}%",
            (2*param.csize.height/3), (2*param.csize.height/3) );
    */
    println!(
        "\\def\\hsp{{\\fontsize{{{}pt}}{{{}pt}}\\selectfont$\\triangle$}}%",
        (2 * param.csize.height / 3),
        (2 * param.csize.height / 3)
    );
    println!("\\def\\zsp{{▲}}");

    println!("\\def\\VV{{\\vrule width 0pt height 0.90em depth .25em}}%");
    println!(
        "\\def\\FA#1#2#3{{\\put(#1,#2){{\\makebox({},{}){{\\VV\\mbox{{#3}}}}}}}}%",
        param.csize.width, param.csize.height
    );
    println!(
        "\\def\\FX#1#2#3{{\\put(#1,#2){{\\makebox({},{}){{\\VV\\mbox{{#3}}}}}}}}%",
        param.csize.width, param.csize.height
    );
    println!(
        "\\def\\FR#1#2{{\\put(#1,#2){{\\makebox({},{}){{\\VV$\\triangleright$}}}}}}%",
        param.csize.width, param.csize.height
    );
    //    println!("\\begin{{picture}}({},{})", geo.cvwidth, geo.cvheight);
    println!(
        "\\setbox0\\hbox{{\\begin{{picture}}({},{})",
        geo.cvwidth, gheight
    );

    println!("% frame");

    println!("\\thicklines");

    if param.numbering {
        /*
            println!(" \\put(0,0){{\\circle*{{3}}}}");
            println!(" \\put({},0){{\\circle*{{3}}}}", inmargin);
            println!(" \\put({},0){{\\circle*{{3}}}}", inmargin+numwid);
            println!(" \\put({},0){{\\circle*{{3}}}}", inmargin+numwid+sepmargin);
            //
            for i in 0..=ndigits {
                println!(" \\put({},0){{\\line(0,1){{5}}}}",
                    inmargin+(i as usize)*(numcsize.width as usize) );
            }
        */
    }

    if param.frames == 0xf {
        /*
                println!(
                    " \\put({},0){{\\framebox({},{}){{}}}}",
                    geo.txoffset, geo.txwidth, geo.cvheight
                );
        */
        println!(
            " \\put({},{}){{\\framebox({},{}){{}}}}",
            geo.txoffset,
            gheight - geo.cvheight,
            geo.txwidth,
            geo.cvheight
        );
    } else {
        if (param.frames & 0x01) > 0 {
            println!(
                " \\put({},0){{\\line(0,1){{{}}}}}",
                geo.txoffset, geo.cvheight
            );
        }
        if (param.frames & 0x08) > 0 {
            println!(
                " \\put({},0){{\\line(1,0){{{}}}}}",
                geo.txoffset, geo.txwidth
            );
        }
        if (param.frames & 0x02) > 0 {
            println!(
                " \\put({},{}){{\\line(-1,0){{{}}}}}",
                geo.txoffset + geo.txwidth,
                geo.cvheight,
                geo.txwidth
            );
        }
        if (param.frames & 0x04) > 0 {
            println!(
                " \\put({},{}){{\\line(0,-1){{{}}}}}",
                geo.txoffset + geo.txwidth,
                geo.cvheight,
                geo.cvheight
            );
        }
    }

    println!("\\thinlines");

    if param.grid {
        println!("% grid");
        println!("\\linethickness{{0.1pt}}");

        for gx in 0..=geo.nchars {
            if gx % (param.ghpitch as isize) == 0 {
                println!(
                    "  \\put({},{}){{\\line(0,1){{{}}}}}",
                    geo.txoffset + (param.inmargin + (gx as usize) * param.csize.width) as isize,
                    gheight - geo.cvheight,
                    geo.cvheight //                    gheight
                );
            }
        }

        //        for gy in 0..=crow {
        for gy in 0..=(chunk.len() as isize) {
            if gy % (param.gvpitch as isize) == 0 {
                println!(
                    "  \\put({},{}){{\\line(1,0){{{}}}}}",
                    geo.txoffset,
                    //                    geo.cvheight - (param.inmargin + (gy as usize) * param.lheight) as isize,
                    gheight - (param.inmargin + (gy as usize) * param.lheight) as isize,
                    geo.txwidth
                );
            }
        }

        println!("\\thinlines");
    }

    println!("% body");

    let mut gline = 1;
    let mut gx: isize;
    let mut gy: isize;
    for r in chunk {
        //        gy = cvheight - (param.lheight * gline) as isize - param.inmargin as isize;
        gy = gheight - (param.lheight * gline) as isize - param.inmargin as isize;
        /*
        eprintln!("gline {} gy {}", gline, gy);
        */

        if param.numbering && r.lineno > 0 {
            let numstr = format!(
                "{:>width$}",
                param.lnooffset + r.lineno as usize,
                width = geo.ndigits as usize
            );
            for (c, ch) in numstr.chars().enumerate() {
                gx = (param.inmargin + param.numcsize.width * c) as isize;
                if ch != ' ' {
                    println!("{{\\numfont\\FA{{{}}}{{{}}}{{{}}}}}", gx, gy, ch);
                }
            }
        }

        gx = geo.txoffset + param.inmargin as isize;
        for tk in r.tokens {
            match tk.kind {
                TokenKind::Ascii(ch) => {
                    let mut och: String = "".to_string();
                    if let Some(_x) = cmdchars.find(&ch) {
                        if ch == "~" {
                            och.push_str("\\textasciitilde");
                        } else if ch == "^" {
                            och.push_str("\\textasciicircum");
                        } else if ch == "\\" {
                            och.push_str("\\textbackslash");
                        } else {
                            och.push('\\');
                            och.push_str(&ch);
                        }
                    } else if ch == " " {
                        if param.spcmarking {
                            println!(" \\FA{{{}}}{{{}}}{{\\hsp}}", gx, gy - param.braise);
                        }
                    } else {
                        och.push_str(&ch);
                    }

                    println!(" \\FA{{{}}}{{{}}}{{{}}}", gx, gy - param.braise, och);
                }
                TokenKind::Misc(ch) => {
                    if param.spcmarking && ch == "　" {
                        println!(
                            " \\FX{{{}}}{{{}}}{{\\zsp}}",
                            gx + (param.csize.width as isize) / 2,
                            gy
                        );
                    } else {
                        println!(
                            " \\FX{{{}}}{{{}}}{{{}}}",
                            gx + (param.csize.width as isize) / 2,
                            gy,
                            ch
                        );
                    }
                }
                TokenKind::Escape(_) => {}
                TokenKind::Skip => {}
                TokenKind::Hole(label) => {
                    println!(
                        " \\FA{{{}}}{{{}}}{{\\fbox{{\\hbox to 2em{{\\hss
\\VV{{}}{}\\hss}}}}}}",
                        gx + 3 * (param.csize.width as isize) / 2,
                        gy,
                        label
                    );
                }
                _ => {}
            }
            gx += tk.width * param.csize.width as isize;
        }

        if r.setret {
            println!(
                " \\FR{{{}}}{{{}}}",
                geo.txoffset + geo.txwidth - (param.inmargin as isize) / 2,
                gy
            );
        }

        gline += 1;
    }

    println!("\\end{{picture}}}}");
    println!("%");

    if !param.abovegap.is_empty() {
        println!("\\vspace*{{{}}}% above", param.abovegap);
    }
    println!("\\noindent%");
    if !param.leftgap.is_empty() {
        println!("\\hspace{{{}}}% left", param.leftgap);
    }
    if param.raise.is_empty() {
        println!("\\copy0%");
    } else {
        println!("\\raise{}\\copy0%", param.raise);
    }
    if !param.rightgap.is_empty() {
        println!("\\hspace{{{}}}% right", param.rightgap);
    }
    if !param.belowgap.is_empty() {
        println!("\\vspace*{{{}}}% below", param.belowgap);
    }

    println!("}}%");

    if param.pagebreaking {
        println!("\\newpage");
    }
    println!();
}

#[allow(clippy::too_many_arguments)]
fn fwtype(fp: &mut dyn BufRead, param: &Param) {
    let mut maxwidth = 0;

    let mut geo = Geo {
        nchars: 0,
        ndigits: 0,
        txoffset: 0,
        txwidth: 0,
        txwmin: 0,
        cvwidth: 0,
        cvheight: 0,
        cvhmin: 0,
    };

    eprintln!(
        "numbering {} width {} offset {}",
        param.numbering, param.lnowidth, param.lnooffset
    );

    let mut fullrow: RowChunk = Vec::new();

    /*
     * phase 1: estimate columns and rows
     */
    /*
        linec = 0;
        rowc = 0;
    */

    let mut cline: isize = 0;
    let mut crow: isize = 0;

    for line_result in fp.lines() {
        let line = line_result.unwrap();
        /*
        eprintln!("; line |{}|", line);
        */

        let chunk = parse_line(&line, param.tabstop as isize, param.wmax as isize);
        /*
        eprintln!("; {} chunk {:?}", _line_num, chunk);
        */
        cline += 1;
        for (r_per_i, mut x) in chunk.into_iter().enumerate() {
            if x.width > maxwidth {
                maxwidth = x.width;
            }
            if r_per_i == 0 {
                x.lineno = cline;
            }
            fullrow.push(x);
            crow += 1;
        }
    }
    geo.nchars = maxwidth;

    /*
        view_chunk("full", &fullrow);
    */

    if param.lnowidth == DIME_AUTO {
        geo.ndigits = if crow <= 0 {
            1
        } else {
            (cline + param.lnooffset as isize).ilog10() + 1
        } as isize;
    } else {
        geo.ndigits = param.lnowidth as isize;
    }

    eprintln!(
        "csize {}x{} lheight {}",
        param.csize.width, param.csize.height, param.lheight
    );
    eprintln!(
        "nchars {} cline {} crow {} braise {}",
        geo.nchars, cline, crow, param.braise
    );

    eprintln!("inmargin {} sepmargin {}", param.inmargin, param.sepmargin);

    let numwid: isize = if param.numbering {
        (param.numcsize.width as isize) * (geo.ndigits + 1)
    } else {
        0
    };

    eprintln!(
        "numcsize {}x{} ndights {} numwid {}",
        param.numcsize.width, param.numcsize.height, geo.ndigits, numwid
    );

    eprintln!("wmax {}", param.wmax);
    eprintln!("wmin {}", param.wmin);
    eprintln!("lmax {}", param.lmax);
    eprintln!("lmin {}", param.lmin);

    geo.txoffset = if param.numbering {
        param.inmargin as isize + numwid + param.sepmargin as isize
    } else {
        0
    };
    /*
        eprintln!("txoffset {}", txoffset);
    */
    geo.txwidth = (param.inmargin as isize)
        + geo.nchars * (param.csize.width as isize)
        + (param.inmargin as isize);
    geo.txwmin = (param.inmargin as isize)
        + (param.wmin as isize) * (param.csize.width as isize)
        + (param.inmargin as isize);
    geo.cvwidth = geo.txoffset + geo.txwidth;
    geo.cvheight = (crow as usize * param.lheight + param.inmargin * 2) as isize;
    geo.cvhmin = (param.lmin * param.lheight + param.inmargin * 2) as isize;
    eprintln!("geo {:?}", geo);

    let mut curpic: RowChunk = Vec::new();
    let mut lineperpage: usize;
    let mut lineoffset: usize;
    let mut picno: usize = 0;

    lineperpage = 0;
    lineoffset = 0;
    loop {
        //        eprintln!("lineperpage {}", lineperpage);
        if fullrow.is_empty() {
            break;
        }

        curpic.push(fullrow.remove(0));
        lineperpage += 1;

        if lineperpage >= param.lmax {
            eprintln!("call pagepring picno# {} {} lines", picno, lineperpage);
            print_picture(curpic.clone(), lineoffset, crow, &geo, param);

            curpic.clear();

            lineoffset += lineperpage;

            picno += 1;
            lineperpage = 0;
        }
    }

    if lineperpage > 0 {
        eprintln!("call pagepring picno# {} {} lines", picno, lineperpage);
        print_picture(curpic.clone(), lineoffset, crow, &geo, param);

        // lineoffset += lineperpage;
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let _num_files = config.files.len();
    let param = config.params;

    //  dbg!(&config);

    if param.standalone {
        println!("\\documentclass{{article}} %%% fwtype-opt");
        println!("\\usepackage[T1]{{fontenc}} %%% fwtype-opt");
        //        println!("\\usepackage{{times}} %%% fwtypw-opt");
        println!("\\begin{{document}} %%% fwtypw-opt");
        println!("\\par %%% fwtypw-opt");
    }

    for filename in config.files.iter() {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut file) => {
                fwtype(&mut file, &param);
            }
        }
    }

    if param.standalone {
        println!("\\end{{document}} %%% fwtypw-opt");
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn parse_2d_int(val: &str) -> MyResult<CharSize> {
    let mut h: usize = 10;
    let mut w: usize = 5;
    let pat: Vec<String> = val.split("x").map(str::to_string).collect();

    match pat.len() {
        0 => {
            panic!();
        }
        1 => {
            h = pat[0].parse().unwrap();
            w = h / 2;
        }
        2 => {
            h = pat[0].parse().unwrap();
            w = pat[1].parse().unwrap();
        }
        _ => {}
    }

    Ok(CharSize {
        width: w,
        height: h,
    })
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

fn parse_int(val: &str) -> MyResult<isize> {
    match val.parse() {
        Ok(n) => Ok(n),
        _ => Err(From::from(val)),
    }
}

fn parse_uint(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) => Ok(n),
        _ => Err(From::from(val)),
    }
}

// --------------------------------------------------
#[test]
fn test_parse_int() {
    // -3 is an OK integer
    let res = parse_int("-3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), -3);

    // 3 is an OK integer
    let res = parse_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // Any string is an error
    let res = parse_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    /*
        // A zero is an error
        let res = parse_int("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "0".to_string());
    */
}

#[test]
fn test_parse_positive_int() {
    // 3 is an OK integer
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // Any string is an error
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    // A zero is an error
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

#[test]
fn test_parse_2d_int() {
    // 3 is an OK integer
    let res = parse_2d_int("8");
    assert!(res.is_ok());
    assert_eq!(
        res.unwrap(),
        CharSize {
            height: 8,
            width: 4
        }
    );

    // 12x6 is an OK integer
    let res = parse_2d_int("12x6");
    assert!(res.is_ok());
    assert_eq!(
        res.unwrap(),
        CharSize {
            height: 12,
            width: 6
        }
    );
}
