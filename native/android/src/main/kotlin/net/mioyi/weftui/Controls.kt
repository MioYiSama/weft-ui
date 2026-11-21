package net.mioyi.weftui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DividerDefaults
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuAnchorType
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.ProgressIndicatorDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import kotlin.math.floor

// Synchronous edits go through the runtime; only the value slot drives the
// committed value shown by the control. IME composition is kept local until commit.

/** Intrinsic width of an `Input` (CSS inputs default to about 20 characters). */
private val inputWidth = 200.dp

/**
 * `Input`: an outlined text field showing [value], the value slot. [edit]
 * reports an edit and returns the value to show again when the app refused
 * it, or null when it was accepted.
 */
@Composable
fun WeftInput(
    value: String,
    placeholder: String? = null,
    disabled: Boolean = false,
    modifier: Modifier = Modifier,
    type: WeftInputType = WeftInputType.TEXT,
    returnKey: WeftReturnKey? = null,
    maxLength: Float = -1f,
    focused: Boolean = false,
    onSubmit: (() -> Unit)? = null,
    onFocusChange: ((Boolean) -> Unit)? = null,
    edit: (String) -> String?,
) {
    val keyboardType = when (type) {
        WeftInputType.TEXT -> KeyboardType.Text
        WeftInputType.PASSWORD -> KeyboardType.Password
        WeftInputType.NUMBER -> KeyboardType.Number
        WeftInputType.EMAIL -> KeyboardType.Email
        WeftInputType.PHONE -> KeyboardType.Phone
    }
    val imeAction = when (returnKey) {
        WeftReturnKey.DONE -> ImeAction.Done
        WeftReturnKey.GO -> ImeAction.Go
        WeftReturnKey.NEXT -> ImeAction.Next
        WeftReturnKey.SEARCH -> ImeAction.Search
        WeftReturnKey.SEND -> ImeAction.Send
        // Compose doesn't dispatch a keyboard callback for ImeAction.Default.
        null -> if (onSubmit == null) ImeAction.Default else ImeAction.Done
    }
    val keyboardOptions = remember(keyboardType, imeAction) {
        KeyboardOptions(keyboardType = keyboardType, imeAction = imeAction)
    }
    val keyboardActions = remember(onSubmit) {
        if (onSubmit == null) KeyboardActions.Default else KeyboardActions(onAny = { onSubmit() })
    }
    val visualTransformation = if (type == WeftInputType.PASSWORD) {
        remember { PasswordVisualTransformation() }
    } else {
        VisualTransformation.None
    }
    WeftEditableText(
        value, placeholder, disabled, modifier, lines = 1, maxLength, focused, onFocusChange,
        keyboardOptions, keyboardActions, visualTransformation, edit,
    )
}

/** `TextArea`: a three-line Material field with the same controlled edits as [WeftInput]. */
@Composable
fun WeftTextArea(
    value: String,
    placeholder: String? = null,
    disabled: Boolean = false,
    modifier: Modifier = Modifier,
    maxLength: Float = -1f,
    focused: Boolean = false,
    onFocusChange: ((Boolean) -> Unit)? = null,
    edit: (String) -> String?,
) = WeftEditableText(value, placeholder, disabled, modifier, lines = 3, maxLength, focused, onFocusChange, edit = edit)

@Composable
private fun WeftEditableText(
    value: String,
    placeholder: String?,
    disabled: Boolean,
    modifier: Modifier,
    lines: Int,
    maxLength: Float,
    focused: Boolean,
    onFocusChange: ((Boolean) -> Unit)?,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    keyboardActions: KeyboardActions = KeyboardActions.Default,
    visualTransformation: VisualTransformation = VisualTransformation.None,
    edit: (String) -> String?,
) {
    var field by remember { mutableStateOf(TextFieldValue(value, TextRange(value.length))) }
    // A value the app wrote replaces the field's text, unless an input method is composing.
    val shown = if (field.text != value && field.composition == null) {
        TextFieldValue(value, TextRange(value.length))
    } else {
        field
    }
    val onValueChange: (TextFieldValue) -> Unit = { new ->
        field = if (new.composition != null || new.text == value) {
            // Composing text and moves of the cursor are local to the field.
            new
        } else {
            val limited = new.text.limitUtf16(maxLength.toInt())
            val restored = if (limited == value) null else edit(limited)
            when (restored) {
                null -> if (limited.length == new.text.length) new else TextFieldValue(limited, new.selection)
                shown.text -> shown
                else -> TextFieldValue(restored, TextRange(restored.length))
            }
        }
    }
    val requester = remember { FocusRequester() }
    val focus = remember { TextFocus() }
    val focusManager = LocalFocusManager.current
    // Keep the request separate from actual focus, which the user can move independently.
    LaunchedEffect(focused) {
        if (focus.requested != focused) {
            focus.requested = focused
            if (focused) requester.requestFocus() else if (focus.active) focusManager.clearFocus()
        }
    }
    val fieldModifier = modifier.width(inputWidth).focusRequester(requester).onFocusChanged {
        if (focus.active != it.isFocused) {
            focus.active = it.isFocused
            onFocusChange?.invoke(it.isFocused)
        }
    }
    val hint: (@Composable () -> Unit)? = placeholder?.let { { Text(it) } }
    if (lines == 1) {
        OutlinedTextField(
            value = shown, onValueChange = onValueChange, enabled = !disabled,
            singleLine = true, placeholder = hint, modifier = fieldModifier,
            keyboardOptions = keyboardOptions, keyboardActions = keyboardActions,
            visualTransformation = visualTransformation,
        )
    } else {
        TextField(
            value = shown, onValueChange = onValueChange, enabled = !disabled,
            minLines = lines, maxLines = lines, placeholder = hint, modifier = fieldModifier,
        )
    }
}

