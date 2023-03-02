//
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

import SwiftUI
import UIKit

/// Defines accessibility identifiers shared between the UIKit and the SwiftUI example apps.
public enum WysiwygSharedAccessibilityIdentifier: String {
    // Composer text view needs to be in sync with the value set for the text view in the library
    // Unfortunately trying to expose it from the lib results in undefined symbols error in a UI Test context.
    case composerTextView = "WysiwygComposer"
    case boldButton = "WysiwygBoldButton"
    case italicButton = "WysiwygItalicButton"
    case strikeThroughButton = "WysiwygStrikeThroughButton"
    case underlineButton = "WysiwygUnderlineButton"
    case inlineCodeButton = "WysiwygInlineCodeButton"
    case linkButton = "WysiwygLinkButton"
    case undoButton = "WysiwygUndoButton"
    case redoButton = "WysiwygRedoButton"
    case orderedListButton = "WysiwygOrderedListButton"
    case unorderedListButton = "WysiwygUnorderedListButton"
    case indentButton = "WysiwygIndentButton"
    case unindentButton = "WysiwygUnindentButton"
    case codeBlockButton = "WysiwygCodeBlockButton"
    case quoteButton = "WysiwygQuoteButton"
    case sendButton = "WysiwygSendButton"
    case minMaxButton = "WysiwygMinMaxButton"
    case plainRichButton = "WysiwygPlainRichButton"
    case contentText = "WysiwygContentText"
    case htmlContentText = "WysiwygHtmlContentText"
    case linkUrlTextField = "WysiwygLinkUrlTextField"
    case linkTextTextField = "WysiwygLinkTextTextField"
    case showTreeButton = "WysiwygShowTreeButton"
    case treeText = "WysiwygTreeText"
    case forceCrashButton = "WysiwygForceCrashButton"
    case setHtmlButton = "WysiwygSetHtmlButton"
}

public extension View {
    /// Sets up an accessibility identifier to the view from the enum
    /// of expected accessibilityIdentifiers.
    ///
    /// - Parameters:
    ///   - identifier: the accessibility identifier to setup
    /// - Returns: modified view
    func accessibilityIdentifier(_ identifier: WysiwygSharedAccessibilityIdentifier)
        -> ModifiedContent<Self, AccessibilityAttachmentModifier> {
        accessibilityIdentifier(identifier.rawValue)
    }
}
