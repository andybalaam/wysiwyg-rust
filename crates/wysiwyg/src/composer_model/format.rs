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

use crate::composer_model::base::{slice, slice_from, slice_to};
use crate::dom::nodes::{ContainerNodeKind, DomNode, TextNode};
use crate::dom::{
    Dom, DomHandle, DomLocation, MultipleNodesRange, Range, SameNodeRange,
};
use crate::{ComposerModel, ComposerUpdate, InlineFormatType, UnicodeString};

#[derive(Eq, PartialEq, Debug)]
enum FormatSelectionType {
    Extend,
    Remove,
}

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn format(&mut self, format: InlineFormatType) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                self.format_same_node(range, format);
                // TODO: for now, we replace every time, to check ourselves, but
                // at least some of the time we should not
                self.create_update_replace_all()
            }

            Range::NoNode => {
                self.state.dom.append_child(DomNode::new_formatting(
                    format,
                    vec![DomNode::new_text(S::from_str(""))],
                ));
                ComposerUpdate::keep()
            }

            Range::MultipleNodes(range) => {
                self.format_several_nodes(&range, format);
                self.create_update_replace_all()
            }
        }
    }

    fn format_same_node(
        &mut self,
        range: SameNodeRange,
        format: InlineFormatType,
    ) {
        let node = self.state.dom.lookup_node(range.node_handle.clone());
        if let DomNode::Text(t) = node {
            let text = t.data();
            // TODO: can we be globally smart about not leaving empty text nodes ?
            let before = slice_to(text, ..range.start_offset);
            let during = slice(text, range.start_offset..range.end_offset);
            let after = slice_from(text, range.end_offset..);
            let new_nodes = vec![
                DomNode::new_text(before),
                DomNode::new_formatting(
                    format,
                    vec![DomNode::new_text(during)],
                ),
                DomNode::new_text(after),
            ];
            self.state.dom.replace(range.node_handle, new_nodes);
        } else {
            panic!("Trying to bold a non-text node")
        }
    }

    fn check_format_selection_type(
        &self,
        locations: &Vec<DomLocation>,
        format: &InlineFormatType,
    ) -> FormatSelectionType {
        // First sweep to understand what the underlying DOM looks like
        let found_format_locations: Vec<&DomLocation> = locations
            .iter()
            .filter(|l| {
                let node = self.state.dom.lookup_node(l.node_handle.clone());
                Self::is_format_node(node, format)
            })
            .collect();

        // No format nodes found, so it can we can only create new formatting nodes by Extend
        if found_format_locations.is_empty() {
            FormatSelectionType::Extend
        } else {
            // Find text nodes inside the selection that are not formatted with this format
            let non_formatted_leaf_locations = locations.iter().filter(|l| {
                l.is_leaf
                    && Self::path_contains_format_node(
                        &self.state.dom,
                        l.node_handle.clone(),
                        &format,
                    )
                    .is_none()
            });
            // If there are selected non-formatted text nodes, we need to extend the format to them
            let is_extend = non_formatted_leaf_locations.count() > 0;
            if is_extend {
                FormatSelectionType::Extend
            } else {
                // Otherwise, we found only format notes partially or totally covered by the
                // selection, we need to remove formatting in the selection range
                FormatSelectionType::Remove
            }
        }
    }

    fn format_several_nodes(
        &mut self,
        range: &MultipleNodesRange,
        format: InlineFormatType,
    ) {
        let selection_type =
            self.check_format_selection_type(&range.locations, &format);
        match selection_type {
            FormatSelectionType::Remove => {} // TODO: actually implement this
            FormatSelectionType::Extend => self
                .extend_format_in_multiple_nodes(
                    range.locations.clone(),
                    &format,
                ),
        }
    }

    fn needs_format(
        dom: &Dom<S>,
        loc: &DomLocation,
        format: &InlineFormatType,
    ) -> bool {
        let handle = loc.node_handle.clone();
        loc.is_leaf
            && Self::path_contains_format_node(dom, handle, format).is_none()
    }

    fn extend_format_in_multiple_nodes(
        &mut self,
        locations: Vec<DomLocation>,
        format: &InlineFormatType,
    ) {
        // Go through the locations in reverse order to prevent Dom modification issues
        for loc in locations.iter().rev() {
            if Self::needs_format(&self.state.dom, loc, &format) {
                if let DomNode::Container(parent) = self
                    .state
                    .dom
                    .lookup_node_mut(loc.node_handle.parent_handle())
                {
                    let index = loc.node_handle.index_in_parent();
                    let node = parent.remove_child(index);
                    if loc.is_covered() {
                        // Node completely covered by selection, happy path. Just replace the old
                        // text node with a formatting node that contains a copy.
                        let format_node =
                            DomNode::new_formatting(format.clone(), vec![node]);
                        parent.insert_child(index, format_node);
                    } else {
                        // Node only partially covered by selection, we need to split the text node,
                        // add one part to a new formatting node, then replace the original text
                        // node with both the new formatting node and the other half of the text
                        // node to their original parent.
                        let position = if loc.is_start() {
                            loc.start_offset
                        } else {
                            loc.end_offset
                        };
                        if let Some((orig, new)) =
                            Self::split_text_node(node, position)
                        {
                            if loc.is_start() {
                                let new = DomNode::new_formatting(
                                    format.clone(),
                                    vec![DomNode::Text(new)],
                                );
                                parent.insert_child(index, new);
                                parent.insert_child(index, DomNode::Text(orig));
                            } else {
                                let orig = DomNode::new_formatting(
                                    format.clone(),
                                    vec![DomNode::Text(orig)],
                                );
                                parent.insert_child(index, DomNode::Text(new));
                                parent.insert_child(index, orig);
                            }
                        } else {
                            panic!("Node was not a text node so it cannot be split.");
                        }
                    }
                    // Clean up by removing any empty text nodes and merging formatting nodes
                    self.remove_empty_text_nodes(
                        loc.node_handle.parent_handle(),
                    );
                    self.merge_formatting_node_with_siblings(
                        loc.node_handle.clone(),
                    );
                }
            }
        }
    }

    fn path_contains_format_node(
        dom: &Dom<S>,
        handle: DomHandle,
        format: &InlineFormatType,
    ) -> Option<DomHandle> {
        if Self::is_format_node(dom.lookup_node(handle.clone()), format) {
            Some(handle)
        } else if handle.has_parent() {
            let parent_handle = handle.parent_handle();
            if Self::is_format_node(
                dom.lookup_node(parent_handle.clone()),
                format,
            ) {
                Some(parent_handle)
            } else {
                Self::path_contains_format_node(dom, parent_handle, format)
            }
        } else {
            None
        }
    }

    fn is_format_node(node: &DomNode<S>, format: &InlineFormatType) -> bool {
        if let DomNode::Container(n) = node {
            if let ContainerNodeKind::Formatting(kind) = n.kind() {
                if kind == format {
                    return true;
                }
            }
        }
        false
    }

    fn split_text_node(
        node: DomNode<S>,
        position: usize,
    ) -> Option<(TextNode<S>, TextNode<S>)> {
        if let DomNode::Text(text_node) = node {
            let split_data_orig = slice_to(text_node.data(), ..position);
            let split_data_new = slice_from(text_node.data(), position..);
            let mut orig = TextNode::from(split_data_orig);
            orig.set_handle(text_node.handle());
            let new = TextNode::from(split_data_new);
            Some((orig, new))
        } else {
            None
        }
    }

    fn remove_empty_text_nodes(&mut self, handle: DomHandle) {
        if let DomNode::Container(parent) =
            self.state.dom.lookup_node_mut(handle.clone())
        {
            let mut indexes_to_remove = Vec::new();
            let children = parent.children();
            for child in children.iter().rev() {
                if let DomNode::Text(n) = child {
                    if n.data().is_empty() {
                        indexes_to_remove.push(n.handle().index_in_parent());
                    }
                }
            }
            for i in indexes_to_remove {
                parent.remove_child(i);
            }
        }
    }

    fn merge_formatting_node_with_siblings(&mut self, handle: DomHandle) {
        // If has next sibling, try to join it with the current node
        if let DomNode::Container(parent) =
            self.state.dom.lookup_node(handle.parent_handle())
        {
            if parent.children().len() - handle.index_in_parent() > 1 {
                self.join_format_node_with_prev(handle.next_sibling());
            }
        }
        // Merge current node with previous if possible
        self.join_format_node_with_prev(handle);
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::cm;

    use super::*;

    fn find_range<S: UnicodeString>(model: &ComposerModel<S>) -> Range {
        let (start, end) = model.get_selection();
        model.state.dom.find_range(start.into(), end.into())
    }

    fn find_multiple_range<S: UnicodeString>(
        model: &ComposerModel<S>,
    ) -> MultipleNodesRange {
        let range = find_range(&model);
        if let Range::MultipleNodes(r) = range {
            r
        } else {
            panic!("Should have been a multiple nodes range, {:?}", range);
        }
    }

    #[test]
    fn selection_type_extend_if_different_type() {
        let model = cm("{hello <b>wor}|ld</b>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Italic,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_extending_start() {
        let model = cm("hell{o <b>w}|orld</b>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_extending_end() {
        let model = cm("<b>hell{o </b>wor}|ld");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_extending_middle() {
        let model = cm("<b>h{el</b>lo <b>wor}|ld</b>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Extend);
    }

    #[test]
    fn selection_type_remove() {
        let model = cm("<b>hel{lo</b><i><b>wor}|ld</b></i>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Remove);
    }

    #[test]
    fn selection_type_remove_on_start_edge() {
        let model = cm("{<b>hello </b><i><b>wor}|ld</b></i>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Remove);
    }

    #[test]
    fn selection_type_remove_on_ending_edge() {
        let model = cm("<b>hel{lo </b><i><b>world}|</b></i>");
        let range = find_multiple_range(&model);
        let selection_type = model.check_format_selection_type(
            &range.locations,
            &InlineFormatType::Bold,
        );
        assert_eq!(selection_type, FormatSelectionType::Remove);
    }

    #[test]
    fn formatting_several_nodes_works_with_different_format() {
        let mut model = cm("{hello <i>wor}|ld</i>");
        model.format(InlineFormatType::Bold);
        assert_eq!(
            model.state.dom.to_string(),
            "<strong>hello </strong><i><strong>wor</strong>ld</i>"
        );
    }

    #[test]
    fn formatting_several_nodes_works_with_same_format() {
        let mut model = cm("{hello <b>wor}|ld</b>");
        model.format(InlineFormatType::Bold);
        assert_eq!(model.state.dom.to_string(), "<strong>hello world</strong>");
    }

    #[test]
    fn formatting_several_nodes_works_with_plain_text_between() {
        let mut model = cm("<b>{hello</b> <b>wor}|ld</b>");
        model.format(InlineFormatType::Bold);
        assert_eq!(model.state.dom.to_string(), "<b>hello world</b>");
    }

    #[test]
    fn formatting_several_nodes_works_with_same_format_rev() {
        let mut model = cm("|{hello <b>wor}ld</b>");
        model.format(InlineFormatType::Bold);
        assert_eq!(model.state.dom.to_string(), "<strong>hello world</strong>");
    }
}
