// Platform modules of the shell; byte formats in
// protocol/SPEC.md ("Platform calls").

#pragma once

#include <gio/gio.h>

#include <cstdint>
#include <functional>
#include <string>
#include <string_view>
#include <vector>

namespace weft::platform {

/// Completes a call: whether it succeeded, and its result or error message.
using Done = std::function<void(bool ok, std::vector<uint8_t> result)>;

/// Whether the shell implements module `module`.
bool supported(std::string_view module);

/// Starts `module.method` with `args`. `done` runs later on the main loop,
/// never before this returns, and never once `cancellable` is cancelled.
void call(const std::string &module, const std::string &method, std::vector<uint8_t> args,
          GCancellable *cancellable, Done done);

}  // namespace weft::platform
