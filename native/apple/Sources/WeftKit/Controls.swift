import SwiftUI

// The `edit` closure dispatches the event and returns the slot's value to
// restore when an edit was refused, or nil when accepted. Input
// delegates skip marked text so it is never rolled back mid-composition.

/// Intrinsic width of an `Input` (CSS inputs default to about 20 characters).
private let inputWidth: CGFloat = 200

#if os(macOS)
import AppKit

/// `Input`: an `NSTextField`.
public struct WeftInput: NSViewRepresentable {
    let text: String
    let placeholder: String?
    let edit: @MainActor (String) -> String?
    private var sizing = WeftSizing()

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
        sizing.nativeSize(proposal, ideal: CGSize(width: inputWidth, height: field.intrinsicContentSize.height))
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

#else
import UIKit

/// `Input`: a `UITextField`.
public struct WeftInput: UIViewRepresentable {
    let text: String
    let placeholder: String?
    let edit: @MainActor (String) -> String?
    private var sizing = WeftSizing()

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
        sizing.nativeSize(proposal, ideal: CGSize(width: inputWidth, height: field.intrinsicContentSize.height))
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

#endif

/// A binding to the value slot, rather than independent state in the control.
/// Refused edits invalidate the control even when the value slot did not change.
private struct WeftControl<Value, Content: View>: View {
    let value: Value
    let edit: @MainActor (Value) -> Value?
    let content: (Binding<Value>) -> Content
    @State private var rejection = 0

    var body: some View {
        _ = rejection
        return content(Binding(get: { value }, set: {
            if edit($0) != nil { rejection &+= 1 }
        }))
    }
}

/// A native checkbox on macOS and a square checkmark button on iOS.
@MainActor public func WeftCheckbox(checked: Bool, edit: @escaping @MainActor (Bool) -> Bool?) -> some View {
    #if os(macOS)
    WeftControl(value: checked, edit: edit) { binding in
        Toggle("", isOn: binding).labelsHidden().toggleStyle(.checkbox)
    }
    #else
    Button {
        _ = edit(!checked)
    } label: {
        Image(systemName: checked ? "checkmark.square.fill" : "square")
    }
    .accessibilityLabel("Checkbox")
    .accessibilityValue(checked ? "1" : "0")
    #endif
}

/// A native on/off switch.
@MainActor public func weftSwitch(checked: Bool, edit: @escaping @MainActor (Bool) -> Bool?) -> some View {
    WeftControl(value: checked, edit: edit) { binding in
        Toggle("", isOn: binding).labelsHidden().toggleStyle(.switch)
    }
}

/// A multiline text field with three lines reserved for its content.
public struct WeftTextArea: View {
    private let value: String
    private let placeholder: String
    private let edit: @MainActor (String) -> String?
    @State private var draft: String

    public init(value: String, placeholder: String, edit: @escaping @MainActor (String) -> String?) {
        self.value = value
        self.placeholder = placeholder
        self.edit = edit
        _draft = State(initialValue: value)
    }

    public var body: some View {
        TextField("", text: $draft, prompt: Text(placeholder), axis: .vertical)
            .lineLimit(3, reservesSpace: true)
            .textFieldStyle(.roundedBorder)
            .onChange(of: value) { _, value in draft = value }
            .onChange(of: draft) { _, proposed in
                // The vertical field keeps its edit buffer if only a computed binding
                // is invalidated. Set the draft back to the slot explicitly on refusal.
                if proposed != value, let restored = edit(proposed) {
                    draft = restored
                }
            }
    }
}

/// A native slider with an integral event payload.
@MainActor public func weftSlider(
    value: CGFloat, min: CGFloat, max: CGFloat, step: CGFloat, edit: @escaping @MainActor (Double) -> Double?
) -> some View {
    let lower = Double(min)
    let upper = Double(max)
    let increment = Double(step)
    let valid = lower.isFinite && upper.isFinite && lower < upper
    return WeftControl(value: valid ? Double(value) : 0, edit: { edit(roundedSliderValue($0)) }) { binding in
        Slider(value: binding,
               in: valid ? lower...upper : 0...1,
               step: increment.isFinite && increment > 0 ? increment : 1)
            .disabled(!valid)
    }
}

func roundedSliderValue(_ value: Double) -> Double {
    value.rounded()
}

/// A menu labeled by the selected option, or the placeholder if none is selected.
@MainActor public func weftPicker(
    selected: CGFloat, options: [String], placeholder: String, edit: @escaping @MainActor (Double) -> Double?
) -> some View {
    Menu {
        ForEach(options.indices, id: \.self) { index in
            Button(options[index]) { _ = edit(Double(index)) }
        }
    } label: {
        Text(pickerLabel(selected: selected, options: options, placeholder: placeholder))
    }
}

func pickerLabel(selected: CGFloat, options: [String], placeholder: String) -> String {
    guard let index = Int(exactly: selected), options.indices.contains(index) else { return placeholder }
    return options[index]
}

/// A native indeterminate progress indicator.
@MainActor @ViewBuilder public func weftSpinner(color: WeftColor?) -> some View {
    #if os(macOS)
    if let color {
        // AppKit's busy indicator ignores tint and SwiftUI color effects.
        let tint = color.resolved
        TimelineView(.animation) { timeline in
            let phase = timeline.date.timeIntervalSinceReferenceDate.truncatingRemainder(dividingBy: 1)
            Circle().trim(from: 0, to: 0.75)
                .stroke(tint, style: StrokeStyle(lineWidth: 2, lineCap: .round))
                .rotationEffect(.degrees(phase * 360))
                .frame(width: 18, height: 18)
                .padding(1)
        }
        .accessibilityRepresentation { ProgressView() }
    } else {
        ProgressView()
    }
    #else
    ProgressView().tint(color?.resolved)
    #endif
}

/// A native progress bar displaying a fraction from zero to one.
@MainActor public func weftProgress(value: CGFloat, color: WeftColor?) -> some View {
    ProgressView(value: Double(value)).tint(color?.resolved)
}

/// A horizontal or vertical native separator.
@MainActor @ViewBuilder public func weftDivider(vertical: Bool, color: WeftColor?) -> some View {
    let line = Divider().overlay { color?.resolved }
    if vertical {
        HStack(spacing: 0) { line }
    } else {
        VStack(spacing: 0) { line }
    }
}
