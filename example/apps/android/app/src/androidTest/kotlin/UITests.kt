package net.mioyi.weftui.app

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsOff
import androidx.compose.ui.test.assertIsOn
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.isToggleable
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.text.AnnotatedString
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

/**
 * The example's click flow over every built-in element; run with
 * `pnpm --filter example test:android` on a device or emulator.
 */
@RunWith(AndroidJUnit4::class)
class UITests {
    @get:Rule
    val compose = createAndroidComposeRule<MainActivity>()

    @Test
    fun clickFlow() {
        // Button and `if`.
        compose.onNodeWithText("Click me").performClick()
        compose.onNodeWithText("You clicked!").assertExists()

        // A controlled input: drafts longer than 12 characters are refused and rolled back.
        val field = compose.onNode(hasSetTextAction())
        val add = compose.onNodeWithText("Add")
        add.assertIsNotEnabled()
        type(field, "Buy milk and bread")
        field.assert(hasEditableText("Buy milk and"))
        add.assertIsEnabled()

        // `for` rows.
        add.performClick()
        compose.onNodeWithText("Buy milk and").assertExists()
        field.assert(hasEditableText(""))
        type(field, "Eggs")
        add.performClick()
        compose.onNodeWithText("2 todos").assertExists()

        // Checkboxes in the rows.
        compose.onAllNodes(isToggleable())[0].assertIsOff().performClick()
        compose.onAllNodes(isToggleable())[0].assertIsOn()
        compose.onAllNodes(isToggleable())[1].assertIsOff()

        // Removing the first row keeps the second.
        compose.onAllNodesWithText("Remove")[0].performClick()
        compose.onNodeWithText("1 todos").assertExists()
        compose.onNodeWithText("Buy milk and").assertDoesNotExist()
        compose.onNodeWithText("Eggs").assertExists()
        compose.onAllNodes(isToggleable())[0].assertIsOff()
    }

    /** Types [text] one character at a time, each an edit at the cursor, as a keyboard does. */
    private fun type(field: SemanticsNodeInteraction, text: String) {
        for (c in text) field.performTextInput(c.toString())
    }

    private fun hasEditableText(text: String) =
        SemanticsMatcher.expectValue(SemanticsProperties.EditableText, AnnotatedString(text))
}
