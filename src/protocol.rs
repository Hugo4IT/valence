use std::io::{Read, Write};
use std::mem;

use anyhow::{anyhow, ensure};
use bitvec::prelude::*;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::var_int::VarInt;

/// Trait for types that can be written to the Minecraft protocol.
pub trait Encode {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()>;
}

/// Trait for types that can be constructed from the Minecraft protocol.
pub trait Decode: Sized {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self>;
}

/// The maximum number of bytes in a single packet.
pub const MAX_PACKET_SIZE: i32 = 2097151;

impl Encode for () {
    fn encode(&self, _w: &mut impl Write) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Decode for () {
    fn decode(_r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(())
    }
}

impl<T: Encode> Encode for &T {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        (*self).encode(w)
    }
}

impl Encode for bool {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_u8(*self as u8)?;
        Ok(())
    }
}

impl Decode for bool {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        let n = r.read_u8()?;
        ensure!(n < 2, "boolean is not 0 or 1");
        Ok(n == 1)
    }
}

impl Encode for u8 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_u8(*self)?;
        Ok(())
    }
}

impl Decode for u8 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(r.read_u8()?)
    }
}

impl Encode for i8 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_i8(*self)?;
        Ok(())
    }
}

impl Decode for i8 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(r.read_i8()?)
    }
}

impl Encode for u16 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_u16::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode for u16 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(r.read_u16::<BigEndian>()?)
    }
}

impl Encode for i16 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_i16::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode for i16 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(r.read_i16::<BigEndian>()?)
    }
}

impl Encode for u32 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_u32::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode for u32 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(r.read_u32::<BigEndian>()?)
    }
}

impl Encode for i32 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_i32::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode for i32 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(r.read_i32::<BigEndian>()?)
    }
}

impl Encode for u64 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_u64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode for u64 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(r.read_u64::<BigEndian>()?)
    }
}

impl Encode for i64 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_i64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode for i64 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(r.read_i64::<BigEndian>()?)
    }
}

impl Encode for f32 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        ensure!(
            self.is_finite(),
            "attempt to encode non-finite f32 ({})",
            self
        );
        w.write_f32::<BigEndian>(*self)?;
        Ok(())
    }
}
impl Decode for f32 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        let f = r.read_f32::<BigEndian>()?;
        ensure!(f.is_finite(), "attempt to decode non-finite f32 ({f})");
        Ok(f)
    }
}

impl Encode for f64 {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        ensure!(
            self.is_finite(),
            "attempt to encode non-finite f64 ({})",
            self
        );
        w.write_f64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl Decode for f64 {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        let f = r.read_f64::<BigEndian>()?;
        ensure!(f.is_finite(), "attempt to decode non-finite f64 ({f})");
        Ok(f)
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        match self {
            Some(t) => true.encode(w).and_then(|_| t.encode(w)),
            None => false.encode(w),
        }
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        if bool::decode(r)? {
            Ok(Some(T::decode(r)?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BoundedInt<T, const MIN: i64, const MAX: i64>(pub T);

impl<T, const MIN: i64, const MAX: i64> From<T> for BoundedInt<T, MIN, MAX> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

impl<T, const MIN: i64, const MAX: i64> Encode for BoundedInt<T, MIN, MAX>
where
    T: Encode + Copy + Into<i64>,
{
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        let val = self.0.into();
        ensure!(
            (MIN..=MAX).contains(&val),
            "Integer is not in bounds while encoding (got {val}, expected {MIN}..={MAX})"
        );

        self.0.encode(w)
    }
}

impl<T, const MIN: i64, const MAX: i64> Decode for BoundedInt<T, MIN, MAX>
where
    T: Decode + Copy + Into<i64>,
{
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        let res = T::decode(r)?;
        let val = res.into();

        ensure!(
            (MIN..=MAX).contains(&val),
            "Integer is not in bounds while decoding (got {val}, expected {MIN}..={MAX})"
        );

        Ok(Self(res))
    }
}

// TODO: bounded float?

impl Encode for String {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        encode_string_bounded(self, 0, 32767, w)
    }
}

impl Decode for String {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        decode_string_bounded(0, 32767, r)
    }
}

/// A string with a minimum and maximum character length known at compile time.
/// If the string is not in bounds, an error is generated while
/// encoding/decoding.
///
/// Note that the length is a count of the characters in the string, not bytes.
///
/// When encoded and decoded, the string is VarInt prefixed.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Debug)]
pub struct BoundedString<const MIN: usize, const MAX: usize>(pub String);

impl<const MIN: usize, const MAX: usize> Encode for BoundedString<MIN, MAX> {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        encode_string_bounded(&self.0, MIN, MAX, w)
    }
}

impl<const MIN: usize, const MAX: usize> Decode for BoundedString<MIN, MAX> {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        decode_string_bounded(MIN, MAX, r).map(Self)
    }
}

impl<const MIN: usize, const MAX: usize> From<String> for BoundedString<MIN, MAX> {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        encode_array_bounded(self, 0, usize::MAX, w)
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        decode_array_bounded(0, usize::MAX, r)
    }
}

impl<T: Encode> Encode for Box<[T]> {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        encode_array_bounded(self, 0, usize::MAX, w)
    }
}

impl<T: Decode> Decode for Box<[T]> {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        decode_array_bounded(0, usize::MAX, r).map(|v| v.into_boxed_slice())
    }
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        encode_array_bounded(self, N, N, w)
    }
}

impl<T: Decode, const N: usize> Decode for [T; N] {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        let vec = decode_array_bounded(N, N, r)?;
        match vec.try_into() {
            Ok(arr) => Ok(arr),
            Err(_) => unreachable!("array size does not match"),
        }
    }
}

