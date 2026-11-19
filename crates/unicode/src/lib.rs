//! Private UTF-16 C ABI for the Linux kernel's Hermes Unicode backend.

use std::{env, fmt, slice, sync::LazyLock};

use arrayvec::ArrayString;
use calendrical_calculations::gregorian;
use icu_calendar::{Date, Iso};
use icu_casemap::CaseMapper;
use icu_collator::{Collator, CollatorBorrowed};
use icu_datetime::{
    DateTimeFormatter,
    fieldsets::{T, YMD, YMDT},
    input::{DateTime, Time},
    options::TimePrecision,
};
use icu_locale_core::{LanguageIdentifier, Locale};
use icu_normalizer::{ComposingNormalizerBorrowed, DecomposingNormalizerBorrowed};
use write16::Write16;
use writeable::Writeable;

static DEFAULT_LOCALE: LazyLock<Locale> = LazyLock::new(|| {
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|key| env::var_os(key).filter(|value| !value.is_empty()));
    locale
        .as_deref()
        .and_then(|value| value.to_str())
        .map_or(Locale::UNKNOWN, parse_locale)
});

fn parse_locale(value: &str) -> Locale {
    let name = value.split(['.', '@']).next().unwrap_or_default();
    if matches!(name, "C" | "POSIX") {
        return Locale::UNKNOWN;
    }
    name.replace('_', "-").parse().unwrap_or(Locale::UNKNOWN)
}

fn with_root_fallback<T, E>(locale: &Locale, load: impl Fn(&Locale) -> Result<T, E>) -> Option<T> {
    load(locale).or_else(|_| load(&Locale::UNKNOWN)).ok()
}

fn collator(locale: &Locale) -> Option<CollatorBorrowed<'static>> {
    with_root_fallback(locale, |locale| {
        Collator::try_new(locale.into(), Default::default())
    })
}

static COLLATOR: LazyLock<Option<CollatorBorrowed<'static>>> =
    LazyLock::new(|| collator(&DEFAULT_LOCALE));

/// Compare UTF-16 strings using the process's default locale, returning −1, 0, or 1.
///
/// # Safety
/// Nonempty inputs must point to valid, initialized UTF-16 code units for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn weft_unicode_compare(
    left: *const u16,
    left_len: usize,
    right: *const u16,
    right_len: usize,
) -> i32 {
    // SAFETY: The caller provides readable input for each nonempty slice.
    let (left, right) = unsafe { (input_slice(left, left_len), input_slice(right, right_len)) };
    COLLATOR.as_ref().map_or_else(
        || left.cmp(right) as i32,
        |collator| collator.compare_utf16(left, right) as i32,
    )
}

struct Output<'a> {
    units: &'a mut [u16],
    required: usize,
}

impl Write16 for Output<'_> {
    fn write_slice(&mut self, text: &[u16]) -> fmt::Result {
        let start = self.required.min(self.units.len());
        let written = text.len().min(self.units.len() - start);
        self.units[start..start + written].copy_from_slice(&text[..written]);
        self.required = self.required.saturating_add(text.len());
        Ok(())
    }
}

impl fmt::Write for Output<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for ch in text.chars() {
            Write16::write_char(self, ch)?;
        }
        Ok(())
    }
}

/// # Safety
/// For a nonzero length, `data` must be valid and readable for `len` units for `'a`.
unsafe fn input_slice<'a>(data: *const u16, len: usize) -> &'a [u16] {
    if len == 0 {
        &[]
    } else {
        // SAFETY: The caller guarantees the input memory for nonzero lengths.
        unsafe { slice::from_raw_parts(data, len) }
    }
}

/// # Safety
/// For a nonzero capacity, `data` must be valid, initialized and exclusively borrowed
/// for `capacity` units for `'a`.
unsafe fn output_sink<'a>(data: *mut u16, capacity: usize) -> Output<'a> {
    let units = if capacity == 0 {
        &mut []
    } else {
        // SAFETY: The caller guarantees exclusive access for nonzero capacities.
        unsafe { slice::from_raw_parts_mut(data, capacity) }
    };
    Output { units, required: 0 }
}

// ICU4X case mapping takes UTF-8 and the UTF-16 normalizer substitutes U+FFFD on
// malformed input. Lone surrogates are inert in both operations and must survive.
fn write_utf16_runs(
    input: &[u16],
    output: &mut Output<'_>,
    mut write_run: impl FnMut(&[u16], &mut Output<'_>) -> fmt::Result,
) -> fmt::Result {
    let mut start = 0;
    let mut remaining = input;
    while let Some((&unit, next)) = remaining.split_first() {
        let pos = input.len() - remaining.len();
        remaining = next;
        match unit {
            0xD800..=0xDBFF if matches!(next.first(), Some(0xDC00..=0xDFFF)) => {
                remaining = &next[1..];
            }
            0xD800..=0xDFFF => {
                if pos > start {
                    write_run(&input[start..pos], output)?;
                }
                output.write_slice(&[unit])?;
                start = pos + 1;
            }
            _ => (),
        }
    }
    if start < input.len() {
        write_run(&input[start..], output)?;
    }
    Ok(())
}

fn convert_case(
    input: &[u16],
    output: &mut Output<'_>,
    locale: &LanguageIdentifier,
    uppercase: bool,
) -> fmt::Result {
    let mapper = CaseMapper::new();
    let map = |text: &str, output: &mut Output<'_>| {
        if uppercase {
            mapper.uppercase(text, locale).write_to(output)
        } else {
            mapper.lowercase(text, locale).write_to(output)
        }
    };
    write_utf16_runs(input, output, |run, output| {
        const INLINE_UNITS: usize = 64;
        if run.len() <= INLINE_UNITS {
            // Each UTF-16 code unit needs at most three UTF-8 bytes.
            let mut text = ArrayString::<{ INLINE_UNITS * 3 }>::new();
            for ch in char::decode_utf16(run.iter().copied()) {
                let ch = ch.map_err(|_| fmt::Error)?;
                text.try_push(ch).map_err(|_| fmt::Error)?;
            }
            map(&text, output)
        } else {
            let text = String::from_utf16(run).map_err(|_| fmt::Error)?;
            map(&text, output)
        }
    })
}

