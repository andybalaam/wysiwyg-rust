package io.element.android.wysiwyg

import android.app.Application
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Context.CLIPBOARD_SERVICE
import android.os.Build
import android.os.Parcelable
import android.text.Selection
import android.text.Spannable
import android.text.SpannableStringBuilder
import android.util.AttributeSet
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import androidx.lifecycle.*
import com.google.android.material.textfield.TextInputEditText
import io.element.android.wysiwyg.inputhandlers.InterceptInputConnection
import io.element.android.wysiwyg.inputhandlers.models.EditorInputAction
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.inputhandlers.models.ReplaceTextResult
import io.element.android.wysiwyg.utils.*
import io.element.android.wysiwyg.utils.HtmlToSpansParser.FormattingSpans.removeFormattingSpans
import io.element.android.wysiwyg.viewmodel.EditorViewModel
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction
import uniffi.wysiwyg_composer.newComposerModel

class EditorEditText : TextInputEditText {

    private var inputConnection: InterceptInputConnection? = null
    private val viewModel: EditorViewModel by viewModel(viewModelInitializer = {
        val applicationContext = context.applicationContext as Application
        val resourcesProvider = AndroidResourcesProvider(applicationContext)
        val htmlConverter = AndroidHtmlConverter(
            provideHtmlToSpansParser = { html -> HtmlToSpansParser(resourcesProvider, html) },
        )
        val composer = if (!isInEditMode) newComposerModel() else null
        EditorViewModel(composer, htmlConverter)
    })

    private val spannableFactory = object : Spannable.Factory() {
        override fun newSpannable(source: CharSequence?): Spannable {
            // Try to reuse current source if possible to improve performance
            return source as? Spannable ?: SpannableStringBuilder(source)
        }
    }

    private var isInitialized = false

