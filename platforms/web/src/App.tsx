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

import { MouseEventHandler, useState } from 'react';

import { useWysiwyg } from '../lib/useWysiwyg';
import boldImage from './images/bold.svg';
import undoImage from './images/undo.svg';
import redoImage from './images/redo.svg';
import italicImage from './images/italic.svg';
import underlineImage from './images/underline.svg';
import strikeTroughImage from './images/strike_through.svg';
import listUnorderedImage from './images/list-unordered.svg';
import listOrderedImage from './images/list-ordered.svg';
import { Wysiwyg, WysiwygInputEvent } from '../lib/types';

type ButtonProps = {
    onClick: MouseEventHandler<HTMLButtonElement>;
    imagePath: string;
    alt: string;
    state: 'enabled' | 'disabled' | 'reversed';
};

function Button({ onClick, imagePath, alt, state }: ButtonProps) {
    const isReversed = state === 'reversed';
    const isDisabled = state === 'disabled';
    return (
        <button
            type="button"
            onClick={onClick}
            style={{
                ...(isReversed && { backgroundColor: 'lightgray' }),
                ...(isDisabled && { backgroundColor: 'firebrick' }),
            }}
        >
            <img alt={alt} src={imagePath} />
        </button>
    );
}

function App() {
    const [enterToSend, setEnterToSend] = useState(true);

    const inputEventProcessor = (
        e: WysiwygInputEvent,
        wysiwyg: Wysiwyg,
    ): WysiwygInputEvent | null => {
        if (e instanceof ClipboardEvent) {
            return e;
        } else if (enterToSend && e.inputType === 'insertParagraph') {
            console.log(`SENDING: ${wysiwyg.content()}`);
            e.preventDefault();
            e.stopPropagation();
            wysiwyg.actions.clear();
            return null;
        }
        return e;
    };

    const { ref, isWysiwygReady, formattingStates, wysiwyg, debug } =
        useWysiwyg({
            isAutoFocusEnabled: true,
            inputEventProcessor,
        });

    const onEnterToSendChanged = () => {
        setEnterToSend(!enterToSend);
    };

    return (
        <div className="wrapper">
            <div>
                <div className="editor_container">
                    <div className="editor_toolbar">
                        <Button
                            onClick={wysiwyg.undo}
                            alt="undo"
                            imagePath={undoImage}
                            state={formattingStates.undo}
                        />
                        <Button
                            onClick={wysiwyg.redo}
                            alt="redo"
                            imagePath={redoImage}
                            state={formattingStates.redo}
                        />
                        <Button
                            onClick={wysiwyg.bold}
                            alt="bold"
                            imagePath={boldImage}
                            state={formattingStates.bold}
                        />
                        <Button
                            onClick={wysiwyg.italic}
                            alt="italic"
                            imagePath={italicImage}
                            state={formattingStates.italic}
                        />
                        <Button
                            onClick={wysiwyg.underline}
                            alt="underline"
                            imagePath={underlineImage}
                            state={formattingStates.underline}
                        />
                        <Button
                            onClick={wysiwyg.strikeThrough}
                            alt="strike through"
                            imagePath={strikeTroughImage}
                            state={formattingStates.strikeThrough}
                        />
                        <Button
                            onClick={wysiwyg.unorderedList}
                            alt="list unordered"
                            imagePath={listUnorderedImage}
                            state={formattingStates.unorderedList}
                        />
                        <Button
                            onClick={wysiwyg.orderedList}
                            alt="list ordered"
                            imagePath={listOrderedImage}
                            state={formattingStates.orderedList}
                        />
                        <Button
                            onClick={wysiwyg.inlineCode}
                            alt="inline code"
                            imagePath={listOrderedImage}
                            state={formattingStates.inlineCode}
                        />
                        <button type="button" onClick={(e) => wysiwyg.clear()}>
                            clear
                        </button>
                    </div>
                    <div
                        className="editor"
                        ref={ref}
                        contentEditable={isWysiwygReady}
                    />
                </div>
                <div className="editor_options">
                    <input
                        type="checkbox"
                        id="enterToSend"
                        checked={enterToSend}
                        onChange={onEnterToSendChanged}
                    />
                    <label htmlFor="enterToSend">Enter to "send"</label>
                </div>
            </div>
            <h2>Model:</h2>
            <div className="dom" ref={debug.modelRef} />
            <h2>
                Test case:{' '}
                <button type="button" onClick={debug.resetTestCase}>
                    Start from here
                </button>
            </h2>
            <div className="testCase" ref={debug.testRef}>
                let mut model = cm("");
                <br />
                assert_eq!(tx(&amp;model), "");
            </div>
        </div>
    );
}

export default App;
