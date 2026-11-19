using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.UI;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Weft.WinUI.Protocol;

namespace Weft.WinUI;

/// <summary>
/// One Weft app running in the native kernel: owns the kernel runtime and the
/// scope tree its ops build.
/// </summary>
/// <remarks>
/// The kernel runs on the UI thread: <c>commit</c> applies each batch of ops
/// to the scopes as it arrives, timers run on the thread's dispatcher queue,
/// and platform calls complete on it. The session lives as long as the app;
/// the kernel's callbacks keep it alive.
/// </remarks>
public sealed class WeftSession
{
    /// <summary>Creates the scope of template <paramref name="template"/> for a <c>create</c> op.</summary>
    public delegate WeftScope? Factory(uint template, uint id, WeftSession session);

    private readonly Factory make;
    private readonly Dictionary<uint, WeftScope> scopes = [];
    private readonly Panel root;
    private readonly DispatcherQueue queue;
    private readonly DispatcherQueueTimer timer;
    /// <summary>When the timer fires, in <see cref="Environment.TickCount64"/> milliseconds.</summary>
    private long due;
    /// <summary>Whether a batch of ops is being applied; the kernel is not reentrant.</summary>
    private bool committing;
    /// <summary>The ops of the batch being applied, reused across commits.</summary>
    private readonly List<Op> batch = [];
    private bool failed;
    /// <summary>The last error the kernel logged: why <c>weft_runtime_new</c> failed.</summary>
    private string? lastError;
    private readonly nint runtime;
    /// <summary>The bytecode, 16-byte aligned, alive as long as the runtime.</summary>
    private readonly unsafe byte* bytecode;

    /// <summary>
    /// Starts the app compiled to <c>weft/bundle.hbc</c> next to the executable,
    /// whose views implement the layout with hash <paramref name="layoutHash"/>,
    /// and returns its root view: a vertically scrolling page whose content
    /// starts at the top left corner, or why the app failed.
    /// </summary>
    public static UIElement Start(ulong layoutHash, Factory make)
    {
        var page = new StackPanel { HorizontalAlignment = HorizontalAlignment.Left, VerticalAlignment = VerticalAlignment.Top };
        var view = new ScrollViewer { Content = page };
        if (Application.Current?.Resources.TryGetValue("ApplicationPageBackgroundThemeBrush", out var brush) == true)
        {
            view.Background = (Brush)brush;
        }
        _ = new WeftSession(Path.Combine(AppContext.BaseDirectory, "weft", "bundle.hbc"), layoutHash, make, page);
        return view;
    }

    private unsafe WeftSession(string bundle, ulong layoutHash, Factory make, Panel root)
    {
        this.make = make;
        this.root = root;
        queue = DispatcherQueue.GetForCurrentThread()
            ?? throw new InvalidOperationException("weft: the session must start on a UI thread");
        timer = queue.CreateTimer();
        timer.IsRepeating = false;
        timer.Tick += (_, _) =>
        {
            timer.Stop();
            if (runtime != 0) Native.RuntimeTick(runtime);
        };
        scopes[0] = new WeftRootScope(this, root);

        byte[] data;
        try
        {
            data = File.ReadAllBytes(bundle);
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException)
        {
            Fail($"weft: cannot read the bundle {bundle}: {error.Message}");
            return;
        }
        // The kernel needs 4-byte aligned bytecode that outlives the runtime.
        bytecode = (byte*)NativeMemory.AlignedAlloc((nuint)Math.Max(data.Length, 1), 16);
        data.CopyTo(new Span<byte>(bytecode, data.Length));

        var version = Encoding.UTF8.GetBytes(Environment.OSVersion.Version + "\0");
        fixed (byte* versionBytes = version)
        {
            var host = new Native.Host
            {
                // The runtime calls back for as long as the app runs, so the handle is never freed.
                Ctx = GCHandle.ToIntPtr(GCHandle.Alloc(this)),
                Commit = &OnCommit,
                Schedule = &OnSchedule,
                Supports = &OnSupports,
                Call = &OnCall,
                Log = &OnLog,
                Version = versionBytes,
            };
            runtime = Native.RuntimeNew(&host, bytecode, (nuint)data.Length, layoutHash);
        }
        if (runtime == 0) Fail(lastError ?? "weft: the app failed to start");
    }

    /// <summary>Reports event slot <paramref name="slot"/> of <paramref name="scope"/>; see <see cref="WeftScope.Dispatch"/>.</summary>
    internal unsafe bool Dispatch(uint scope, uint slot, WeftValue? payload)
    {
        if (committing)
        {
            Debug.Fail("weft: an event was dispatched while ops were applied");
            return true;
        }
        if (runtime == 0) return true;
        var bytes = new List<byte>();
        payload?.Encode(bytes);
        fixed (byte* data = CollectionsMarshal.AsSpan(bytes))
        {
            return Native.RuntimeDispatch(runtime, scope, slot, data, (nuint)bytes.Count) != 0;
        }
    }

    private void Commit(ReadOnlySpan<byte> bytes)
    {
        if (failed) return;
        committing = true;
        try
        {
            OpReader.ReadAll(bytes, batch);
            foreach (ref readonly var op in CollectionsMarshal.AsSpan(batch)) Apply(op);
        }
        catch (Exception error)
        {
            Fail($"weft: cannot apply ops ({error.Message})");
        }
        finally
        {
            batch.Clear();
            committing = false;
        }
    }

