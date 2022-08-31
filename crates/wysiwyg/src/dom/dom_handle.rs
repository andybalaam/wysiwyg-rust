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

#[derive(Clone, Debug, PartialEq)]
pub struct DomHandle {
    // The location of a node in the tree, or None if we don't know yet
    path: Option<Vec<usize>>,
}

impl DomHandle {
    /// Create a new handle with the provided path.
    /// So long as the path provided points to a valid node, this handle
    /// can be used (it is set).
    pub fn from_raw(path: Vec<usize>) -> Self {
        Self { path: Some(path) }
    }

    /// Create a new UNSET handle
    /// Don't use this handle for anything until it has been set.
    /// Most methods will panic!
    pub fn new_unset() -> Self {
        Self { path: None }
    }

    /// Returns true if this handle has been set to a value
    pub fn is_set(&self) -> bool {
        !self.path.is_none()
    }

    /// Returns true if this handle refers to a root node
    /// Panics if this handle is unset.
    pub fn is_root(&self) -> bool {
        self.raw().is_empty()
    }

    /// Return the handle of this node's parent, or None if this is the
    /// root node.
    /// Panics if this handle is unset
    /// Panics if we have no parent (i.e. this handle is the root)
    pub fn parent_handle(&self) -> DomHandle {
        let path = self.raw();
        if path.is_empty() {
            panic!("Root handle has no parent!");
        } else {
            let mut new_path = path.clone();
            new_path.pop();
            DomHandle::from_raw(new_path)
        }
    }

    /// Return a new handle for one of our children, with the supplied index.
    /// Panics if this handle is unset
    pub fn child_handle(&self, child_index: usize) -> DomHandle {
        let mut new_path = self.raw().clone();
        new_path.push(child_index);
        DomHandle::from_raw(new_path)
    }

    /// Return true if this handle has a parent i.e. it is not the root. If
    /// this returns true, it is safe to call index_in_parent() and
    /// parent_handle().
    /// Panics if this handle is unset
    pub fn has_parent(&self) -> bool {
        self.raw().len() > 0
    }

    /// Return this handle's position within its parent.
    /// Panics if this handle is unset
    /// Panics if we have no parent (i.e. this handle is the root)
    pub fn index_in_parent(&self) -> usize {
        self.raw()
            .last()
            .expect("Root handle has no parent!")
            .clone()
    }

    /// Return the underlying path used to represent this handle.
    /// Panics if this handle is unset
    pub fn raw(&self) -> &Vec<usize> {
        let path = self.path.as_ref().expect("Handle is unset!");
        &path
    }
}
