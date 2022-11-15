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

use crate::composer_model::action_state::ActionState;
use crate::composer_model::menu_state::MenuStateComputeType;
use crate::composer_state::ComposerState;
use crate::dom::parser::parse;
use crate::dom::UnicodeString;
use crate::markdown_html_parser::MarkdownHTMLParser;
use crate::{
    ComposerAction, ComposerUpdate, Location, ToHtml, ToMarkdown, ToTree,
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct ComposerModel<S>
where
    S: UnicodeString,
{
    /// The current state of the model
    pub state: ComposerState<S>,

    /// Old states that may be restored by calling undo()
    pub(crate) previous_states: Vec<ComposerState<S>>,

    /// States after the current one that may be restored by calling redo()
    pub(crate) next_states: Vec<ComposerState<S>>,

    /// The states of the buttons for each action e.g. bold, undo
    pub(crate) action_states: HashMap<ComposerAction, ActionState>,
}

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn new() -> Self {
        let mut instance = Self {
            state: ComposerState::new(),
            previous_states: Vec::new(),
            next_states: Vec::new(),
            action_states: HashMap::new(), // TODO: Calculate state based on ComposerState
        };
        instance.compute_menu_state(MenuStateComputeType::AlwaysUpdate);
        instance
    }

    pub fn from_state(state: ComposerState<S>) -> Self {
        Self {
            state,
            previous_states: Vec::new(),
            next_states: Vec::new(),
            action_states: HashMap::new(), // TODO: Calculate state based on ComposerState
        }
    }

    /// Create a UTF-16 model from an HTML string, or panic if HTML parsing
    /// fails.
    pub fn from_html(
        html: &str,
        start_codeunit: usize,
        end_codeunit: usize,
    ) -> Self {
        let mut model = Self {
            state: ComposerState {
                dom: parse(html).expect("HTML parsing failed"),
                start: Location::from(start_codeunit),
                end: Location::from(end_codeunit),
                toggled_format_types: Vec::new(),
            },
            previous_states: Vec::new(),
            next_states: Vec::new(),
            action_states: HashMap::new(), // TODO: Calculate state based on ComposerState
        };
        model.compute_menu_state(MenuStateComputeType::AlwaysUpdate);
        model
    }

    /// Replace the entire content of the model with given HTML string.
    /// This will remove all previous and next states, effectively disabling
    /// undo and redo until further updates.
    pub fn set_content_from_html(&mut self, html: &S) -> ComposerUpdate<S> {
        let dom = parse(&html.to_string());

        match dom {
            Ok(dom) => {
                self.state.dom = dom;
                self.state.start = Location::from(self.state.dom.text_len());
                self.state.end = self.state.start;
                self.previous_states.clear();
                self.next_states.clear();
                self.create_update_replace_all_with_menu_state()
            }
            Err(e) => {
                // We should log here - internal task PSU-741
                self.state.dom = e.dom;
                self.previous_states.clear();
                self.next_states.clear();
                self.create_update_replace_all_with_menu_state()
            }
        }
    }

    pub fn set_content_from_markdown(
        &mut self,
        markdown: &S,
    ) -> ComposerUpdate<S> {
        let html = MarkdownHTMLParser::to_html(markdown);

        self.set_content_from_html(&html)
    }

    pub fn action_states(&self) -> &HashMap<ComposerAction, ActionState> {
        &self.action_states
    }

    #[cfg(test)]
    pub(crate) fn action_is_enabled(&self, action: ComposerAction) -> bool {
        self.action_states.get(&action) == Some(&ActionState::Enabled)
    }

    pub(crate) fn action_is_reversed(&self, action: ComposerAction) -> bool {
        self.action_states.get(&action) == Some(&ActionState::Reversed)
    }

    #[cfg(test)]
    pub(crate) fn action_is_disabled(&self, action: ComposerAction) -> bool {
        self.action_states.get(&action) == Some(&ActionState::Disabled)
    }

    pub(crate) fn create_update_replace_all(&mut self) -> ComposerUpdate<S> {
        ComposerUpdate::replace_all(
            self.state.dom.to_html(),
            self.state.start,
            self.state.end,
            self.compute_menu_state(MenuStateComputeType::KeepIfUnchanged),
        )
    }

    pub(crate) fn create_update_replace_all_with_menu_state(
        &mut self,
    ) -> ComposerUpdate<S> {
        ComposerUpdate::replace_all(
            self.state.dom.to_html(),
            self.state.start,
            self.state.end,
            self.compute_menu_state(MenuStateComputeType::AlwaysUpdate),
        )
    }

    pub fn get_selection(&self) -> (Location, Location) {
        (self.state.start, self.state.end)
    }

    pub fn get_content_as_html(&self) -> S {
        self.state.dom.to_html()
    }

    pub fn get_content_as_markdown(&self) -> S {
        self.state.dom.to_markdown().unwrap()
    }

    pub fn get_current_state(&self) -> &ComposerState<S> {
        &self.state
    }

    pub fn to_tree(&self) -> S {
        self.state.dom.to_tree()
    }

    pub fn clear(&mut self) -> ComposerUpdate<S> {
        self.set_content_from_html(&"".into())
    }
}

#[cfg(test)]
mod test {
    use widestring::Utf16String;

    use crate::tests::testutils_composer_model::cm;

    use super::*;

    // Most tests for ComposerModel are inside the tests/ modules

    #[test]
    fn completely_replacing_html_works() {
        let mut model = cm("{hello}| world");
        model.set_content_from_html(&Utf16String::from_str("foo <b>bar</b>"));
        assert_eq!(model.state.dom.to_string(), "foo <b>bar</b>");
    }

    #[test]
    fn action_states_are_reported() {
        let mut model = ComposerModel::new();
        model.replace_text(Utf16String::from("a"));
        model.select(Location::from(0), Location::from(1));
        model.bold();

        assert!(model.action_is_reversed(ComposerAction::Bold));
        assert!(model.action_is_enabled(ComposerAction::StrikeThrough));
        assert!(model.action_is_disabled(ComposerAction::Redo));
    }
}
