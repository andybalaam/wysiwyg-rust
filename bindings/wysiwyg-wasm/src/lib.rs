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

use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use wasm_bindgen::prelude::*;
use widestring::Utf16String;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn new_composer_model() -> ComposerModel {
    ComposerModel {
        inner: wysiwyg::ComposerModel::new(),
    }
}

#[wasm_bindgen]
pub fn new_composer_model_from_html(
    html: &str,
    start_utf16_codeunit: u32,
    end_utf16_codeunit: u32,
) -> ComposerModel {
    ComposerModel {
        inner: wysiwyg::ComposerModel::<Utf16String>::from_html(
            html,
            usize::try_from(start_utf16_codeunit).unwrap(),
            usize::try_from(end_utf16_codeunit).unwrap(),
        ),
    }
}

#[wasm_bindgen]
pub enum ActionState {
    /// The button can be clicked, and will perform its normal action
    /// e.g. make something bold
    Enabled,

    /// The button can be clicked, and will perform the reverse of its
    /// normal action e.g. stop something being bold
    Reversed,

    /// The button cannot be clicked
    Disabled,
}

trait IntoFfi {
    fn into_ffi(self) -> js_sys::Map;
}

impl IntoFfi for &HashMap<wysiwyg::ComposerAction, wysiwyg::ActionState> {
    fn into_ffi(self) -> js_sys::Map {
        let ret = js_sys::Map::new();
        for (k, v) in self.iter() {
            ret.set(&k.as_ref().into(), &v.as_ref().into());
        }
        ret
    }
}

trait ToUtf16TupleVec {
    fn into_vec(self) -> Vec<(Utf16String, Utf16String)>;
}

impl ToUtf16TupleVec for js_sys::Map {
    fn into_vec(self) -> Vec<(Utf16String, Utf16String)> {
        let mut vec = vec![];
        self.for_each(&mut |value, key| {
            vec.push((
                Utf16String::from_str(&key.as_string().unwrap()),
                Utf16String::from_str(&value.as_string().unwrap()),
            ));
        });
        vec
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct ComposerModel {
    inner: wysiwyg::ComposerModel<Utf16String>,
}

#[wasm_bindgen]
impl ComposerModel {
    pub fn new() -> Self {
        Self {
            inner: wysiwyg::ComposerModel::new(),
        }
    }

    pub fn from_example_format(text: &str) -> Self {
        Self {
            inner: wysiwyg::ComposerModel::from_example_format(text),
        }
    }

    pub fn to_example_format(&self) -> String {
        self.inner.to_example_format()
    }

    pub fn get_content_as_html(&self) -> String {
        self.inner.get_content_as_html().to_string()
    }

    pub fn get_content_as_markdown(&self) -> String {
        self.inner.get_content_as_markdown().to_string()
    }

    pub fn get_content_as_plain_text(&self) -> String {
        self.inner.get_content_as_plain_text().to_string()
    }

    pub fn document(&self) -> DomHandle {
        DomHandle {
            inner: self.inner.state.dom.document().handle(),
        }
    }

    pub fn action_states(&self) -> js_sys::Map {
        self.inner.action_states().into_ffi()
    }

    pub fn select(
        &mut self,
        start_utf16_codeunit: u32,
        end_utf16_codeunit: u32,
    ) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.select(
            wysiwyg::Location::from(
                usize::try_from(start_utf16_codeunit).unwrap(),
            ),
            wysiwyg::Location::from(
                usize::try_from(end_utf16_codeunit).unwrap(),
            ),
        ))
    }

    pub fn selection_start(&self) -> u32 {
        let ret: usize = self.inner.state.start.into();
        ret as u32
    }

    pub fn selection_end(&self) -> u32 {
        let ret: usize = self.inner.state.end.into();
        ret as u32
    }

    pub fn replace_text(&mut self, new_text: &str) -> ComposerUpdate {
        // Conversion here to UTF-16, which has presumably just been
        // converted to UTF-8 in the FFI bindings layer.
        // If the performance is a problem, we could fix this.
        // Internal task to track this: PSU-739
        ComposerUpdate::from(
            self.inner.replace_text(Utf16String::from_str(new_text)),
        )
    }

