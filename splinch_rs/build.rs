use std::io::Write;

use clap::{Arg, Command};

fn build_cli() -> Command {
    // NOTE: Keep this in sync with the Cli struct in src/main.rs
    Command::new("splinch")
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
        .arg(
            Arg::new("secure-delete")
                .short('s')
                .long("secure-delete")
                .help("Securely delete the original file after splitting (overwrite with random data)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("passes")
                .short('p')
                .long("passes")
                .help("Number of overwrite passes for secure delete (default: 1)")
                .default_value("1")
                .value_name("N"),
        )
}

fn custom_troff_sections() -> &'static str {
    r#"
.SH EXAMPLES
.PP
Split a file into two XOR-complementary parts:
.RS 4
.nf
splinch \-i secret.pdf
.fi
.RE
.PP
Split a file and verify the output:
.RS 4
.nf
splinch \-i secret.pdf \-v
.fi
.RE
.PP
Split a file and securely delete the original (3 overwrite passes):
.RS 4
.nf
splinch \-i secret.pdf \-s \-p 3
.fi
.RE
.PP
Combine two parts back into the original:
.RS 4
.nf
splinch \-i secret.pdf.xor1 \-c
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
\fBsplinch\fR implements one-time pad (OTP) file splitting. Each output
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
.PP
The \fB\-s\fR (secure delete) option overwrites the original file with
cryptographically random data before removing it. Each pass is flushed to
physical storage with \fBfsync\fR(2). However:
.IP \(bu 2
On \fBcopy-on-write filesystems\fR (btrfs, ZFS), overwriting a file may write
to new physical blocks, leaving old data intact. Use filesystem-level secure
erase if available.
.IP \(bu 2
On \fBSSDs with wear leveling\fR, the controller may remap sectors, so old
data could persist in unmapped blocks. Full-disk encryption is the recommended
defense.
.IP \(bu 2
On traditional \fBext4/XFS on HDDs\fR, in-place overwrite with fsync is
effective for destroying the original data.
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

    let out_path = std::path::Path::new(&out_dir).join("splinch.1");
    std::fs::write(&out_path, buf).expect("failed to write man page");

    println!("cargo:rerun-if-changed=src/main.rs");
}
