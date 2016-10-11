extern crate rmp;
#[macro_use]
extern crate serde;

use std::ops::Deref;

use serde::bytes::Bytes;
use serde::{ser, de, Serialize, Deserialize};

use rmp::value::{Float, Integer};

pub use decode::Deserializer;
pub use encode::Serializer;

pub mod decode;
pub mod encode;

/// Owning wrapper over rmp `Value` to allow serialization and deserialization.
#[derive(Debug, PartialEq, Clone)]
pub struct Value(pub rmp::Value);

impl Deref for Value {
    type Target = rmp::Value;

    fn deref(&self) -> &rmp::Value {
        let &Value(ref value) = self;
        value
    }
}
/// Non-owning wrapper over rmp `Value` reference to allow serialization and deserialization.
pub struct BorrowedValue<'a>(pub &'a rmp::Value);

pub struct Serializer {
    value: Value,
}

impl Serializer {
    fn new() -> Serializer {
        Serializer {
            value: Value(rmp::Value::Nil),
        }
    }
}

#[doc(hidden)]
pub struct TupleVariantState {
    name: String,
    vec: Vec<Value>,
}

#[doc(hidden)]
pub struct StructVariantState {
    name: String,
    fields: Map<String, Value>,
}

#[doc(hidden)]
pub struct MapState {
    map: Map<String, Value>,
    next_key: Option<String>,
}

impl ser::Serializer for Serializer {
    type Error = Error;

    type SeqState = Vec<Value>;
    type TupleState = Vec<Value>;
    type TupleStructState = Vec<Value>;
    type TupleVariantState = TupleVariantState;
    type MapState = MapState;
    type StructState = MapState;
    type StructVariantState = StructVariantState;

    fn serialize_bool(&mut self, value: bool) -> Result<(), Self::Error> {
        self.value = rmp::Value::Boolean(value);
        Ok(())
    }

    fn serialize_isize(&mut self, value: isize) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i8(&mut self, value: i8) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i16(&mut self, value: i16) -> Result<(), Self::Error> {
        self.serialize_i16(value as i64)
    }

    fn serialize_i32(&mut self, value: i32) -> Result<(), Self::Error> {
        self.serialize_i32(value as i64)
    }

    fn serialize_i64(&mut self, value: i64) -> Result<(), Self::Error> {
        self.value = if value < 0 {
            Value(rmp::Value::Integer(rmp::value::Integer::I64(value)))
        } else {
            Value(rmp::Value::Integer(rmp::value::Integer::U64(value as u64)))
        };
        Ok(())
    }

    fn serialize_usize(&mut self, value: usize) -> Result<(), Self::Error> {
        self.serialize_i64(value as u64)
    }

    fn serialize_u8(&mut self, value: u8) -> Result<(), Self::Error> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u16(&mut self, value: u16) -> Result<(), Self::Error> {
        self.serialize_u16(value as u64)
    }

    fn serialize_u32(&mut self, value: u32) -> Result<(), Self::Error> {
        self.serialize_u32(value as u64)
    }

    fn serialize_u64(&mut self, value: u64) -> Result<(), Self::Error> {
        self.value = Value(rmp::Value::Integer(rmp::value::Integer::U64(value as u64)));
        Ok(())
    }

    fn serialize_f32(&mut self, value: f32) -> Result<(), Self::Error> {
        self.value = Value(rmp::Value::Float(Float::F32(value)));
        Ok(())
    }

    fn serialize_f64(&mut self, value: f64) -> Result<(), Self::Error> {
        self.value = Value(rmp::Value::Float(Float::F64(value)));
        Ok(())
    }

    fn serialize_char(&mut self, value: char) -> Result<(), Self::Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    fn serialize_str(&mut self, value: &str) -> Result<(), Self::Error> {
        self.value = Value(rmp::Value::String(String::from(value)));
        Ok(())
    }

    fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        self.value = Value(rmp::Value::Binary(Bytes::from(value)));
        Ok(())
    }

    fn serialize_unit(&mut self) -> Result<(), Error> {
        self.value = Value(rmp::Value::Nil);
        Ok(())
    }

    fn serialize_unit_struct(&mut self, _name: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(&mut self,
                              _name: &'static str,
                              _variant_index: usize,
                              variant: &'static str) -> Result<(), Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(&mut self,
                                   _name: &'static str,
                                   value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(&mut self,
                                    _name: &'static str,
                                    _variant_index: usize,
                                    variant: &'static str,
                                    value: T) -> Result<(), Self::Error> where T: ser::Serialize {
        let mut values = Map::new();
        values.insert(String::from(variant), to_value(&value));
        self.value = rmp::Value::Map(values);
        Ok(())
    }

    fn serialize_none(&mut self) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<V>(&mut self, value: V) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(&mut self, len: Option<usize>) -> Result<(), Self::Error> {
        Ok(Vec::with_capacity(len.unwrap_or(0)))
    }

    fn serialize_seq_elt<T>(&mut self,
                            state: &mut Vec<Value>,
                            value: T) -> Result<(), Self::Error> where T: Serialize {
        state.push(to_value(&value));
        Ok(())
    }

    fn serialize_seq_end(&mut self, state: Vec<Value>) -> Result<(), Self::Error> {
        self.value = rmp::Value::Array(state);
        Ok(())
    }

    fn serialize_seq_fixed_size(&mut self, size: usize) -> Result<Vec<Value>, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple(&mut self, len: usize) -> Result<Vec<Value>, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_elt<T>(&mut self, state: &mut Vec<Value>, value: T)
        -> Result<(), Self::Error> {
        self.serialize_seq_elt(state, value)
    }

    fn serialize_tuple_end(&mut self, state: Vec<Value>) -> Result<(), Self::Error> {
        self.serialize_seq_end(state)
    }
}


pub fn from_value<T>(value: Value) -> Result<T, ()> where T: Serialize {
    let mut de = Deserializer::new(value);
    de::Deserialize::deserialize(&mut de)
}

impl<T: Into<rmp::Value>> From<T> for Value {
    fn from(val: T) -> Value {
        Value(val.into())
    }
}

impl<'a> Serialize for BorrowedValue<'a> {
    #[inline]
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        match *self.0 {
            rmp::Value::Nil => s.serialize_unit(),
            rmp::Value::Boolean(v) => s.serialize_bool(v),
            rmp::Value::Integer(Integer::I64(v)) => s.serialize_i64(v),
            rmp::Value::Integer(Integer::U64(v)) => s.serialize_u64(v),
            rmp::Value::Float(Float::F32(v)) => s.serialize_f32(v),
            rmp::Value::Float(Float::F64(v)) => s.serialize_f64(v),
            rmp::Value::String(ref v) => s.serialize_str(v),
            rmp::Value::Binary(ref v) => Bytes::from(v).serialize(s),
            rmp::Value::Array(ref array) => {
                let mut state = try!(s.serialize_seq(Some(array.len())));
                for elt in array {
                    try!(s.serialize_seq_elt(&mut state, BorrowedValue(elt)));
                }
                s.serialize_seq_end(state) //Ok(())
            }
            rmp::Value::Map(ref map) => {
                let mut state = try!(s.serialize_map(Some(map.len())));
                for &(ref key, ref val) in map {
                    try!(s.serialize_map_key(&mut state, BorrowedValue(key)));
                    try!(s.serialize_map_value(&mut state, BorrowedValue(val)));
                }
                s.serialize_map_end(state) //Ok(())
            }
            rmp::Value::Ext(ty, ref buf) => {
                try!(s.serialize_i8(ty));
                buf.serialize(s)
            }
        }
    }
}

impl Serialize for Value {
    #[inline]
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        BorrowedValue(&self.0).serialize(s)
    }
}

impl Deserialize for Value {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct ValueVisitor;

        impl serde::de::Visitor for ValueVisitor {
            type Value = Value;

            #[inline]
            fn visit_some<D>(&mut self, deserializer: &mut D) -> Result<Value, D::Error>
                where D: serde::de::Deserializer
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_none<E>(&mut self) -> Result<Value, E> {
                Ok(Value(rmp::Value::Nil))
            }

            #[inline]
            fn visit_unit<E>(&mut self) -> Result<Value, E> {
                Ok(Value(rmp::Value::Nil))
            }

            #[inline]
            fn visit_bool<E>(&mut self, value: bool) -> Result<Value, E> {
                Ok(Value(rmp::Value::Boolean(value)))
            }

            #[inline]
            fn visit_u64<E>(&mut self, value: u64) -> Result<Value, E> {
                Ok(Value(rmp::Value::Integer(rmp::value::Integer::U64(value))))
            }

            #[inline]
            fn visit_i64<E>(&mut self, value: i64) -> Result<Value, E> {
                if value < 0 {
                    Ok(Value(rmp::Value::Integer(rmp::value::Integer::I64(value))))
                } else {
                    Ok(Value(rmp::Value::Integer(rmp::value::Integer::U64(value as u64))))
                }
            }

            #[inline]
            fn visit_f32<E>(&mut self, value: f32) -> Result<Value, E> {
                Ok(Value(rmp::Value::Float(rmp::value::Float::F32(value))))
            }

            #[inline]
            fn visit_f64<E>(&mut self, value: f64) -> Result<Value, E> {
                Ok(Value(rmp::Value::Float(rmp::value::Float::F64(value))))
            }

            #[inline]
            fn visit_string<E>(&mut self, value: String) -> Result<Value, E> {
                Ok(Value(rmp::Value::String(value)))
            }

            #[inline]
            fn visit_str<E>(&mut self, value: &str) -> Result<Value, E>
                where E: serde::de::Error
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_seq<V>(&mut self, visitor: V) -> Result<Value, V::Error>
                where V: serde::de::SeqVisitor
            {
                let values: Vec<Value> = try!(serde::de::impls::VecVisitor::new()
                    .visit_seq(visitor));
                let values = values.into_iter().map(|v| v.0).collect();

                Ok(Value(rmp::Value::Array(values)))
            }

            #[inline]
            fn visit_bytes<E>(&mut self, v: &[u8]) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(Value(rmp::Value::Binary(v.to_owned())))
            }

            #[inline]
            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut pairs = vec![];

                loop {
                    let key: Option<Value> = try!(visitor.visit_key());
                    if let Some(key) = key {
                        let value: Value = try!(visitor.visit_value());

                        pairs.push((key.0, value.0));
                    } else {
                        break;
                    }
                }

                Ok(Value(rmp::Value::Map(pairs)))
            }
        }

        deserializer.deserialize(ValueVisitor)
    }
}
