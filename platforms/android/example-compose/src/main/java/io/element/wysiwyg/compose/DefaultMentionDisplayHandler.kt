package io.element.wysiwyg.compose

import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay

class DefaultMentionDisplayHandler : MentionDisplayHandler {

    override fun resolveMentionDisplay(
        text: String, url: String
    ): TextDisplay {
        return TextDisplay.Pill
    }

    override fun resolveAtRoomMentionDisplay(): TextDisplay {
        return TextDisplay.Pill
    }
}
