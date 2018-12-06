extern crate base64;
extern crate byteorder;
extern crate xml;

mod err;

use std::io::{Cursor, Read};
use std::str::FromStr;

use byteorder::{BigEndian, ReadBytesExt};
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{Events, XmlEvent::*},
    ParserConfig,
};

pub use err::Error;

pub fn parse<R: Read>(r: R) -> Result<VOTable, Error> {
    VOTable::parse(r)
}

#[derive(Debug, Clone, Default)]
pub struct VOTable {
    description: Option<Description>,
    resources: Vec<Resource>,
}

#[derive(Debug, Clone, Default)]
pub struct Resource {
    description: Option<Description>,
    infos: Vec<Info>,
    tables: Vec<Table>,
    child_resources: Vec<Resource>,
}

#[derive(Debug, Clone, Default)]
struct Info {}

#[derive(Debug, Clone, Default)]
pub struct Table {
    description: Option<Description>,
    fields: Vec<Field>,
    data: Option<Data>,
}

#[derive(Debug, Clone)]
struct Description {
    content: String,
}

#[derive(Debug, Clone, Default)]
struct Field {
    id: Option<String>,
    name: Option<String>,
    datatype: Option<DataType>,
    arraysize: Option<ArraySize>,
    width: Option<usize>,
    precision: Option<Precision>,
    xtype: Option<XType>,
    unit: Option<String>,
    ucd: Option<String>,
    description: Option<Description>,
    values: Option<Values>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy)]
enum ArraySize {
    Unbounded,
    Variable { max: usize },
    Fixed(usize),
}

#[derive(Debug, Clone)]
enum Precision {
    ///  Number of significant digits after decimal point
    AfterDecimalPoint(usize),
    ///  Number of significant figures
    SignificantFigures(usize),
}

#[derive(Debug, Clone)]
/// The standard says: "The actual values of the xtype attribute are not defined
/// in this VOTable specification."
struct XType {
    value: String,
}

#[derive(Debug, Clone, Default)]
struct Values {
    null: Option<NullableDataValue>,
}

#[derive(Debug, Clone, Default)]
struct Data {
    rows: Vec<OwnedRow>,
}

#[derive(Debug, Clone, Default)]
struct OwnedRow {
    cells: Vec<Cell>,
}

