#include "hermes/Platform/Unicode/PlatformUnicode.h"

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <ctime>
#include <limits>
#include <utility>

namespace {
// Private ABI shared with crates/unicode. Lengths and capacities count UTF-16
// code units; output never aliases input and has no trailing terminator.
enum class CaseMode : uint8_t { LowerRoot, UpperRoot, LowerLocal, UpperLocal };
enum class Normalization : uint8_t { NFC, NFD, NFKC, NFKD };
struct CivilTime {
  int32_t year;
  uint8_t month;
  uint8_t day;
  uint8_t hour;
  uint8_t minute;
  uint8_t second;
};

extern "C" {
int32_t weft_unicode_compare(const char16_t *, size_t, const char16_t *, size_t);
size_t weft_unicode_case(const char16_t *, size_t, CaseMode, char16_t *, size_t);
size_t weft_unicode_normalize(const char16_t *, size_t, Normalization, char16_t *, size_t);
size_t weft_unicode_format(CivilTime, uint8_t, char16_t *, size_t);
}

template <typename Write>
void writeOutput(llvh::SmallVectorImpl<char16_t> &out, size_t initialSize, Write write) {
  out.resize(initialSize);
  const size_t length = write(out.data(), out.size());
  out.resize(length);
  if (length > initialSize) {
    const size_t written = write(out.data(), out.size());
    out.resize(written);
  }
}

template <typename Write>
void transform(llvh::SmallVectorImpl<char16_t> &buf, Write write) {
  llvh::SmallVector<char16_t, 64> out;
  writeOutput(out, buf.size(), [&](char16_t *output, size_t capacity) {
    return write(buf.data(), buf.size(), output, capacity);
  });
  buf = std::move(out);
}
}  // namespace

namespace hermes::platform_unicode {

int localeCompare(llvh::ArrayRef<char16_t> left, llvh::ArrayRef<char16_t> right) {
  return weft_unicode_compare(left.data(), left.size(), right.data(), right.size());
}

void convertToCase(llvh::SmallVectorImpl<char16_t> &buf,
                   CaseConversion targetCase,
                   bool useCurrentLocale) {
  const bool upper = targetCase == CaseConversion::ToUpper;
  const auto mode = useCurrentLocale ? (upper ? CaseMode::UpperLocal : CaseMode::LowerLocal)
                                     : (upper ? CaseMode::UpperRoot : CaseMode::LowerRoot);
  transform(buf, [mode](const char16_t *input, size_t length, char16_t *output, size_t capacity) {
    return weft_unicode_case(input, length, mode, output, capacity);
  });
}

void normalize(llvh::SmallVectorImpl<char16_t> &buf, NormalizationForm form) {
  static_assert(static_cast<uint8_t>(NormalizationForm::C) == static_cast<uint8_t>(Normalization::NFC));
  static_assert(static_cast<uint8_t>(NormalizationForm::D) == static_cast<uint8_t>(Normalization::NFD));
  static_assert(static_cast<uint8_t>(NormalizationForm::KC) == static_cast<uint8_t>(Normalization::NFKC));
  static_assert(static_cast<uint8_t>(NormalizationForm::KD) == static_cast<uint8_t>(Normalization::NFKD));
  const auto mode = static_cast<Normalization>(form);
  transform(buf, [mode](const char16_t *input, size_t length, char16_t *output, size_t capacity) {
    return weft_unicode_normalize(input, length, mode, output, capacity);
  });
}

void dateFormat(double unixtimeMs, bool formatDate, bool formatTime,
                llvh::SmallVectorImpl<char16_t> &buf) {
  const uint8_t fields = (formatDate ? 1 : 0) | (formatTime ? 2 : 0);
  if (std::isnan(unixtimeMs) || fields == 0) {
    buf.clear();
    return;
  }

  // Like ICU's lenient Calendar, saturate dates outside its Julian-day bounds.
  // Floor before converting to time_t to retain the correct second before 1970.
  constexpr double epochJulianDay = 2440588;
  constexpr double dayLimit = 0x7F000000;
  constexpr double millisPerDay = 86400000;
  constexpr double earliest = (-dayLimit - epochJulianDay) * millisPerDay;
  constexpr double latest = (dayLimit - epochJulianDay) * millisPerDay;
  static_assert(std::numeric_limits<std::time_t>::digits >= 48);
  const auto seconds = static_cast<std::time_t>(
      std::floor(std::clamp(unixtimeMs, earliest, latest) / 1000));

  std::tm local;
  if (!localtime_r(&seconds, &local)) {
    buf.clear();
    return;
  }
  const CivilTime civil{
      local.tm_year + 1900,
      static_cast<uint8_t>(local.tm_mon + 1),
      static_cast<uint8_t>(local.tm_mday),
      static_cast<uint8_t>(local.tm_hour),
      static_cast<uint8_t>(local.tm_min),
      // ICU4X Time does not represent leap seconds from right/ time zones.
      static_cast<uint8_t>(std::min(local.tm_sec, 59)),
  };
  writeOutput(buf, 128, [&](char16_t *output, size_t capacity) {
    return weft_unicode_format(civil, fields, output, capacity);
  });
}

}  // namespace hermes::platform_unicode
