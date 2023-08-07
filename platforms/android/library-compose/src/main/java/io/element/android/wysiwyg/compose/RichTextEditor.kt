package io.element.android.wysiwyg.compose

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.widget.addTextChangedListener
import io.element.android.wysiwyg.EditorEditText
import io.element.android.wysiwyg.compose.internal.ViewConnection
import io.element.android.wysiwyg.view.models.InlineFormat

/**
 * A composable rich text editor.
 *
 * This composable is a wrapper around the [EditorEditText] view.
 *
 * @param state The state holder for this composable. See [rememberRichTextEditorState].
 * @param modifier The modifier for the layout
 */
@Composable
fun RichTextEditor(
    state: RichTextEditorState,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    AndroidView(
        modifier = modifier,
        factory = {
            val view = EditorEditText(context).apply {
                actionStatesChangedListener =
                    EditorEditText.OnActionStatesChangedListener { actionStates ->
                        state.actions = actionStates
                    }

                selectionChangeListener =
                    EditorEditText.OnSelectionChangeListener { start, end ->
                        state.selection = start to end
                    }
                menuActionListener = EditorEditText.OnMenuActionChangedListener { menuAction ->
                    state.menuAction = menuAction
                }

                addTextChangedListener {
                    state.messageHtml = getContentAsMessageHtml()
                    state.messageMarkdown = getMarkdown()
                }

                // Restore the state of the view with the saved state
                setHtml(state.messageHtml)
            }

            state.viewConnection = object : ViewConnection {
                override fun toggleBold() = view.toggleInlineFormat(InlineFormat.Bold)

                override fun toggleItalic() = view.toggleInlineFormat(InlineFormat.Italic)

                override fun setHtml(html: String) = view.setHtml(html)
            }

            view
        },
    )
}