    pub fn replace_text_suggestion(
        &mut self,
        new_text: &str,
        suggestion: &SuggestionPattern,
    ) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.replace_text_suggestion(
            Utf16String::from_str(new_text),
            wysiwyg::SuggestionPattern::from(suggestion.clone()),
        ))
    }

    pub fn set_content_from_html(
        &mut self,
        text: &str,
    ) -> Result<ComposerUpdate, DomCreationError> {
        let update = self
            .inner
            .set_content_from_html(&Utf16String::from_str(text))?;
        Ok(ComposerUpdate::from(update))
    }

    pub fn set_content_from_markdown(
        &mut self,
        text: &str,
    ) -> Result<ComposerUpdate, DomCreationError> {
        let markdown = self
            .inner
            .set_content_from_markdown(&Utf16String::from_str(text))?;
        Ok(ComposerUpdate::from(markdown))
    }

    pub fn clear(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.clear())
    }

    pub fn enter(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.enter())
    }

    pub fn backspace(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.backspace())
    }

    pub fn backspace_word(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.backspace_word())
    }

    pub fn delete(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.delete())
    }

    pub fn delete_word(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.delete_word())
    }

    pub fn bold(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.bold())
    }

    pub fn italic(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.italic())
    }

    pub fn strike_through(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.strike_through())
    }

    pub fn underline(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.underline())
    }

    pub fn quote(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.quote())
    }

    pub fn inline_code(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.inline_code())
    }

    pub fn code_block(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.code_block())
    }

    pub fn undo(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.undo())
    }

    pub fn redo(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.redo())
    }

    pub fn ordered_list(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.ordered_list())
    }

    pub fn unordered_list(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.unordered_list())
    }

    pub fn indent(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.indent())
    }

    pub fn unindent(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.unindent())
    }

    pub fn get_link_action(&self) -> LinkAction {
        self.inner.get_link_action().into()
    }

    pub fn set_link(&mut self, url: &str) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.set_link(Utf16String::from_str(url)))
    }

    pub fn set_link_with_text(
        &mut self,
        url: &str,
        text: &str,
    ) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.set_link_with_text(
            Utf16String::from_str(url),
            Utf16String::from_str(text),
        ))
    }

    /// This function creates a link with the first argument being the href, the second being the
    /// display text, the third being the (rust model) suggestion that is being replaced and the
    /// final argument being a map of html attributes that will be added to the mention.

    // TODO should this be renamed? We're now creating a mention container, but that is still a link node
    pub fn set_link_suggestion(
        &mut self,
        url: &str,
        text: &str,
        suggestion: &SuggestionPattern,
        attributes: js_sys::Map,
    ) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.set_mention_from_suggestion(
            Utf16String::from_str(url),
            Utf16String::from_str(text),
            wysiwyg::SuggestionPattern::from(suggestion.clone()),
            attributes.into_vec(),
        ))
    }

    pub fn remove_links(&mut self) -> ComposerUpdate {
        ComposerUpdate::from(self.inner.remove_links())
    }
}

#[wasm_bindgen]
pub struct ComposerUpdate {
    inner: wysiwyg::ComposerUpdate<Utf16String>,
}

impl ComposerUpdate {
    fn from(inner: wysiwyg::ComposerUpdate<Utf16String>) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
impl ComposerUpdate {
    pub fn text_update(&self) -> TextUpdate {
        TextUpdate::from(self.inner.text_update.clone())
    }

    pub fn menu_state(&self) -> MenuState {
        MenuState::from(self.inner.menu_state.clone())
    }

    pub fn menu_action(&self) -> MenuAction {
        MenuAction::from(self.inner.menu_action.clone())
    }
}

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub enum DomCreationError {
    HtmlParseError,
    MarkdownParseError,
}

impl Display for DomCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DomCreationError::HtmlParseError => {
                "could not create dom from html"
            }
            DomCreationError::MarkdownParseError => {
                "could not create dom from markdown"
            }
        })
    }
}

impl From<wysiwyg::DomCreationError> for DomCreationError {
    fn from(error: wysiwyg::DomCreationError) -> Self {
        match error {
            wysiwyg::DomCreationError::HtmlParseError(_) => {
                Self::HtmlParseError
            }
            wysiwyg::DomCreationError::MarkdownParseError(_) => {
                Self::MarkdownParseError
            }
        }
    }
}

impl From<DomCreationError> for wysiwyg::DomCreationError {
    fn from(_: DomCreationError) -> Self {
        unimplemented!("Error is not needed as input")
    }
}