/// Write case-mapped UTF-16 and return the required number of code units.
///
/// Modes 0–3 are root lowercase, root uppercase, local lowercase, and local uppercase.
///
/// # Safety
/// Nonempty input and output must be initialized, nonoverlapping, and readable or
/// writable respectively for the specified lengths. Pointers may be null for zero lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn weft_unicode_case(
    input: *const u16,
    len: usize,
    mode: u8,
    output: *mut u16,
    capacity: usize,
) -> usize {
    let locale = match mode {
        0 | 1 => &LanguageIdentifier::UNKNOWN,
        2 | 3 => &DEFAULT_LOCALE.id,
        _ => return 0,
    };
    // SAFETY: The caller guarantees that the slices are valid and do not alias.
    let (input, mut output) = unsafe { (input_slice(input, len), output_sink(output, capacity)) };
    convert_case(input, &mut output, locale, mode & 1 != 0).map_or(0, |()| output.required)
}

/// Write normalized UTF-16 and return the required number of code units.
///
/// Forms 0–3 are NFC, NFD, NFKC, and NFKD.
///
/// # Safety
/// Nonempty input and output must be initialized, nonoverlapping, and readable or
/// writable respectively for the specified lengths. Pointers may be null for zero lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn weft_unicode_normalize(
    input: *const u16,
    len: usize,
    form: u8,
    output: *mut u16,
    capacity: usize,
) -> usize {
    // SAFETY: The caller guarantees that the slices are valid and do not alias.
    let (input, mut output) = unsafe { (input_slice(input, len), output_sink(output, capacity)) };
    let result = match form {
        0 => write_utf16_runs(input, &mut output, |run, output| {
            ComposingNormalizerBorrowed::new_nfc().normalize_utf16_to(run, output)
        }),
        1 => write_utf16_runs(input, &mut output, |run, output| {
            DecomposingNormalizerBorrowed::new_nfd().normalize_utf16_to(run, output)
        }),
        2 => write_utf16_runs(input, &mut output, |run, output| {
            ComposingNormalizerBorrowed::new_nfkc().normalize_utf16_to(run, output)
        }),
        3 => write_utf16_runs(input, &mut output, |run, output| {
            DecomposingNormalizerBorrowed::new_nfkd().normalize_utf16_to(run, output)
        }),
        _ => return 0,
    };
    result.map_or(0, |()| output.required)
}

/// Local calendar and clock fields supplied by the C++ kernel's `localtime_r`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CivilTime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

static DATE: LazyLock<Option<DateTimeFormatter<YMD>>> = LazyLock::new(|| {
    with_root_fallback(&DEFAULT_LOCALE, |locale| {
        DateTimeFormatter::try_new(locale.into(), YMD::medium())
    })
});
static TIME: LazyLock<Option<DateTimeFormatter<T>>> = LazyLock::new(|| {
    with_root_fallback(&DEFAULT_LOCALE, |locale| {
        DateTimeFormatter::try_new(
            locale.into(),
            T::medium().with_time_precision(TimePrecision::Second),
        )
    })
});
static DATETIME: LazyLock<Option<DateTimeFormatter<YMDT>>> = LazyLock::new(|| {
    with_root_fallback(&DEFAULT_LOCALE, |locale| {
        DateTimeFormatter::try_new(locale.into(), YMD::medium().with_time_hms())
    })
});

fn format_date_time(civil: CivilTime, fields: u8, output: &mut Output<'_>) -> Option<()> {
    let date = || {
        // The ISO field constructor only accepts four-digit years. The day-count
        // constructor covers the full JS Date range and clamps beyond ICU4X's range.
        let day = gregorian::fixed_from_gregorian(civil.year, civil.month, civil.day);
        Date::from_rata_die(day, Iso)
    };
    let time = || Time::try_new(civil.hour, civil.minute, civil.second, 0).ok();
    match fields {
        1 => DATE.as_ref()?.format(&date()).write_to(output).ok(),
        2 => TIME.as_ref()?.format(&time()?).write_to(output).ok(),
        3 => DATETIME
            .as_ref()?
            .format(&DateTime {
                date: date(),
                time: time()?,
            })
            .write_to(output)
            .ok(),
        _ => None,
    }
}

/// Write a medium local date and/or time and return the required number of code units.
///
/// Field bits 1 and 2 select the date and time respectively.
///
/// # Safety
/// For nonzero capacity, `output` must point to that many initialized, writable code units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn weft_unicode_format(
    civil: CivilTime,
    fields: u8,
    output: *mut u16,
    capacity: usize,
) -> usize {
    // SAFETY: The caller guarantees exclusive access to the output for nonzero capacity.
    let mut output = unsafe { output_sink(output, capacity) };
    format_date_time(civil, fields, &mut output).map_or(0, |()| output.required)
}

#[cfg(test)]
mod tests;
