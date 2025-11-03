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

fn dime_auto_str() -> &'static str {
    Box::leak(DIME_AUTO.to_string().into_boxed_str())
}

type MyResult<T> = Result<T, Box<dyn Error>>;

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
    kind:   TokenKind,
    width:  isize,
}

impl TokenKind {
    pub fn fmt(&self) -> String {
        match &self {
            TokenKind::_Text(s) =>  format!("t{:?}", s),
            TokenKind::Escape(s) => format!("e{:?}", s),
            TokenKind::Ascii(s) =>  format!("a{:?}", s),
            TokenKind::Misc(s) =>   format!("m{:?}", s),
            TokenKind::Hole(s) =>   format!("h{:?}", s),
            TokenKind::Skip =>      format!("skip"),
            TokenKind::_Nop =>      format!("nop"),
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
    width:  isize,
    setret: bool,
    tokens: Vec<Token>,
}

impl Row {
    pub fn clear(&mut self) {
        self.lineno = -1;
        self.width  = -1;
        self.setret = false;
        self.tokens.clear();
    }
    pub fn fmt(self) -> String {
        let pre = format!("{:5}:{:<3}", self.lineno, self.width);
        let mark = if self.setret { "*" } else { "." };
        let tks : Vec<String> = self.tokens.iter()
            .map(|t| t.fmt())
            .collect::<Vec<_>>();
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
    let mut _c = 0;
    let mut x = 0;
    let mut _y = 0;
    let mut tk: Token;
    let mut rchk : RowChunk = Vec::new();
    let mut currow : Row = Row {lineno: -1, width: -1,
            setret: false, tokens: Vec::new() };
    loop {
        let Some(q) = iter.next() else { break };

        if q.is_ascii() {

            if q == "\t" {
                let nx = ((x) / tabstop) * tabstop + tabstop;
                tk = Token { kind: TokenKind::Skip, width: nx-x};
            }
            else if q == "\x1b" {
                let mut seq = String::from(q);
                while let Some(&next) = iter.peek() {
                    seq.push_str(next); 
                    iter.next();
                    if next.ends_with('m') {
                        break;
                    }
                }
                tk = Token { kind: TokenKind::Escape(seq), width: 1};
            }
            else if q == "\x1d" {
                let Some(q2) = iter.next() else { break };
                tk = Token { kind: TokenKind::Hole(q2.to_string()), width: 4};
            }   
            else {
                tk = Token { kind: TokenKind::Ascii(q.to_string()), width: 1};
           }
        }
        else {
            tk = Token { kind: TokenKind::Misc(q.to_string()), width: 2};
        }

// eprintln!("c {} x {} y {} tk.width {} tk |{}|", c, x, y, tk.width, tk.fmt());
// eprintln!("x {} tk.width {} {} vs {}", x, tk.width, x+tk.width, wcolumn);
        if x+tk.width>wcolumn {
/*
eprintln!("overrun!!!");
*/
            currow.setret = true;
            currow.calcwidth();
            rchk.push(currow.clone());

            currow.tokens.clear();
            currow.clear();
            x = 0;

            _y += 1;
        }
        else {
        }
        x += tk.width;
        currow.tokens.push(tk);

/*
eprint_row("cur", &currow);
*/

/*
println!("currow {}", fmt_row(&currow));
        println!("currow {:?}", currow);
*/

        _c += 0;
    }

    if ! currow.tokens.is_empty() {
/*
println!("push currow {}", fmt_row(&currow));
*/
        currow.calcwidth();
        rchk.push(currow.clone());
        
        currow.tokens.clear();

        _y += 1;
    }

/*
    eprint_chunk("rchk ", &rchk);
    view_chunk(  "rchk ", &rchk);
*/

    rchk
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    csize: CharSize,
    lineheight: usize,
    outmargin: usize,
    sepmargin: usize,
    frames: usize,
    tabstop: usize,
    wlimit: usize,
    _llimit: usize,
    basedrift_a: isize,
    gridhpitch: usize,
    gridvpitch: usize,
    abovegap: String,
    belowgap: String,
    linenumoffset: usize,
    linenumwidth: usize,
    //
    spcmarking: bool,
    grid: bool,
    numbering: bool,
    standalone: bool,
}

// --------------------------------------------------
pub fn get_args() -> MyResult<Config> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(long_version_static())
        .author("Ken-ichi Chinen <k-chinen@metro-cit.ac.jp>")
        .about("generate fix width printing for LaTeX from plain text")
        .arg(
            Arg::with_name("linenumoffset")
                .short("N")
                .long("linenumoffset")
                .takes_value(true)
                .help("linenumber offset")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("linenumwidth")
                .short("W")
                .long("linenumwidth")
                .takes_value(true)
                .help("linenumber width")
                .default_value(dime_auto_str()),
        )
        .arg(
            Arg::with_name("abovegap")
                .short("A")
                .long("above")
                .takes_value(true)
                .help("above gap like \".5em\"")
                .default_value(""),
        )
        .arg(
            Arg::with_name("belowgap")
                .short("B")
                .long("below")
                .takes_value(true)
                .help("below gap like \"12pt\"")
                .default_value(""),
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
            Arg::with_name("gridhpitch")
                .short("G")
                .long("gridhpitch")
                .takes_value(true)
                .help("grid pitch in horizontal")
                .default_value("5"),
        )
        .arg(
            Arg::with_name("gridvpitch")
                .short("Z")
                .long("gridvpitch")
                .takes_value(true)
                .help("grid pitch in vertical")
                .default_value("5"),
        )

        .arg(
            Arg::with_name("llimit")
                .short("l")
                .long("llimit")
                .takes_value(true)
                .help("line limit")
                .default_value("9999"),
        )

        .arg(
            Arg::with_name("wlimit")
                .short("w")
                .long("wlimit")
                .takes_value(true)
                .help("width limit; column per line")
                .default_value("64"),
        )

        .arg(
            Arg::with_name("basedrift_a")
                .short("b")
                .long("basedrift_a")
                .takes_value(true)
                .help("basedrift for ASCII")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("outmargin")
                .short("m")
                .long("outmargin")
                .takes_value(true)
                .help("out margin width")
                .default_value("5"),
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
            Arg::with_name("lineheight")
                .short("H")
                .long("lineheight")
                .takes_value(true)
                .help("lineheight; if not specified csize *1.2")
                .default_value(dime_auto_str()),
        )
        .arg(
            Arg::with_name("csize")
                .short("c")
                .long("csize")
                .takes_value(true)
                .help("charctor size like \"17\" or \"20x10\" in pt")
                .default_value("10x5"),
        )
        .arg(
            Arg::with_name("grid")
                .short("g")
                .long("grid")
                .takes_value(false)
                .help("grid"),
        )
        .arg(
            Arg::with_name("spcmarking")
                .short("u")
                .long("spcmakring")
                .takes_value(false)
                .help("Space marking"),
        )
        .arg(
            Arg::with_name("numbering")
                .short("n")
                .long("numbering")
                .takes_value(false)
                .help("Numbering"),
        )
        .arg(
            Arg::with_name("standalone")
                .short("S")
                .long("standalone")
                .takes_value(false)
                .help("add preamble and begin/end document"),
        )
        .arg(
            Arg::with_name("frames")
                .short("f")
                .long("frames")
                .takes_value(true)
                .help("set of frames")
                .default_value("15"),
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

    let mut lineheight = matches
        .value_of("lineheight")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal lineheight -- {}", e))?;

    let linenumoffset = matches
        .value_of("linenumoffset")
        .map(parse_uint)
        .transpose()
        .map_err(|e| format!("illegal linenumoffset -- {}", e))?;

    let linenumwidth = matches
        .value_of("linenumwidth")
        .map(parse_uint)
        .transpose()
        .map_err(|e| format!("illegal linenumwidth -- {}", e))?;

    let tabstop = matches
        .value_of("tabstop")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal tabstop -- {}", e))?;

    let llimit = matches
        .value_of("llimit")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal llimit -- {}", e))?;

    let wlimit = matches
        .value_of("wlimit")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal wlimit -- {}", e))?;

    let basedrift_a = matches
        .value_of("basedrift_a")
        .map(parse_int)
        .transpose()
        .map_err(|e| format!("illegal basedrift_a  -- {}", e))?;

    let outmargin = matches
        .value_of("outmargin")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal outmargin width -- {}", e))?;

    let sepmargin = matches
        .value_of("sepmargin")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal sepmargin width -- {}", e))?;

    let gridvpitch = matches
        .value_of("gridvpitch")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal grid v pitch -- {}", e))?;

    let gridhpitch = matches
        .value_of("gridhpitch")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal grid h pitch -- {}", e))?;

    let frames = matches
        .value_of("frames")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal frame -- {}", e))?;

    // not specified, set automatically csize.height * 1.2
    if lineheight == Some(DIME_AUTO) {
        lineheight = Some((csize.clone().unwrap().height * 12) / 10);
    }
    if linenumwidth == Some(DIME_AUTO) {
        eprintln!("auto linenumwidth");
        //        linenumwidth = Some((csize.clone().unwrap().height*12)/10);
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        csize: csize.unwrap(),
        lineheight: lineheight.unwrap(),
        outmargin: outmargin.unwrap(),
        sepmargin: sepmargin.unwrap(),
        tabstop: tabstop.unwrap(),
        _llimit: llimit.unwrap(),
        wlimit: wlimit.unwrap(),
        basedrift_a: basedrift_a.unwrap() as isize,
        frames: frames.unwrap(),
        gridhpitch: gridhpitch.unwrap(),
        gridvpitch: gridvpitch.unwrap(),
        abovegap: matches.value_of("abovegap").unwrap().to_string(),
        belowgap: matches.value_of("belowgap").unwrap().to_string(),
        linenumoffset: linenumoffset.unwrap(),
        linenumwidth: linenumwidth.unwrap(),
        grid: matches.is_present("grid"),
        spcmarking: matches.is_present("spcmarking"),
        numbering: matches.is_present("numbering"),
        standalone: matches.is_present("standalone"),
    })
}

#[allow(clippy::too_many_arguments)]
fn fwtype(
    fp: &mut dyn BufRead,
    csize: &CharSize,
    lineheight: usize,
    outmargin: usize,
    sepmargin: usize,
    frames: usize,
    tabstop: usize,
    wlimit: usize,
    basedrift_a: isize,
    gridhpitch: usize,
    gridvpitch: usize,
    grid: bool,
    dospcmarking: bool,
    donumbering: bool,
    abovegap: String,
    belowgap: String,
    linenumoffset: usize,
    linenumwidth: usize,
) {
//    let nchars;
/*
    let mut linec;
    let mut rowc;
    let mut maxline = 0;
    let mut cont: Vec<String> = vec![];
*/
    let mut maxwidth= 0;

    //  let cmdchars = r"#$%&^_{}\\|~";
    let cmdchars = "#$%&^_{}\\~";


    eprintln!(
        "numbering {} width {} offset {}",
        donumbering, linenumwidth, linenumoffset
    );

    let mut fullrow: RowChunk = Vec::new();

    /*
     * phase 1: estimate columns and rows
     */
/*
    linec = 0;
    rowc = 0;
*/

    let mut cline = 0;
    let mut crow  : isize = 0;

    for (_line_num, line_result) in fp.lines().enumerate() {
        let line = line_result.unwrap();
/*
eprintln!("; line |{}|", line);
*/

        let chunk = parse_line(&line, tabstop as isize, wlimit as isize);
/*
eprintln!("; tokens {:?}", tokens);
*/
        cline += 1;
        let mut r_per_i = 0;
        for mut x in chunk {
            if x.width > maxwidth{
                maxwidth = x.width;
            }
            if r_per_i==0 {
                x.lineno = cline as isize;
            }
            else {
            }
            fullrow.push(x);
            crow += 1;
            r_per_i += 1;
        }
    }
    let nchars : isize = maxwidth as isize;

/*
    view_chunk("full", &fullrow);
*/

    let ndigits: isize;
    if linenumwidth == DIME_AUTO {
        ndigits = if crow <= 0 { 1 } else { crow.ilog10() + 1 } as isize;
    } else {
        ndigits = linenumwidth as isize;
    }

    eprintln!(
        "csize {}x{} lineheight {}",
        csize.width, csize.height, lineheight
    );
    eprintln!(
        "nchars {} cline {} crow {} basedrift_a {} ndigts {}",
        nchars, cline, crow, basedrift_a, ndigits
    );
    /*
    panic!();
     */

    eprintln!("outmagin {} sepmargin {}", outmargin, sepmargin);

    let numwid: isize = if donumbering {
        (csize.width as isize) * ndigits
    } else {
        0
    };

    eprintln!("numwid {}", numwid);

    let txoffset: isize = if donumbering {
        outmargin as isize + numwid + sepmargin as isize
    } else {
        0
    };
    let txwidth: isize = (outmargin as isize) + 
                nchars * (csize.width as isize) + (outmargin as isize);
    let cvwidth: isize = txoffset + txwidth;
/*
    let cvheight: isize = rowc * lineheight + (outmargin as usize * 2) as isize;
*/
    let cvheight: isize = (crow as usize * lineheight + outmargin * 2) as isize;
    eprintln!("txwidth {} cvwidth {} cvheight {}", txwidth, cvwidth, cvheight);

/*
    let mut x;
    let mut y;
    let mut c;
    let mut q;
    let mut rinline;
*/

    println!("");

    if abovegap == "" {
    } else {
        println!("\\vspace*{{{}}} % above", abovegap);
    }

    println!("\\noindent%");
    println!("{{%");

    println!("\\fboxsep=-.5pt%");
    println!("\\setlength{{\\unitlength}}{{1pt}}%");
    //    println!("\\tt%");
    println!("\\ttfamily\\gtfamily%");
    println!("% csize w,h={}, {}", csize.width, csize.height);
    println!(
        "\\fontsize{{{}pt}}{{{}pt}}\\selectfont%",
        csize.height, csize.height
    );
    println!(
        "\\def\\numfont{{\\fontsize{{{}pt}}{{{}pt}}\\selectfont}}%",
        (2 * csize.height / 3),
        (2 * csize.height / 3)
    );
    /*
        println!("\\def\\spcmark{{\\fontsize{{{}pt}}{{{}pt}}\\selectfont$\\diamond$}}%",
            (2*csize.height/3), (2*csize.height/3) );
    */
    println!(
        "\\def\\spcmark{{\\fontsize{{{}pt}}{{{}pt}}\\selectfont$\\triangle$}}%",
        (2 * csize.height / 3),
        (2 * csize.height / 3)
    );

    println!("\\def\\VV{{\\vrule width 0pt height 0.90em depth .25em}}%");
    println!(
        "\\def\\FA#1#2#3{{\\put(#1,#2){{\\makebox({},{}){{\\VV\\mbox{{#3}}}}}}}}%",
        csize.width, csize.height
    );
    println!(
        "\\def\\FX#1#2#3{{\\put(#1,#2){{\\makebox({},{}){{\\VV\\mbox{{#3}}}}}}}}%",
        csize.width, csize.height
    );
    println!("\\begin{{picture}}({},{})", cvwidth, cvheight);

    println!("% frame");

    println!("\\thicklines");

    if donumbering {
        /*
                println!(" \\put(0,0){{\\circle*{{3}}}}");
                println!(" \\put({},0){{\\circle*{{3}}}}", outmargin);
                println!(" \\put({},0){{\\circle*{{3}}}}", outmargin+numwid);
                println!(" \\put({},0){{\\circle*{{3}}}}", outmargin+numwid+sepmargin);
                //
                for i in 0..=ndigits {
                    println!(" \\put({},0){{\\line(0,1){{5}}}}",
                        outmargin+i*csize.width );
                }
        */
    }

    if frames == 0xf {
        println!(
            " \\put({},0){{\\framebox({},{}){{}}}}",
            txoffset, txwidth, cvheight
        );
    } else {
        if (frames & 0x01) > 0 {
            println!(" \\put({},0){{\\line(0,1){{{}}}}}", txoffset, cvheight);
        }
        if (frames & 0x08) > 0 {
            println!(" \\put({},0){{\\line(1,0){{{}}}}}", txoffset, txwidth);
        }
        if (frames & 0x02) > 0 {
            println!(
                " \\put({},{}){{\\line(-1,0){{{}}}}}",
                txoffset + txwidth,
                cvheight,
                txwidth
            );
        }
        if (frames & 0x04) > 0 {
            println!(
                " \\put({},{}){{\\line(0,-1){{{}}}}}",
                txoffset + txwidth,
                cvheight,
                cvheight
            );
        }
    }

    println!("\\thinlines");

    if grid {
        println!("% grid");
        println!("\\linethickness{{0.1pt}}");

        for gx in 0..=nchars {
            if gx % (gridhpitch as isize) == 0 {
                println!(
                    "  \\put({},{}){{\\line(0,1){{ {} }} }}",
                    txoffset + (outmargin + (gx as usize) * csize.width) as isize,
                    cvheight * 0,
                    cvheight
                );
            }
        }

        for gy in 0..=crow {
            if gy % (gridvpitch as isize) == 0 {
                println!(
                    "  \\put({},{}){{\\line(1,0){{ {} }} }}",
                    txoffset,
                    cvheight - (outmargin + (gy as usize) * lineheight) as isize,
                    txwidth
                );
            }
        }

        println!("\\thinlines");
    }

    println!("% body");

    let mut gline = 1;
    let mut gx : isize;
    let mut gy : isize;
    for r in fullrow {
        gy = cvheight as isize - (lineheight * gline) as isize
                - outmargin as isize;
/*
eprintln!("gline {} gy {}", gline, gy);
*/

        if donumbering {
            if r.lineno > 0 {

                    let numstr = format!("{:>width$}",
                                linenumoffset + r.lineno as usize,
                                width = ndigits as usize);
                    let mut c = 0;
                    for ch in numstr.chars() {
                        gx = (outmargin + csize.width * c) as isize;
                        if ch != ' ' {
                            println!("{{\\numfont\\FA{{{}}}{{{}}}{{{}}}}}", 
                                gx, gy, ch);
                        }
                        c += 1;
                    }

            }
                
        }

        gx = txoffset + outmargin as isize;
        for tk in r.tokens {
            match tk.kind {
            TokenKind::Ascii(ch) => {
                        let mut och: String = "".to_string();
                        if let Some(_x) = cmdchars.find(&ch) {
                            if ch == "\\" {
                                och.push_str("\\textbackslash");
                            } else {
                                och.push('\\');
                                och.push_str(&ch);
                            }
                        } else {
                            if ch == " " {
                                if dospcmarking {
                                    println!(
                                        " \\FA{{{}}}{{{}}}{{\\spcmark}}",
                                        gx,
                                        (gy as isize) - basedrift_a*0
                                    );
                                }
                            }
                            else {
                                och.push_str(&ch);
                            }
                        }
                    
                    println!(
                        " \\FA{{{}}}{{{}}}{{{}}}",
                        gx,
                        (gy as isize) - basedrift_a*0,
                        och
                    );
                },
            TokenKind::Misc(ch) => {
                    println!(
                        " \\FX{{{}}}{{{}}}{{{}}}",
                        gx + (csize.width as isize) / 2,
                        (gy as isize) - basedrift_a*0,
                        ch
                    );
                },
            TokenKind::Escape(_) =>  {
            },
            TokenKind::Skip =>  {
            },
            TokenKind::Hole(label) => {
                    println!(
                        " \\FA{{{}}}{{{}}}{{\\fbox{{\\hbox to 2em{{\\hss
\\VV{{}}{}\\hss}}}}}}",
                        gx + 3 * (csize.width as isize) / 2,
                        (gy as isize) - basedrift_a*0,
                        label
                    );
            }
            _ => {},
            }
            gx += tk.width * csize.width as isize;
        }

    
        if r.setret {
            
/*
                    println!(
                        " \\put({},{}){{{}}}",
                        txoffset + txwidth,
                        (gy as isize) - basedrift_a*0,
                        "$\\hookleftarrow$");
*/
            
                    println!(
                        " \\FA{{{}}}{{{}}}{{{}}}",
                        txoffset + txwidth,
                        (gy as isize) - basedrift_a*0,
                        "$\\ast$");

        }


        gline += 1;
    }

    println!("\\end{{picture}}");
    println!("}}");

    if belowgap == "" {
    } else {
        println!("\\vspace*{{{}}} % below", belowgap);
    }

    println!("");
}

