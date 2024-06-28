use std::{
    borrow::Cow,
    collections::HashMap,
    convert::TryFrom,
    fmt::{self, Formatter},
};

use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{Error, Map};

/// Number.
#[derive(Debug, Copy, Clone)]
pub enum Number {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
}

macro_rules! try_from_number {
    ( $x:ty, $y:ident ) => {
        impl TryFrom<Number> for $x {
            type Error = Error;

            #[inline]
            fn try_from(n: Number) -> Result<Self, Self::Error> {
                match n {
                    Number::$y(v) => Ok(v),
                    _ => Err(Error::UnsupportedConversion),
                }
            }
        }
    };
}

try_from_number!(i8, I8);
try_from_number!(i16, I16);
try_from_number!(i32, I32);
try_from_number!(i64, I64);
try_from_number!(i128, I128);

try_from_number!(u8, U8);
try_from_number!(u16, U16);
try_from_number!(u32, U32);
try_from_number!(u64, U64);
try_from_number!(u128, U128);

try_from_number!(f32, F32);
try_from_number!(f64, F64);

impl TryFrom<Number> for isize {
    type Error = Error;
    fn try_from(n: Number) -> Result<Self, Self::Error> {
        match n {
            Number::I64(v) => Ok(v.try_into().map_err(|_| Error::UnsupportedConversion)?),
            Number::I32(v) => Ok(v.try_into().map_err(|_| Error::UnsupportedConversion)?),
            _ => Err(Error::UnsupportedConversion),
        }
    }
}

impl TryFrom<Number> for usize {
    type Error = Error;
    fn try_from(n: Number) -> Result<Self, Self::Error> {
        match n {
            Number::U64(v) => Ok(v.try_into().map_err(|_| Error::UnsupportedConversion)?),
            Number::U32(v) => Ok(v.try_into().map_err(|_| Error::UnsupportedConversion)?),
            _ => Err(Error::UnsupportedConversion),
        }
    }
}

impl Serialize for Number {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Number::I8(v) => serializer.serialize_i8(v),
            Number::I16(v) => serializer.serialize_i16(v),
            Number::I32(v) => serializer.serialize_i32(v),
            Number::I64(v) => serializer.serialize_i64(v),
            Number::I128(v) => serializer.serialize_i128(v),
            Number::U8(v) => serializer.serialize_u8(v),
            Number::U16(v) => serializer.serialize_u16(v),
            Number::U32(v) => serializer.serialize_u32(v),
            Number::U64(v) => serializer.serialize_u64(v),
            Number::U128(v) => serializer.serialize_u128(v),
            Number::F32(v) => serializer.serialize_f32(v),
            Number::F64(v) => serializer.serialize_f64(v),
        }
    }
}

impl<'de> Deserialize<'de> for Number {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NumberVisitor;

        impl<'a> Visitor<'a> for NumberVisitor {
            type Value = Number;

            #[inline]
            fn expecting(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
                f.write_str("a number")
            }

            #[inline]
            fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E> {
                Ok(Number::I8(value))
            }

            #[inline]
            fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E> {
                Ok(Number::I16(value))
            }

            #[inline]
            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E> {
                Ok(Number::I32(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Number::I64(value))
            }

            #[inline]
            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E> {
                Ok(Number::U8(value))
            }

            #[inline]
            fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E> {
                Ok(Number::U16(value))
            }

            #[inline]
            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E> {
                Ok(Number::U32(value))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Number::U64(value))
            }