#[derive(Debug, Clone)]
pub enum Cell {
    Logical(Vec<Option<bool>>),
    Bit(Vec<bool>),
    Byte(Vec<u8>),
    Character(String),
    UnicodeCharacter(String),
    Integer16(Vec<Option<i16>>),
    Integer32(Vec<Option<i32>>),
    Integer64(Vec<Option<i64>>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    Complex32(Vec<(f32, f32)>),
    Complex64(Vec<(f64, f64)>),
}

#[derive(Debug, Clone, PartialEq)]
enum NullableDataValue {
    Integer16(i16),
    Integer32(i32),
    Integer64(i64),
}

trait Nullable {
    fn to_nullable(self) -> NullableDataValue;
}

impl Nullable for i16 {
    fn to_nullable(self) -> NullableDataValue {
        NullableDataValue::Integer16(self)
    }
}

impl Nullable for i32 {
    fn to_nullable(self) -> NullableDataValue {
        NullableDataValue::Integer32(self)
    }
}

impl Nullable for i64 {
    fn to_nullable(self) -> NullableDataValue {
        NullableDataValue::Integer64(self)
    }
}

impl VOTable {
    pub fn parse<R: Read>(r: R) -> Result<Self, Error> {
        let parser = ParserConfig::new()
            // Cannot trim whitespaces as white spaces are significant for some string types
            // .trim_whitespace(true)
            .cdata_to_characters(true)
            .create_reader(r);

        let mut events = parser.into_iter();
        let mut table = VOTable::default();
        while let Some(event) = events.next() {
            let event = event?;
            if let StartElement {
                name: OwnedName { local_name, .. },
                ..
            } = event
            {
                match local_name.as_str() {
                    "DESCRIPTION" => if table.description.is_none() {
                        table.description = Some(Description::parse(&mut events)?);
                    },
                    "RESOURCE" => table.resources.push(Resource::parse(&mut events)?),
                    _ => (),
                }
            }
        }
        Ok(table)
    }

    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Iterate over all the tables in the VOTable, included nested ones.
    pub fn tables(&self) -> impl Iterator<Item = &Table> {
        self.resources
            .iter()
            .map(|resource| {
                resource.tables().iter().chain(
                    resource
                        .children()
                        .iter()
                        .map(|child_resource| child_resource.tables())
                        .flatten(),
                )
            }).flatten()
    }
}

impl Description {
    fn parse<R: Read>(events: &mut Events<R>) -> Result<Self, Error> {
        let mut description = None;
        for event in events {
            match event? {
                Characters(content) => description = Some(Description { content }),
                EndElement { .. } => break,
                _ => (),
            }
        }
        description.ok_or(Error::ContentNotFound { tag: "DESCRIPTION" })
    }
}

impl Resource {
    fn parse<R: Read>(events: &mut Events<R>) -> Result<Self, Error> {
        let mut resource = Resource::default();
        let mut depth = 0;
        while let Some(event) = events.next() {
            match event? {
                StartElement {
                    name: OwnedName { local_name, .. },
                    ..
                } => match local_name.as_str() {
                    "INFO" => resource.infos.push(Info::parse(events)?),
                    "TABLE" => resource.tables.push(Table::parse(events)?),
                    "RESOURCE" => resource.child_resources.push(Resource::parse(events)?),
                    _ => depth += 1,
                },
                EndElement { .. } => {
                    depth -= 1;
                    if depth == -1 {
                        break;
                    }
                }
                _ => (),
            }
        }
        Ok(resource)
    }

    pub fn tables(&self) -> &[Table] {
        &self.tables
    }

    pub fn children(&self) -> &[Resource] {
        &self.child_resources
    }
}

impl Info {
    fn parse<R: Read>(events: &mut Events<R>) -> Result<Self, Error> {
        // TODO
        let info = Info::default();
        let mut depth = 0;
        for event in events {
            match event? {
                StartElement { .. } => depth += 1,
                EndElement { .. } => {
                    depth -= 1;
                    if depth == -1 {
                        break;
                    }
                }
                _ => (),
            }
        }
        Ok(info)
    }
}

impl Table {
    fn parse<R: Read>(events: &mut Events<R>) -> Result<Self, Error> {
        let mut table = Table::default();
        let mut depth = 0;
        while let Some(event) = events.next() {
            match event? {
                StartElement {
                    name: OwnedName { local_name, .. },
                    attributes,
                    ..
                } => match local_name.as_str() {
                    "FIELD" => {
                        let field = Field::parse(attributes, events)?;
                        table.fields.push(field);
                    }
                    "DATA" => {
                        let data = Data::parse(&table.fields, events)?;
                        table.data = Some(data);
                    }
                    _ => depth += 1,
                },
                EndElement { .. } => {
                    depth -= 1;
                    if depth == -1 {
                        break;
                    }
                }
                _ => (),
            }
        }
        Ok(table)
    }

    pub fn rows(&self) -> Option<impl Iterator<Item = Row<'_>>> {
        let fields = &self.fields;
        self.data
            .as_ref()
            .map(|data| data.rows.iter().map(move |row| Row { fields, row }))
    }
}

pub struct Row<'a> {
    fields: &'a [Field],
    row: &'a OwnedRow,
}

impl<'a> Row<'a> {
    pub fn get_by_ucd(&self, ucd: &str) -> Option<&Cell> {
        for (cell, field) in self.row.cells.iter().zip(self.fields) {
            if let Some(check_ucd) = &field.ucd {
                if check_ucd == ucd {
                    return Some(cell);
                }
            }
        }
        None
    }
}

