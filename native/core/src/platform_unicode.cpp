#include "hermes/Platform/Unicode/PlatformUnicode.h"

#include "hermes/Support/UTF8.h"
#include "llvh/Support/ConvertUTF.h"

#include <icu4x/CaseMapper.hpp>
#include <icu4x/Collator.hpp>
#include <icu4x/ComposingNormalizer.hpp>
#include <icu4x/DateFormatter.hpp>
#include <icu4x/DateTimeFormatter.hpp>
#include <icu4x/DecomposingNormalizer.hpp>
#include <icu4x/TimeFormatter.hpp>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <ctime>
#include <iterator>
#include <limits>
#include <memory>
#include <string>
#include <string_view>
#include <utility>

// Let the bindings write directly into the available storage, including the
// inline buffer. Diplomat measures both UTF-8 and UTF-16 storage in bytes.
template <typename Char, unsigned Inline>
struct icu4x::diplomat::WriteTrait<llvh::SmallVector<Char, Inline>> {
  using Buffer = llvh::SmallVector<Char, Inline>;
  using Write = icu4x::diplomat::capi::DiplomatWrite;

  static void flush(Write *write) {
    static_cast<Buffer *>(write->context)->set_size(write->len / sizeof(Char));
  }

  static bool grow(Write *write, size_t bytes) {
    auto &buf = *static_cast<Buffer *>(write->context);
    flush(write);  // Give reserve() the full initialized prefix to preserve.
    buf.reserve((bytes + sizeof(Char) - 1) / sizeof(Char));
    write->buf = reinterpret_cast<char *>(buf.data());
    write->cap = buf.capacity() * sizeof(Char);
    return true;
  }

  static Write Construct(Buffer &buf) {
    return {&buf, reinterpret_cast<char *>(buf.data()), buf.size() * sizeof(Char),
            buf.capacity() * sizeof(Char), false, flush, grow};
  }
};

namespace {

const icu4x::Locale &defaultLocale() {
  static const auto locale = []() -> std::unique_ptr<icu4x::Locale> {
    for (const char *key : {"LC_ALL", "LC_MESSAGES", "LANG"}) {
      const char *value = std::getenv(key);
      if (!value || !*value) continue;

      const std::string_view posix(value);
      const auto *bytes = reinterpret_cast<const llvh::UTF8 *>(posix.data());
      if (!llvh::isLegalUTF8String(&bytes, bytes + posix.size())) return nullptr;

      const auto name = posix.substr(0, posix.find_first_of(".@"));
      if (name == "C" || name == "POSIX") return nullptr;

      std::string bcp47(name);
      std::replace(bcp47.begin(), bcp47.end(), '_', '-');
      return icu4x::Locale::from_string(bcp47).ok().value_or(nullptr);
    }
    return nullptr;
  }();
  return locale ? *locale : icu4x::Locale::unknown_ref();
}

template <typename Load>
auto withRootFallback(Load load) {
  const auto &locale = defaultLocale();
  const auto &root = icu4x::Locale::unknown_ref();
  auto result = load(locale).ok();
  if (!result && &locale != &root) result = load(root).ok();
  return std::move(result).value_or(nullptr);
}

using Utf16Buffer = llvh::SmallVector<char16_t, 64>;

// Case mapping requires valid UTF-8, and the normalizers replace malformed
// UTF-16. Process well-formed runs so lone surrogates stay inert in both.
template <typename WriteRun>
void transform(llvh::SmallVectorImpl<char16_t> &buf, WriteRun writeRun) {
  if (buf.empty()) return;
  Utf16Buffer out;
  out.reserve(buf.size());
  const std::u16string_view input(buf.data(), buf.size());
  size_t start = 0;
  for (size_t i = 0; i < input.size(); ++i) {
    const auto unit = input[i];
    if (hermes::isHighSurrogate(unit)) {
      if (i + 1 < input.size() && hermes::isLowSurrogate(input[i + 1])) {
        ++i;
        continue;
      }
    } else if (!hermes::isLowSurrogate(unit)) {
      continue;
    }
    if (i > start) writeRun(input.substr(start, i - start), out);
    out.push_back(unit);
    start = i + 1;
  }
  if (start < input.size()) writeRun(input.substr(start), out);
  buf = std::move(out);
}

template <typename Normalizer>
void normalizeRuns(llvh::SmallVectorImpl<char16_t> &buf, const Normalizer &normalizer) {
  transform(buf, [&normalizer](std::u16string_view run, Utf16Buffer &out) {
    normalizer.normalize16_write(run, out);
  });
}

void appendUtf8(llvh::SmallVectorImpl<char16_t> &out, const llvh::SmallVectorImpl<char> &text) {
  hermes::convertUTF8WithSurrogatesToUTF16(std::back_inserter(out), text.begin(), text.end());
}

int64_t localRataDie(const std::tm &local) {
  // ICU4X's day-count constructor covers the full JS Date range. Rata die 1
  // is 0001-01-01; count days in preceding years with floored leap-year terms.
  const int64_t year = static_cast<int64_t>(local.tm_year) + 1899;
  const auto floorDiv = [year](int divisor) { return year / divisor - (year % divisor < 0); };
  return 365 * year + floorDiv(4) - floorDiv(100) + floorDiv(400) + local.tm_yday + 1;
}

}  // namespace