    private void Apply(in Op op)
    {
        switch (op.Code)
        {
            case OpCode.Create:
                scopes[op.Scope] = make(op.Arg, op.Scope, this)
                    ?? throw new OpException($"the views have no template {op.Arg}");
                break;
            case OpCode.Set:
                if (scopes.TryGetValue(op.Scope, out var scope)) scope.Apply((int)op.Arg, op.Value);
                break;
            case OpCode.Branch:
                Mount(op.Scope, op.Arg, op.Child == 0 ? [] : [op.Child]);
                break;
            case OpCode.Attach:
                Mount(op.Scope, op.Arg, [op.Child]);
                break;
            case OpCode.Rows:
                Mount(op.Scope, op.Arg, op.Rows!);
                break;
        }
    }

    /// <summary>
    /// Mounts the scopes <paramref name="ids"/> at <paramref name="index"/> of
    /// <paramref name="parent"/>; children no longer mounted there are
    /// destroyed with their descendants.
    /// </summary>
    private void Mount(uint parent, uint index, uint[] ids)
    {
        if (!scopes.TryGetValue(parent, out var scope)) return;
        if (index >= scope.Points.Length) throw new OpException($"scope {parent} has no mount point {index}");
        var point = scope.Points[(int)index];
        var kept = new HashSet<uint>(ids);
        foreach (var old in point.Children)
        {
            if (!kept.Contains(old.Id)) Destroy(old);
        }
        var children = new List<WeftScope>(ids.Length);
        foreach (var id in ids)
        {
            if (scopes.TryGetValue(id, out var child)) children.Add(child);
        }
        point.Mount(children);
    }

    private void Destroy(WeftScope scope)
    {
        scopes.Remove(scope.Id);
        foreach (var point in scope.Points)
        {
            foreach (var child in point.Children) Destroy(child);
        }
    }

    private void Schedule(ulong delay)
    {
        var ms = (long)Math.Min(delay, int.MaxValue);
        var at = Environment.TickCount64 + ms;
        if (timer.IsRunning && at >= due) return;
        due = at;
        timer.Stop();
        timer.Interval = TimeSpan.FromMilliseconds(ms);
        timer.Start();
    }

    private void Call(uint id, string module, string method, byte[] args)
    {
        // Runs after the kernel's callback returns: the kernel is not reentrant.
        queue.TryEnqueue(async () =>
        {
            bool ok;
            byte[] result;
            try
            {
                result = await PlatformModules.Call(module, method, args);
                ok = true;
            }
            catch (Exception error)
            {
                result = Encoding.UTF8.GetBytes(error.Message);
                ok = false;
            }
            Resolve(id, ok, result);
        });
    }

    private unsafe void Resolve(uint id, bool ok, byte[] result)
    {
        if (runtime == 0) return;
        fixed (byte* data = result)
        {
            Native.RuntimeResolve(runtime, id, ok ? (byte)1 : (byte)0, data, (nuint)result.Length);
        }
    }

    /// <summary>Shows <paramref name="message"/> instead of the app's views.</summary>
    private void Fail(string message)
    {
        WeftLog.Write(Native.LogError, message);
        failed = true;
        root.Children.Clear();
        root.Children.Add(new TextBlock
        {
            Text = message,
            Foreground = new SolidColorBrush(Colors.Red),
            Margin = new Thickness(16),
            TextWrapping = TextWrapping.Wrap,
        });
    }

    // Kernel callbacks; every one arrives on the UI thread. Exceptions must not
    // unwind into the kernel.

    private static WeftSession From(nint ctx) => (WeftSession)GCHandle.FromIntPtr(ctx).Target!;

    private static unsafe string Utf8(byte* text) => Marshal.PtrToStringUTF8((nint)text) ?? "";

    [UnmanagedCallersOnly]
    private static unsafe void OnCommit(nint ctx, byte* ops, nuint len)
    {
        var session = From(ctx);
        session.Commit(new ReadOnlySpan<byte>(ops, checked((int)len)));
    }

    [UnmanagedCallersOnly]
    private static void OnSchedule(nint ctx, ulong delay)
    {
        try
        {
            From(ctx).Schedule(delay);
        }
        catch (Exception error)
        {
            WeftLog.Write(Native.LogError, $"weft: cannot schedule a tick ({error.Message})");
        }
    }

    [UnmanagedCallersOnly]
    private static unsafe byte OnSupports(nint ctx, byte* module) =>
        module != null && PlatformModules.Supported.Contains(Utf8(module)) ? (byte)1 : (byte)0;

    [UnmanagedCallersOnly]
    private static unsafe void OnCall(nint ctx, uint id, byte* module, byte* method, byte* args, nuint len)
    {
        var bytes = new ReadOnlySpan<byte>(args, checked((int)len)).ToArray();
        From(ctx).Call(id, Utf8(module), Utf8(method), bytes);
    }

    [UnmanagedCallersOnly]
    private static unsafe void OnLog(nint ctx, int level, byte* message, nuint len)
    {
        var text = Encoding.UTF8.GetString(message, checked((int)len));
        WeftLog.Write(level, text);
        if (level >= Native.LogError) From(ctx).lastError = text;
    }
}

/// <summary>Console messages and diagnostics, written with <c>OutputDebugString</c>.</summary>
internal static class WeftLog
{
    private static readonly string[] Levels = ["debug", "info", "warn", "error"];

    public static void Write(int level, string message) =>
        Native.OutputDebugString($"[weft {Levels[Math.Clamp(level, 0, Levels.Length - 1)]}] {message}\n");
}
