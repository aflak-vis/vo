use std::io::Read;

pub fn parse<R: Read>(r: R) -> VOTable {
    unimplemented!()
}

pub struct VOTable {
    description: Option<Description>,
    resources: Vec<Resource>,
}

struct Resource {
    description: Option<Description>,
    infos: Vec<Info>,
    tables: Vec<Table>,
    child_resources: Vec<Resource>,
}

struct Info {}

struct Table {
    description: Option<Description>,
    fields: Vec<Field>,
    data: Option<Data>,
}

struct Description {
    content: String,
}

struct Field {
    id: Option<String>,
    name: Option<String>,
    datatype: DataType,
    arraysize: Option<ArraySize>,
    width: Option<usize>,
    precision: Option<usize>,
    xtype: Option<XType>,
    unit: Option<String>,
    ucd: Option<String>,
    description: Option<Description>,
    values: Option<Values>,
}

enum DataType {
    Logical,
    BitArray,
    Byte,
    Character,
    UnicodeCharacter,
    Integer16,
    Integer32,
    Integer64,
    Float32,
    Float64,
    Complex32,
    Complex64,
}

enum ArraySize {
    Variable,
    Fixed(usize),
}

enum XType {
    Polygon,
}

struct Values {
    null: Option<String>,
}

struct Data {
    rows: Vec<Row>,
}

struct Row {
    cells: Vec<Cell>,
}

struct Cell {
    v: Vec<DataValue>,
}

enum DataValue {
    Logical(bool),
    BitArray(Vec<u8>),
    Byte(u8),
    Character(u8),
    UnicodeCharacter(char),
    Integer16(i16),
    Integer32(i32),
    Integer64(i64),
    Float32(f32),
    Float64(f64),
    Complex32(f32, f32),
    Complex64(f64, f64),
}
