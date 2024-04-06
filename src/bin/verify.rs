use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;

use anyhow::{Result, bail};
use ndjson_updater::groupby::{group_by, print_group_sizes};

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


fn main() -> Result<()> {
    let mut args = std::env::args();
    let cmd = args.next().unwrap();
    let args: Vec<_> = args.collect();
    if let [tsvpath] = &*args {
        let tsventries = read_tsv_by_gisaid_epi_isl(&tsvpath)?;

        let by_test_boolean_column = group_by(
            tsventries.values(),
            |e| { &e.test_boolean_column },
            |e| { &e.gisaid_epi_isl })?;
        print_group_sizes(&by_test_boolean_column);

        dbg!(tsventries.values().filter(|e| e.test_boolean_column == "true").count());
        dbg!(tsventries.values().filter(|e| e.test_boolean_column == "false").count());
        dbg!(tsventries.values().filter(|e| e.test_boolean_column == "").count());

        dbg!(tsventries.values().filter(|e| {
            (e.test_boolean_column == "false"
             || e.test_boolean_column == "")
                &&
                (e.pango_lineage == "B.1"
                 || e.pango_lineage.starts_with("B.1."))
        }).count());

        let by_pango_lineage = group_by(
            tsventries.values(),
            |e| { &e.pango_lineage },
            |e| { &e.gisaid_epi_isl })?;
        print_group_sizes(&by_pango_lineage);
        
    } else {
        bail!("usage: {cmd} tsvpath");
    }
    
    Ok(())
}
