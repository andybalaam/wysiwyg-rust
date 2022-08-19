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

#![cfg(test)]

use crate::tests::testutils_composer_model::{cm, tx};

use crate::Location;

#[test]
fn selecting_ascii_characters() {
    let mut model = cm("abcdefgh|");
    model.select(Location::from(0), Location::from(1));
    assert_eq!(tx(&model), "{a}|bcdefgh");

    model.select(Location::from(1), Location::from(3));
    assert_eq!(tx(&model), "a{bc}|defgh");

    model.select(Location::from(4), Location::from(8));
    assert_eq!(tx(&model), "abcd{efgh}|");
}

// TODO: Test selecting invalid ranges, including starting and ending off
// the end.

#[test]
fn selecting_single_utf16_code_unit_characters() {
    let mut model = cm("\u{03A9}\u{03A9}\u{03A9}|");

    model.select(Location::from(0), Location::from(1));
    assert_eq!(tx(&model), "{\u{03A9}}|\u{03A9}\u{03A9}");

    model.select(Location::from(0), Location::from(3));
    assert_eq!(tx(&model), "{\u{03A9}\u{03A9}\u{03A9}}|");

    model.select(Location::from(1), Location::from(2));
    assert_eq!(tx(&model), "\u{03A9}{\u{03A9}}|\u{03A9}");
}

#[test]
fn selecting_multiple_utf16_code_unit_characters() {
    let mut model = cm("\u{1F4A9}\u{1F4A9}\u{1F4A9}|");

    model.select(Location::from(0), Location::from(2));
    assert_eq!(tx(&model), "{\u{1F4A9}}|\u{1F4A9}\u{1F4A9}");

    model.select(Location::from(0), Location::from(6));
    assert_eq!(tx(&model), "{\u{1F4A9}\u{1F4A9}\u{1F4A9}}|");

    model.select(Location::from(2), Location::from(4));
    assert_eq!(tx(&model), "\u{1F4A9}{\u{1F4A9}}|\u{1F4A9}");
}

#[test]
fn selecting_complex_characters() {
    let mut model = cm("aaa\u{03A9}bbb\u{1F469}\u{1F3FF}\u{200D}\u{1F680}ccc|");

    model.select(Location::from(0), Location::from(3));
    assert_eq!(
        tx(&model),
        "{aaa}|\u{03A9}bbb\u{1F469}\u{1F3FF}\u{200D}\u{1F680}ccc"
    );

    model.select(Location::from(0), Location::from(4));
    assert_eq!(
        tx(&model),
        "{aaa\u{03A9}}|bbb\u{1F469}\u{1F3FF}\u{200D}\u{1F680}ccc"
    );

    model.select(Location::from(7), Location::from(14));
    assert_eq!(
        tx(&model),
        "aaa\u{03A9}bbb{\u{1F469}\u{1F3FF}\u{200D}\u{1F680}}|ccc"
    );

    model.select(Location::from(7), Location::from(15));
    assert_eq!(
        tx(&model),
        "aaa\u{03A9}bbb{\u{1F469}\u{1F3FF}\u{200D}\u{1F680}c}|cc"
    );
}
