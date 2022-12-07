package io.element.android.wysiwyg.test

import android.graphics.Typeface
import android.text.Editable
import android.text.style.BulletSpan
import android.text.style.StyleSpan
import android.view.KeyEvent
import android.view.View
import android.widget.EditText
import android.widget.TextView
import androidx.core.text.getSpans
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.accessibility.AccessibilityChecks
import androidx.test.espresso.action.ViewActions.*
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.BoundedDiagnosingMatcher
import androidx.test.espresso.matcher.ViewMatchers
import androidx.test.espresso.matcher.ViewMatchers.withId
import androidx.test.espresso.matcher.ViewMatchers.withText
import androidx.test.ext.junit.rules.ActivityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.spans.LinkSpan
import io.element.android.wysiwyg.spans.OrderedListSpan
import io.element.android.wysiwyg.test.utils.*
import io.mockk.confirmVerified
import io.mockk.spyk
import io.mockk.verify
import org.hamcrest.CoreMatchers
import org.hamcrest.Description
import org.junit.*
import org.junit.runner.RunWith
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction

@RunWith(AndroidJUnit4::class)
class EditorEditTextInputTests {

    @get:Rule
    val scenarioRule = ActivityScenarioRule(TestActivity::class.java)

    private val ipsum = "Lorem Ipsum is simply dummy text of the printing and typesetting industry."

    init {
        AccessibilityChecks.enable()
    }

    @After
    fun cleanUp() {
        // Finish composing just in case, to prevent clashes between test cases
        onView(withId(R.id.rich_text_edit_text)).perform(ImeActions.finishComposingText())
    }

