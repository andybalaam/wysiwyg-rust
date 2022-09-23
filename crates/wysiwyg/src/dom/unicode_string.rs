// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::iter;
use std::ops::Deref;

use widestring::{Utf16String, Utf32String};

/// The type of string being used inside a [Dom] instance. Must
/// contain valid Unicode, and allow slicing by code unit positions.
/// We implement this for String, Utf16String and Utf32String (from the
/// widestring crate).
pub trait UnicodeString:
    Clone
    + std::fmt::Debug
    + std::fmt::Display
    + Default
    + PartialEq
    + AsRef<[Self::CodeUnit]>
    + for<'a> From<&'a str>
    + Deref
    + for<'a> Extend<&'a <Self as Deref>::Target>
    + Extend<Self>
    + Extend<char>
    + for<'a> Extend<&'a str>
{
    type CodeUnit: Copy + From<u8> + PartialEq;

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String>;

    /// Convert this character to a code unit.
    /// Panics if this character requires more than one code unit
    fn c_from_char(ch: char) -> Self::CodeUnit;
}

impl UnicodeString for String {
    type CodeUnit = u8;

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String> {
        String::from_utf8(v.into()).map_err(|e| e.to_string())
    }

    fn c_from_char(ch: char) -> Self::CodeUnit {
        assert!(ch.len_utf8() == 1);
        let mut buf = [0; 1];
        ch.encode_utf8(&mut buf);
        buf[0]
    }
}

impl UnicodeString for Utf16String {
    type CodeUnit = u16;

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String> {
        Utf16String::from_vec(v.into()).map_err(|e| e.to_string())
    }

    fn c_from_char(ch: char) -> Self::CodeUnit {
        let mut ret = Utf16String::new();
        ret.push(ch);
        assert!(ret.len() == 1);
        ret.into_vec()[0]
    }
}

impl UnicodeString for Utf32String {
    type CodeUnit = u32;

    fn from_vec(v: impl Into<Vec<Self::CodeUnit>>) -> Result<Self, String> {
        Utf32String::from_vec(v.into()).map_err(|e| e.to_string())
    }

    fn c_from_char(ch: char) -> Self::CodeUnit {
        let mut ret = Utf32String::new();
        ret.push(ch);
        assert!(ret.len() == 1);
        ret.into_vec()[0]
    }
}

pub trait UnicodeStringExt: UnicodeString {
    fn push<T>(&mut self, s: T)
    where
        Self: Extend<T>;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

impl<S: UnicodeString> UnicodeStringExt for S {
    fn push<T>(&mut self, s: T)
    where
        Self: Extend<T>,
    {
        self.extend(iter::once(s))
    }

    fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }
}
