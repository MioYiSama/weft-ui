import SwiftUI

// Controlled `Input` and `Checkbox`. The native control reports every user
// edit through `edit`, which dispatches it synchronously and returns the
// value to show again when the app refused it (`nil` when accepted). Text
// being composed by an input method is not reported, so it is never rolled
// back mid-composition; the committed text is.

/// Width of an `Input`, which has no width property (CSS inputs default to
/// about 20 characters).
private let inputWidth: CGFloat = 200

#if os(macOS)
import AppKit

/// `Input`: an `NSTextField`.
public struct WeftInput: NSViewRepresentable {
    let text: String
    let placeholder: String?
    let edit: @MainActor (String) -> String?

    public init(text: String, placeholder: String?, edit: @escaping @MainActor (String) -> String?) {
        self.text = text
        self.placeholder = placeholder
        self.edit = edit
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    public func makeNSView(context: Context) -> NSTextField {
        let field = NSTextField(string: text)
        field.delegate = context.coordinator
        field.usesSingleLineMode = true
        field.cell?.isScrollable = true
        field.cell?.wraps = false
        return field
    }

    public func updateNSView(_ field: NSTextField, context: Context) {
        context.coordinator.input = self
        field.placeholderString = placeholder
        field.isEnabled = context.environment.isEnabled
        if field.stringValue != text, !Self.composing(field) {
            field.stringValue = text
        }
    }

    public func sizeThatFits(_ proposal: ProposedViewSize, nsView field: NSTextField, context: Context) -> CGSize? {
        CGSize(width: inputWidth, height: field.intrinsicContentSize.height)
    }

    static func composing(_ field: NSTextField) -> Bool {
        (field.currentEditor() as? NSTextView)?.hasMarkedText() ?? false
    }

    @MainActor
    public final class Coordinator: NSObject, NSTextFieldDelegate {
        var input: WeftInput

        init(_ input: WeftInput) {
            self.input = input
        }

        public func controlTextDidChange(_ notification: Notification) {
            guard let field = notification.object as? NSTextField, !WeftInput.composing(field) else { return }
            if let restored = input.edit(field.stringValue) {
                field.stringValue = restored
            }
        }
    }
}

/// `Checkbox`: an `NSButton` checkbox.
public struct WeftCheckbox: NSViewRepresentable {
    let checked: Bool
    let edit: @MainActor (Bool) -> Bool?

    public init(checked: Bool, edit: @escaping @MainActor (Bool) -> Bool?) {
        self.checked = checked
        self.edit = edit
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    public func makeNSView(context: Context) -> NSButton {
        let button = NSButton(checkboxWithTitle: "", target: context.coordinator, action: #selector(Coordinator.toggled(_:)))
        button.state = checked ? .on : .off
        return button
    }

    public func updateNSView(_ button: NSButton, context: Context) {
        context.coordinator.checkbox = self
        button.isEnabled = context.environment.isEnabled
        button.state = checked ? .on : .off
    }

    @MainActor
    public final class Coordinator: NSObject {
        var checkbox: WeftCheckbox

        init(_ checkbox: WeftCheckbox) {
            self.checkbox = checkbox
        }

        @objc func toggled(_ button: NSButton) {
            if let restored = checkbox.edit(button.state == .on) {
                button.state = restored ? .on : .off
            }
        }
    }
}

#else
import UIKit

/// `Input`: a `UITextField`.
public struct WeftInput: UIViewRepresentable {
    let text: String
    let placeholder: String?
    let edit: @MainActor (String) -> String?

    public init(text: String, placeholder: String?, edit: @escaping @MainActor (String) -> String?) {
        self.text = text
        self.placeholder = placeholder
        self.edit = edit
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    public func makeUIView(context: Context) -> UITextField {
        let field = UITextField()
        field.text = text
        field.borderStyle = .roundedRect
        field.addTarget(context.coordinator, action: #selector(Coordinator.changed(_:)), for: .editingChanged)
        return field
    }

    public func updateUIView(_ field: UITextField, context: Context) {
        context.coordinator.input = self
        field.placeholder = placeholder
        field.isEnabled = context.environment.isEnabled
        if field.text != text, field.markedTextRange == nil {
            field.text = text
        }
    }

    public func sizeThatFits(_ proposal: ProposedViewSize, uiView field: UITextField, context: Context) -> CGSize? {
        CGSize(width: inputWidth, height: field.intrinsicContentSize.height)
    }

    @MainActor
    public final class Coordinator: NSObject {
        var input: WeftInput

        init(_ input: WeftInput) {
            self.input = input
        }

        @objc func changed(_ field: UITextField) {
            guard field.markedTextRange == nil else { return }
            if let restored = input.edit(field.text ?? "") {
                field.text = restored
            }
        }
    }
}

/// `Checkbox`: a `UISwitch`, the iOS control for a boolean.
public struct WeftCheckbox: UIViewRepresentable {
    let checked: Bool
    let edit: @MainActor (Bool) -> Bool?

    public init(checked: Bool, edit: @escaping @MainActor (Bool) -> Bool?) {
        self.checked = checked
        self.edit = edit
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }

    public func makeUIView(context: Context) -> UISwitch {
        let toggle = UISwitch()
        toggle.isOn = checked
        toggle.addTarget(context.coordinator, action: #selector(Coordinator.toggled(_:)), for: .valueChanged)
        return toggle
    }

    public func updateUIView(_ toggle: UISwitch, context: Context) {
        context.coordinator.checkbox = self
        toggle.isEnabled = context.environment.isEnabled
        if toggle.isOn != checked {
            toggle.setOn(checked, animated: true)
        }
    }

    @MainActor
    public final class Coordinator: NSObject {
        var checkbox: WeftCheckbox

        init(_ checkbox: WeftCheckbox) {
            self.checkbox = checkbox
        }

        @objc func toggled(_ toggle: UISwitch) {
            if let restored = checkbox.edit(toggle.isOn) {
                toggle.setOn(restored, animated: true)
            }
        }
    }
}
#endif
