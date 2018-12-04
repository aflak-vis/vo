#[derive(Debug)]
pub struct SiaService<'a> {
    url: &'a str,
}

impl<'a> SiaService<'a> {
    pub const UNI_HEIDELBERG: SiaService<'static> = SiaService {
        url: "http://dc.zah.uni-heidelberg.de/hppunion/q/im/siap.xml",
    };

    pub fn create_query(&self, pos: (f32, f32)) -> SiaQuery<'a> {
        SiaQuery {
            url: self.url,
            pos,
            size: (1.0, 1.0),
            format: Format::All,
        }
    }
}

#[derive(Debug)]
pub struct SiaQuery<'a> {
    url: &'a str,
    pos: (f32, f32),
    size: (f32, f32),
    format: Format,
    // intersect
    // verbosity
    // other keywords
}

#[derive(Debug)]
pub enum Format {
    All,
    Graphic,
    Metadata,
    Fits,
    Jpeg,
    Png,
}

impl<'a> SiaQuery<'a> {
    pub fn execute(&self) {
        ()
    }
}
