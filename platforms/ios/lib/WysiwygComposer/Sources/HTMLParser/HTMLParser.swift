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

import DTCoreText
import UIKit

/// Provides tools to parse from HTML to NSAttributedString with a standard style.
public final class HTMLParser {
    // MARK: - Private
    
    private static var defaultCSS: String {
        """
        blockquote {
            background-color: \(TempColor.quote.toHexString());
            display: block;
        }
        pre {
            background-color: \(TempColor.codeBlock.toHexString());
            display: block;
            font-family: monospace;
            white-space: pre;
            -coretext-fontname: Menlo-Regular;
            font-size: inherit;
        }
        code {
            background-color: \(TempColor.inlineCode.toHexString());
            display: inline;
            font-family: monospace;
            white-space: pre;
            -coretext-fontname: Menlo-Regular;
            font-size: inherit;
        }
        h1,h2,h3 {
            font-size: 1.2em;
        }
        """
    }
    
    private init() { }
    
    // MARK: - Public
    
    /// Parse given HTML to NSAttributedString with a standard style.
    ///
    /// - Parameters:
    ///   - html: HTML to parse
    ///   - encoding: string encoding to use
    ///   - style: style to apply for HTML parsing
    /// - Returns: an attributed string representation of the HTML content
    public static func parse(html: String,
                             encoding: String.Encoding = .utf16,
                             style: HTMLParserStyle = .standard) throws -> NSAttributedString {
        guard !html.isEmpty else {
            return NSAttributedString(string: "")
        }

        guard let data = html.data(using: encoding) else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }
        
        let defaultFont = UIFont.preferredFont(forTextStyle: .body)
        
        let parsingOptions: [String: Any] = [
            DTUseiOS6Attributes: true,
            DTDefaultFontDescriptor: defaultFont.fontDescriptor,
            DTDefaultStyleSheet: DTCSSStylesheet(styleBlock: defaultCSS) as Any,
        ]
        
        guard let builder = DTHTMLAttributedStringBuilder(html: data, options: parsingOptions, documentAttributes: nil) else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }

        builder.willFlushCallback = { element in
            guard let element else { return }
            // Removing NBSP character from <p>&nbsp;</p> since it is only used to
            // make DTCoreText able to easily parse new lines.
            element.clearNbspNodes()
        }
        
        guard let attributedString = builder.generatedAttributedString() else {
            throw BuildHtmlAttributedError.dataError(encoding: encoding)
        }
        
        let mutableAttributedString = NSMutableAttributedString(attributedString: attributedString)
        
        mutableAttributedString.addAttributes(
            [.foregroundColor: style.textColor], range: NSRange(location: 0, length: mutableAttributedString.length)
        )
        
        // This fixes an iOS bug where if some text is typed after a link, and then a whitespace is added the link color is overridden.
        mutableAttributedString.enumerateAttribute(
            .link,
            in: NSRange(location: 0, length: mutableAttributedString.length)
        ) { value, range, _ in
            if value != nil {
                mutableAttributedString.removeAttribute(.underlineStyle, range: range)
                mutableAttributedString.removeAttribute(.underlineColor, range: range)
                mutableAttributedString.addAttributes([.foregroundColor: style.linkColor], range: range)
            }
        }
        
        mutableAttributedString.applyBackgroundStyles(style: style)
        mutableAttributedString.applyInlineCodeBackgroundStyle(codeBackgroundColor: style.codeBackgroundColor)
        mutableAttributedString.removeDiscardableText()
        
        // FIXME: This solution might not fit for everything.
        mutableAttributedString.addAttribute(.paragraphStyle,
                                             value: NSParagraphStyle.default,
                                             range: .init(location: 0, length: mutableAttributedString.length))
        
        removeTrailingNewlineIfNeeded(from: mutableAttributedString, given: html)
        return mutableAttributedString
    }
    
    private static func removeTrailingNewlineIfNeeded(from mutableAttributedString: NSMutableAttributedString, given html: String) {
        // DTCoreText always adds a \n at the end of the document, which we need to remove
        // however it does not add it if </code> </a> are the last nodes.
        // Also we don't want to remove it if blockquote and codeblock contain that newline
        // and are not empty, because DTCoreText does not add a newline if these blocks
        // contain one at the end.
        if mutableAttributedString.string.last == "\n",
           !html.hasSuffix("</code>"),
           !html.hasSuffix("</a>"),
           !html.hasSuffix("</p><p>\(Character.nbsp)</p></blockquote>"),
           !html.hasSuffix("</ul><p>\(Character.nbsp)</p></blockquote>"),
           !html.hasSuffix("</ol><p>\(Character.nbsp)</p></blockquote>"),
           !html.hasSuffix("\n</pre>") {
            mutableAttributedString.deleteCharacters(in: NSRange(location: mutableAttributedString.length - 1, length: 1))
        }
    }
}
