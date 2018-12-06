use std::sync::Mutex;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use failure::{Error, err_msg};
use xcb::Connection;
use xcb::xproto;
use lazy_static::*;

lazy_static! {
    static ref ATOM_CACHE: Mutex<HashMap<&'static str, xproto::Atom>> = Mutex::new(HashMap::new());
}

pub fn get(conn: &Connection, name: &'static str) -> Result<xproto::Atom, Error> {
    let mut map = ATOM_CACHE.lock()
        .map_err(|_| err_msg("Failed to access atom cache"))?;
    match map.entry(name) {
        Entry::Occupied(entry) => Ok(*entry.get()),
        Entry::Vacant(entry) => {
            let interned = xproto::intern_atom(conn, true, name)
                .get_reply()?
                .atom();
            Ok(*entry.insert(interned))
        }
    }
}


