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

