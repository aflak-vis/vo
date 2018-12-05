extern crate vo_table;

use std::io::Cursor;

fn main() {
    let string = include_bytes!("sia-result.xml");
    let cursor = Cursor::new(string.as_ref());
    let votable = vo_table::parse(cursor).unwrap();
    for resouce in votable.resources() {
        for table in resouce.tables() {
            for (i, row) in table.rows().unwrap().enumerate() {
                println!("{}. {:?}", i, row.get_by_ucd("VOX:Image_AccessReference"));
            }
        }
    }
}