impl From<DomCreationError> for JsValue {
    fn from(error: DomCreationError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct TextUpdate {
    pub keep: Option<Keep>,
    pub replace_all: Option<ReplaceAll>,
    pub select: Option<Selection>,
}

impl TextUpdate {
    pub fn from(inner: wysiwyg::TextUpdate<Utf16String>) -> Self {
        match inner {
            wysiwyg::TextUpdate::Keep => Self {
                keep: Some(Keep),
                replace_all: None,
                select: None,
            },
            wysiwyg::TextUpdate::ReplaceAll(r) => {
                let start_utf16_codeunit: usize = r.start.into();
                let end_utf16_codeunit: usize = r.end.into();
                Self {
                    keep: None,
                    replace_all: Some(ReplaceAll {
                        replacement_html: r.replacement_html.to_string(),
                        start_utf16_codeunit: u32::try_from(
                            start_utf16_codeunit,
                        )
                        .unwrap(),
                        end_utf16_codeunit: u32::try_from(end_utf16_codeunit)
                            .unwrap(),
                    }),
                    select: None,
                }
            }
            wysiwyg::TextUpdate::Select(s) => {
                let start_utf16_codeunit: usize = s.start.into();
                let end_utf16_codeunit: usize = s.end.into();
                Self {
                    keep: None,
                    replace_all: None,
                    select: Some(Selection {
                        start_utf16_codeunit: u32::try_from(
                            start_utf16_codeunit,
                        )
                        .unwrap(),
                        end_utf16_codeunit: u32::try_from(end_utf16_codeunit)
                            .unwrap(),
                    }),
                }
            }
        }
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct Keep;

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct ReplaceAll {
    pub replacement_html: String,
    pub start_utf16_codeunit: u32,
    pub end_utf16_codeunit: u32,
}

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct Selection {
    pub start_utf16_codeunit: u32,
    pub end_utf16_codeunit: u32,
}

#[wasm_bindgen]
pub struct MenuState {
    inner: wysiwyg::MenuState,
}

impl MenuState {
    pub fn from(inner: wysiwyg::MenuState) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
impl MenuState {
    pub fn keep(&self) -> bool {
        matches!(self.inner, wysiwyg::MenuState::Keep)
    }

    pub fn update(&self) -> Option<MenuStateUpdate> {
        match &self.inner {
            wysiwyg::MenuState::Update(update) => {
                Some(MenuStateUpdate::from(update))
            }
            _ => None,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug)]
pub struct MenuStateUpdate {
    pub action_states: js_sys::Map,
}

impl MenuStateUpdate {
    pub fn from(inner: &wysiwyg::MenuStateUpdate) -> Self {
        Self {
            action_states: inner.action_states.into_ffi(),
        }
    }
}

#[wasm_bindgen]
pub struct MenuAction {
    inner: wysiwyg::MenuAction,
}

impl MenuAction {
    pub fn from(inner: wysiwyg::MenuAction) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
impl MenuAction {
    pub fn keep(&self) -> bool {
        matches!(self.inner, wysiwyg::MenuAction::Keep)
    }

    pub fn none(&self) -> bool {
        matches!(self.inner, wysiwyg::MenuAction::None)
    }

    pub fn suggestion(&self) -> Option<MenuActionSuggestion> {
        match &self.inner {
            wysiwyg::MenuAction::Suggestion(suggestion) => {
                Some(MenuActionSuggestion {
                    suggestion_pattern: SuggestionPattern::from(
                        suggestion.clone(),
                    ),
                })
            }
            _ => None,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct MenuActionSuggestion {
    pub suggestion_pattern: SuggestionPattern,
}

#[wasm_bindgen]
#[derive(Clone)]
pub enum ComposerAction {
    Bold,
    Italic,
    StrikeThrough,
    Underline,
    InlineCode,
    Link,
    Undo,
    Redo,
    OrderedList,
    UnorderedList,
    Indent,
    Unindent,
    CodeBlock,
    Quote,
}

impl ComposerAction {
    pub fn from(inner: &wysiwyg::ComposerAction) -> Self {
        match inner {
            wysiwyg::ComposerAction::Bold => Self::Bold,
            wysiwyg::ComposerAction::Italic => Self::Italic,
            wysiwyg::ComposerAction::StrikeThrough => Self::StrikeThrough,
            wysiwyg::ComposerAction::Underline => Self::Underline,
            wysiwyg::ComposerAction::InlineCode => Self::InlineCode,
            wysiwyg::ComposerAction::Link => Self::Link,
            wysiwyg::ComposerAction::Undo => Self::Undo,
            wysiwyg::ComposerAction::Redo => Self::Redo,
            wysiwyg::ComposerAction::OrderedList => Self::OrderedList,
            wysiwyg::ComposerAction::UnorderedList => Self::UnorderedList,
            wysiwyg::ComposerAction::Indent => Self::Indent,
            wysiwyg::ComposerAction::Unindent => Self::Unindent,
            wysiwyg::ComposerAction::CodeBlock => Self::CodeBlock,
            wysiwyg::ComposerAction::Quote => Self::Quote,
        }
    }
}

impl From<&ComposerAction> for wysiwyg::ComposerAction {
    fn from(action: &ComposerAction) -> Self {
        match action {
            ComposerAction::Bold => Self::Bold,
            ComposerAction::Italic => Self::Italic,
            ComposerAction::StrikeThrough => Self::StrikeThrough,
            ComposerAction::Underline => Self::Underline,
            ComposerAction::InlineCode => Self::InlineCode,
            ComposerAction::Link => Self::Link,
            ComposerAction::Undo => Self::Undo,
            ComposerAction::Redo => Self::Redo,
            ComposerAction::OrderedList => Self::OrderedList,
            ComposerAction::UnorderedList => Self::UnorderedList,
            ComposerAction::Indent => Self::Indent,
            ComposerAction::Unindent => Self::Unindent,
            ComposerAction::CodeBlock => Self::CodeBlock,
            ComposerAction::Quote => Self::Quote,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct SuggestionPattern {
    pub key: PatternKey,
    pub text: String,
    pub start: u32,
    pub end: u32,
}

impl From<wysiwyg::SuggestionPattern> for SuggestionPattern {
    fn from(inner: wysiwyg::SuggestionPattern) -> Self {
        Self {
            key: PatternKey::from(inner.key),
            text: inner.text,
            start: u32::try_from(inner.start).unwrap(),
            end: u32::try_from(inner.end).unwrap(),
        }
    }
}

impl From<SuggestionPattern> for wysiwyg::SuggestionPattern {
    fn from(pattern: SuggestionPattern) -> Self {
        Self {
            key: wysiwyg::PatternKey::from(pattern.key),
            text: pattern.text,
            start: usize::try_from(pattern.start).unwrap(),
            end: usize::try_from(pattern.end).unwrap(),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub enum PatternKey {
    At,
    Hash,
    Slash,
}

impl From<wysiwyg::PatternKey> for PatternKey {
    fn from(inner: wysiwyg::PatternKey) -> Self {
        match inner {
            wysiwyg::PatternKey::At => Self::At,
            wysiwyg::PatternKey::Hash => Self::Hash,
            wysiwyg::PatternKey::Slash => Self::Slash,
        }
    }
}

impl From<PatternKey> for wysiwyg::PatternKey {
    fn from(key: PatternKey) -> Self {
        match key {
            PatternKey::At => Self::At,
            PatternKey::Hash => Self::Hash,
            PatternKey::Slash => Self::Slash,
        }
    }
}

/// An iterator-like view of a DomHandle's children, written to work around
/// the lack of support for returning Vec<T> in wasm_bindgen.
#[wasm_bindgen]
pub struct DomChildren {
    inner: VecDeque<DomHandle>,
}

#[wasm_bindgen]
impl DomChildren {
    fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }

    pub fn next_child(&mut self) -> Option<DomHandle> {
        self.inner.pop_front()
    }
}

impl FromIterator<DomHandle> for DomChildren {
    fn from_iter<T: IntoIterator<Item = DomHandle>>(iter: T) -> Self {
        Self {
            inner: VecDeque::from_iter(iter),
        }
    }
}

#[wasm_bindgen]
/// Refers to a node in the composer model.
pub struct DomHandle {
    inner: wysiwyg::DomHandle,
}

#[wasm_bindgen]
impl DomHandle {
    /// Returns "container", "line_break", "text" or "zwsp" depending on the type of
    /// node we refer to.
    /// Panics if we are not a valid reference (because the model has changed
    /// since we were created, or because you passed in a different model
    /// from the one that created us.)
    pub fn node_type(&self, model: &ComposerModel) -> String {
        let node = model.inner.state.dom.lookup_node(&self.inner);
        String::from(match node {
            wysiwyg::DomNode::Container(_) => "container",
            wysiwyg::DomNode::LineBreak(_) => "line_break",
            wysiwyg::DomNode::Text(_) => "text",
        })
    }

    /// Returns a list of our children nodes, or an empty list if we refer
    /// to a text or line break node.
    /// Panics if we are not a valid reference (because the model has changed
    /// since we were created, or because you passed in a different model
    /// from the one that created us.)
    pub fn children(&self, model: &ComposerModel) -> DomChildren {
        let node = model.inner.state.dom.lookup_node(&self.inner);
        match node {
            wysiwyg::DomNode::Container(node) => node
                .children()
                .iter()
                .map(|child| DomHandle {
                    inner: child.handle(),
                })
                .collect(),
            _ => DomChildren::new(),
        }
    }

    /// Returns the text of this node, or an empty string if this is a
    /// container or line break.
    /// Panics if we are not a valid reference (because the model has changed
    /// since we were created, or because you passed in a different model
    /// from the one that created us.)
    pub fn text(&self, model: &ComposerModel) -> String {
        let node = model.inner.state.dom.lookup_node(&self.inner);
        match node {
            wysiwyg::DomNode::Container(_) => String::from(""),
            wysiwyg::DomNode::LineBreak(_) => String::from(""),
            wysiwyg::DomNode::Text(node) => node.data().to_string(),
        }
    }

    /// Returns our tagname, or "-text-"/"-zwsp-" if we are a text/zwsp node.
    /// Panics if we are not a valid reference (because the model has changed
    /// since we were created, or because you passed in a different model
    /// from the one that created us.)
    pub fn tag(&self, model: &ComposerModel) -> String {
        let node = model.inner.state.dom.lookup_node(&self.inner);
        match node {
            wysiwyg::DomNode::Container(node) => node.name().to_string(),
            wysiwyg::DomNode::LineBreak(node) => node.name().to_string(),
            wysiwyg::DomNode::Text(_) => String::from("-text-"),
        }
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct CreateWithText;

#[derive(Clone)]
#[wasm_bindgen]
pub struct Create;

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct Edit {
    pub url: String,
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct Disabled;

#[wasm_bindgen(getter_with_clone)]
pub struct LinkAction {
    pub create_with_text: Option<CreateWithText>,
    pub create: Option<Create>,
    pub edit_link: Option<Edit>,
    pub disabled: Option<Disabled>,
}

impl From<wysiwyg::LinkAction<Utf16String>> for LinkAction {
    fn from(inner: wysiwyg::LinkAction<Utf16String>) -> Self {
        match inner {
            wysiwyg::LinkAction::CreateWithText => Self {
                create_with_text: Some(CreateWithText),
                create: None,
                edit_link: None,
                disabled: None,
            },
            wysiwyg::LinkAction::Create => Self {
                create_with_text: None,
                create: Some(Create),
                edit_link: None,
                disabled: None,
            },
            wysiwyg::LinkAction::Edit(url) => {
                let url = url.to_string();
                Self {
                    create_with_text: None,
                    create: None,
                    edit_link: Some(Edit { url }),
                    disabled: None,
                }
            }
            wysiwyg::LinkAction::Disabled => Self {
                create_with_text: None,
                create: None,
                edit_link: None,
                disabled: Some(Disabled),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::ComposerModel;

    #[test]
    fn can_find_types_of_nodes() {
        let mut model = ComposerModel::new();
        model.replace_text("foo");

        assert_eq!(model.document().node_type(&model), "container");
        assert_eq!(
            model.document().children(&model).inner[0].node_type(&model),
            "text"
        );
    }

    #[test]
    fn can_enumerate_children_of_nodes() {
        let mut model = ComposerModel::new();
        model.replace_text("01234");
        model.select(1, 4);
        model.bold();
        model.select(2, 3);
        model.italic();

        let children = model.document().children(&model).inner;
        let grandchildren = children[1].children(&model).inner;
        let great_grandchildren = grandchildren[1].children(&model).inner;

        assert_eq!(children[0].node_type(&model), "text");
        assert_eq!(children[0].text(&model), "0");
        assert_eq!(children[1].node_type(&model), "container");
        assert_eq!(children[1].tag(&model), "strong");
        assert_eq!(grandchildren[0].node_type(&model), "text");
        assert_eq!(grandchildren[0].text(&model), "1");
        assert_eq!(grandchildren[1].node_type(&model), "container");
        assert_eq!(grandchildren[1].tag(&model), "em");
        assert_eq!(great_grandchildren[0].node_type(&model), "text");
        assert_eq!(great_grandchildren[0].text(&model), "2");
        assert_eq!(grandchildren[2].node_type(&model), "text");
        assert_eq!(grandchildren[2].text(&model), "3");
        assert_eq!(children[2].node_type(&model), "text");
        assert_eq!(children[2].text(&model), "4");
    }
}
