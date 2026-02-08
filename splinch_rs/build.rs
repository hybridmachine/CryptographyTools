use std::io::Write;

use clap::{Arg, Command};

fn build_cli() -> Command {
    // NOTE: Keep this in sync with the Cli struct in src/main.rs
    Command::new("splinch-rs")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Split a file into two XOR-complementary parts for secure transport, or combine them back")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .help("Path to the input file to split or a .xor1/.xor2 file to combine")
                .required(true)
                .value_name("FILE"),
        )
        .arg(
            Arg::new("verify")
                .short('v')
                .long("verify")
                .help("Verify the split files against the original after splitting")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("combine")
                .short('c')
                .long("combine")
                .help("Combine two XOR files back into the original")
                .action(clap::ArgAction::SetTrue),
        )
}

fn custom_troff_sections() -> &'static str {
    r#"
.SH EXAMPLES
.PP
Split a file into two XOR-complementary parts:
.RS 4
.nf
splinch\-rs \-i secret.pdf
.fi
.RE
.PP
Split a file and verify the output:
.RS 4
.nf
splinch\-rs \-i secret.pdf \-v
.fi
.RE
.PP
Combine two parts back into the original:
.RS 4
.nf
splinch\-rs \-i secret.pdf.xor1 \-c
.fi
.RE
.SH EXIT STATUS
.TP
.B 0
Successful operation.
.TP
.B 1
An error occurred (missing files, verification failure, invalid arguments, etc.).
.SH FILES
.TP
.I <input>.xor1
Random byte stream (one-time pad) generated during splitting.
.TP
.I <input>.xor2
XOR of the original file and the .xor1 pad.
.PP
If the output filename already exists, a numeric suffix is inserted
(e.g., \fIsecret.1.pdf\fR) to avoid overwriting.
.SH SECURITY CONSIDERATIONS
.PP
\fBsplinch\-rs\fR implements one-time pad (OTP) file splitting. Each output
file is statistically indistinguishable from random data. Neither file alone
reveals any information about the original.
.PP
For secure transport, the two output files \fBmust\fR be sent over separate,
independent channels. Sending both files over the same channel defeats the
security guarantee.
.PP
For files larger than 10\ MB, the \fB\-v\fR flag uses sampled verification
(10 random 64\ KB chunks) rather than a full byte-by-byte comparison.
This is fast but not exhaustive.
.SH SEE ALSO
.BR xor (1),
.BR split (1),
.BR openssl (1)
.SH AUTHORS
Brian Tabone
.SH BUGS
Report bugs at: https://github.com/hybridmachine/CryptographyTools/issues
"#
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let cmd = build_cli();

    let man = clap_mangen::Man::new(cmd);
    let mut buf = Vec::new();
    man.render(&mut buf).expect("failed to render man page");

    buf.write_all(custom_troff_sections().as_bytes())
        .expect("failed to append custom sections");

    let out_path = std::path::Path::new(&out_dir).join("splinch-rs.1");
    std::fs::write(&out_path, buf).expect("failed to write man page");

    println!("cargo:rerun-if-changed=src/main.rs");
}