impl Field {
    fn parse<R: Read>(
        attributes: Vec<OwnedAttribute>,
        events: &mut Events<R>,
    ) -> Result<Self, Error> {
        let mut field = Field::default();

        for OwnedAttribute {
            name: OwnedName { local_name, .. },
            value,
        } in attributes
        {
            match local_name.as_str() {
                "ID" => field.id = Some(value),
                "name" => field.name = Some(value),
                "datatype" => field.datatype = Some(DataType::from_str(&value)?),
                "arraysize" => field.arraysize = Some(ArraySize::from_str(&value)?),
                "width" => {
                    field.width = Some(match FromStr::from_str(&value) {
                        Ok(width) => width,
                        Err(e) => {
                            return Err(Error::CannotParseIntAttribute {
                                e,
                                attribute: "width",
                            })
                        }
                    })
                }
                "precision" => field.precision = Some(Precision::from_str(&value)?),
                "xtype" => field.xtype = Some(XType::from_str(&value)?),
                "unit" => field.unit = Some(value),
                "ucd" => field.ucd = Some(value),
                _ => (),
            }
        }

        let mut depth = 0;
        while let Some(event) = events.next() {
            match event? {
                StartElement {
                    name: OwnedName { local_name, .. },
                    attributes,
                    ..
                } => match local_name.as_str() {
                    "DESCRIPTION" => field.description = Some(Description::parse(events)?),
                    "VALUES" => if let Some(datatype) = field.datatype {
                        field.values = Some(Values::parse(datatype, attributes, events)?)
                    },
                    _ => depth += 1,
                },
                EndElement { .. } => {
                    depth -= 1;
                    if depth == -1 {
                        break;
                    }
                }
                _ => (),
            }
        }
        Ok(field)
    }

    // Return None if variable length, some length otherwise (in number of records).
    fn len(&self) -> Option<usize> {
        match self.arraysize {
            Some(ArraySize::Fixed(max)) => Some(max),
            Some(_) => None,
            None => Some(1),
        }
    }

    fn is_null<T: Nullable>(&self, t: T) -> bool {
        if let Some(values) = &self.values {
            values.is_null(t)
        } else {
            false
        }
    }
}

impl Values {
    fn parse<R: Read>(
        datatype: DataType,
        attributes: Vec<OwnedAttribute>,
        events: &mut Events<R>,
    ) -> Result<Self, Error> {
        let mut values = Values::default();

        for OwnedAttribute {
            name: OwnedName { local_name, .. },
            value,
        } in attributes
        {
            if local_name == "null" {
                values.null = Some(NullableDataValue::parse(datatype, &value)?);
            }
        }

        let mut depth = 0;
        for event in events {
            match event? {
                StartElement { .. } => depth += 1,
                EndElement { .. } => {
                    depth -= 1;
                    if depth == -1 {
                        break;
                    }
                }
                _ => (),
            }
        }
        Ok(values)
    }

    fn is_null<T: Nullable>(&self, t: T) -> bool {
        if let Some(null) = &self.null {
            null == &t.to_nullable()
        } else {
            false
        }
    }
}

impl Data {
    fn parse<R: Read>(fields: &[Field], events: &mut Events<R>) -> Result<Self, Error> {
        let mut data = Data::default();

        let mut depth = 0;
        while let Some(event) = events.next() {
            match event? {
                StartElement {
                    name: OwnedName { local_name, .. },
                    ..
                } => match local_name.as_str() {
                    "TABLEDATA" => unimplemented!("TABLEDATA"),
                    "FITS" => unimplemented!("FITS"),
                    "BINARY" => data = Data::parse_binary(fields, events)?,
                    "BINARY2" => unimplemented!("BINARY2"),
                    _ => depth += 1,
                },
                EndElement { .. } => {
                    depth -= 1;
                    if depth == -1 {
                        break;
                    }
                }
                _ => (),
            }
        }

        Ok(data)
    }

