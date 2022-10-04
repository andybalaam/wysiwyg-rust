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

import { MouseEvent as ReactMouseEvent } from 'react';

import { ComposerModel } from '../../generated/wysiwyg';
import { processInput } from '../composer';
import {
    getCurrentSelection,
    refreshComposerView,
    replaceEditor,
} from '../dom';
import { BlockType, WysiwygInputEvent } from '../types';
import { TestUtilities } from '../useTestCases/types';

export function sendWysiwygInputEvent(
    e: ReactMouseEvent<HTMLElement, MouseEvent> | KeyboardEvent,
    editor: HTMLElement,
    blockType: BlockType,
) {
    e.preventDefault();
    e.stopPropagation();
    editor.dispatchEvent(
        new CustomEvent('wysiwygInput', { detail: { blockType } }),
    );
}

function getInputFromKeyDown(e: KeyboardEvent) {
    if (e.shiftKey && e.altKey) {
        switch (e.key) {
            case '5':
                return 'formatStrikeThrough';
        }
    }

    if (e.ctrlKey || e.metaKey) {
        switch (e.key) {
            case 'b':
                return 'formatBold';
            case 'i':
                return 'formatItalic';
            case 'u':
                return 'formatUnderline';
            case 'y':
                return 'historyRedo';
            case 'z':
                return 'historyUndo';
            case 'Z':
                return 'historyRedo';
        }
    }

    return null;
}

export function handleKeyDown(e: KeyboardEvent, editor: HTMLElement) {
    const inputType = getInputFromKeyDown(e);
    if (inputType) {
        sendWysiwygInputEvent(e, editor, inputType);
    }
}

export function handleInput(
    e: WysiwygInputEvent,
    editor: HTMLElement,
    composerModel: ComposerModel,
    modelNode: HTMLElement | null,
    testUtilities: TestUtilities,
) {
    const update = processInput(e, composerModel, testUtilities.traceAction);
    if (update) {
        const repl = update.text_update().replace_all;
        if (repl) {
            replaceEditor(
                editor,
                repl.replacement_html,
                repl.start_utf16_codeunit,
                repl.end_utf16_codeunit,
            );
            testUtilities.setEditorHtml(repl.replacement_html);
        }

        // Only when
        if (modelNode) {
            refreshComposerView(modelNode, composerModel);
        }
    }
}

export function handleSelectionChange(
    editor: HTMLElement,
    composeModel: ComposerModel,
    { traceAction, getSelectionAccordingToActions }: TestUtilities,
) {
    const isInEditor = document.activeElement === editor;

    // Skip the selection behavior when the focus is not in the editor
    if (!isInEditor) {
        return;
    }

    const [start, end] = getCurrentSelection(editor, document.getSelection());

    const prevStart = composeModel.selection_start();
    const prevEnd = composeModel.selection_end();

    const [actStart, actEnd] = getSelectionAccordingToActions();

    // Ignore selection changes that do nothing
    if (
        start === prevStart &&
        start === actStart &&
        end === prevEnd &&
        end === actEnd
    ) {
        return;
    }

    // Ignore selection changes that just reverse the selection - all
    // backwards selections actually do this, because the browser can't
    // support backwards selections.
    if (
        start === prevEnd &&
        start === actEnd &&
        end === prevStart &&
        end === actStart
    ) {
        return;
    }
    composeModel.select(start, end);
    traceAction(null, 'select', start, end);
}
