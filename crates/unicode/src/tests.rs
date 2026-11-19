use super::*;
use std::{cmp::Ordering, ptr};

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[test]
fn parses_posix_locales() {
    for (os, bcp47) in [
        ("zh_CN.UTF-8@euro", "zh-CN"),
        ("sr_Latn_RS.UTF-8", "sr-Latn-RS"),
        ("tr-TR", "tr-TR"),
        ("C.UTF-8", "und"),
        ("POSIX", "und"),
        ("not a locale", "und"),
        ("", "und"),
    ] {
        assert_eq!(parse_locale(os).to_string(), bcp47, "locale {os:?}");
    }
}

#[test]
fn collates_canonical_equivalents_and_chinese_pinyin() {
    let root = collator(&Locale::UNKNOWN).expect("root collation data");
    assert_eq!(
        root.compare_utf16(&utf16("e\u{301}"), &utf16("é")),
        Ordering::Equal
    );
    let zh = collator(&parse_locale("zh_CN.UTF-8")).expect("Chinese collation data");
    assert_eq!(zh.compare_utf16(&utf16("李"), &utf16("张")), Ordering::Less);
}

#[test]
fn maps_turkish_case() {
    let turkish = parse_locale("tr_TR.UTF-8");
    for (input, upper, expected) in [("iı", true, "İI"), ("Iİ", false, "ıi")] {
        let mut units = [0; 8];
        let mut output = Output {
            units: &mut units,
            required: 0,
        };
        convert_case(&utf16(input), &mut output, &turkish.id, upper).expect("case mapping");
        assert_eq!(&output.units[..output.required], utf16(expected));
    }
}

#[test]
fn case_mapping_preserves_lone_surrogates_and_maps_pairs() {
    let mut input = vec![0xD800];
    input.extend(utf16("ß𐐨"));
    input.extend([0xDC00, 0xD800]);
    let mut expected = vec![0xD800];
    expected.extend(utf16("SS𐐀"));
    expected.extend([0xDC00, 0xD800]);

    let mut output = [0; 8];
    // SAFETY: Both nonoverlapping arrays are valid for the supplied lengths.
    let length = unsafe {
        weft_unicode_case(
            input.as_ptr(),
            input.len(),
            1,
            output.as_mut_ptr(),
            output.len(),
        )
    };
    assert_eq!(&output[..length], expected);
}

#[test]
fn normalizes_all_forms_without_replacing_lone_surrogates() {
    for (form, input, expected) in [
        (0, "e\u{301}", "é"),
        (1, "é", "e\u{301}"),
        (2, "①e\u{301}", "1é"),
        (3, "①é", "1e\u{301}"),
    ] {
        let mut input = utf16(input);
        input.extend([0xD800, b'e' as u16, 0xDC00, 0x0301]);
        let mut expected = utf16(expected);
        expected.extend([0xD800, b'e' as u16, 0xDC00, 0x0301]);
        let mut output = [0; 8];
        // SAFETY: Both nonoverlapping arrays are valid for the supplied lengths.
        let length = unsafe {
            weft_unicode_normalize(
                input.as_ptr(),
                input.len(),
                form,
                output.as_mut_ptr(),
                output.len(),
            )
        };
        assert_eq!(&output[..length], expected, "normalization form {form}");
    }
}

#[test]
fn returns_required_utf16_capacity_without_overwriting_the_buffer() {
    let input = utf16("ß𐐨");
    let mut output = [0; 5];
    // SAFETY: The first output is empty. The subsequent buffers are disjoint and valid.
    unsafe {
        let needed = weft_unicode_case(input.as_ptr(), input.len(), 1, ptr::null_mut(), 0);
        assert_eq!(needed, 4);
        assert_eq!(
            weft_unicode_case(input.as_ptr(), input.len(), 1, output.as_mut_ptr(), 1),
            needed
        );
        assert_eq!(output, [b'S' as u16, 0, 0, 0, 0]);
        assert_eq!(
            weft_unicode_case(input.as_ptr(), input.len(), 1, output.as_mut_ptr(), needed),
            needed
        );
        assert_eq!(output[..needed], utf16("SS𐐀"));
    }
}

#[test]
fn formats_dates_at_the_edges_of_the_js_range() {
    for (year, month, day) in [(275760, 9, 13), (-271821, 4, 20)] {
        let civil = CivilTime {
            year,
            month,
            day,
            hour: 0,
            minute: 0,
            second: 0,
        };
        let mut units = [0; 128];
        let mut output = Output {
            units: &mut units,
            required: 0,
        };
        assert!(
            format_date_time(civil, 3, &mut output).is_some(),
            "year {year}"
        );
        assert!(output.required > 0, "year {year}");
    }
}
