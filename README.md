fwtype
======

fwtype generates TeX picture environments instead of verbatim ones,
since verbatim spacing is not uniform and causes misalignment.

Characters are rendered in a monospace grid:
ASCII characters occupy 1 column, and non-ASCII characters occupy 2 columns.

fwtype accepts ASCII and UTF-8. But it is tested only japanese.

	ASCIIは1桁
	漢字は2桁

<img src="sample/ascii_japanese_mix.png">

When you use -n, line numbers are added.

<img src="sample/ascii_japanese_mix_wnum.png">

Here, I show usage of this program.  You can get them using -h.

	fwtype 0.3.5 (35f4dc8) [2025-11-14T04:20:14.370428Z]
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
