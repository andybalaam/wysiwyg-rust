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

use crate::tests::testutils_composer_model::cm;
use crate::tests::testutils_conversion::utf16;

use crate::TextUpdate;

#[test]
fn cant_set_link_to_empty_selection() {
    let mut model = cm("hello |world");
    let update = model.set_link(utf16("https://element.io"));
    assert!(matches!(update.text_update, TextUpdate::Keep));
}

#[test]
fn set_link_wraps_selection_in_link_tag() {
    let mut model = cm("{hello}| world");
    model.set_link(utf16("https://element.io"));
    assert_eq!(
        model.state.dom.to_string(),
        "<a href=\"https://element.io\">hello</a> world"
    );
}

#[test]
fn set_link_in_multiple_leaves_of_formatted_text() {
    let mut model = cm("{<i>test_italic<b>test_italic_bold</b></i>}|");
    model.set_link(utf16("https://element.io"));
    assert_eq!(
        model.state.dom.to_string(),
        "<i><a href=\"https://element.io\">test_italic</a><b><a href=\"https://element.io\">test_italic_bold</a></b></i>"
    )
}

#[test]
fn set_link_in_multiple_leaves_of_formatted_text_partially_covered() {
    let mut model = cm("<i>test_it{alic<b>test_ital}|ic_bold</b></i>");
    model.set_link(utf16("https://element.io"));
    assert_eq!(
        model.state.dom.to_string(),
        "<i>test_it<a href=\"https://element.io\">alic</a><b><a href=\"https://element.io\">test_ital</a>ic_bold</b></i>"
    )
}

#[test]
fn set_link_in_multiple_leaves_of_formatted_text_partially_covered_2() {
    let mut model = cm("<i><u>test_it{alic_underline</u>test_italic<b>test_ital}|ic_bold</b></i>");
    model.set_link(utf16("https://element.io"));
    assert_eq!(
        model.state.dom.to_string(),
        "<i><u>test_it<a href=\"https://element.io\">alic_underline</a></u><a href=\"https://element.io\">test_italic</a><b><a href=\"https://element.io\">test_ital</a>ic_bold</b></i>"
    )
}

#[test]
fn set_link_in_already_linked_text() {
    let mut model = cm("{<a href=\"https://element.io\">link_text</a>}|");
    model.set_link(utf16("https://matrix.org"));
    assert_eq!(
        model.state.dom.to_string(),
        "<a href=\"https://matrix.org\">link_text</a>"
    )
}

#[test]
fn set_link_in_already_linked_text_with_partial_selection() {
    let mut model = cm("<a href=\"https://element.io\">link_{text}|</a>");
    model.set_link(utf16("https://matrix.org"));
    assert_eq!(
        model.state.dom.to_string(),
        "<a href=\"https://matrix.org\">link_text</a>"
    )
}

#[test]
fn set_link_in_text_and_already_linked_text() {
    let mut model =
        cm("{non_link_text<a href=\"https://element.io\">link_text</a>}|");
    model.set_link(utf16("https://matrix.org"));
    assert_eq!(
        model.state.dom.to_string(),
        "<a href=\"https://matrix.org\">non_link_text</a><a href=\"https://matrix.org\">link_text</a>"
    )
}

#[test]
fn set_link_in_multiple_leaves_of_formatted_text_with_link() {
    let mut model = cm("{<i><a href=\"https://element.io\">test_italic</a><b><a href=\"https://element.io\">test_italic_bold</a></b></i>}|");
    model.set_link(utf16("https://matrix.org"));
    assert_eq!(
        model.state.dom.to_string(),
        "<i><a href=\"https://matrix.org\">test_italic</a><b><a href=\"https://matrix.org\">test_italic_bold</a></b></i>"
    )
}
