# fwtype

See: **[README-ja.md](README-ja.md)**

`fwtype` is a command-line formatter that preserves whitespace and
terminal-style layout, allowing text to be printed or embedded in
documents exactly as it appears in a terminal.

For typesetting, ASCII characters are treated as having a width of 1,
while all non-ASCII characters are treated as having a width of 2,
ensuring stable alignment in the output.

## Motivation

When the output of Unix tools is embedded directly into TeX, the
whitespace width, indentation, and column alignment often change,
making it impossible to reproduce the terminal layout faithfully.

`fwtype` is designed specifically to transfer terminal-style layout
**correctly into TeX**, preserving whitespace and alignment exactly as
they appear on the terminal.

## Features

- Precise preservation of whitespace, spacing, and alignment  
  - ASCII characters occupy 1 cell; all other characters occupy 2 cells
- Uses only the standard LaTeX picture environment (no epic, eepic, or \special)
- Optional line numbering (-n)
- Configurable column width per picture (-w)
- Configurable line count per picture (-l)
- Configurable font size (-c)
- Configurable frame (-f)
- Optional generation of a standalone TeX document (-S)
- Optional grid drawing (-g)

## Example

### Example 1

The output of `ls` contains both tabs and spaces. When typeset in TeX,
the layout becomes distorted, and the same is likely to happen in most
web browsers.

    % ls -F /
    Applications/   etc@        private/    Users/
    bin/        home@       sbin/       usr/
    cores/      Library/    System/     var@
    dev/        opt/        tmp@        Volumes/


`fwtype` reproduces the above layout exactly, including alignment and spacing.

Using the `Verbatim` environment (fancyvrb.sty):

<img src="sample/ls-F-slash-Verb.png">

Using `fwtype`:

<img src="sample/ls-F-slash-fwtype.png">

### Example 2

`fwtype` accepts ASCII and UTF-8, but it has been tested primarily with Japanese text.

    ASCIIは1桁
    漢字は2桁

<img src="sample/ascii_japanese_mix.png">

When you use `-n`, line numbers are added:

<img src="sample/ascii_japanese_mix_wnum.png">

## Installation

Build from source:

    git clone https://github.com/k-chinen/fwtype
    cd fwtype
    cargo build --release


## Case Study

First, generate a data file using any program
and convert it to a TeX file with `fwtype`:

    % ls -C -F / > /tmp/ls-F-out.txt
    % fwtype /tmp/ls-F-out.txt > ls-F-out.tex

Next, prepare a `main.tex` file that includes `ls-F-out.tex`:

    ...
    \input{ls-F-out}
    ...

Finally, typeset the document using your TeX workflow:

    % platex main.tex
    % dvipdfmx main.dvi


## Limitations

- Combining characters may not align perfectly  
- Emoji and wide-character layout can vary by platform  
- ANSI escape sequences are not interpreted  

## Help

    fwtype 0.3.5 (cb87028) [2025-11-23T06:19:00.808308Z]
    Ken-ichi Chinen <k-chinen@metro-cit.ac.jp>
    generate fixed-width printing for LaTeX from plain-text

    USAGE:
        fwtype [FLAGS] [OPTIONS] [FILE]...

    FLAGS:
        -g, --grid            Enable grid. See -G and -Z
        -h, --help            Prints help information
        -n, --numbering       Line numbering
        -p, --pagebreaking    Insert a page break after each picture. See -l
        -u, --spcmarking      Space marking by triangle
        -S, --standalone      Insert preamble and begin/end document in first
        -V, --version         Prints version information

    OPTIONS:
        -A, --above <abovegap>         above gap like ".5em" [default: ]
        -B, --below <belowgap>         below gap like "12pt" [default: ]
        -b, --braise <braise>          baseline raise for ASCII [default: 0]
        -c, --csize <csize>            character size, e.g., "17" or "20x10" in pt [default: 10x5]
        -F, --font <font>              base font [default: \ttfamily\gtfamily]
        -f, --frames <frames>          set of frames [default: 15]
        -G, --ghpitch <ghpitch>        grid pitch in horizontal [default: 5]
        -Z, --gvpitch <gvpitch>        grid pitch in vertical [default: 5]
        -H, --lheight <lheight>        lheight; if not specified csize *1.2 [default: 99999]
        -l, --llimit <llimit>          line limit per picture [default: 9999]
        -N, --lnooffset <lnooffset>    linenumber offset [default: 0]
        -W, --lnowidth <lnowidth>      linenumber width [default: 99999]
        -C, --numcsize <numcsize>      character size of line numbers, e.g., "12x6" in pt [default: 6x3]
        -m, --outmargin <outmargin>    out margin width [default: 5]
        -s, --sepmargin <sepmargin>    sep margin width [default: 2]
        -t, --tabstop <tabstop>        tabstop [default: 8]
        -w, --wlimit <wlimit>          width limit; column per line [default: 64]

    ARGS:
        <FILE>...    Input file(s) [default: -]

    EXAMPLES:
        % fwtype input.txt
        % fwtype -n -u input.txt
        % fwtype -w 80 input.txt
        % fwtype -l 50 -p -n input.txt
        % fwtype -g -G 4 src/*.txt
        % fwtype -S input.txt > output.tex
