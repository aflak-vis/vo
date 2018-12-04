extern crate xml;

use std::error;
use std::io::Read;
use std::str::FromStr;

use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{
        self, Events,
        XmlEvent::{self, *},
    },
    ParserConfig,
};

pub fn parse<R: Read>(r: R) -> Result<VOTable, Error> {
    VOTable::parse(r)
}

#[derive(Debug, Clone, Default)]
pub struct VOTable {
    description: Option<Description>,
    resources: Vec<Resource>,
}

#[derive(Debug, Clone, Default)]
struct Resource {
    description: Option<Description>,
    infos: Vec<Info>,
    tables: Vec<Table>,
    child_resources: Vec<Resource>,
}

#[derive(Debug, Clone, Default)]
struct Info {}

#[derive(Debug, Clone, Default)]
struct Table {
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
    // precision: Option<usize>,
    xtype: Option<XType>,
    unit: Option<String>,
    ucd: Option<String>,
    description: Option<Description>,
    values: Option<Values>,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
enum ArraySize {
    Unbounded,
    Variable { max: usize },
    Fixed(usize),
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

#[derive(Debug, Clone)]
struct Data {
    rows: Vec<Row>,
}

#[derive(Debug, Clone)]
struct Row {
    cells: Vec<Cell>,
}

#[derive(Debug, Clone)]
struct Cell {
    v: Vec<DataValue>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
enum NullableDataValue {
    Byte(u8),
    Character(u8),
    UnicodeCharacter(char),
    Integer16(i16),
    Integer32(i32),
    Integer64(i64),
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
                            return Err(Error::CannotParseAttribute {
                                e: Box::new(e),
                                attribute: "width",
                            })
                        }
                    })
                }
                // Precision uses "Fn"/"En"/"n" syntaxes
                // "precision" => {
                //     field.precision = Some(match FromStr::from_str(&value) {
                //         Ok(precision) => precision,
                //         Err(e) => {
                //             return Err(Error::CannotParseAttribute {
                //                 e: Box::new(e),
                //                 attribute: "precision",
                //             })
                //         }
                //     })
                // }
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
}

#[derive(Debug)]
pub enum Error {
    XmlReaderError(reader::Error),
    ContentNotFound {
        tag: &'static str,
    },
    CannotParseAttribute {
        e: Box<error::Error>,
        attribute: &'static str,
    },
    CannotParse {
        got: String,
        target: &'static str,
    },
}

impl From<reader::Error> for Error {
    fn from(e: reader::Error) -> Self {
        Error::XmlReaderError(e)
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
            DataType::Byte => {
                let b = s.parse().map_err(|_| Error::CannotParse {
                    got: s.to_owned(),
                    target: "null",
                })?;
                Ok(NullableDataValue::Byte(b))
            }
            DataType::Character => {
                let chars = s.as_bytes();
                if chars.len() > 0 {
                    Ok(NullableDataValue::Character(chars[0]))
                } else {
                    Err(Error::CannotParse {
                        got: s.to_owned(),
                        target: "null",
                    })
                }
            }
            DataType::UnicodeCharacter => {
                let mut chars = s.chars();
                if let Some(c) = chars.next() {
                    Ok(NullableDataValue::UnicodeCharacter(c))
                } else {
                    Err(Error::CannotParse {
                        got: s.to_owned(),
                        target: "null",
                    })
                }
            }
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
