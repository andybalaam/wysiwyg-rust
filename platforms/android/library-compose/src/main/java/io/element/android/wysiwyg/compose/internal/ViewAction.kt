package io.element.android.wysiwyg.compose.internal

import io.element.android.wysiwyg.view.models.InlineFormat

internal sealed class ViewAction {
    data class ToggleInlineFormat(val inlineFormat: InlineFormat): ViewAction()
    data class ToggleList(val ordered: Boolean): ViewAction()
    data object ToggleCodeBlock: ViewAction()
    data object ToggleQuote: ViewAction()
    data object Undo: ViewAction()
    data object Redo: ViewAction()
    data object Indent: ViewAction()
    data object Unindent: ViewAction()
    data class SetHtml(val html: String): ViewAction()
    data object RequestFocus: ViewAction()
    data class SetLink(val url: String?): ViewAction()
    data object RemoveLink: ViewAction()
    data class InsertLink(val url: String, val text: String): ViewAction()
    data class EditLink(val url: String, val text: String): ViewAction()
}