    fn parse_binary<R: Read>(fields: &[Field], events: &mut Events<R>) -> Result<Self, Error> {
        let mut data = Data::default();

        let mut depth = 0;
        while let Some(event) = events.next() {
            match event? {
                StartElement {
                    name: OwnedName { local_name, .. },
                    attributes,
                    ..
                } => match local_name.as_str() {
                    "STREAM" => data = Data::parse_binary_stream(fields, &attributes, events)?,
                    _ => depth += 1,
                },
                EndElement { .. } => {
                    depth -= 1;
                    if depth == -1 {
                        break;
                    }
                }
                _ => (),
            }
        }

        Ok(data)
    }

    fn parse_binary_stream<R: Read>(
        fields: &[Field],
        attributes: &[OwnedAttribute],
        events: &mut Events<R>,
    ) -> Result<Self, Error> {
        let encoding = attributes
            .iter()
            .find(|attr| attr.name.local_name == "encoding")
            .ok_or_else(|| Error::CannotParse {
                got: "encoding is missing".to_owned(),
                target: "BINARY > STREAM",
            })?;

        let mut depth = 0;
        let mut some_input = None;
        for event in events {
            match event? {
                Characters(input) => some_input = Some(input),
                StartElement { .. } => depth += 1,
                EndElement { .. } => {
                    depth -= 1;
                    if depth == -1 {
                        break;
                    }
                }
                _ => (),
            }
        }

        let bytes = match encoding.value.as_str() {
            "base64" => if let Some(input) = some_input {
                // We need to strip spaces and newlines from input before decoding it
                let mut stripped_input = String::with_capacity(input.len());
                for chunk in input.split_whitespace() {
                    stripped_input.push_str(chunk);
                }
                match base64::decode(&stripped_input) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        return Err(Error::CannotParse {
                            got: format!("{}", e),
                            target: "BINARY > STREAM",
                        })
                    }
                }
            } else {
                return Err(Error::CannotParse {
                    got: "No input defined in STREAM!".to_owned(),
                    target: "BINARY > STREAM",
                });
            },
            encoding => {
                return Err(Error::CannotParse {
                    got: format!("Cannot parse encoding {}", encoding),
                    target: "BINARY > STREAM",
                })
            }
        };

        let mut bytes = Cursor::new(bytes);
        let mut data = Data::default();
        'end: loop {
            let mut row = OwnedRow::default();
            for field in fields {
                let cell;
                let len = if let Some(len) = field.len() {
                    len
                } else {
                    match bytes.read_i32::<BigEndian>() {
                        Ok(len) => {
                            assert!(len >= 0, "Length must bust be positive");
                            len as usize
                        }
                        Err(_) => break 'end,
                    }
                };
                match field.datatype.ok_or_else(|| Error::CannotParse {
                    got: format!("Cannot parse field {:?}. Missing datatype", field.name),
                    target: "BINARY > STREAM",
                })? {
                    DataType::Byte => {
                        let mut buf = vec![0; len];
                        bytes.read_exact(&mut buf).expect("No read error");
                        cell = Cell::Byte(buf)
                    }
                    DataType::Character => {
                        let mut buf = vec![0; len];
                        bytes.read_exact(&mut buf).expect("No read error");
                        if let Some(last) = buf.iter().position(|b| *b == 0) {
                            buf.truncate(last);
                        }
                        cell = Cell::Character(String::from_utf8_lossy(&buf).to_string())
                    }
                    DataType::Integer32 => {
                        let mut buf = vec![0; len];
                        bytes
                            .read_i32_into::<BigEndian>(&mut buf)
                            .expect("No read error");
                        cell = Cell::Integer32(
                            buf.into_iter()
                                .map(|int| if field.is_null(int) { None } else { Some(int) })
                                .collect(),
                        )
                    }
                    DataType::Float32 => {
                        let mut buf = vec![0.0; len];
                        bytes
                            .read_f32_into::<BigEndian>(&mut buf)
                            .expect("No read error");
                        cell = Cell::Float32(buf)
                    }
                    DataType::Float64 => {
                        let mut buf = vec![0.0; len];
                        bytes
                            .read_f64_into::<BigEndian>(&mut buf)
                            .expect("No read error");
                        cell = Cell::Float64(buf)
                    }
                    e => unimplemented!("{:?}", e),
                }
                row.cells.push(cell);
            }
            data.rows.push(row);
        }
        Ok(data)
    }
}