private class TextFocus {
    var requested = false
    var active = false
}

/** Limits an edit to UTF-16 code units without splitting a surrogate pair. */
internal fun String.limitUtf16(limit: Int): String {
    if (limit < 0 || length <= limit) return this
    val splitsPair = limit > 0 && this[limit - 1].isHighSurrogate() && this[limit].isLowSurrogate()
    return substring(0, if (splitsPair) limit - 1 else limit)
}

/** `Checkbox`: a Material checkbox showing [checked], the value slot; [edit] reports a toggle. */
@Composable
fun WeftCheckbox(checked: Boolean, disabled: Boolean = false, modifier: Modifier = Modifier, edit: (Boolean) -> Unit) {
    Checkbox(checked = checked, onCheckedChange = edit, enabled = !disabled, modifier = modifier)
}

/** `Switch`: a Material switch whose checked state follows the value slot. */
@Composable
fun WeftSwitch(checked: Boolean, disabled: Boolean = false, modifier: Modifier = Modifier, edit: (Boolean) -> Unit) {
    Switch(checked = checked, onCheckedChange = edit, enabled = !disabled, modifier = modifier)
}

/** `Slider`: a Material slider that proposes integer values on the specified step grid. */
@Composable
fun WeftSlider(
    value: Float,
    min: Float = 0f,
    max: Float = 100f,
    step: Float = 1f,
    disabled: Boolean = false,
    modifier: Modifier = Modifier,
    edit: (Float) -> Unit,
) {
    val upper = max.coerceAtLeast(min)
    val increment = step.coerceAtLeast(1f)
    val intervals = (upper.toDouble() - min) / increment
    // Compose divides discrete tracks evenly and allocates a tick for each step.
    // Use a continuous track for uneven or very dense grids, snapping proposals ourselves.
    val steps = if (intervals in 2.0..1000.0 && intervals == floor(intervals)) intervals.toInt() - 1 else 0
    val shown = snapSliderValue(value, min, upper, increment)
    Slider(
        value = shown,
        onValueChange = { proposed ->
            val snapped = snapSliderValue(proposed, min, upper, increment)
            if (snapped != shown) edit(snapped)
        },
        modifier = modifier,
        enabled = !disabled && upper > min,
        valueRange = min..upper,
        steps = steps,
    )
}

/** Snaps to the nearest in-range point on the step grid, breaking ties upwards. */
internal fun snapSliderValue(value: Float, min: Float, max: Float, step: Float): Float {
    val offset = (value.toDouble() - min) / step
    val last = floor((max.toDouble() - min) / step)
    val tick = floor(offset + 0.5).coerceIn(0.0, last)
    return (min + tick * step).toFloat()
}

/** `Picker`: a read-only Material field showing the selected option or its placeholder. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun WeftPicker(
    selected: Float,
    options: List<String>,
    placeholder: String? = null,
    disabled: Boolean = false,
    modifier: Modifier = Modifier,
    edit: (Int) -> Unit,
) {
    var open by remember { mutableStateOf(false) }
    val expanded = open && !disabled
    val index = selected.toInt()
    val label = options.getOrNull(index)
    val state = remember(label) { TextFieldState(label.orEmpty()) }
    val textStyle = MaterialTheme.typography.bodyMedium
    ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { open = it }, modifier = modifier) {
        TextField(
            state = state,
            readOnly = true,
            enabled = !disabled,
            lineLimits = TextFieldLineLimits.SingleLine,
            textStyle = textStyle,
            placeholder = placeholder?.takeIf { label == null }?.let { { Text(it, style = textStyle) } },
            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded) },
            colors = ExposedDropdownMenuDefaults.textFieldColors(),
            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 16.dp),
            modifier = Modifier.menuAnchor(ExposedDropdownMenuAnchorType.PrimaryNotEditable, enabled = !disabled),
        )
        ExposedDropdownMenu(expanded = expanded, onDismissRequest = { open = false }) {
            options.forEachIndexed { i, option ->
                DropdownMenuItem(text = { Text(option) }, onClick = {
                    open = false
                    if (i != index) edit(i)
                })
            }
        }
    }
}

/** `Spinner`: a 20 dp indeterminate circular progress indicator. */
@Composable
fun WeftSpinner(color: WeftColor? = null, modifier: Modifier = Modifier) {
    CircularProgressIndicator(
        modifier = modifier.size(20.dp),
        color = color?.resolve() ?: ProgressIndicatorDefaults.circularColor,
        strokeWidth = 2.dp,
    )
}

/** `Progress`: a Material progress bar showing a completed fraction. */
@Composable
fun WeftProgress(value: Float, color: WeftColor? = null, modifier: Modifier = Modifier) {
    LinearProgressIndicator(progress = { value }, modifier = modifier, color = color?.resolve() ?: Color(0xff007aff))
}

/** `Divider`: a Material separator oriented along the requested axis. */
@Composable
fun WeftDivider(vertical: Boolean = false, color: WeftColor? = null, modifier: Modifier = Modifier) {
    val lineColor = color?.resolve() ?: DividerDefaults.color
    if (vertical) VerticalDivider(modifier = modifier, color = lineColor)
    else HorizontalDivider(modifier = modifier, color = lineColor)
}