/// An array with a minimum and maximum character length known at compile time.
/// If the array is not in bounds, an error is generated while
/// encoding/decoding.
///
/// When encoding/decoding, the array is VarInt prefixed.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Debug)]
pub struct BoundedArray<T, const MIN: usize = 0, const MAX: usize = { usize::MAX }>(pub Vec<T>);

impl<T: Encode, const MIN: usize, const MAX: usize> Encode for BoundedArray<T, MIN, MAX> {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        encode_array_bounded(&self.0, MIN, MAX, w)
    }
}

impl<T: Decode, const MIN: usize, const MAX: usize> Decode for BoundedArray<T, MIN, MAX> {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        decode_array_bounded(MIN, MAX, r).map(Self)
    }
}

impl<T, const MIN: usize, const MAX: usize> From<Vec<T>> for BoundedArray<T, MIN, MAX> {
    fn from(v: Vec<T>) -> Self {
        Self(v)
    }
}

impl Encode for Uuid {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_u128::<BigEndian>(self.as_u128())?;
        Ok(())
    }
}

impl Decode for Uuid {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(Uuid::from_u128(r.read_u128::<BigEndian>()?))
    }
}

impl Encode for nbt::Blob {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        Ok(nbt::to_writer(w, self, None)?)
    }
}

impl Decode for nbt::Blob {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        Ok(nbt::from_reader(r)?)
    }
}

/// Wrapper type acting as a bridge between Serde and [Encode]/[Decode] through
/// the NBT format.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Debug)]
pub struct Nbt<T>(pub T);

impl<T: Serialize> Encode for Nbt<T> {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        let mut enc = nbt::ser::Encoder::new(w, None);
        self.0.serialize(&mut enc)?;
        Ok(())
    }
}

impl<'a, T: Deserialize<'a>> Decode for Nbt<T> {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        let mut dec = nbt::de::Decoder::new(r);
        Ok(Nbt(Deserialize::deserialize(&mut dec)?))
    }
}

impl Encode for BitVec<u64> {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        encode_array_bounded(self.as_raw_slice(), 0, usize::MAX, w)
    }
}

impl Decode for BitVec<u64> {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        BitVec::try_from_vec(Vec::decode(r)?)
            .map_err(|_| anyhow!("Array is too long for bit vector"))
    }
}

impl Encode for BitBox<u64> {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        encode_array_bounded(self.as_raw_slice(), 0, usize::MAX, w)
    }
}

impl Decode for BitBox<u64> {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        BitVec::decode(r).map(|v| v.into_boxed_bitslice())
    }
}

/// When decoding, reads the rest of the data in a packet and stuffs it into a
/// `Vec<u8>`. When encoding, the data is inserted into the packet with no
/// length prefix.
#[derive(Clone, Debug)]
pub struct ReadToEnd(pub Vec<u8>);

impl Decode for ReadToEnd {
    fn decode(r: &mut impl Read) -> anyhow::Result<Self> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        Ok(ReadToEnd(buf))
    }
}

impl Encode for ReadToEnd {
    fn encode(&self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write_all(&self.0).map_err(|e| e.into())
    }
}

fn encode_array_bounded<T: Encode>(
    s: &[T],
    min: usize,
    max: usize,
    w: &mut impl Write,
) -> anyhow::Result<()> {
    assert!(min <= max);

    let len = s.len();

    ensure!(
        (min..=max).contains(&len),
        "Length of array is out of bounds while encoding (got {len}, expected {min}..={max})"
    );

    ensure!(
        len <= i32::MAX as usize,
        "Length of array ({len}) exceeds i32::MAX"
    );

    VarInt(len as i32).encode(w)?;
    for t in s {
        t.encode(w)?;
    }
    Ok(())
}

pub(crate) fn encode_string_bounded(
    s: &str,
    min: usize,
    max: usize,
    w: &mut impl Write,
) -> anyhow::Result<()> {
    assert!(min <= max, "Bad min and max");

    let char_count = s.chars().count();

    ensure!(
        (min..=max).contains(&char_count),
        "Char count of string is out of bounds while encoding (got {char_count}, expected \
         {min}..={max})"
    );

    encode_array_bounded(s.as_bytes(), 0, usize::MAX, w)
}

pub(crate) fn decode_string_bounded(
    min: usize,
    max: usize,
    r: &mut impl Read,
) -> anyhow::Result<String> {
    assert!(min <= max);

    let bytes = decode_array_bounded(min, max.saturating_mul(4), r)?;
    let string = String::from_utf8(bytes)?;

    let char_count = string.chars().count();
    ensure!(
        (min..=max).contains(&char_count),
        "Char count of string is out of bounds while decoding (got {char_count}, expected \
         {min}..={max}"
    );

    Ok(string)
}

pub(crate) fn decode_array_bounded<T: Decode>(
    min: usize,
    max: usize,
    r: &mut impl Read,
) -> anyhow::Result<Vec<T>> {
    assert!(min <= max);

    let len = VarInt::decode(r)?.0;
    ensure!(
        len >= 0 && (min..=max).contains(&(len as usize)),
        "Length of array is out of bounds while decoding (got {len}, needed {min}..={max})",
    );

    // Don't allocate more than what would roughly fit in a single packet in case we
    // get a malicious array length.
    let cap = (MAX_PACKET_SIZE as usize / mem::size_of::<T>().max(1)).min(len as usize);

    let mut res = Vec::with_capacity(cap);
    for _ in 0..len {
        res.push(T::decode(r)?);
    }

    Ok(res)
}
