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

use crate::{ComposerAction, UnicodeString};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InlineFormatType {
    Bold,
    Italic,
    StrikeThrough,
    Underline,
    InlineCode,
}

impl InlineFormatType {
    pub fn tag(&self) -> &'static str {
        match self {
            InlineFormatType::Bold => "strong",
            InlineFormatType::Italic => "em",
            InlineFormatType::StrikeThrough => "del",
            InlineFormatType::Underline => "u",
            InlineFormatType::InlineCode => "code",
        }
    }

    pub fn action(&self) -> ComposerAction {
        match self {
            InlineFormatType::Bold => ComposerAction::Bold,
            InlineFormatType::Italic => ComposerAction::Italic,
            InlineFormatType::StrikeThrough => ComposerAction::StrikeThrough,
            InlineFormatType::Underline => ComposerAction::Underline,
            InlineFormatType::InlineCode => ComposerAction::InlineCode,
        }
    }
}

impl<S: UnicodeString> From<S> for InlineFormatType {
    fn from(value: S) -> Self {
        match value.to_string().as_str() {
            "b" | "strong" => InlineFormatType::Bold,
            "i" | "em" => InlineFormatType::Italic,
            "del" => InlineFormatType::StrikeThrough,
            "u" => InlineFormatType::Underline,
            "code" => InlineFormatType::InlineCode,
            _ => {
                panic!("Unknown format type {}", value.to_string().as_str());
            }
        }
    }
}
