package net.mioyi.weftui

import androidx.compose.foundation.layout.width
import androidx.compose.material3.Checkbox
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp

// Controlled `Input` and `Checkbox`. Every user edit is dispatched
// synchronously through `edit`; the control then shows the value slot, so a
// refused edit is rolled back on the spot. Text being composed by an input
// method is not reported, so it is never rolled back mid-composition; the
// committed text is.

/** Width of an `Input`, which has no width property (CSS inputs default to about 20 characters). */
private val inputWidth = 200.dp

/**
 * `Input`: an outlined text field showing [text], the value slot. [edit]
 * reports an edit and returns the value to show again when the app refused
 * it, or null when it was accepted.
 */
@Composable
fun WeftInput(text: String, placeholder: String? = null, disabled: Boolean = false, edit: (String) -> String?) {
    var field by remember { mutableStateOf(TextFieldValue(text, TextRange(text.length))) }
    // A value the app wrote replaces the field's text, unless an input method is composing.
    val shown = if (field.text != text && field.composition == null) {
        TextFieldValue(text, TextRange(text.length))
    } else {
        field
    }
    OutlinedTextField(
        value = shown,
        onValueChange = { new ->
            field = when {
                // Composing text and moves of the cursor are local to the field.
                new.composition != null || new.text == text -> new
                else -> when (val restored = edit(new.text)) {
                    null -> new
                    shown.text -> shown
                    else -> TextFieldValue(restored, TextRange(restored.length))
                }
            }
        },
        enabled = !disabled,
        singleLine = true,
        placeholder = placeholder?.let { { Text(it) } },
        modifier = Modifier.width(inputWidth),
    )
}

/** `Checkbox`: a Material checkbox showing [checked], the value slot; [edit] reports a toggle. */
@Composable
fun WeftCheckbox(checked: Boolean, disabled: Boolean = false, edit: (Boolean) -> Unit) {
    Checkbox(checked = checked, onCheckedChange = edit, enabled = !disabled)
}
