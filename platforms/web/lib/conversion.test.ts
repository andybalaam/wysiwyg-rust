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

import init from '../generated/wysiwyg';
import { richToPlain, plainToRich } from './conversion';

const testCases = [
    { rich: 'plain', plain: 'plain' },
    { rich: '<strong>bold</strong>', plain: '__bold__' },
    { rich: '<em>italic</em>', plain: '*italic*' },
    { rich: '<u>underline</u>', plain: '<u>underline</u>' },
    { rich: '<del>strike</del>', plain: '~~strike~~' },
];

beforeAll(async () => {
    await init();
});

const mappedTestCases = testCases.map(({ rich, plain }) => [rich, plain]);

test.each(mappedTestCases)('rich: `%s` - plain: `%s`', (rich, plain) => {
    const convertedRichText = plainToRich(plain);
    const convertedPlainText = richToPlain(rich);

    expect(convertedRichText).toBe(rich);
    expect(convertedPlainText).toBe(plain);
});

it('converts linebreaks for display rich => plain', () => {
    const richText = 'multi<br />line';
    const convertedPlainText = richToPlain(richText);
    const expectedPlainText = `multi\nline`;

    expect(convertedPlainText).toBe(expectedPlainText);
});

it('converts linebreaks for display plain => rich', () => {
    const plainText = 'multi\nline';
    const convertedRichText = plainToRich(plainText);
    const expectedRichText = 'multi<br />line';

    expect(convertedRichText).toBe(expectedRichText);
});