namespace hermes::platform_unicode {

int localeCompare(llvh::ArrayRef<char16_t> left, llvh::ArrayRef<char16_t> right) {
  static const auto collator = withRootFallback([](const icu4x::Locale &locale) {
    return icu4x::Collator::create_v1(locale, {});
  });
  const std::u16string_view lhs(left.data(), left.size());
  const std::u16string_view rhs(right.data(), right.size());
  if (collator) return collator->compare16(lhs, rhs);
  const int order = lhs.compare(rhs);
  return (order > 0) - (order < 0);
}

void convertToCase(llvh::SmallVectorImpl<char16_t> &buf,
                   CaseConversion targetCase,
                   bool useCurrentLocale) {
  if (buf.empty()) return;
  const auto &locale = useCurrentLocale ? defaultLocale() : icu4x::Locale::unknown_ref();
  auto map = targetCase == CaseConversion::ToUpper
                 ? icu4x::CaseMapper::uppercase_with_compiled_data_write<llvh::SmallVector<char, 192>>
                 : icu4x::CaseMapper::lowercase_with_compiled_data_write<llvh::SmallVector<char, 192>>;
  llvh::SmallVector<uint8_t, 192> input;
  llvh::SmallVector<char, 192> mapped;
  transform(buf, [&](std::u16string_view run, Utf16Buffer &out) {
    const size_t capacity = run.size() * 3;  // Maximum bytes per UTF-16 unit.
    input.reserve(capacity);
    const auto bytes = convertUTF16ToUTF8BufferWithReplacements(
        {input.data(), capacity}, {run.data(), run.size()}).second;
    const std::string_view utf8(reinterpret_cast<const char *>(input.data()), bytes);
    mapped.clear();
    map(utf8, locale, mapped);
    appendUtf8(out, mapped);
  });
}

void normalize(llvh::SmallVectorImpl<char16_t> &buf, NormalizationForm form) {
  switch (form) {
    case NormalizationForm::C: {
      static const auto nfc = icu4x::ComposingNormalizer::create_nfc();
      return normalizeRuns(buf, *nfc);
    }
    case NormalizationForm::D: {
      static const auto nfd = icu4x::DecomposingNormalizer::create_nfd();
      return normalizeRuns(buf, *nfd);
    }
    case NormalizationForm::KC: {
      static const auto nfkc = icu4x::ComposingNormalizer::create_nfkc();
      return normalizeRuns(buf, *nfkc);
    }
    case NormalizationForm::KD: {
      static const auto nfkd = icu4x::DecomposingNormalizer::create_nfkd();
      return normalizeRuns(buf, *nfkd);
    }
  }
}

void dateFormat(double unixtimeMs, bool formatDate, bool formatTime,
                llvh::SmallVectorImpl<char16_t> &buf) {
  buf.clear();
  if (std::isnan(unixtimeMs) || (!formatDate && !formatTime)) return;

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
  if (!localtime_r(&seconds, &local)) return;

  llvh::SmallVector<char, 128> text;
  if (formatTime) {
    // ICU4X Time does not represent leap seconds from right/ time zones.
    const auto time = icu4x::Time::create(
        local.tm_hour, local.tm_min, std::min(local.tm_sec, 59), 0).ok();
    if (!time) return;
    if (formatDate) {
      static const auto formatter = withRootFallback([](const icu4x::Locale &locale) {
        return icu4x::DateTimeFormatter::create_ymdt(
            locale, icu4x::DateTimeLength::Medium, icu4x::TimePrecision::Second, {}, {});
      });
      if (formatter) {
        const auto date = icu4x::IsoDate::from_rata_die(localRataDie(local));
        formatter->format_iso_write(*date, **time, text);
      }
    } else {
      static const auto formatter = withRootFallback([](const icu4x::Locale &locale) {
        return icu4x::TimeFormatter::create(
            locale, icu4x::DateTimeLength::Medium, icu4x::TimePrecision::Second, {});
      });
      if (formatter) formatter->format_write(**time, text);
    }
  } else {
    static const auto formatter = withRootFallback([](const icu4x::Locale &locale) {
      return icu4x::DateFormatter::create_ymd(locale, icu4x::DateTimeLength::Medium, {}, {});
    });
    if (formatter) {
      const auto date = icu4x::IsoDate::from_rata_die(localRataDie(local));
      formatter->format_iso_write(*date, text);
    }
  }
  appendUtf8(buf, text);
}

}  // namespace hermes::platform_unicode
