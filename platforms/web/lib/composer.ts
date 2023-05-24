/*
Copyright 2022 The Matrix.org Foundation C.I.C.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

import { ComposerModel, SuggestionPattern } from '../generated/wysiwyg';
import {
    WysiwygInputEvent,
    InputEventProcessor,
    Wysiwyg,
    FormattingFunctions,
    WysiwygEvent,
} from './types';
import {
    isClipboardEvent,
    isLinkEvent,
    isSuggestionEvent,
} from './useListeners/assert';
import { TestUtilities } from './useTestCases/types';

export function processEvent<T extends WysiwygEvent>(
    e: T,
    wysiwyg: Wysiwyg,
    editor: HTMLElement,
    inputEventProcessor?: InputEventProcessor,
): T | null {
    if (inputEventProcessor) {
        return inputEventProcessor(e, wysiwyg, editor) as T | null;
    } else {
        return e;
    }
}

export function processInput(
    e: WysiwygInputEvent,
    composerModel: ComposerModel,
    action: TestUtilities['traceAction'],
    formattingFunctions: FormattingFunctions,
    editor: HTMLElement,
    suggestion: SuggestionPattern | null,
    inputEventProcessor?: InputEventProcessor,
) {
    const event = processEvent(
        e,
        {
            actions: formattingFunctions,
            content: () => composerModel.get_content_as_html(),
        },
        editor,
        inputEventProcessor,
    );
    if (!event) {
        return;
    }

    if (isClipboardEvent(event)) {
        const data = event.clipboardData?.getData('text/plain') ?? '';
        return action(composerModel.replace_text(data), 'paste');
    }

    switch (event.inputType) {
        case 'insertSuggestion': {
            if (suggestion && isSuggestionEvent(event)) {
                const { text, url, attributes } = event.data;
                const attributesMap = new Map(Object.entries(attributes));

                return action(
                    composerModel.set_link_suggestion(
                        url,
                        text,
                        suggestion,
                        attributesMap,
                    ),
                    'set_link_suggestion',
                );
            }
            break;
        }
        case 'insertCommand': {
            if (suggestion && event.data) {
                return action(
                    composerModel.replace_text_suggestion(
                        event.data,
                        suggestion,
                    ),
                    'replace_text_suggestion',
                );
            }
            break;
        }
        case 'clear':
            return action(composerModel.clear(), 'clear');
        case 'deleteContentBackward':
            return action(composerModel.backspace(), 'backspace');
        case 'deleteWordBackward':
            return action(composerModel.backspace_word(), 'backspace_word');
        case 'deleteSoftLineBackward': {
            const selection = document.getSelection();
            if (selection) {
                selection.modify('extend', 'backward', 'lineboundary');
                document.dispatchEvent(new CustomEvent('selectionchange'));
            }
            return action(composerModel.delete(), 'backspace_line');
        }
        case 'deleteContentForward':
            return action(composerModel.delete(), 'delete');
        case 'deleteWordForward':
            return action(composerModel.delete_word(), 'delete_word');
        case 'deleteByCut':
            return action(composerModel.delete(), 'delete');
        case 'formatBold':
            return action(composerModel.bold(), 'bold');
        case 'formatItalic':
            return action(composerModel.italic(), 'italic');
        case 'formatStrikeThrough':
            return action(composerModel.strike_through(), 'strike_through');
        case 'formatUnderline':
            return action(composerModel.underline(), 'underline');
        case 'formatInlineCode':
            return action(composerModel.inline_code(), 'inline_code');
        case 'historyRedo':
            return action(composerModel.redo(), 'redo');
        case 'historyUndo':
            return action(composerModel.undo(), 'undo');
        case 'insertCodeBlock':
            return action(composerModel.code_block(), 'code_block');
        case 'insertQuote':
            return action(composerModel.quote(), 'quote');
        case 'insertFromPaste':
            // Paste is already handled by catching the 'paste' event, which
            // results in a ClipboardEvent, handled above. Ideally, we would
            // do it here, but Chrome does not provide data inside this
            // InputEvent, only in the original ClipboardEvent.
            return;
        case 'insertOrderedList':
            return action(composerModel.ordered_list(), 'ordered_list');
        case 'insertLineBreak':
        case 'insertParagraph':
            return action(composerModel.enter(), 'enter');
        case 'insertReplacementText': {
            // Remove br tag
            const newContent = editor.innerHTML.slice(
                0,
                editor.innerHTML.length - 4,
            );
            return action(
                composerModel.set_content_from_html(newContent),
                'set_content_from_html',
                newContent,
            );
        }
        case 'insertCompositionText':
        case 'insertFromComposition':
        case 'insertText':
            if (event.data) {
                return action(
                    composerModel.replace_text(event.data),
                    'replace_text',
                    event.data,
                );
            }
            break;
        case 'insertUnorderedList':
            return action(composerModel.unordered_list(), 'unordered_list');
        case 'insertLink':
            if (isLinkEvent(event)) {
                const { text, url } = event.data;
                return action(
                    text
                        ? composerModel.set_link_with_text(url, text)
                        : composerModel.set_link(url),
                    'insertLink',
                );
            }
            break;
        case 'removeLinks':
            return action(composerModel.remove_links(), 'remove_links');
        case 'formatIndent':
            return action(composerModel.indent(), 'indent');
        case 'formatOutdent':
            return action(composerModel.unindent(), 'unindent');
        case 'sendMessage':
            // We create this event type when the user presses Ctrl+Enter.
            // We don't do anythign here, but the user may want to hook in
            // using inputEventProcessor to perform behaviour here.
            return null;
        default:
            // We should cover all of
            // eslint-disable-next-line max-len
            // https://rawgit.com/w3c/input-events/v1/index.html#interface-InputEvent-Attributes
            // Internal task to make sure we cover all inputs: PSU-740
            console.error(`Unknown input type: ${event.inputType}`);
            console.error(e);
            return null;
    }
}
