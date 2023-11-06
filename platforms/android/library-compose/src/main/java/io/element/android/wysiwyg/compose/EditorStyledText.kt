package io.element.android.wysiwyg.compose

import android.text.Spanned
import android.widget.TextView
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import io.element.android.wysiwyg.EditorStyledTextView
import io.element.android.wysiwyg.compose.internal.applyStyleInCompose
import io.element.android.wysiwyg.compose.internal.rememberTypeface
import io.element.android.wysiwyg.compose.internal.toStyleConfig
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay

/**
 * A composable EditorStyledText.
 * This composable is a wrapper around the [EditorStyledTextView] view.
 *
 * @param text The text to render.
 * If it's spanned it will be rendered as is, otherwise it will go first to HtmlConverter.
 * Your might want to use HtmlConverter before the rendering to avoid the conversion at each recomposition.
 * @param modifier The modifier for the layout.
 * @param resolveMentionDisplay A function to resolve the [TextDisplay] of a mention.
 * @param resolveRoomMentionDisplay A function to resolve the [TextDisplay] of an `@room` mention.
 * @param style The styles to use for any customisable elements.
 */
@Composable
fun EditorStyledText(
    text: CharSequence,
    modifier: Modifier = Modifier,
    resolveMentionDisplay: (text: String, url: String) -> TextDisplay = RichTextEditorDefaults.MentionDisplay,
    resolveRoomMentionDisplay: () -> TextDisplay = RichTextEditorDefaults.RoomMentionDisplay,
    onLinkClickedListener: ((String) -> Unit) = {},
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
) {
    val typeface by style.text.rememberTypeface()
    val mentionDisplayHandler = remember(resolveMentionDisplay, resolveRoomMentionDisplay) {
        object : MentionDisplayHandler {
            override fun resolveMentionDisplay(text: String, url: String): TextDisplay {
                return resolveMentionDisplay(text, url)
            }

            override fun resolveAtRoomMentionDisplay(): TextDisplay {
                return resolveRoomMentionDisplay()
            }
        }
    }
    AndroidView(
        modifier = modifier,
        factory = { context ->
            EditorStyledTextView(context)
        },
        // The `update` lambda is called when the view is first created, and then again whenever the actual `update` lambda changes. That is, it's replaced with
        // a new lambda capturing different variables from the surrounding scope. However, there seems to be an issue that causes the `update` lambda to change
        // more than it's strictly necessary. To avoid this, we can use a `remember` block to cache the `update` lambda, and only update it when needed.
        update = remember(style, typeface, mentionDisplayHandler, text, onLinkClickedListener) {
            { view ->
                view.applyStyleInCompose(style)
                view.typeface = typeface
                view.updateStyle(style.toStyleConfig(view.context), mentionDisplayHandler)
                if (text is Spanned) {
                    view.setText(text, TextView.BufferType.SPANNABLE)
                } else {
                    view.setHtml(text.toString())
                }
                view.onLinkClickedListener = onLinkClickedListener
            }
        }
    )
}
