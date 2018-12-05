extern crate vo_sia;

use vo_sia::hyper::rt::{self, Future};
use vo_sia::SiaService;

fn main() {
    let query = SiaService::UNI_HEIDELBERG.create_query((161.2647341982576, -59.6844592879577));
    let results = query.execute();
    rt::run(
        results
            .map(|results| {
                for (i, record) in results.records().enumerate() {
                    if let Some(acref) = record.acref() {
                        println!("{}. {}", i, acref)
                    }
                }
            }).map_err(|e| {
                eprintln!("{:?}", e);
            }),
    )
}