impl FromStr for DataType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s {
            "boolean" => DataType::Logical,
            "bit" => DataType::BitArray,
            "unsignedByte" => DataType::Byte,
            "short" => DataType::Integer16,
            "int" => DataType::Integer32,
            "long" => DataType::Integer64,
            "char" => DataType::Character,
            "unicodeChar" => DataType::UnicodeCharacter,
            "float" => DataType::Float32,
            "double" => DataType::Float64,
            "floatComplex" => DataType::Complex32,
            "doubleComplex" => DataType::Complex64,
            s => {
                return Err(Error::CannotParse {
                    got: s.to_owned(),
                    target: "datatype",
                })
            }
        })
    }
}
impl FromStr for ArraySize {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        if s == "*" {
            Ok(ArraySize::Unbounded)
        } else if s.ends_with('*') {
            let mut split = s.split('*');
            match split.next().unwrap().parse() {
                Ok(max) => Ok(ArraySize::Variable { max }),
                Err(_) => Err(Error::CannotParse {
                    got: s.to_owned(),
                    target: "arraysize",
                }),
            }
        } else {
            match s.parse() {
                Ok(max) => Ok(ArraySize::Fixed(max)),
                Err(_) => Err(Error::CannotParse {
                    got: s.to_owned(),
                    target: "arraysize",
                }),
            }
        }
    }
}

impl FromStr for Precision {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        if s.starts_with('E') {
            let mut split = s.split('E');
            split.next();
            if let Some(number) = split.next() {
                let precision = number.parse().map_err(|_| Error::CannotParse {
                    got: s.to_owned(),
                    target: "precision",
                })?;
                Ok(Precision::SignificantFigures(precision))
            } else {
                Err(Error::CannotParse {
                    got: s.to_owned(),
                    target: "precision",
                })
            }
        } else {
            let number = if s.starts_with('F') {
                let mut split = s.split('F');
                split.next();
                if let Some(number) = split.next() {
                    number
                } else {
                    return Err(Error::CannotParse {
                        got: s.to_owned(),
                        target: "precision",
                    });
                }
            } else {
                s
            };
            let precision = number.parse().map_err(|_| Error::CannotParse {
                got: s.to_owned(),
                target: "precision",
            })?;
            Ok(Precision::AfterDecimalPoint(precision))
        }
    }
}

impl FromStr for XType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(Self {
            value: s.to_owned(),
        })
    }
}

impl NullableDataValue {
    fn parse(datatype: DataType, s: &str) -> Result<Self, Error> {
        match datatype {
            DataType::Integer16 => {
                let int = s.parse().map_err(|_| Error::CannotParse {
                    got: s.to_owned(),
                    target: "null",
                })?;
                Ok(NullableDataValue::Integer16(int))
            }
            DataType::Integer32 => {
                let int = s.parse().map_err(|_| Error::CannotParse {
                    got: s.to_owned(),
                    target: "null",
                })?;
                Ok(NullableDataValue::Integer32(int))
            }
            DataType::Integer64 => {
                let int = s.parse().map_err(|_| Error::CannotParse {
                    got: s.to_owned(),
                    target: "null",
                })?;
                Ok(NullableDataValue::Integer64(int))
            }
            _ => Err(Error::CannotParse {
                got: format!("{} as {:?}", s, datatype),
                target: "null",
            }),
        }
    }
}
