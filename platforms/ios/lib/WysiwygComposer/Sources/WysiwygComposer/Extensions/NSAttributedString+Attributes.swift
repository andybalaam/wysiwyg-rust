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

import UIKit

extension NSAttributedString {
    /// Retrieves character at given attributed index.
    ///
    /// - Parameters:
    ///   - index: the attributed string index to lookup
    /// - Returns: the character at given location
    func character(at index: Int) -> Character? {
        let substring = attributedSubstring(from: .init(location: index, length: 1))
        return substring.string.first
    }
    
    /// Enumerate attribute for given key and conveniently ignore any attribute that doesn't match given generic type.
    ///
    /// - Parameters:
    ///   - attrName: The name of the attribute to enumerate.
    ///   - enumerationRange: The range over which the attribute values are enumerated. If omitted, the entire range is used.
    ///   - opts: The options used by the enumeration. For possible values, see NSAttributedStringEnumerationOptions.
    ///   - block: The block to apply to ranges of the specified attribute in the attributed string.
    func enumerateTypedAttribute<T>(_ attrName: NSAttributedString.Key,
                                    in enumerationRange: NSRange? = nil,
                                    options opts: NSAttributedString.EnumerationOptions = [],
                                    using block: (T, NSRange, UnsafeMutablePointer<ObjCBool>) -> Void) {
        enumerateAttribute(attrName,
                           in: enumerationRange ?? .init(location: 0, length: length),
                           options: opts) { (attr: Any?, range: NSRange, stop: UnsafeMutablePointer<ObjCBool>) in
            guard let typedAttr = attr as? T else { return }
            
            block(typedAttr, range, stop)
        }
    }
    
    /// Retrieve font symbolic traits at a given attributed index.
    ///
    /// - Parameters:
    ///   - index: the attributed string index to lookup
    /// - Returns: the symbolic traits at givem location, empty if no font is defined
    func fontSymbolicTraits(at index: Int) -> UIFontDescriptor.SymbolicTraits {
        let font = attribute(.font, at: index, effectiveRange: nil) as? UIFont
        return font?.fontDescriptor.symbolicTraits ?? []
    }
    
    /// Changes the attribute of foregroundColor for the whole attributed string
    ///
    /// - Parameters:
    ///   - color: the new UIColor to update the attributed string
    ///   - linkColor: the new UIColor for links inside the attributed string
    ///   - codeBackgroundColor: the new UIColor for the background of code blocks
    /// - Returns: a new attributed string with the same content and attributes, but its foregroundColor is changed
    func changeColor(to color: UIColor, linkColor: UIColor, codeBackgroundColor: UIColor) -> NSAttributedString {
        let mutableAttributed = NSMutableAttributedString(attributedString: self)
        mutableAttributed.addAttributes(
            [.foregroundColor: color], range: NSRange(location: 0, length: mutableAttributed.length)
        )
        
        // This fixes an iOS bug where if some text is typed after a link, and then a whitespace is added the link color is overridden.
        mutableAttributed.enumerateAttribute(.link, in: NSRange(location: 0, length: mutableAttributed.length)) { value, range, _ in
            if value != nil {
                mutableAttributed.addAttributes([.foregroundColor: linkColor], range: range)
            }
        }
        
        // Adding background to inline code
        mutableAttributed.enumerateTypedAttribute(.font) { (font: UIFont, range, _) in
            if font.familyName == "Courier" {
                mutableAttributed.addAttributes([.backgroundColor: codeBackgroundColor], range: range)
            }
        }
        let newSelf = NSAttributedString(attributedString: mutableAttributed)
        return newSelf
    }
}
