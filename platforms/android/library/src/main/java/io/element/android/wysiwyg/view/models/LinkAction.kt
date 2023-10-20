package io.element.android.wysiwyg.view.models

/**
 * Link related editor actions, depending on the current selection.
 */
sealed class LinkAction {
    /**
     * Insert new text with a link (only available when no text is selected)
     */
    data object InsertLink : LinkAction()

    /**
     * Add or change the link url for the current selection, without supplying text.
     */
    data class SetLink(val currentUrl: String?) : LinkAction()

    /**
     * Change the link url and text for the current selection.
     */
    data class EditLink(val currentUrl: String?, val currentText: String?) : LinkAction()
}
