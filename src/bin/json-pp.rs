use std::{io::{Write, BufWriter, stdin, stdout}, fs::read_to_string};

use anyhow::{Result, Context, anyhow, bail};
use jzon::codegen::{Generator, PrettyWriterGenerator};
use ndjson_updater::{tempfile::named_tempfile_for, io_read_to_string::io_read_to_string};


fn json_pp(input: &str, outp: &mut impl Write) -> Result<()> {
    let entry = jzon::parse(input)?;
    let mut jsonwriter = PrettyWriterGenerator::new(outp, 2);
    jsonwriter.write_json(&entry)?;
    jsonwriter.get_writer().write(b"\n")?;
    Ok(())
}

fn inplace_json_pp(path: &str) -> Result<()> {
    (|| -> Result<_> {
        let input = read_to_string(path)?;
        let mut tmp = named_tempfile_for(path)?;
        {
            let mut outp = BufWriter::new(tmp.as_file_mut());
            json_pp(&input, &mut outp)?;
            outp.flush()?;
            // fsync (makes it slower but safe):
            outp.get_mut().sync_data()?;
        }
        std::fs::rename(tmp.path(), path)?;
        Ok(())
    })().with_context(|| anyhow!("processing file {path:?}"))
}

fn pipeline_json_pp() -> Result<()> {
    (|| -> Result<_> {
        let input = io_read_to_string(stdin())?;
        let mut outp = BufWriter::new(stdout());
        json_pp(&input, &mut outp)
    })().with_context(|| anyhow!("processing stdin to stdout"))
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let cmd = args.next().expect("program name");

    let mut opt_inplace = false;
    let mut sourcepaths = Vec::new();

    while let Some(arg) = args.next() {
        match &*arg {
            "--inplace" | "-i" =>
                opt_inplace = true,
            "--" => {
                // unstable lib feature: args.collect_into(&mut sourcepaths);
                sourcepaths = args.collect();
                break;
            }
            _ =>
                if opt_inplace {
                    sourcepaths.push(arg);
                } else {
                    bail!("{cmd}: unknown argument {arg:?} -- \
                           hint: for processing files in-place, pass --inplace or -i first")
                }
        }
    }

    (|| {
        if opt_inplace {
            for sourcepath in sourcepaths {
                inplace_json_pp(&sourcepath)?;
            }
            Ok(())
        } else {
            pipeline_json_pp()
        }
    })().with_context(|| anyhow!("{cmd}"))
}
