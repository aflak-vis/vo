extern crate vo_sia;

use vo_sia::SiaService;

fn main() {
    let query = SiaService::UNI_HEIDELBERG
        .create_query((161.027341982576, -59.6844592879577))
        .with_size((100.0, 100.0));

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