pub fn run(config: Config) -> MyResult<()> {
    let _num_files = config.files.len();

    //  dbg!(&config);

    if config.standalone {
        println!("\\documentclass{{article}} %%% fwtype-opt");
        println!("\\begin{{document}} %%% fwtypw-opt");
        println!("\\par %%% fwtypw-opt");
    }

    for (_file_num, filename) in config.files.iter().enumerate() {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut file) => {
                fwtype(
                    &mut file,
                    &config.csize,
                    config.lineheight,
                    config.outmargin,
                    config.sepmargin,
                    config.frames,
                    config.tabstop,
                    config.wlimit,
                    config.basedrift_a,
                    config.gridhpitch,
                    config.gridvpitch,
                    config.grid,
                    config.spcmarking,
                    config.numbering,
                    config.abovegap.clone(),
                    config.belowgap.clone(),
                    config.linenumoffset,
                    config.linenumwidth,
                );
            }
        }
    }

    if config.standalone {
        println!("\\end{{document}} %%% fwtypw-opt");
    }

    Ok(())
}

// --------------------------------------------------
fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

// --------------------------------------------------
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

// --------------------------------------------------
fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

// --------------------------------------------------
fn parse_int(val: &str) -> MyResult<isize> {
    match val.parse() {
        Ok(n) => Ok(n),
        _ => Err(From::from(val)),
    }
}

// --------------------------------------------------
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
