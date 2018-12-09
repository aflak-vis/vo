extern crate hyper;
extern crate tokio;
extern crate url;
extern crate vo_table;

mod err;

use hyper::rt::{Future, Stream};
use hyper::Client;
use vo_table::VOTable;

pub use err::Error;

#[derive(Debug, Copy, Clone)]
pub struct SiaService<U> {
    url: U,
}

impl SiaService<&'static str> {
    pub const CADC: SiaService<&'static str> = SiaService {
        url: "http://www.cadc-ccda.hia-iha.nrc-cnrc.gc.ca/sia/v2query",
    };
    pub const GAVO: SiaService<&'static str> = SiaService {
        url: "http://dc.zah.uni-heidelberg.de/__system__/siap2/sitewide/siap2.xml",
    };
    pub const GAVO_OLD_V1: SiaService<&'static str> = SiaService {
        url: "http://dc.zah.uni-heidelberg.de/hppunion/q/im/siap.xml",
    };
}

impl<U> SiaService<U>
where
    U: ToString,
{
    pub fn new(url: U) -> SiaService<U> {
        SiaService { url }
    }

    pub fn map<F, V>(self, mut f: F) -> SiaService<V>
    where
        F: FnMut(U) -> V,
        V: ToString,
    {
        SiaService { url: f(self.url) }
    }

    pub fn create_query<'k, P: Into<Pos>>(&self, pos: P) -> SiaQuery<'k> {
        SiaQuery {
            base_url: self.url.to_string(),
            pos: pos.into(),
            // size: (1.0, 1.0),
            format: None,
            // intersect: Intersect::Overlaps,
            // verbosity: Verbosity::VV,
            keywords: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SiaQuery<'k> {
    base_url: String,
    pos: Pos,
    // size: (f64, f64),
    format: Option<Format>,
    // intersect: Intersect,
    // verbosity: Verbosity,
    keywords: Vec<(&'k str, &'k str)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pos {
    Circle {
        longitude: f64,
        latitude: f64,
        radius: f64,
    },
    Range {
        longitude1: f64,
        longitude2: f64,
        latitude1: f64,
        latitude2: f64,
    },
    Polygon(PolygonPos),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolygonPos(Vec<(f64, f64)>);

impl Pos {
    fn serialize(&self) -> String {
        match self {
            Pos::Circle {
                longitude,
                latitude,
                radius,
            } => format!("CIRCLE {} {} {}", longitude, latitude, radius),
            Pos::Range {
                longitude1,
                longitude2,
                latitude1,
                latitude2,
            } => format!(
                "RANGE {} {} {} {}",
                longitude1, longitude2, latitude1, latitude2
            ),
            Pos::Polygon(pos) => pos.serialize(),
        }
    }
}

impl From<(f64, f64)> for Pos {
    fn from(pos: (f64, f64)) -> Pos {
        Pos::Circle {
            longitude: pos.0,
            latitude: pos.1,
            radius: 1.0,
        }
    }
}

impl PolygonPos {
    fn serialize(&self) -> String {
        use std::fmt::Write;

        let mut string = String::from("POLYGON");
        for &(long, lat) in self.0.iter() {
            let _ = write!(&mut string, "{} {}", long, lat);
        }
        string
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Format {
    // All,
    // Graphic,
    // Metadata,
    Fits,
    // Jpeg,
    // Png,
}

impl From<Format> for &'static str {
    fn from(format: Format) -> &'static str {
        match format {
            // Format::All => "ALL",
            // Format::Graphic => "GRAPHIC",
            // Format::Metadata => "METADATA",
            Format::Fits => "application/fits",
            // Format::Jpeg => "IMAGE/JPEG",
            // Format::Png => "IMAGE/PNG",
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Intersect {
    Covers,
    Enclosed,
    Overlaps,
    Center,
}

impl From<Intersect> for &'static str {
    fn from(intersect: Intersect) -> &'static str {
        match intersect {
            Intersect::Covers => "COVERS",
            Intersect::Enclosed => "ENCLOSED",
            Intersect::Overlaps => "OVERLAPS",
            Intersect::Center => "CENTER",
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Verbosity {
    Zero = 0,
    V = 1,
    VV = 2,
    VVV = 3,
}

impl<'k> SiaQuery<'k> {
    pub fn with_format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }

    pub fn execute(&self) -> impl Future<Item = SIAResults, Error = Error> {
        let client = Client::new();
        let uri = self.query_url().parse().unwrap();
        client
            .get(uri)
            .and_then(|res| res.into_body().concat2())
            .map_err(Error::Hyper)
            .and_then(|body| {
                use std::io::Cursor;
                let read = Cursor::new(body);
                vo_table::parse(read)
                    .map(|table| SIAResults { table })
                    .map_err(Error::VOTable)
            })
    }

    pub fn execute_sync(&self) -> Result<SIAResults, Error> {
        let mut runtime = tokio::runtime::Runtime::new()
            .map_err(|e| Error::RuntimeError(e, "Could not initialize a Tokio runtime."))?;
        runtime.block_on(self.execute())
    }

    fn query_url(&self) -> String {
        let pos_val = self.pos.serialize();
        // let size_val = format!("{},{}", self.size.0, self.size.1);
        // let verb_val = format!("{}", self.verbosity as usize);
        let query_string = {
            let mut serializer = url::form_urlencoded::Serializer::new(String::new());

            serializer
                .append_pair("POS", &pos_val)
                // .append_pair("SIZE", &size_val)
                // .append_pair("INTERSECT", self.intersect.into())
                // .append_pair("VERB", &verb_val)
                .extend_pairs(&self.keywords);

            if let Some(format) = self.format {
                serializer.append_pair("FORMAT", format.into());
            }

            serializer.finish()
        };
        format!("{}?{}", self.base_url, query_string)
    }
}

#[derive(Debug, Clone)]
pub struct SIAResults {
    table: VOTable,
}

impl SIAResults {
    pub fn records(&self) -> impl Iterator<Item = SIARecord<'_>> {
        self.table
            .resources()
            .iter()
            .map(|resource| {
                resource
                    .tables()
                    .iter()
                    .filter(|table| table.rows().is_some())
                    .map(|table| table.rows().unwrap().map(|row| SIARecord { row }))
                    .flatten()
            }).flatten()
    }

    pub fn table(&self) -> &VOTable {
        &self.table
    }

    pub fn into_table(self) -> VOTable {
        self.table
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SIARecord<'a> {
    row: vo_table::Row<'a>,
}

impl<'a> SIARecord<'a> {
    pub fn access_url(&self) -> Option<&str> {
        self.row
            .get_by_ucd("VOX:Image_AccessReference")
            .or_else(|| self.row.get_by_id("access_url"))
            .or_else(|| self.row.get_by_name("access_url"))
            .and_then(|cell| match cell {
                vo_table::Cell::Character(link) | vo_table::Cell::UnicodeCharacter(link) => {
                    Some(link.as_ref())
                }
                _ => None,
            })
    }

    pub fn access_format(&self) -> Option<&str> {
        self.row
            .get_by_id("access_format")
            .or_else(|| self.row.get_by_name("access_format"))
            .and_then(|cell| match cell {
                vo_table::Cell::Character(link) | vo_table::Cell::UnicodeCharacter(link) => {
                    Some(link.as_ref())
                }
                _ => None,
            })
    }

    pub fn access_estsize(&self) -> Option<i64> {
        self.row
            .get_by_id("access_estsize")
            .or_else(|| self.row.get_by_name("access_estsize"))
            .and_then(|cell| match cell {
                vo_table::Cell::Integer16(size) => size[0].map(|int| int as i64),
                vo_table::Cell::Integer32(size) => size[0].map(|int| int as i64),
                vo_table::Cell::Integer64(size) => size[0],
                _ => None,
            })
    }
}
