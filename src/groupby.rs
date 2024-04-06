use std::{hash::Hash, fmt::Debug, collections::HashMap};

use anyhow::{Result, bail};


pub fn group_by<'k, 'pk, 'v: 'pk, K: Hash + Eq, PK: Hash + Eq + Debug, V>(
    vals: impl Iterator<Item = &'v V>,
    getkey: impl Fn(&'v V) -> &'k K,
    get_primary_key: impl Fn(&'v V) -> &'pk PK,
) -> Result<HashMap<&'k K, HashMap<&'pk PK, &'v V>>>
{
    let mut m: HashMap<&'k K, HashMap<&'pk PK, &'v V>> = HashMap::new();
    for val in vals {
        let key = getkey(val);
        let pkey = get_primary_key(val);
        if let Some(v) = m.get_mut(key) {
            if v.contains_key(pkey) {
                bail!("duplicate pkey {pkey:?}");
            }
            v.insert(pkey, val);
        } else {
            let mut g = HashMap::new();
            g.insert(pkey, val);
            m.insert(key, g);
        }
    }
    Ok(m)
}
