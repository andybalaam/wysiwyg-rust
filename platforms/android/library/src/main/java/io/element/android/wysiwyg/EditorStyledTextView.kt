package io.element.android.wysiwyg

import android.content.Context
import android.graphics.Canvas
import android.text.Spanned
import android.util.AttributeSet
import android.view.MotionEvent
import androidx.appcompat.widget.AppCompatTextView
import androidx.core.graphics.withTranslation
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.internal.view.EditorEditTextAttributeReader
import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.view.StyleConfig
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelper
import io.element.android.wysiwyg.view.inlinebg.SpanBackgroundHelperFactory
import io.element.android.wysiwyg.view.spans.CustomMentionSpan
import io.element.android.wysiwyg.view.spans.LinkSpan
import io.element.android.wysiwyg.view.spans.PillSpan
import io.element.android.wysiwyg.view.spans.ReuseSourceSpannableFactory
import uniffi.wysiwyg_composer.MentionDetector
import uniffi.wysiwyg_composer.newMentionDetector

/**
 * This TextView can display all spans used by the editor.
 */
open class EditorStyledTextView : AppCompatTextView {

    private var mentionDetector: MentionDetector? = null

    private lateinit var inlineCodeBgHelper: SpanBackgroundHelper
    private lateinit var codeBlockBgHelper: SpanBackgroundHelper

    /**
     * The [StyleConfig] used to style the spans generated from the HTML in this TextView.
     */
    lateinit var styleConfig: StyleConfig
        private set

    private var isInit = false

    private val spannableFactory = ReuseSourceSpannableFactory()

    private var mentionDisplayHandler: MentionDisplayHandler? = null
    private var htmlConverter: HtmlConverter? = null

    var onLinkClickedListener: ((String) -> Unit)? = null

    init {
        setSpannableFactory(spannableFactory)
        isInit = true
    }

    constructor(context: Context) : super(context, null)

    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs) {
        styleConfig = EditorEditTextAttributeReader(context, attrs).styleConfig
    }

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(
        context, attrs, defStyleAttr
    ) {
        styleConfig = EditorEditTextAttributeReader(context, attrs).styleConfig
    }

    override fun setText(text: CharSequence?, type: BufferType?) {
        super.setText(text, type)
        // setText may be called during initialisation when we're not yet
        // ready to load the background helpers
        if (!isInit) return
        inlineCodeBgHelper.clearCachedPositions()
        codeBlockBgHelper.clearCachedPositions()
    }

    /**
     * Sets up the styling used to translate HTML to Spanned text.
     * @param styleConfig The styles to use for the generated spans.
     * @param mentionDisplayHandler Used to decide how to display any mentions found in the HTML text.
     */
    fun updateStyle(styleConfig: StyleConfig, mentionDisplayHandler: MentionDisplayHandler?) {
        this.styleConfig = styleConfig
        this.mentionDisplayHandler = mentionDisplayHandler

        inlineCodeBgHelper = SpanBackgroundHelperFactory.createInlineCodeBackgroundHelper(styleConfig.inlineCode)
        codeBlockBgHelper = SpanBackgroundHelperFactory.createCodeBlockBackgroundHelper(styleConfig.codeBlock)

        htmlConverter = createHtmlConverter(styleConfig, mentionDisplayHandler)
    }

    /**
     * Set the text of the TextView with HTML formatting.
     * @param htmlText The text to display, with HTML formatting.
     * Consider using [HtmlConverter.fromHtmlToSpans] and [setText] instead.
     */
    fun setHtml(htmlText: String) {
        if (!isInit) return
        htmlConverter?.fromHtmlToSpans(htmlText)?.let { setText(it, BufferType.SPANNABLE) }
    }

    override fun onDraw(canvas: Canvas) {
        // need to draw bg first so that text can be on top during super.onDraw()
        if (text is Spanned && layout != null && isInit) {
            canvas.withTranslation(totalPaddingLeft.toFloat(), totalPaddingTop.toFloat()) {
                codeBlockBgHelper.draw(canvas, text as Spanned, layout)
                inlineCodeBgHelper.draw(canvas, text as Spanned, layout)
            }
        }
        super.onDraw(canvas)
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()

        mentionDetector = if (isInEditMode) null else newMentionDetector()

        updateStyle(styleConfig, mentionDisplayHandler)
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()

        mentionDetector?.destroy()
        mentionDetector = null
    }

    private fun createHtmlConverter(styleConfig: StyleConfig, mentionDisplayHandler: MentionDisplayHandler?): HtmlConverter {
        return HtmlConverter.Factory.create(
            context = context,
            styleConfig = styleConfig,
            mentionDisplayHandler = mentionDisplayHandler,
            isMention = mentionDetector?.let { detector ->
                { _, url ->
                    detector.isMention(url)
                }
            }
        )
    }

    override fun onTouchEvent(event: MotionEvent?): Boolean {
        return when (event?.action) {
            MotionEvent.ACTION_DOWN -> {
                // Allow ACTION_UP to be called
                super.onTouchEvent(event)
                true
            }
            MotionEvent.ACTION_UP -> {
                // Find selection matching the pointer coordinates
                val offset = getOffsetForPosition(event.x, event.y)
                // Look for clickable spans in that position
                val spans = (text as? Spanned)?.getSpans(offset, offset, Any::class.java) ?: return false
                val linkSpans = spans.filterIsInstance<LinkSpan>()
                val pillSpans = spans.filterIsInstance<PillSpan>()
                val customMentionSpans = spans.filterIsInstance<CustomMentionSpan>()
                // Notify the link has been clicked
                for (span in linkSpans) {
                    if (span.url == null) continue
                    onLinkClickedListener?.invoke(span.url)
                }
                for (span in pillSpans) {
                    if (span.url == null) continue
                    onLinkClickedListener?.invoke(span.url)
                }
                for (span in customMentionSpans) {
                    if (span.url == null) continue
                    onLinkClickedListener?.invoke(span.url)
                }
                true
            }
            else -> super.onTouchEvent(event)
        }
    }
}
