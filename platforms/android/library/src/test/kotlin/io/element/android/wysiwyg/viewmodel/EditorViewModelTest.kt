package io.element.android.wysiwyg.viewmodel

import io.element.android.wysiwyg.inputhandlers.models.EditorInputAction
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.inputhandlers.models.ReplaceTextResult
import io.element.android.wysiwyg.mocks.MockComposer
import io.element.android.wysiwyg.mocks.MockComposerUpdateFactory
import io.element.android.wysiwyg.mocks.MockTextUpdateFactory
import io.element.android.wysiwyg.utils.BasicHtmlConverter
import io.mockk.mockk
import io.mockk.verify
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Before
import org.junit.Test
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.MenuState

internal class EditorViewModelTest {

    private val composer = MockComposer()
    private val htmlConverter = BasicHtmlConverter()
    private val viewModel = EditorViewModel(
        composer = composer.instance,
        htmlConverter = htmlConverter,
    )
    private val menuStateCallback = mockk<(MenuState) -> Unit>(relaxed = true)

    companion object {
        private const val paragraph =
            "Lorem Ipsum is simply dummy text of the printing and typesetting industry."
        private const val updatedParagraph =
            "Lorem Ipsum is updated!"
        private const val htmlParagraphs =
            "<p><b>$paragraph</b></p>" +
                    "<p><i>$paragraph</i></p>"
        private const val plainTextParagraphs = "$paragraph$paragraph"
        private val menuStateUpdate =
            MenuState.Update(listOf(ComposerAction.Bold), listOf(ComposerAction.Link))
        private val composerStateUpdate = MockComposerUpdateFactory.create(
            textUpdate = MockTextUpdateFactory.createReplaceAll(updatedParagraph, 2, 3),
            menuState = menuStateUpdate,
        )
        private val replaceTextResult = ReplaceTextResult(updatedParagraph, 2..3)
    }

    @Before
    fun setUp() {
        viewModel.setActionStatesCallback(menuStateCallback)
    }

    @Test
    fun `when menu state callback is not set, it processes input without an error`() {
        composer.givenReplaceTextResult(composerStateUpdate)
        viewModel.setActionStatesCallback(null)

        val result = viewModel.processInput(EditorInputAction.ReplaceText(paragraph))

        verify(inverse = true) {
            menuStateCallback(menuStateUpdate)
        }

        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process replace text action, it returns a text update`() {
        composer.givenReplaceTextResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ReplaceText(paragraph))

        verify {
            composer.instance.replaceText(paragraph)
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process insert paragraph action, it returns a text update`() {
        composer.givenEnterResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.InsertParagraph)

        verify {
            composer.instance.enter()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process backspace action, it returns a text update`() {
        composer.givenBackspaceResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.BackPress)

        verify {
            composer.instance.backspace()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process bold action, it returns a text update`() {
        composer.givenBoldResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Bold))

        verify {
            composer.instance.bold()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process italic action, it returns a text update`() {
        composer.givenItalicResult(composerStateUpdate)

        val result =
            viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Italic))

        verify {
            composer.instance.italic()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }


    @Test
    fun `when process underline action, it returns a text update`() {
        composer.givenUnderlineResult(composerStateUpdate)

        val result =
            viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.Underline))

        verify {
            composer.instance.underline()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process strike through action, it returns a text update`() {
        composer.givenStrikeThroughResult(composerStateUpdate)

        val result =
            viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.StrikeThrough))

        verify {
            composer.instance.strikeThrough()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process inline code action, it returns a text update`() {
        composer.givenInlineCodeResult(composerStateUpdate)

        val result =
            viewModel.processInput(EditorInputAction.ApplyInlineFormat(InlineFormat.InlineCode))

        verify {
            composer.instance.inlineCode()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process delete in action, it returns a text update`() {
        composer.givenDeleteInResult(3, 4, composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.DeleteIn(3, 4))

        verify {
            composer.instance.deleteIn(3.toUInt(), 4.toUInt())
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process set link action, it returns a text update`() {
        composer.givenSetLinkResult("https://element.io", composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.SetLink("https://element.io"))

        verify {
            composer.instance.setLink("https://element.io")
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process replace all html action, it returns a text update`() {
        composer.givenReplaceAllHtmlResult("new html", composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ReplaceAllHtml("new html"))

        verify {
            composer.instance.setContentFromHtml("new html")
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process undo action, it returns a text update`() {
        composer.givenUndoResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.Undo)

        verify {
            composer.instance.undo()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process redo action, it returns a text update`() {
        composer.givenRedoResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.Redo)

        verify {
            composer.instance.redo()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process toggle ordered list action, it returns a text update`() {
        composer.givenToggleOrderedListResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ToggleList(ordered = true))

        verify {
            composer.instance.orderedList()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `when process toggle unordered list action, it returns a text update`() {
        composer.givenToggleUnorderedListResult(composerStateUpdate)

        val result = viewModel.processInput(EditorInputAction.ToggleList(ordered = false))

        verify {
            composer.instance.unorderedList()
            menuStateCallback(menuStateUpdate)
        }
        assertThat(result, equalTo(replaceTextResult))
    }

    @Test
    fun `given formatted text, getHtml returns formatted HTML`() {
        composer.givenCurrentDomState(htmlParagraphs)

        val html = viewModel.getHtml()

        assertThat(html, equalTo(htmlParagraphs))
    }

    @Test
    fun `given formatted text, getPlainText returns plain text`() {
        composer.givenCurrentDomState(htmlParagraphs)

        val plainText = viewModel.getPlainText()

        assertThat(plainText, equalTo(plainTextParagraphs))
    }

}