    @Test
    fun testHardwareKeyboardTyping() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(typeText(ipsum))
            .check(matches(withText(ipsum)))
    }

    @Test
    fun testHardwareKeyboardBackspace() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(typeText("Test"))
            .perform(pressKey(KeyEvent.KEYCODE_DEL))
            .check(matches(withText("Tes")))
            // Type a character again to make sure the composer and the UI match
            .perform(typeText("t"))
            .check(matches(withText("Test")))
    }

    @Test
    fun testHardwareKeyboardDelete() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(typeText("Test"))
            .perform(ImeActions.setSelection(0))
            .perform(pressKey(KeyEvent.KEYCODE_FORWARD_DEL))
            .check(matches(withText("est")))
    }

    @Test
    fun testHardwareKeyboardBackspaceEmoji() {
        onView(withId(R.id.rich_text_edit_text))
            // pressKey doesn't seem to work if no `typeText` is used before
            .perform(pressKey(KeyEvent.KEYCODE_A))
            .perform(replaceText("\uD83D\uDE2E\u200D\uD83D\uDCA8"))
            .perform(pressKey(KeyEvent.KEYCODE_DEL))
            .check(matches(withText("")))
    }

    @Test
    fun testHardwareKeyboardDeleteEmoji() {
        onView(withId(R.id.rich_text_edit_text))
            // pressKey doesn't seem to work if no `typeText` is used before
            .perform(pressKey(KeyEvent.KEYCODE_A))
            .perform(replaceText("\uD83D\uDE2E\u200D\uD83D\uDCA8"))
            .perform(AnyViewAction { view -> (view as EditText).setSelection(0) })
            .perform(pressKey(KeyEvent.KEYCODE_FORWARD_DEL))
            .check(matches(withText("")))
    }

    @Test
    fun testReplace() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(replaceText(ipsum))
            .check(matches(withText(ipsum)))
    }

    @Test
    fun testImeSetComposingText() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
            .check(matches(withText("Test")))
    }

    @Test
    fun testImeCommitText() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
                // This should actually be automatic
            .perform(ImeActions.setComposingRegion(0, 4))
                // This should replace "Test" with "Testing"
            .perform(ImeActions.commitText("Testing"))
            .check(matches(withText("Testing")))
    }

    @Test
    fun testImeBackspace() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.backspace())
            .check(matches(withText("Tes")))
    }

    @Test
    fun testSetSelection() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.setSelection(2))
            .check(matches(selectionIsAt(2)))
    }

    @Test
    fun testImeDeleteSurroundingText() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Test"))
            .perform(ImeActions.setSelection(2))
            .perform(ImeActions.deleteSurrounding(1, 1))
            .check(matches(withText("Tt")))
    }

    @Test
    fun testHardwareKeyMovementNotIntercepted() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(typeText("Test"))
            .perform(pressKey(KeyEvent.KEYCODE_DPAD_LEFT))
            .check(matches(selectionIsAt(3)))
            .perform(pressKey(KeyEvent.KEYCODE_DPAD_LEFT))
            .check(matches(selectionIsAt(2)))
    }

    @Test
    fun testJapaneseInputHiraganaToKanji() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("う")) // U (Hiragana)
            .perform(ImeActions.setComposingText("み")) // Mi (Hiragana)
            .perform(ImeActions.commitText("海")) // Umi (Kanji through autocomplete)
            .check(matches(withText("海")))
    }

    @Test
    fun testJapaneseInputHiraganaDeletion() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("うみ")) // Umi (Hiragana)
            .perform(ImeActions.backspace())
            .check(matches(withText("う"))) // U (Hiragana)
    }

    @Test
    fun testJapaneseInputKanjiDeletion() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.commitText("海")) // Umi (Kanji through autocomplete)
            .perform(ImeActions.backspace())
            .check(matches(withText("")))
    }

    @Test
    fun testKoreanInputSeparateCharactersJoined() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("ㅂ")) // B/P (Piup)
            .perform(ImeActions.setComposingText("바")) // B/P + A
            .perform(ImeActions.setComposingText("밥")) // B/P + A + B/P
            .check(matches(withText("밥")))
    }

    @Test
    fun testSettingLink() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("a link to set"))
            .perform(ImeActions.setSelection(2, 6))
            .perform(EditorActions.setLink("https://element.io"))
            .check(matches(TextViewMatcher {
                it.editableText.getSpans<LinkSpan>().isNotEmpty()
            }))
    }

    @Test
    fun testSettingLink_withoutSelection_hasNoEffect() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("a link to set"))
            .perform(ImeActions.setSelection(2, 2))
            .perform(EditorActions.setLink("https://element.io"))
            .check(matches(TextViewMatcher {
                it.editableText.getSpans<LinkSpan>().isEmpty()
            }))
    }

    @Test
    fun testRemovingLink() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("a link to set"))
            .perform(ImeActions.setSelection(2, 6))
            .perform(EditorActions.setLink("https://element.io"))
            .perform(EditorActions.removeLink())
            .check(matches(TextViewMatcher {
                it.editableText.getSpans<LinkSpan>().isEmpty()
            }))
    }

    @Test
    fun testInsertingLink_inSpace() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("a  b"))
            .perform(ImeActions.setSelection(2, 2))
            .perform(EditorActions.insertLink("Element", "https://element.io"))
            .check(matches(TextViewMatcher {
                it.editableText.getSpans<LinkSpan>().isNotEmpty()
            }))
    }

    @Test
    fun testInsertingLink_onSelection_hasNoEffect() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("a link to set"))
            .perform(ImeActions.setSelection(2, 6))
            .perform(EditorActions.insertLink("Element", "https://element.io"))
            .check(matches(TextViewMatcher {
                it.editableText.getSpans<LinkSpan>().isEmpty()
            }))
    }

    @Test
    @Ignore("Lists are being refactored at the moment")
    fun testAddingOrderedList() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(EditorActions.toggleList(true))
            .perform(ImeActions.setComposingText("A list item"))
            .perform(ImeActions.enter())
            .perform(ImeActions.setComposingText("Another list item"))
            .check(matches(withText("\u200bA list item\n\u200bAnother list item")))
            .check(matches(TextViewMatcher {
                // Has 2 OrderedListSpans (prefixes, 1 per line)
                it.editableText.getSpans<OrderedListSpan>().count() == 2
            }))
    }

    @Test
    @Ignore("Lists are being refactored at the moment")
    fun testAddingUnorderedList() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(EditorActions.toggleList(false))
            .perform(ImeActions.setComposingText("A list item"))
            .perform(ImeActions.enter())
            .perform(ImeActions.setComposingText("Another list item"))
            .check(matches(withText("\u200bA list item\n\u200bAnother list item")))
            .check(matches(TextViewMatcher {
                // Has 2 OrderedListSpans (prefixes, 1 per line)
                it.editableText.getSpans<BulletSpan>().count() == 2
            }))
    }

    @Test
    fun testUndo() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Some text to undo"))
            .check(matches(withText("Some text to undo")))
            .perform(EditorActions.undo())
            .check(matches(withText("")))
    }

    @Test
    fun testRedo() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("Some text to undo"))
            .check(matches(withText("Some text to undo")))
            .perform(EditorActions.undo())
            .check(matches(withText("")))
            .perform(EditorActions.redo())
            .check(matches(withText("Some text to undo")))
    }

    // About IME backspace on Korean, that's handled by the IME, which automatically seems to either
    // remove the last code unit from the code point, or 'undo' the last action and send the last
    // compositing text.
    @Test
    @Ignore("These are failing at the moment. The whole text is deleted. Note that this backspace action mimicks HW keyboard backspace, not IME.")
    fun testKoreanInputSeparateCharactersDeletion() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.setComposingText("ㅂ")) // B/P (Piup)
            .perform(ImeActions.setComposingText("바")) // B/P + A
            .perform(ImeActions.backspace())
            .check(matches(withText("ㅂ")))
    }

    @Test
    @Ignore("These are failing at the moment. The whole text is deleted. Note that this backspace action mimicks HW keyboard backspace, not IME.")
    fun testKoreanInputJoinedCharactersDeletion() {
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.commitText("밥")) // Bap (autocomplete)
            .perform(ImeActions.backspace())
            .check(matches(withText("바")))
    }

    @Test
    fun testBoldFormatting() {
        val start = 6
        val end = 11
        // Write and select text
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.commitText(ipsum))
            .perform(ImeActions.setSelection(start, end))
            .perform(EditorActions.toggleFormat(InlineFormat.Bold))
            // Check text contains a Bold StyleSpan
            .check(matches(TextViewMatcher { view ->
                view.editableText.getSpans<StyleSpan>(start, end)
                    .any { (it as? StyleSpan)?.style == Typeface.BOLD }
            }))
    }

    @Test
    fun testMenuStateChangedListener() {
        var isItalicHighlighted = false
        scenarioRule.scenario.onActivity {
            it.findViewById<EditorEditText>(R.id.rich_text_edit_text).actionStatesChangedListener =
                EditorEditText.OnActionStatesChangedListener { actionStates ->
                    if (actionStates.get(ComposerAction.ITALIC) == ActionState.REVERSED) {
                        isItalicHighlighted = true
                    }
                }
        }

        val start = 6
        val end = 11
        onView(withId(R.id.rich_text_edit_text))
            .perform(ImeActions.commitText(ipsum))
            .perform(ImeActions.setSelection(start, end))
            .perform(EditorActions.toggleFormat(InlineFormat.Italic))

        Assert.assertTrue(isItalicHighlighted)
    }

    @Test
    fun testTextWatcher() {
        val textWatcher = spyk<(text: Editable?) -> Unit>({ })
        onView(withId(R.id.rich_text_edit_text))
            .perform(EditorActions.addTextWatcher(textWatcher))
            .perform(EditorActions.setText("text"))
            .perform(ImeActions.setSelection(0, 4))
            .perform(EditorActions.toggleFormat(InlineFormat.Bold))
            .perform(EditorActions.toggleFormat(InlineFormat.Underline))
            .perform(EditorActions.toggleFormat(InlineFormat.Italic))
            .perform(EditorActions.toggleFormat(InlineFormat.StrikeThrough))
            .perform(EditorActions.toggleFormat(InlineFormat.InlineCode))
            .perform(EditorActions.toggleList(ordered = true))
            .perform(EditorActions.toggleList(ordered = false))
            .perform(EditorActions.setHtml("<b>text</b>"))

        verify(exactly = 9) {
            textWatcher.invoke(match { it.toString() == "text" })
        }
        verify(inverse = true) {
            textWatcher.invoke(match { it.toString() == "" })
        }
        confirmVerified(textWatcher)
    }

    @Test
    fun getMarkdownTranslatesDomToMarkdown() {
        scenarioRule.scenario.onActivity { activity ->
            val editor = activity.findViewById<EditorEditText>(R.id.rich_text_edit_text)
            editor.setHtml("<b>Test</b>")
            val markdown = editor.getMarkdown()
            ViewMatchers.assertThat(markdown, CoreMatchers.equalTo("__Test__"))
        }
    }

    @Test
    fun setMarkdownCanParseProperMarkdownIntoDom() {
        scenarioRule.scenario.onActivity { activity ->
            val editor = activity.findViewById<EditorEditText>(R.id.rich_text_edit_text)
            editor.setMarkdown("__Test__")
            ViewMatchers.assertThat(
                editor.getHtmlOutput(),
                CoreMatchers.equalTo("<strong>Test</strong>")
            )
            editor.setMarkdown("**Test**")
            ViewMatchers.assertThat(
                editor.getHtmlOutput(),
                CoreMatchers.equalTo("<strong>Test</strong>")
            )
            editor.setMarkdown("**Test*")
            ViewMatchers.assertThat(editor.getHtmlOutput(), CoreMatchers.equalTo("*<em>Test</em>"))
            editor.setMarkdown("<u>*Test*</u>")
            ViewMatchers.assertThat(
                editor.getHtmlOutput(),
                CoreMatchers.equalTo("<u><em>Test</em></u>")
            )
        }
    }
}

class TextViewMatcher(
    private val check: (TextView) -> Boolean
) : BoundedDiagnosingMatcher<View, EditorEditText>(EditorEditText::class.java) {
    override fun matchesSafely(item: EditorEditText?, mismatchDescription: Description?): Boolean {
        return if (item != null && check(item)) {
            true
        } else {
            mismatchDescription?.appendText("Did not match TextViewMatcher")
            false
        }
    }

    override fun describeMoreTo(description: Description?) {
        description?.appendText("Matches TextViewMatcher")
    }

}
