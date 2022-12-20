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

use crate::dom::nodes::dom_node::DomNodeKind::{Generic, Quote};
use crate::dom::DomLocation;
use crate::{
    ComposerAction, ComposerModel, ComposerUpdate, DomHandle, DomNode, ToHtml,
    UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn quote(&mut self) -> ComposerUpdate<S> {
        if self.action_is_reversed(ComposerAction::Quote) {
            self.remove_quote()
        } else {
            self.add_quote()
        }
    }

    fn add_quote(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let Some(wrap_result) = self.find_nodes_to_wrap_in_block(s, e) else {
            // No suitable nodes found to be wrapped inside the quote. The Dom should be empty
            self.state.dom.append_at_end_of_document(DomNode::new_quote(vec![DomNode::new_text(S::zwsp())]));
            self.state.start += 1;
            self.state.end += 1;
            return self.create_update_replace_all();
        };

        let parent_handle = wrap_result.ancestor_handle;

        let start_handle = wrap_result.start_handle;
        let end_handle = wrap_result.end_handle;

        let mut subtree = self.state.dom.split_sub_tree_between(
            &start_handle,
            0,
            &end_handle,
            usize::MAX,
            parent_handle.depth(),
        );
        subtree.set_handle(DomHandle::root());
        let dom_html = self.state.dom.to_html().to_string();
        let subtree_html = subtree.to_html().to_string();

        let start_handle_is_start_at_depth =
            start_handle.raw().iter().all(|i| *i == 0);
        let mut insert_at_handle =
            if subtree.is_block_node() && subtree.kind() != Generic {
                start_handle.sub_handle_up_to(parent_handle.depth())
            } else {
                start_handle.sub_handle_up_to(parent_handle.depth() + 1)
            };
        if !start_handle_is_start_at_depth
            && self.state.dom.contains(&insert_at_handle)
        {
            insert_at_handle = insert_at_handle.next_sibling();
        } else if self.state.dom.document().children().is_empty() {
            insert_at_handle = self.state.dom.document_handle().child_handle(0);
        }

        if self.state.dom.document().children().is_empty() {
            insert_at_handle = self.state.dom.document_handle().child_handle(0);
        }
        let quote_node = if subtree.is_block_node() && subtree.kind() != Generic
        {
            self.state.start += 1;
            self.state.end += 1;
            DomNode::new_quote(vec![DomNode::new_zwsp(), subtree])
        } else if let Some(subtree_container) = subtree.as_container_mut() {
            if !subtree_container
                .children()
                .first()
                .map_or(false, |n| n.is_zwsp())
            {
                subtree_container.insert_child(0, DomNode::new_zwsp());
                self.state.start += 1;
                self.state.end += 1;
            }
            DomNode::new_quote(subtree_container.children().clone())
        } else {
            panic!("Subtree node must be a container");
        };
        self.state.dom.insert_at(&insert_at_handle, quote_node);

        let parent = self.state.dom.parent_mut(&insert_at_handle);
        // If quote was added to a list item and we removed the leading ZWSP, add it again
        if parent.is_list_item()
            && !parent.children().first().map_or(false, |n| n.is_zwsp())
        {
            parent.insert_child(0, DomNode::new_zwsp());
            self.state.start += 1;
            self.state.end += 1;
        }

        self.state.dom.join_nodes_in_container(&parent_handle);

        self.state.dom.remove_empty_container_nodes(false);

        self.create_update_replace_all()
    }

    fn remove_quote(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let Some(quote_location) = range.locations.iter().find(|l| l.kind == Quote) else {
            return ComposerUpdate::keep();
        };

        if let DomNode::Container(quote_container) =
            self.state.dom.lookup_node_mut(&quote_location.node_handle)
        {
            if quote_container.remove_leading_zwsp()
                && quote_location.index_in_dom() <= s
            {
                self.state.start -= 1;
                self.state.end -= 1;
            }
        } else {
            panic!("Quote node must be a container node");
        }

        self.state
            .dom
            .remove_and_keep_children(&quote_location.node_handle);

        self.create_update_replace_all()
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};

    #[test]
    fn apply_quote_to_empty_dom() {
        let mut model = cm("|");
        model.quote();
        assert_eq!(tx(&model), "<blockquote>~|</blockquote>")
    }

    #[test]
    fn apply_quote_to_simple_text() {
        let mut model = cm("Some text|");
        model.quote();
        assert_eq!(tx(&model), "<blockquote>~Some text|</blockquote>")
    }

    #[test]
    fn apply_quote_to_formatted_text() {
        let mut model = cm("<i>Some text|</i>");
        model.quote();
        assert_eq!(tx(&model), "<blockquote>~<i>Some text|</i></blockquote>")
    }

    #[test]
    fn apply_quote_to_nested_formatted_text() {
        let mut model = cm("<i><b>Some text|</b></i>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote>~<i><b>Some text|</b></i></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_text_and_formatted_text() {
        let mut model = cm("Plain text and <i><b>Some formatted text|</b></i>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote>~Plain text and <i><b>Some formatted text|</b></i></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_single_list_item() {
        let mut model = cm("<ul><li>~List item|</li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<ul><li>~<blockquote>~List item|</blockquote></li></ul>"
        )
    }

    #[test]
    fn apply_quote_to_list() {
        let mut model =
            cm("<ul><li>~First {item</li><li>~Second}| item</li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote>~<ul><li>~First {item</li><li>~Second}| item</li></ul></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_list_and_formatted_text() {
        let mut model =
            cm("Plain text <b>bold {text</b><ul><li>~First item</li><li>~Second}| item</li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote>~Plain text <b>bold {text</b><ul><li>~First item</li><li>~Second}| item</li></ul></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_simple_text_with_line_breaks() {
        let mut model = cm("Some |text<br />Next line");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote>~Some |text</blockquote><br />Next line"
        )
    }

    #[test]
    fn apply_quote_to_several_nodes_with_line_breaks() {
        let mut model = cm("Some {text<br /><b>Next}| line</b>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote>~Some {text<br /><b>Next}| line</b></blockquote>"
        )
    }

    #[test]
    fn apply_quote_to_formatted_text_with_line_breaks() {
        let mut model = cm("<b>Some |text<br />Next line</b>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote>~<b>Some |text</b></blockquote><b><br />Next line</b>"
        )
    }

    #[test]
    fn apply_quote_to_single_list_item_with_formatted_text_and_line_breaks() {
        let mut model =
            cm("<ul><li>~<b>Some |text<br />Next line</b></li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<ul><li>~<blockquote>~<b>Some |text</b></blockquote><b><br />Next line</b></li></ul>"
        )
    }

    #[test]
    fn apply_quote_to_several_list_items_with_formatted_text_and_line_breaks() {
        let mut model =
            cm("<ul><li>~<b>Some {text<br />Next line</b></li><li>~<i>Second}| item</i></li><li>~Third item</li></ul>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote>~<ul><li>~<b>Some {text<br />Next line</b></li><li>~<i>Second}| item</i></li></ul></blockquote><ul><li>~Third item</li></ul>"
        )
    }

    #[test]
    fn apply_quote_to_code_block() {
        let mut model = cm("<pre>~Some| code</pre>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<blockquote>~<pre>~Some| code</pre></blockquote>"
        )
    }

    #[test]
    fn remove_quote_with_simple_text() {
        let mut model = cm("<blockquote>~Text|</blockquote>");
        model.quote();
        assert_eq!(tx(&model), "Text|");
    }

    #[test]
    fn remove_quote_with_simple_text_with_adjacent_nodes() {
        let mut model = cm("<blockquote>~Text|</blockquote> with plain text");
        model.quote();
        assert_eq!(tx(&model), "Text| with plain text");
    }

    #[test]
    fn remove_quote_with_nested_formatted_text() {
        let mut model = cm("<blockquote>~<b><i>Text|<br /> Some other text</i></b></blockquote>");
        model.quote();
        assert_eq!(tx(&model), "<b><i>Text|<br /> Some other text</i></b>");
    }

    #[test]
    fn remove_quote_with_list() {
        let mut model = cm("<blockquote>~<b><i>Text|</i></b><ul><li>~Fist item</li></ul></blockquote>");
        model.quote();
        assert_eq!(
            tx(&model),
            "<b><i>Text|</i></b><ul><li>~Fist item</li></ul>"
        );
    }

    #[test]
    fn remove_quote_containing_code_block() {
        let mut model = cm("<blockquote>~<pre>~Some| code</pre></blockquote>");
        model.quote();
        assert_eq!(tx(&model), "<pre>~Some| code</pre>");
    }
}