    constructor(context: Context) : super(context)

    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) :
            super(context, attrs, defStyleAttr)

    init {
        setSpannableFactory(spannableFactory)
        addHardwareKeyInterceptor()

        isInitialized = true
    }

    fun interface OnSelectionChangeListener {
        fun selectionChanged(start: Int, end: Int)
    }

    fun interface OnActionStatesChangedListener {
        fun actionStatesChanged(actionStates: Map<ComposerAction, ActionState>)
    }

    var selectionChangeListener: OnSelectionChangeListener? = null
    var actionStatesChangedListener: OnActionStatesChangedListener? = null
        set(value) {
            field = value

            viewModel.setActionStatesCallback { actionStates ->
                value?.actionStatesChanged(actionStates)
            }
        }

    /**
     * We'll do our own text restoration.
     */
    override fun getFreezesText(): Boolean {
        return false
    }

    override fun onRestoreInstanceState(state: Parcelable?) {
        val spannedText = viewModel.getCurrentFormattedText()
        editableText.replace(0, editableText.length, spannedText)

        super.onRestoreInstanceState(state)

        val start = Selection.getSelectionStart(editableText)
        val end = Selection.getSelectionEnd(editableText)
        viewModel.updateSelection(editableText, start, end)
    }

    override fun onSelectionChanged(selStart: Int, selEnd: Int) {
        super.onSelectionChanged(selStart, selEnd)
        if (this.isInitialized) {
            this.viewModel.updateSelection(editableText, selStart, selEnd)
        }
        selectionChangeListener?.selectionChanged(selStart, selEnd)
    }

    /**
     * We wrap the internal `com.android.internal.widget.EditableInputConnection` as it's not
     * public, but its internal behavior is needed to work properly with the EditText.
     */
    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        val baseInputConnection = requireNotNull(super.onCreateInputConnection(outAttrs))
        val inputConnection =
            InterceptInputConnection(baseInputConnection, this, viewModel)
        this.inputConnection = inputConnection
        return inputConnection
    }

    /**
     * Override cut & paste events so output is redirected to the [inputProcessor].
     */
    override fun onTextContextMenuItem(id: Int): Boolean {
        when (id) {
            android.R.id.cut -> {
                val clipboardManager =
                    context.getSystemService(CLIPBOARD_SERVICE) as ClipboardManager
                val clpData = ClipData.newPlainText(
                    "newText",
                    this.editableText.slice(this.selectionStart until this.selectionEnd)
                )
                clipboardManager.setPrimaryClip(clpData)

                val result = viewModel.processInput(
                    EditorInputAction.DeleteIn(
                        this.selectionStart,
                        this.selectionEnd
                    )
                )

                if (result != null) {
                    setTextFromComposerUpdate(result)
                    setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
                }

                return false
            }
            android.R.id.paste, android.R.id.pasteAsPlainText -> {
                val clipBoardManager =
                    context.getSystemService(CLIPBOARD_SERVICE) as ClipboardManager
                val copiedString = clipBoardManager.primaryClip?.getItemAt(0)?.text ?: return false
                val result = viewModel.processInput(EditorInputAction.ReplaceText(copiedString))

                if (result != null) {
                    setTextFromComposerUpdate(result)
                    setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
                }

                return false
            }
            else -> return super.onTextContextMenuItem(id)
        }
    }

    private fun addHardwareKeyInterceptor() {
        // This seems to be the only way to prevent EditText from automatically handling key strokes
        setOnKeyListener { _, keyCode, event ->
            println("Key Event: $event")
            if (keyCode == KeyEvent.KEYCODE_ENTER) {
                if (event.action == MotionEvent.ACTION_DOWN) {
                    inputConnection?.sendHardwareKeyboardInput(event)
                }
                true
            } else if (event.action != MotionEvent.ACTION_DOWN) {
                false
            } else if (event.isMovementKey()) {
                false
            } else if (event.metaState != 0 && event.unicodeChar == 0) {
                // Is a modifier key
                false
            } else if (event.isPrintableCharacter() ||
                keyCode == KeyEvent.KEYCODE_DEL ||
                keyCode == KeyEvent.KEYCODE_FORWARD_DEL) {
                // Consume printable characters
                inputConnection?.sendHardwareKeyboardInput(event)
                true
            } else {
                // Don't consume other key codes (HW back button, i.e.)
                false
            }
        }
    }

    override fun setText(text: CharSequence?, type: BufferType?) {
        val currentText = this.text
        val end = currentText?.length ?: 0
        // Although inputProcessor is assured to be not null, that's not the case while inflating.
        // We have to add this here to prevent some NullPointerExceptions from being thrown.
        if (!this.isInitialized) {
            return super.setText(text, type)
        }

        viewModel.updateSelection(editableText, 0, end)

        val result = viewModel.processInput(EditorInputAction.ReplaceText(text.toString()))
            ?: return super.setText(text, type)

        setTextFromComposerUpdate(result)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
    }

    override fun append(text: CharSequence?, start: Int, end: Int) {
        val result = viewModel.processInput(EditorInputAction.ReplaceText(text.toString()))
            ?: return super.append(text, start, end)

        setTextFromComposerUpdate(result)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
    }

    fun toggleInlineFormat(inlineFormat: InlineFormat): Boolean {
        val result = viewModel.processInput(EditorInputAction.ApplyInlineFormat(inlineFormat))
            ?: return false

        setTextFromComposerUpdate(result)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
        return true
    }

    fun undo() {
        val result = viewModel.processInput(EditorInputAction.Undo) ?: return

        setTextFromComposerUpdate(result)
        setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
    }

    fun redo() {
        val result = viewModel.processInput(EditorInputAction.Redo) ?: return

        setTextFromComposerUpdate(result)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    fun setLink(link: String) {
        val result = viewModel.processInput(EditorInputAction.SetLink(link)) ?: return

        setTextFromComposerUpdate(result)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    fun toggleList(ordered: Boolean) {
        val result = viewModel.processInput(EditorInputAction.ToggleList(ordered)) ?: return

        setTextFromComposerUpdate(result)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    fun setHtml(html: String) {
        val result = viewModel.processInput(EditorInputAction.ReplaceAllHtml(html)) ?: return

        setTextFromComposerUpdate(result)
        setSelectionFromComposerUpdate(result.selection.last)
    }

    fun getHtmlOutput(): String {
        return viewModel.getHtml()
    }

    /**
     * Get the text as plain text (without any HTML formatting).
     * Note that markdown is not currently supported.
     * TODO: Return markdown formatted plain text
     */
    fun getPlainText(): String = viewModel.getPlainText()

    /**
     * Set the text as plain text, ignoring HTML formatting.
     * Note that markdown is not currently supported.
     * TODO: Accept markdown formatted plain text
     */
    fun setPlainText(plainText: String) =
        setText(plainText)

    private fun setTextFromComposerUpdate(result: ReplaceTextResult) {
        beginBatchEdit()
        editableText.removeFormattingSpans()
        editableText.replace(0, editableText.length, result.text)
        endBatchEdit()
    }

    private fun setSelectionFromComposerUpdate(start: Int, end: Int = start) {
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditor(start, end, editableText)
        setSelection(newStart, newEnd)
    }
}

private fun KeyEvent.isMovementKey(): Boolean {
    val baseIsMovement = this.keyCode in KeyEvent.KEYCODE_DPAD_UP..KeyEvent.KEYCODE_DPAD_CENTER
    val api24IsMovement = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
        this.keyCode in KeyEvent.KEYCODE_DPAD_UP_LEFT..KeyEvent.KEYCODE_DPAD_DOWN_RIGHT
    } else false
    return baseIsMovement || api24IsMovement
}

private fun KeyEvent.isPrintableCharacter(): Boolean {
    return isPrintingKey || keyCode == KeyEvent.KEYCODE_SPACE
}
