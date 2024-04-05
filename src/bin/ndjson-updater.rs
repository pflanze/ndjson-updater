use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Write, BufReader, BufRead};

use anyhow::{anyhow, Context, Result, bail};
use jzon::JsonValue;
use jzon::codegen::{Generator, WriterGenerator};
use ndjson_updater::easyjson::{EasyJsonValue, EasyObject};

#[allow(unused, non_snake_case)]
#[derive(Debug, serde::Deserialize)]
struct TsvEntry {
    gisaid_epi_isl: String,
    pango_lineage: String,
    date: String,
    region: String,
    country: String,
    division: String,
    unsorted_date: String,
    age: String,
    qc_value: String,
    nucleotideInsertions: String,
    aminoAcidInsertions: String,
    test_boolean_column: String
}

fn read_tsv_by_gisaid_epi_isl(path: &str) -> Result<HashMap<String, TsvEntry>> {
    let inp = BufReader::new(File::open(path)?);
    
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        // .trim(csv::Trim::All)
        .from_reader(inp);
    
    let mut entries = HashMap::new();
    for r in rdr.deserialize() {
        let r: TsvEntry = r?;
        if let Some(old)= entries.insert(r.gisaid_epi_isl.clone(), r) {
            bail!("duplicate entry in {path:?}: {old:?}");
        }
    }
    
    Ok(entries)
}

fn parse_csv_boolean(valstr: &str) -> Result<JsonValue> {
    Ok(match valstr {
        "true" => true.into(),
        "false" => false.into(),
        "" => JsonValue::Null,
        _=> bail!("invalid value for 'test_boolean_column': {valstr:?}")
    })
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let cmd = args.next().unwrap();
    let args: Vec<_> = args.collect();
    if let [tsvpath, inpath, outpath] = &*args {
        let tsventries = read_tsv_by_gisaid_epi_isl(&tsvpath)?;

        let mut inp = BufReader::new(File::open(&inpath)?);
        let mut outp = File::create(&outpath)?;
        let mut jsonwriter = WriterGenerator::new(&mut outp);

        let mut used_gisaid_epi_isl = HashSet::new();

        let mut line = String::new();
        let mut lineno = 0;
        (|| -> Result<_> {
            while inp.read_line(&mut line)? != 0 {
                lineno += 1;
                let mut entry = jzon::parse(&line)?;
                let metadata = entry.object_mut()?.xget_mut("metadata")?.object_mut()?;
                let id = metadata.xget("gisaid_epi_isl")?.str()?;

                let tsventry = tsventries.get(id).ok_or_else(
                    || anyhow!("unknown \"gisaid_epi_isl\" value {id:?}"))?;

                if used_gisaid_epi_isl.contains(id) {
                    bail!("gisaid_epi_isl {id:?} used multiple times")
                }
                used_gisaid_epi_isl.insert(id.to_string());

                metadata.insert("test_boolean_column",
                                parse_csv_boolean(&tsventry.test_boolean_column)?);

                jsonwriter.write_json(&entry)?;
                jsonwriter.get_writer().write(b"\n")?;
                line.clear();
            }
            Ok(())
        })().with_context(|| anyhow!("on line {lineno}"))?;
    } else {
        bail!("usage: {cmd} tsvpath inpath outpath");
    }
    
    Ok(())
}
