extern crate hyper;
extern crate url;

#[derive(Debug)]
pub struct SiaService<'a> {
    url: &'a str,
}

impl<'a> SiaService<'a> {
    pub const UNI_HEIDELBERG: SiaService<'static> = SiaService {
        url: "http://dc.zah.uni-heidelberg.de/hppunion/q/im/siap.xml",
    };

    pub fn create_query<'k>(&self, pos: (f64, f64)) -> SiaQuery<'a, 'k> {
        SiaQuery {
            base_url: self.url,
            pos,
            size: (1.0, 1.0),
            format: Format::All,
            intersect: Intersect::Overlaps,
            verbosity: Verbosity::VV,
            keywords: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SiaQuery<'a, 'k> {
    base_url: &'a str,
    pos: (f64, f64),
    size: (f64, f64),
    format: Format,
    intersect: Intersect,
    verbosity: Verbosity,
    keywords: Vec<(&'k str, &'k str)>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Format {
    All,
    Graphic,
    Metadata,
    Fits,
    Jpeg,
    Png,
}

impl From<Format> for &'static str {
    fn from(format: Format) -> &'static str {
        match format {
            Format::All => "ALL",
            Format::Graphic => "GRAPHIC",
            Format::Metadata => "METADATA",
            Format::Fits => "APPLICATION/FITS",
            Format::Jpeg => "IMAGE/JPEG",
            Format::Png => "IMAGE/PNG",
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

impl<'a, 'k> SiaQuery<'a, 'k> {
    pub fn execute(&self) {
        use hyper::rt::{self, Future, Stream};
        use hyper::Client;
        use std::io::{self, Write};
        rt::run({
            let client = Client::new();
            let uri = self.query_url().parse().unwrap();
            client
                .get(uri)
                .and_then(|res| {
                    println!("Response: {}", res.status());
                    res.into_body()
                        // Body is a stream, so as each chunk arrives...
                        .for_each(|chunk| {
                            io::stdout()
                                .write_all(&chunk)
                                .map_err(|e| panic!("example expects stdout is open, error={}", e))
                        })
                }).map_err(|err| {
                    println!("Error: {}", err);
                })
        });
    }

    fn query_url(&self) -> String {
        let pos_val = format!("{},{}", self.pos.0, self.pos.1);
        let size_val = format!("{},{}", self.size.0, self.size.1);
        let verb_val = format!("{}", self.verbosity as usize);
        let query_string = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("POS", &pos_val)
            .append_pair("SIZE", &size_val)
            .append_pair("FORMAT", self.format.into())
            .append_pair("INTERSECT", self.intersect.into())
            .append_pair("VERB", &verb_val)
            .extend_pairs(&self.keywords)
            .finish();
        format!("{}?{}", self.base_url, query_string)
    }
}
