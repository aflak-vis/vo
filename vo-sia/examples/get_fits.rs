extern crate vo_sia;

use vo_sia::SiaService;

fn main() {
    let query = SiaService::GAVO.create_query((98.168896625, 4.91167305556));

    match query.execute_sync() {
        Err(e) => eprintln!("Error: {:?}", e),
        Ok(results) => {
            for (i, record) in results.records().enumerate() {
                if let Some(acref) = record.acref() {
                    println!("{}. {}", i, acref);
                }
            }
        }
    };
}
