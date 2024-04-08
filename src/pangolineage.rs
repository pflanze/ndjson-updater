//! Pango lineage parsing

use std::convert::TryFrom;

use anyhow::{bail, Result};
use kstring::KString;
use lazy_static::lazy_static;
use regex::Regex;


pub trait BaseName {
    fn new(basename: KString) -> Result<Self> where Self: Sized;
    fn as_str(&self) -> &str;
    fn as_kstring(&self) -> &KString;
    fn to_kstring(&self) -> KString;
    fn into_kstring(self) -> KString;
}


lazy_static!{
    static ref VALID_BASENAME: Regex = Regex::new(r"^[A-Z]{1,4}\z").unwrap();
}


/// Could be either an original haplotype, or an alias
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndeterminedBaseName(KString);

impl UndeterminedBaseName {
    pub fn into_haplo_type_base_name(self) -> HaplotypeBasename {
        HaplotypeBasename(self.0)
    }
}

impl BaseName for UndeterminedBaseName {
    fn new(basename: KString) -> Result<Self> {
        if ! VALID_BASENAME.is_match(&basename) {
            bail!("invalid string for UndeterminedBaseName {:?}", &*basename)
        }
        Ok(Self(basename))
    }

    fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn as_kstring(&self) -> &KString {
        &self.0
    }

    fn to_kstring(&self) -> KString {
        self.0.clone()
    }

    fn into_kstring(self) -> KString {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HaplotypeBasename(KString);

// stupid copy-paste!
impl BaseName for HaplotypeBasename {
    fn new(basename: KString) -> Result<Self> {
        if ! VALID_BASENAME.is_match(&basename) {
            bail!("invalid string for HaplotypeBasename: {:?}", &*basename)
        }
        Ok(Self(basename))
    }

    fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn as_kstring(&self) -> &KString {
        &self.0
    }

    fn to_kstring(&self) -> KString {
        self.0.clone()
    }

    fn into_kstring(self) -> KString {
        self.0
    }
}
 

// /// Known when it's an alias or not
// pub enum DeterminedBaseName {
//     OriginalHaplotype(KString),
//     Alias(KString),
// }

// impl BaseName for DeterminedBaseName {
//     fn as_str(&self) -> &str {
//         match self {
//             DeterminedBaseName::OriginalHaplotype(s) => s.as_str(),
//             DeterminedBaseName::Alias(s) => s.as_str()
//         }
//     }
// }


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subpath(Vec<u16>);

impl Subpath {
    pub fn new(subpath: Vec<u16>) -> Self {
        Self(subpath)
    }

    pub fn append(&self, further_subpath: &Self) -> Self {
        let mut p = self.0.clone();
        p.extend_from_slice(&further_subpath.0);
        Self(p)
    }

    pub fn is_ancestor_of(&self, possible_other: &Self, include_self: bool) -> bool {
        let selflen = self.0.len();
        if selflen <= possible_other.0.len() && self.0 == &possible_other.0[0..selflen] {
            // prefix is the same
            if include_self {
                true
            } else {
                selflen < possible_other.0.len()
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests1 {
    use super::*;

    #[test]
    fn t_subpath_append() {
        assert_eq!(
            Subpath(vec![1, 13, 7]).append(&Subpath(vec![4, 5])),
            Subpath(vec![1, 13, 7, 4, 5]));
    }

    #[test]
    fn t_subpath_ancestor() {
        assert_eq!(
            Subpath(vec![1, 13, 7]).is_ancestor_of(&Subpath(vec![4, 5]), true),
            false);
        assert_eq!(
            Subpath(vec![1, 13, 7]).is_ancestor_of(&Subpath(vec![1, 13, 7, 12]), true),
            true);
        assert_eq!(
            Subpath(vec![]).is_ancestor_of(&Subpath(vec![12]), true),
            true);
        assert_eq!(
            Subpath(vec![]).is_ancestor_of(&Subpath(vec![]), true),
            true);
        assert_eq!(
            Subpath(vec![]).is_ancestor_of(&Subpath(vec![]), false),
            false);
        assert_eq!(
            Subpath(vec![1, 13, 7]).is_ancestor_of(&Subpath(vec![1, 13, 6, 12]), true),
            false);
        assert_eq!(
            Subpath(vec![1, 13, 7]).is_ancestor_of(&Subpath(vec![1, 13, 7]), true),
            true);
        assert_eq!(
            Subpath(vec![1, 13, 7]).is_ancestor_of(&Subpath(vec![1, 13, 7]), false),
            false);
    }
}


impl std::convert::AsRef<[u16]> for Subpath {
    fn as_ref(&self) -> &[u16] {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PangoLineage<B: BaseName>(pub B, pub Subpath);

impl<B: BaseName> PangoLineage<B> {
    pub fn new(basename: B, subpath: Subpath) -> Self {
        Self(basename, subpath)
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str(self.0.as_str());
        for sublevel in self.1.as_ref() {
            s.push('.');
            s.push_str(&sublevel.to_string()); // XX optim
        }
        s
    }
}

// impl<B: BaseName> TryFrom<&str> for PangoLineage<B> {
// No, only implement it for UndeterminedBaseName ones!
/// `TryFrom<&str>` is only implemented for lineages with undetermined
/// base name kind, as string representations are normally using
/// aliases; this prevents user code accidentally choosing
/// `PangoLineage<HaplotypeBasename>` and over-eagerly satisfy
/// e.g. `is_ancestor_of` checks (omitting the necessary
/// `canonicalize` call).
impl TryFrom<&str> for PangoLineage<UndeterminedBaseName> {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split('.');
        let part0 = parts.next().expect("split returns at least 1 element");
        if part0.is_empty() {
            bail!("empty string can't be a base name")
        }
        Ok(Self(UndeterminedBaseName::new(KString::from_ref(part0))?,
                Subpath(parts.map(|s| Ok(s.parse()?)).collect::<Result<Vec<_>>>()?)))
    }
}

impl PangoLineage<HaplotypeBasename> {
    pub fn is_ancestor_of(&self, possible_sublineage: &Self, include_self: bool) -> bool {
        self.0 == possible_sublineage.0 && self.1.is_ancestor_of(&self.1, include_self)
    }
}

impl PangoLineage<UndeterminedBaseName> {
    /// Force conversion into canonical representation without any
    /// change, i.e. assume that the base name is an original haplo
    /// type name, not an alias. Be careful to uphold this assumption!
    pub fn force_into_canonicalization(self) -> PangoLineage<HaplotypeBasename> {
        let Self(basename, subpath) = self;
        PangoLineage(basename.into_haplo_type_base_name(), subpath)
    }
}

#[cfg(test)]
mod tests2 {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn t_parse() {
        let l : PangoLineage<UndeterminedBaseName> = "A".try_into().unwrap();
        assert_eq!(l.0.as_str(), "A");
        assert_eq!(l.1.as_ref(), &[] as &[u16]);

        let l : PangoLineage<UndeterminedBaseName> = "DV.7.1.2".try_into().unwrap();
        assert_eq!(l.0.as_str(), "DV");
        assert_eq!(l.1.as_ref(), &[7u16, 1, 2]);
    }
}