            #[inline]
            fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E> {
                Ok(Number::F32(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(Number::F64(value))
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}

/// Construct the intermediate value directly using JSON syntax.
///
/// # Example
/// ```rust
/// use serde_lite::intermediate;
///
/// let data = intermediate!({
///     "field1": null,
///     "field2": "foo",
///     "field3": 33,
///     "field4": 44.4,
///     "field5": [1, 2, 3, 4, 5],
///     "field6": {
///         "inner_field": "Hello!",
///     }
/// });
/// ```
#[macro_export]
macro_rules! intermediate {
    ({ $($key:literal : $value:tt),* $(,)? }) => {
        $crate::Intermediate::Map({
            let mut map = $crate::Map::new();
            $(
                map.insert_with_static_key($key, intermediate!($value));
            )*
            map
        })
    };

    ([ $($item:tt),* $(,)? ]) => {
        $crate::Intermediate::Array({
            let mut arr = Vec::new();
            $(
                arr.push(intermediate!($item));
            )*
            arr
        })
    };

    (null) => {
        $crate::Intermediate::None
    };

    ($value:expr) => {
        $crate::Intermediate::from($value)
    };
}

/// Intermediate data representation.
///
/// The format is similar to JSON. It can be serialized/deserialized using
/// serde.
#[derive(Debug, Clone)]
pub enum Intermediate {
    None,
    Bool(bool),
    Number(Number),
    String(Cow<'static, str>),
    Array(Vec<Intermediate>),
    Map(Map),
}

impl Intermediate {
    /// Check if the value is None.
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Get the value as a boolean (if possible).
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    /// Get the numeric value (if possible).
    #[inline]
    pub fn as_number(&self) -> Option<Number> {
        if let Self::Number(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    /// Get the value as a character (if possible).
    #[inline]
    pub fn as_char(&self) -> Option<char> {
        if let Some(s) = self.as_str() {
            let mut chars = s.chars();

            let first = chars.next();
            let second = chars.next();

            if second.is_some() {
                None
            } else {
                first
            }
        } else {
            None
        }
    }

    /// Get the value as a string (if possible).
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Get the value as an array (if possible).
    #[inline]
    pub fn as_array(&self) -> Option<&[Intermediate]> {
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Get the value as a map (if possible).
    #[inline]
    pub fn as_map(&self) -> Option<&Map> {
        if let Self::Map(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl From<()> for Intermediate {
    #[inline]
    fn from(_: ()) -> Self {
        Self::None
    }
}

impl From<bool> for Intermediate {
    #[inline]
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<Number> for Intermediate {
    #[inline]
    fn from(v: Number) -> Self {
        Self::Number(v)
    }
}

macro_rules! intermediate_from_number {
    ( $x:ty, $y:expr ) => {
        impl From<$x> for Intermediate {
            #[inline]
            fn from(v: $x) -> Self {
                Self::from($y(v.try_into().unwrap()))
            }
        }
    };
}

intermediate_from_number!(i8, Number::I8);
intermediate_from_number!(i16, Number::I16);
intermediate_from_number!(i32, Number::I32);
intermediate_from_number!(i64, Number::I64);
intermediate_from_number!(u8, Number::U8);
intermediate_from_number!(u16, Number::U16);
intermediate_from_number!(u32, Number::U32);
intermediate_from_number!(u64, Number::U64);
intermediate_from_number!(f32, Number::F32);
intermediate_from_number!(f64, Number::F64);

impl From<String> for Intermediate {
    #[inline]
    fn from(v: String) -> Self {
        Self::String(Cow::Owned(v))
    }
}

impl From<&str> for Intermediate {
    #[inline]
    fn from(v: &str) -> Self {
        Self::String(Cow::Owned(String::from(v)))
    }
}

impl<T> From<Vec<T>> for Intermediate
where
    Intermediate: From<T>,
{
    fn from(v: Vec<T>) -> Self {
        let mut res = Vec::with_capacity(v.len());

        for elem in v {
            res.push(elem.into());
        }

        Self::Array(res)
    }
}

impl<K, V> From<HashMap<K, V>> for Intermediate
where
    K: Into<Cow<'static, str>>,
    V: Into<Intermediate>,
{
    fn from(map: HashMap<K, V>) -> Self {
        let mut res = Map::with_capacity(map.len());

        for (k, v) in map {
            res.insert(k.into(), v.into());
        }

        Self::Map(res)
    }
}

#[cfg(feature = "preserve-order")]
impl<K, V> From<indexmap::IndexMap<K, V>> for Intermediate
where
    K: Into<Cow<'static, str>>,
    V: Into<Intermediate>,
{
    fn from(map: indexmap::IndexMap<K, V>) -> Self {
        let mut res = Map::with_capacity(map.len());

        for (k, v) in map {
            res.insert(k.into(), v.into());
        }

        Self::Map(res)
    }
}

impl crate::Serialize for Intermediate {
    #[inline]
    fn serialize(&self) -> Result<Intermediate, Error> {
        Ok(self.clone())
    }
}

impl crate::Deserialize for Intermediate {
    #[inline]
    fn deserialize(input: &Intermediate) -> Result<Self, Error> {
        Ok(input.clone())
    }
}

impl crate::Update for Intermediate {
    #[inline]
    fn update(&mut self, other: &Intermediate) -> Result<(), Error> {
        match self {
            Self::Array(arr) => {
                if let Self::Array(_) = other {
                    arr.update(other)?;
                } else {
                    *self = other.clone();
                }
            }
            Self::Map(map) => {
                if let Self::Map(_) = other {
                    map.update(other)?;
                } else {
                    *self = other.clone();
                }
            }
            _ => *self = other.clone(),
        }

        Ok(())
    }
}

impl Serialize for Intermediate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::None => serializer.serialize_none(),
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::Number(v) => v.serialize(serializer),
            Self::String(v) => serializer.serialize_str(v),
            Self::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for e in v {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            Self::Map(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;
                for (k, v) in v {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Intermediate {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Intermediate, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'a> Visitor<'a> for ValueVisitor {
            type Value = Intermediate;

            #[inline]
            fn expecting(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
                f.write_str("a value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(Intermediate::Bool(value))
            }

            #[inline]
            fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::I8(value)))
            }

            #[inline]
            fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::I16(value)))
            }

            #[inline]
            fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::I32(value)))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::I64(value)))
            }

            #[inline]
            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::U8(value)))
            }

            #[inline]
            fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::U16(value)))
            }

            #[inline]
            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::U32(value)))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::U64(value)))
            }

            #[inline]
            fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::F32(value)))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(Intermediate::Number(Number::F64(value)))
            }

            #[inline]
            fn visit_char<E>(self, value: char) -> Result<Self::Value, E> {
                Ok(Intermediate::String(Cow::Owned(value.to_string())))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Intermediate::String(Cow::Owned(String::from(value))))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(Intermediate::String(Cow::Owned(value)))
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E> {
                let mut res = Vec::with_capacity(value.len());
                for b in value {
                    res.push(Intermediate::Number(Number::U8(*b)));
                }
                Ok(Intermediate::Array(res))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Intermediate::None)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'a>,
            {
                Intermediate::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Intermediate::None)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'a>,
            {
                let mut res = Vec::new();

                if let Some(size) = seq.size_hint() {
                    res.reserve(size);
                }

                while let Some(elem) = seq.next_element()? {
                    res.push(elem);
                }

                Ok(Intermediate::Array(res))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'a>,
            {
                let mut res = Map::new();

                if let Some(size) = map.size_hint() {
                    res.reserve(size);
                }

                while let Some((k, v)) = map.next_entry()? {
                    res.insert(Cow::Owned(k), v);
                }

                Ok(Intermediate::Map(res))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
