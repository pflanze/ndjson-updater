//! Indexing pango lineage information from
//! https://github.com/cov-lineages/lineages-website/raw/master/_data/lineage_data.full.json
//! (see https://cov-lineages.org/lineage_list.html)

use std::{collections::HashMap, fs::read_to_string, convert::{TryInto, TryFrom}, io::Write};

use anyhow::{Result, bail};
use itertools::Itertools;
use kstring::KString;

use crate::{lineagelist::Lineage,
            pangolineage::{PangoLineage, HaplotypeBasename, BaseName,
                           UndeterminedBaseName, Subpath},
            easyjson::{EasyJsonValue, EasyObject}};

/// An index of all aliases mentioned in the `lineage_data.json` file.
#[derive(Debug)]
pub struct LineageAliases(HashMap<KString, PangoLineage<HaplotypeBasename>>);

impl LineageAliases {
    pub fn from_file(path: &str) -> Result<LineageAliases> {
        // let raw: HashMap<KString, Lineage> = serde_json::from_reader(
        //     File::open(path)?)?;
        // ^ 200x slower than jzon, so:

        let inp = read_to_string(path)?;
        let data = jzon::parse(&inp)?;
        let raw = {
            let mut raw: HashMap<KString, Lineage> = Default::default();
            for (full_nam, lin_raw) in data.object()?.iter() {
                let o = lin_raw.object()?;
                let l = Lineage {
                    lineage: o.xget("Lineage")?.string()?.into(),
                    description: o.xget("Description")?.string()?.into()
                };
                raw.insert(KString::from_ref(full_nam), l);
            }
            raw
        };
        
        let mut tbl = HashMap::new();
        for (full_nam, lin) in raw.iter() {
            assert_eq!(full_nam.as_str(), lin.lineage.as_str());
            if full_nam.as_str().chars().next() == Some('*') {
                // e.g. "*J.1", recalled names; PangoLineage won't
                // currently parse them, thus skip
                continue;
            }
            // dbg!(&full_nam);
            if let Some(canonicalstr) = lin.get_alias_of() {
                // dbg!(canonicalstr);
                let shortened: PangoLineage<UndeterminedBaseName> =
                    lin.lineage.as_str().try_into()?;
                let canonical: PangoLineage<HaplotypeBasename> =
                    PangoLineage::try_from(canonicalstr)?.force_into_canonicalization();
                let surplus_path = shortened.1.as_ref();
                let surplus_len = surplus_path.len();
                let canonical_fullpath = canonical.1.as_ref();
                let canonical_surplus = &canonical_fullpath[
                    (canonical_fullpath.len() - surplus_len)..];
                if surplus_path != canonical_surplus {
                    eprintln!(
                        "shortened {:?} aliasing {:?} but surplus path doesn't match, \
                         {:?} vs. {:?}",
                        lin.lineage.as_str(),
                        canonicalstr,
                        surplus_path,
                        canonical_surplus);
                    continue;
                }
                let canonical_path = &canonical_fullpath[
                    0..canonical_fullpath.len() - surplus_len];

                let key = KString::from_ref(shortened.0.as_str());
                let value = PangoLineage::new(canonical.0, Subpath::new(canonical_path.into()));
                if let Some(old) = tbl.get(&key) {
                    if old != &value {
                        bail!("alias {:?} previously defined as {:?}, now {:?}",
                              key, old, value)
                    }
                } else {
                    tbl.insert(key, value);
                }
            }
        }
        
        Ok(Self(tbl))
    }

    pub fn print<W: Write>(&self, mut outp: W) -> Result<()> {
        for alias in self.0.keys().sorted() {
            let val = self.0.get(alias).unwrap();
            writeln!(&mut outp, "{} = {}", alias.as_str(), val.to_string())?;
        }
        Ok(())
    }

    /// Look up a name that might be an alias and return the pango
    /// lineage path based on the original haplo types that they were
    /// defined for.
    pub fn get(&self, key: &UndeterminedBaseName) -> Option<&PangoLineage<HaplotypeBasename>> {
        self.0.get(key.as_kstring())
    }

    /// Resolve aliases to the full paths based on the original haplo
    /// types, i.e. translate e.g. `BA.7` to `B.1.1.529.7`.
    pub fn canonicalize(
        &self, inp: PangoLineage<UndeterminedBaseName>
    ) -> PangoLineage<HaplotypeBasename> {
        if let Some(base) = self.get(&inp.0) {
            PangoLineage::new(base.0.clone(),
                              base.1.append(&inp.1))
        } else {
            PangoLineage::new(inp.0.into_haplo_type_base_name(),
                              inp.1)
        }
    }
}
