# What is this

This is a parser for the format of orders shown on the "Orders" page of a game on [WebDiplomacy](http://webdiplomacy.net). The output format is compatible with [jDip](http://jdip.sourceforge.net/)'s multiple order entry dialog.

# Prerequisites

You should install [rustup](https://rustup.rs/).

# Using it

1. Copy all the orders from the orders page into a text file. It should begin with something like `Autumn, 1913 arge map` with whatever the most recent season is, and end with Russia's orders from Spring 1901.
2. save in a file called `data.txt`. Maybe I'll add argument parsing later.
3. Run `cargo run` in the root of this repository. jDip-friendly orders will come over standard output.
4. Paste each chunk of them into jDip's multi-order box. It may complain that "there is no unit in Bulgaria" or similar, when the unit is auto-disbanded and WebDiplomacy wants to destroy it. That's okay.

