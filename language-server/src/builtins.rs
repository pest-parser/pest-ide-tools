pub(crate) const BUILTINS: &[&str] = &[
    "ANY",
    "ASCII_DIGIT",
    "ASCII_NONZERO_DIGIT",
    "ASCII_BIN_DIGIT",
    "ASCII_OCT_DIGIT",
    "ASCII_HEX_DIGIT",
    "ASCII_ALPHA_LOWER",
    "ASCII_ALPHA_UPPER",
    "ASCII_ALPHA",
    "ASCII_ALPHANUMERIC",
    "NEWLINE",
    "LETTER",
    "CASED_LETTER",
    "UPPERCASE_LETTER",
    "LOWERCASE_LETTER",
    "TITLECASE_LETTER",
    "MODIFIER_LETTER",
    "OTHER_LETTER",
    "MARK",
    "NON_SPACING_MARK",
    "SPACING_MARK",
    "ENCLOSING_MARK",
    "NUMBER",
    "DECIMAL_NUMBER",
    "LETTER_NUMBER",
    "OTHER_NUMBER",
    "PUNCTUATION",
    "CONNECTOR_PUNCTUATION",
    "DASH_PUNCTUATION",
    "OPEN_PUNCTUATION",
    "CLOSE_PUNCTUATION",
    "INITIAL_PUNCTUATION",
    "FINAL_PUNCTUATION",
    "OTHER_PUNCTUATION",
    "SYMBOL",
    "MATH_SYMBOL",
    "CURRENCY_SYMBOL",
    "MODIFIER_SYMBOL",
    "OTHER_SYMBOL",
    "SEPARATOR",
    "SPACE_SEPARATOR",
    "LINE_SEPARATOR",
    "PARAGRAPH_SEPARATOR",
    "CONTROL",
    "FORMAT",
    "PRIVATE_USE",
    "SURROGATE",
    "UNASSIGNED",
    "ALPHABETIC",
    "BIDI_CONTROL",
    "BIDI_MIRRORED",
    "CASE_IGNORABLE",
    "CASED",
    "CHANGES_WHEN_CASEFOLDED",
    "CHANGES_WHEN_CASEMAPPED",
    "CHANGES_WHEN_LOWERCASED",
    "CHANGES_WHEN_TITLECASED",
    "CHANGES_WHEN_UPPERCASED",
    "DASH",
    "DEFAULT_IGNORABLE_CODE_POINT",
    "DEPRECATED",
    "DIACRITIC",
    "EMOJI",
    "EMOJI_COMPONENT",
    "EMOJI_MODIFIER",
    "EMOJI_MODIFIER_BASE",
    "EMOJI_PRESENTATION",
    "EXTENDER",
    "GRAPHEME_BASE",
    "GRAPHEME_EXTEND",
    "GRAPHEME_LINK",
    "HEX_DIGIT",
    "HYPHEN",
    "IDS_BINARY_OPERATOR",
    "IDS_TRINARY_OPERATOR",
    "ID_CONTINUE",
    "ID_START",
    "IDEOGRAPHIC",
    "JOIN_CONTROL",
    "LOGICAL_ORDER_EXCEPTION",
    "LOWERCASE",
    "MATH",
    "NONCHARACTER_CODE_POINT",
    "OTHER_ALPHABETIC",
    "OTHER_DEFAULT_IGNORABLE_CODE_POINT",
    "OTHER_GRAPHEME_EXTEND",
    "OTHER_ID_CONTINUE",
    "OTHER_ID_START",
    "OTHER_LOWERCASE",
    "OTHER_MATH",
    "OTHER_UPPERCASE",
    "PATTERN_SYNTAX",
    "PATTERN_WHITE_SPACE",
    "PREPENDED_CONCATENATION_MARK",
    "QUOTATION_MARK",
    "RADICAL",
    "REGIONAL_INDICATOR",
    "SENTENCE_TERMINAL",
    "SOFT_DOTTED",
    "TERMINAL_PUNCTUATION",
    "UNIFIED_IDEOGRAPH",
    "UPPERCASE",
    "VARIATION_SELECTOR",
    "WHITE_SPACE",
    "XID_CONTINUE",
    "XID_START",
    "ADLAM",
    "AHOM",
    "ANATOLIAN_HIEROGLYPHS",
    "ARABIC",
    "ARMENIAN",
    "AVESTAN",
    "BALINESE",
    "BAMUM",
    "BASSA_VAH",
    "BATAK",
    "BENGALI",
    "BHAIKSUKI",
    "BOPOMOFO",
    "BRAHMI",
    "BRAILLE",
    "BUGINESE",
    "BUHID",
    "CANADIAN_ABORIGINAL",
    "CARIAN",
    "CAUCASIAN_ALBANIAN",
    "CHAKMA",
    "CHAM",
    "CHEROKEE",
    "CHORASMIAN",
    "COMMON",
    "COPTIC",
    "CUNEIFORM",
    "CYPRIOT",
    "CYPRO_MINOAN",
    "CYRILLIC",
    "DESERET",
    "DEVANAGARI",
    "DIVES_AKURU",
    "DOGRA",
    "DUPLOYAN",
    "EGYPTIAN_HIEROGLYPHS",
    "ELBASAN",
    "ELYMAIC",
    "ETHIOPIC",
    "GEORGIAN",
    "GLAGOLITIC",
    "GOTHIC",
    "GRANTHA",
    "GREEK",
    "GUJARATI",
    "GUNJALA_GONDI",
    "GURMUKHI",
    "HAN",
    "HANGUL",
    "HANIFI_ROHINGYA",
    "HANUNOO",
    "HATRAN",
    "HEBREW",
    "HIRAGANA",
    "IMPERIAL_ARAMAIC",
    "INHERITED",
    "INSCRIPTIONAL_PAHLAVI",
    "INSCRIPTIONAL_PARTHIAN",
    "JAVANESE",
    "KAITHI",
    "KANNADA",
    "KATAKANA",
    "KAWI",
    "KAYAH_LI",
    "KHAROSHTHI",
    "KHITAN_SMALL_SCRIPT",
    "KHMER",
    "KHOJKI",
    "KHUDAWADI",
    "LAO",
    "LATIN",
    "LEPCHA",
    "LIMBU",
    "LINEAR_A",
    "LINEAR_B",
    "LISU",
    "LYCIAN",
    "LYDIAN",
    "MAHAJANI",
    "MAKASAR",
    "MALAYALAM",
    "MANDAIC",
    "MANICHAEAN",
    "MARCHEN",
    "MASARAM_GONDI",
    "MEDEFAIDRIN",
    "MEETEI_MAYEK",
    "MENDE_KIKAKUI",
    "MEROITIC_CURSIVE",
    "MEROITIC_HIEROGLYPHS",
    "MIAO",
    "MODI",
    "MONGOLIAN",
    "MRO",
    "MULTANI",
    "MYANMAR",
    "NABATAEAN",
    "NAG_MUNDARI",
    "NANDINAGARI",
    "NEW_TAI_LUE",
    "NEWA",
    "NKO",
    "NUSHU",
    "NYIAKENG_PUACHUE_HMONG",
    "OGHAM",
    "OL_CHIKI",
    "OLD_HUNGARIAN",
    "OLD_ITALIC",
    "OLD_NORTH_ARABIAN",
    "OLD_PERMIC",
    "OLD_PERSIAN",
    "OLD_SOGDIAN",
    "OLD_SOUTH_ARABIAN",
    "OLD_TURKIC",
    "OLD_UYGHUR",
    "ORIYA",
    "OSAGE",
    "OSMANYA",
    "PAHAWH_HMONG",
    "PALMYRENE",
    "PAU_CIN_HAU",
    "PHAGS_PA",
    "PHOENICIAN",
    "PSALTER_PAHLAVI",
    "REJANG",
    "RUNIC",
    "SAMARITAN",
    "SAURASHTRA",
    "SHARADA",
    "SHAVIAN",
    "SIDDHAM",
    "SIGNWRITING",
    "SINHALA",
    "SOGDIAN",
    "SORA_SOMPENG",
    "SOYOMBO",
    "SUNDANESE",
    "SYLOTI_NAGRI",
    "SYRIAC",
    "TAGALOG",
    "TAGBANWA",
    "TAI_LE",
    "TAI_THAM",
    "TAI_VIET",
    "TAKRI",
    "TAMIL",
    "TANGSA",
    "TANGUT",
    "TELUGU",
    "THAANA",
    "THAI",
    "TIBETAN",
    "TIFINAGH",
    "TIRHUTA",
    "TOTO",
    "UGARITIC",
    "VAI",
    "VITHKUQI",
    "WANCHO",
    "WARANG_CITI",
    "YEZIDI",
    "YI",
    "ZANABAZAR_SQUARE",
];

pub(crate) fn get_builtin_description(rule: &str) -> Option<&str> {
    match rule {
        "ANY" => Some("Matches any character."),

        "ASCII_DIGIT" => Some("Matches any ASCII digit (0-9)."),
        "ASCII_NONZERO_DIGIT" => Some("Matches any non-zero ASCII digit (1-9)."),
        "ASCII_BIN_DIGIT" => Some("Matches any ASCII binary digit (0-1)."),
        "ASCII_OCT_DIGIT" => Some("Matches any ASCII octal digit (0-7)."),
        "ASCII_HEX_DIGIT" => Some("Matches any ASCII hexadecimal digit (0-9, a-f, A-F)."),

        "ASCII_ALPHA_LOWER" => Some("Matches any ASCII lowercase letter (a-z)."),
        "ASCII_ALPHA_UPPER" => Some("Matches any ASCII uppercase letter (A-Z)."),
        "ASCII_ALPHA" => Some("Matches any ASCII letter (a-z, A-Z)."),

        "ASCII_ALPHANUMERIC" => Some("Matches any ASCII alphanumeric character (0-9, a-z, A-Z)."),
        "NEWLINE" => Some("Matches any newline character (\\n, \\r\\n, \\r)."),

        "LETTER" => Some("Matches any Unicode letter."),
        "CASED_LETTER" => Some("Matches any upper or lower case Unicode letter."),
        "UPPERCASE_LETTER" => Some("Matches any uppercase Unicode letter."),
        "LOWERCASE_LETTER" => Some("Matches any lowercase Unicode letter."),
        "TITLECASE_LETTER" => Some("Matches any titlecase Unicode letter."),
        "MODIFIER_LETTER" => Some("Matches any Unicode modifier letter."),
        "OTHER_LETTER" => {
            Some("Matches any Unicode letter that does not fit into any other defined categories.")
        }

        "MARK" => Some("Matches any Unicode mark."),
        "NON_SPACING_MARK" => Some("Matches any Unicode non-spacing mark."),
        "SPACING_MARK" => Some("Matches any Unicode spacing mark."),
        "ENCLOSING_MARK" => Some("Matches any Unicode enclosing mark."),

        "NUMBER" => Some("Matches any Unicode number."),
        "DECIMAL_NUMBER" => Some("Matches any Unicode decimal number."),
        "LETTER_NUMBER" => Some("Matches any Unicode letter number."),
        "OTHER_NUMBER" => {
            Some("Matches any Unicode number that does not fit into any other defined categories.")
        }

        "PUNCTUATION" => Some("Matches any Unicode punctuation."),
        "CONNECTOR_PUNCTUATION" => Some("Matches any Unicode connector punctuation."),
        "DASH_PUNCTUATION" => Some("Matches any Unicode dash punctuation."),
        "OPEN_PUNCTUATION" => Some("Matches any Unicode open punctuation."),
        "CLOSE_PUNCTUATION" => Some("Matches any Unicode close punctuation."),
        "INITIAL_PUNCTUATION" => Some("Matches any Unicode initial punctuation."),
        "FINAL_PUNCTUATION" => Some("Matches any Unicode final punctuation."),
        "OTHER_PUNCTUATION" => Some(
            "Matches any Unicode punctuation that does not fit into any other defined categories.",
        ),

        "SYMBOL" => Some("Matches any Unicode symbol."),
        "MATH_SYMBOL" => Some("Matches any Unicode math symbol."),
        "CURRENCY_SYMBOL" => Some("Matches any Unicode currency symbol."),
        "MODIFIER_SYMBOL" => Some("Matches any Unicode modifier symbol."),
        "OTHER_SYMBOL" => {
            Some("Matches any Unicode symbol that does not fit into any other defined categories.")
        }

        "SEPARATOR" => Some("Matches any Unicode separator."),
        "SPACE_SEPARATOR" => Some("Matches any Unicode space separator."),
        "LINE_SEPARATOR" => Some("Matches any Unicode line separator."),
        "PARAGRAPH_SEPARATOR" => Some("Matches any Unicode paragraph separator."),

        "OTHER" => Some(
            "Matches any Unicode character that does not fit into any other defined categories.",
        ),
        "CONTROL" => Some("Matches any Unicode control character."),
        "FORMAT" => Some("Matches any Unicode format character."),
        "SURROGATE" => Some("Matches any Unicode surrogate."),
        "PRIVATE_USE" => Some("Matches any Unicode private use character."),
        "UNASSIGNED" => Some("Matches any Unicode unassigned character."),

        "ALPHABETIC" => Some("Matches any Unicode alphabetic character."),
        "BIDI_CONTROL" => Some("Matches any Unicode bidirectional control character."),
        "BIDI_MIRRORED" => Some("Matches any Unicode bidirectional mirrored character."),
        "CASE_IGNORABLE" => Some("Matches any Unicode case-ignorable character."),
        "CASED" => Some("Matches any Unicode cased character."),
        "CHANGES_WHEN_CASEFOLDED" => {
            Some("Matches any Unicode character that changes when casefolded.")
        }
        "CHANGES_WHEN_CASEMAPPED" => {
            Some("Matches any Unicode character that changes when casemapped.")
        }
        "CHANGES_WHEN_LOWERCASED" => {
            Some("Matches any Unicode character that changes when lowercased.")
        }
        "CHANGES_WHEN_TITLECASED" => {
            Some("Matches any Unicode character that changes when titlecased.")
        }
        "CHANGES_WHEN_UPPERCASED" => {
            Some("Matches any Unicode character that changes when uppercased.")
        }
        "DASH" => Some("Matches any Unicode dash character."),
        "DEFAULT_IGNORABLE_CODE_POINT" => Some("Matches any Unicode default-ignorable code point."),
        "DEPRECATED" => Some("Matches any Unicode deprecated character."),
        "DIACRITIC" => Some("Matches any Unicode diacritic character."),
        "EMOJI" => Some("Matches any Unicode emoji character."),
        "EMOJI_COMPONENT" => Some("Matches any Unicode emoji component character."),
        "EMOJI_MODIFIER" => Some("Matches any Unicode emoji modifier character."),
        "EMOJI_MODIFIER_BASE" => Some("Matches any Unicode emoji modifier base character."),
        "EMOJI_PRESENTATION" => Some("Matches any Unicode emoji presentation character."),
        "EXTENDED_PICTOGRAPHIC" => Some("Matches any Unicode extended pictographic character."),
        "EXTENDER" => Some("Matches any Unicode extender character."),
        "GRAPHEME_BASE" => Some("Matches any Unicode grapheme base character."),
        "GRAPHEME_EXTEND" => Some("Matches any Unicode grapheme extend character."),
        "GRAPHEME_LINK" => Some("Matches any Unicode grapheme link character."),
        "HEX_DIGIT" => Some("Matches any Unicode hexadecimal digit character."),
        "HYPHEN" => Some("Matches any Unicode hyphen character."),
        "IDS_BINARY_OPERATOR" => Some("Matches any Unicode IDS binary operator character."),
        "IDS_TRINARY_OPERATOR" => Some("Matches any Unicode IDS trinary operator character."),
        "ID_CONTINUE" => Some("Matches any Unicode ID continue character."),
        "ID_START" => Some("Matches any Unicode ID start character."),
        "IDEOGRAPHIC" => Some("Matches any Unicode ideographic character."),
        "JOIN_CONTROL" => Some("Matches any Unicode join control character."),
        "LOGICAL_ORDER_EXCEPTION" => Some("Matches any Unicode logical order exception character."),
        "LOWERCASE" => Some("Matches any Unicode lowercase character."),
        "MATH" => Some("Matches any Unicode math character."),
        "NONCHARACTER_CODE_POINT" => Some("Matches any Unicode noncharacter code point."),
        "OTHER_ALPHABETIC" => Some("Matches any Unicode other alphabetic character."),
        "OTHER_DEFAULT_IGNORABLE_CODE_POINT" => {
            Some("Matches any Unicode other default-ignorable code point.")
        }
        "OTHER_GRAPHEME_EXTEND" => Some("Matches any Unicode other grapheme extend character."),
        "OTHER_ID_CONTINUE" => Some("Matches any Unicode other ID continue character."),
        "OTHER_ID_START" => Some("Matches any Unicode other ID start character."),
        "OTHER_LOWERCASE" => Some("Matches any Unicode other lowercase character."),
        "OTHER_MATH" => Some("Matches any Unicode other math character."),
        "OTHER_UPPERCASE" => Some("Matches any Unicode other uppercase character."),
        "PATTERN_SYNTAX" => Some("Matches any Unicode pattern syntax character."),
        "PATTERN_WHITE_SPACE" => Some("Matches any Unicode pattern white space character."),
        "PREPENDED_CONCATENATION_MARK" => {
            Some("Matches any Unicode prepended concatenation mark character.")
        }
        "QUOTATION_MARK" => Some("Matches any Unicode quotation mark character."),
        "RADICAL" => Some("Matches any Unicode radical character."),
        "REGIONAL_INDICATOR" => Some("Matches any Unicode regional indicator character."),
        "SENTENCE_TERMINAL" => Some("Matches any Unicode sentence terminal character."),
        "SOFT_DOTTED" => Some("Matches any Unicode soft-dotted character."),
        "TERMINAL_PUNCTUATION" => Some("Matches any Unicode terminal punctuation character."),
        "UNIFIED_IDEOGRAPH" => Some("Matches any Unicode unified ideograph character."),
        "UPPERCASE" => Some("Matches any Unicode uppercase character."),
        "VARIATION_SELECTOR" => Some("Matches any Unicode variation selector character."),
        "WHITE_SPACE" => Some("Matches any Unicode white space character."),
        "XID_CONTINUE" => Some("Matches any Unicode XID continue character."),
        "XID_START" => Some("Matches any Unicode XID start character."),

        "ADLAM" => Some("Matches any Unicode Adlam character."),
        "AHOM" => Some("Matches any Unicode Ahom character."),
        "ANATOLIAN_HIEROGLYPHS" => Some("Matches any Unicode Anatolian Hieroglyphs character."),
        "ARABIC" => Some("Matches any Unicode Arabic character."),
        "ARMENIAN" => Some("Matches any Unicode Armenian character."),
        "AVESTAN" => Some("Matches any Unicode Avestan character."),
        "BALINESE" => Some("Matches any Unicode Balinese character."),
        "BAMUM" => Some("Matches any Unicode Bamum character."),
        "BASSA_VAH" => Some("Matches any Unicode Bassa Vah character."),
        "BATAK" => Some("Matches any Unicode Batak character."),
        "BENGALI" => Some("Matches any Unicode Bengali character."),
        "BHAIKSUKI" => Some("Matches any Unicode Bhaiksuki character."),
        "BOPOMOFO" => Some("Matches any Unicode Bopomofo character."),
        "BRAHMI" => Some("Matches any Unicode Brahmi character."),
        "BRAILLE" => Some("Matches any Unicode Braille character."),
        "BUGINESE" => Some("Matches any Unicode Buginese character."),
        "BUHID" => Some("Matches any Unicode Buhid character."),
        "CANADIAN_ABORIGINAL" => Some("Matches any Unicode Canadian Aboriginal character."),
        "CARIAN" => Some("Matches any Unicode Carian character."),
        "CAUCASIAN_ALBANIAN" => Some("Matches any Unicode Caucasian Albanian character."),
        "CHAKMA" => Some("Matches any Unicode Chakma character."),
        "CHAM" => Some("Matches any Unicode Cham character."),
        "CHEROKEE" => Some("Matches any Unicode Cherokee character."),
        "CHORASMIAN" => Some("Matches any Unicode Chorasmian character."),
        "COMMON" => Some("Matches any Unicode Common character."),
        "COPTIC" => Some("Matches any Unicode Coptic character."),
        "CUNEIFORM" => Some("Matches any Unicode Cuneiform character."),
        "CYPRIOT" => Some("Matches any Unicode Cypriot character."),
        "CYPRO_MINOAN" => Some("Matches any Unicode Cypro-Minoan character."),
        "CYRILLIC" => Some("Matches any Unicode Cyrillic character."),
        "DESERET" => Some("Matches any Unicode Deseret character."),
        "DEVANAGARI" => Some("Matches any Unicode Devanagari character."),
        "DIVES_AKURU" => Some("Matches any Unicode Dives Akuru character."),
        "DOGRA" => Some("Matches any Unicode Dogra character."),
        "DUPLOYAN" => Some("Matches any Unicode Duployan character."),
        "EGYPTIAN_HIEROGLYPHS" => Some("Matches any Unicode Egyptian Hieroglyphs character."),
        "ELBASAN" => Some("Matches any Unicode Elbasan character."),
        "ELYMAIC" => Some("Matches any Unicode Elymaic character."),
        "ETHIOPIC" => Some("Matches any Unicode Ethiopic character."),
        "GEORGIAN" => Some("Matches any Unicode Georgian character."),
        "GLAGOLITIC" => Some("Matches any Unicode Glagolitic character."),
        "GOTHIC" => Some("Matches any Unicode Gothic character."),
        "GRANTHA" => Some("Matches any Unicode Grantha character."),
        "GREEK" => Some("Matches any Unicode Greek character."),
        "GUJARATI" => Some("Matches any Unicode Gujarati character."),
        "GUNJALA_GONDI" => Some("Matches any Unicode Gunjala Gondi character."),
        "GURMUKHI" => Some("Matches any Unicode Gurmukhi character."),
        "HAN" => Some("Matches any Unicode Han character."),
        "HANGUL" => Some("Matches any Unicode Hangul character."),
        "HANIFI_ROHINGYA" => Some("Matches any Unicode Hanifi Rohingya character."),
        "HANUNOO" => Some("Matches any Unicode Hanunoo character."),
        "HATRAN" => Some("Matches any Unicode Hatran character."),
        "HEBREW" => Some("Matches any Unicode Hebrew character."),
        "HIRAGANA" => Some("Matches any Unicode Hiragana character."),
        "IMPERIAL_ARAMAIC" => Some("Matches any Unicode Imperial Aramaic character."),
        "INHERITED" => Some("Matches any Unicode Inherited character."),
        "INSCRIPTIONAL_PAHLAVI" => Some("Matches any Unicode Inscriptional Pahlavi character."),
        "INSCRIPTIONAL_PARTHIAN" => Some("Matches any Unicode Inscriptional Parthian character."),
        "JAVANESE" => Some("Matches any Unicode Javanese character."),
        "KAITHI" => Some("Matches any Unicode Kaithi character."),
        "KANNADA" => Some("Matches any Unicode Kannada character."),
        "KATAKANA" => Some("Matches any Unicode Katakana character."),
        "KAWI" => Some("Matches any Unicode Kawi character."),
        "KAYAH_LI" => Some("Matches any Unicode Kayah Li character."),
        "KHAROSHTHI" => Some("Matches any Unicode Kharoshthi character."),
        "KHITAN_SMALL_SCRIPT" => Some("Matches any Unicode Khitan Small Script character."),
        "KHMER" => Some("Matches any Unicode Khmer character."),
        "KHOJKI" => Some("Matches any Unicode Khojki character."),
        "KHUDAWADI" => Some("Matches any Unicode Khudawadi character."),
        "LAO" => Some("Matches any Unicode Lao character."),
        "LATIN" => Some("Matches any Unicode Latin character."),
        "LEPCHA" => Some("Matches any Unicode Lepcha character."),
        "LIMBU" => Some("Matches any Unicode Limbu character."),
        "LINEAR_A" => Some("Matches any Unicode Linear A character."),
        "LINEAR_B" => Some("Matches any Unicode Linear B character."),
        "LISU" => Some("Matches any Unicode Lisu character."),
        "LYCIAN" => Some("Matches any Unicode Lycian character."),
        "LYDIAN" => Some("Matches any Unicode Lydian character."),
        "MAHAJANI" => Some("Matches any Unicode Mahajani character."),
        "MAKASAR" => Some("Matches any Unicode Makasar character."),
        "MALAYALAM" => Some("Matches any Unicode Malayalam character."),
        "MANDAIC" => Some("Matches any Unicode Mandaic character."),
        "MANICHAEAN" => Some("Matches any Unicode Manichaean character."),
        "MARCHEN" => Some("Matches any Unicode Marchen character."),
        "MASARAM_GONDI" => Some("Matches any Unicode Masaram Gondi character."),
        "MEDEFAIDRIN" => Some("Matches any Unicode Medefaidrin character."),
        "MEETEI_MAYEK" => Some("Matches any Unicode Meetei Mayek character."),
        "MENDE_KIKAKUI" => Some("Matches any Unicode Mende Kikakui character."),
        "MEROITIC_CURSIVE" => Some("Matches any Unicode Meroitic Cursive character."),
        "MEROITIC_HIEROGLYPHS" => Some("Matches any Unicode Meroitic Hieroglyphs character."),
        "MIAO" => Some("Matches any Unicode Miao character."),
        "MODI" => Some("Matches any Unicode Modi character."),
        "MONGOLIAN" => Some("Matches any Unicode Mongolian character."),
        "MRO" => Some("Matches any Unicode Mro character."),
        "MULTANI" => Some("Matches any Unicode Multani character."),
        "MYANMAR" => Some("Matches any Unicode Myanmar character."),
        "NABATAEAN" => Some("Matches any Unicode Nabataean character."),
        "NAG_MUNDARI" => Some("Matches any Unicode Nag Mundari character."),
        "NANDINAGARI" => Some("Matches any Unicode Nandinagari character."),
        "NEW_TAI_LUE" => Some("Matches any Unicode New Tai Lue character."),
        "NEWA" => Some("Matches any Unicode Newa character."),
        "NKO" => Some("Matches any Unicode Nko character."),
        "NUSHU" => Some("Matches any Unicode Nushu character."),
        "NYIAKENG_PUACHUE_HMONG" => Some("Matches any Unicode Nyiakeng Puachue Hmong character."),
        "OGHAM" => Some("Matches any Unicode Ogham character."),
        "OL_CHIKI" => Some("Matches any Unicode Ol Chiki character."),
        "OLD_HUNGARIAN" => Some("Matches any Unicode Old Hungarian character."),
        "OLD_ITALIC" => Some("Matches any Unicode Old Italic character."),
        "OLD_NORTH_ARABIAN" => Some("Matches any Unicode Old North Arabian character."),
        "OLD_PERMIC" => Some("Matches any Unicode Old Permic character."),
        "OLD_PERSIAN" => Some("Matches any Unicode Old Persian character."),
        "OLD_SOGDIAN" => Some("Matches any Unicode Old Sogdian character."),
        "OLD_SOUTH_ARABIAN" => Some("Matches any Unicode Old South Arabian character."),
        "OLD_TURKIC" => Some("Matches any Unicode Old Turkic character."),
        "OLD_UYGHUR" => Some("Matches any Unicode Old Uyghur character."),
        "ORIYA" => Some("Matches any Unicode Oriya character."),
        "OSAGE" => Some("Matches any Unicode Osage character."),
        "OSMANYA" => Some("Matches any Unicode Osmanya character."),
        "PAHAWH_HMONG" => Some("Matches any Unicode Pahawh Hmong character."),
        "PALMYRENE" => Some("Matches any Unicode Palmyrene character."),
        "PAU_CIN_HAU" => Some("Matches any Unicode Pau Cin Hau character."),
        "PHAGS_PA" => Some("Matches any Unicode Phags Pa character."),
        "PHOENICIAN" => Some("Matches any Unicode Phoenician character."),
        "PSALTER_PAHLAVI" => Some("Matches any Unicode Psalter Pahlavi character."),
        "REJANG" => Some("Matches any Unicode Rejang character."),
        "RUNIC" => Some("Matches any Unicode Runic character."),
        "SAMARITAN" => Some("Matches any Unicode Samaritan character."),
        "SAURASHTRA" => Some("Matches any Unicode Saurashtra character."),
        "SHARADA" => Some("Matches any Unicode Sharada character."),
        "SHAVIAN" => Some("Matches any Unicode Shavian character."),
        "SIDDHAM" => Some("Matches any Unicode Siddham character."),
        "SIGNWRITING" => Some("Matches any Unicode SignWriting character."),
        "SINHALA" => Some("Matches any Unicode Sinhala character."),
        "SOGDIAN" => Some("Matches any Unicode Sogdian character."),
        "SORA_SOMPENG" => Some("Matches any Unicode Sora Sompeng character."),
        "SOYOMBO" => Some("Matches any Unicode Soyombo character."),
        "SUNDANESE" => Some("Matches any Unicode Sundanese character."),
        "SYLOTI_NAGRI" => Some("Matches any Unicode Syloti Nagri character."),
        "SYRIAC" => Some("Matches any Unicode Syriac character."),
        "TAGALOG" => Some("Matches any Unicode Tagalog character."),
        "TAGBANWA" => Some("Matches any Unicode Tagbanwa character."),
        "TAI_LE" => Some("Matches any Unicode Tai Le character."),
        "TAI_THAM" => Some("Matches any Unicode Tai Tham character."),
        "TAI_VIET" => Some("Matches any Unicode Tai Viet character."),
        "TAKRI" => Some("Matches any Unicode Takri character."),
        "TAMIL" => Some("Matches any Unicode Tamil character."),
        "TANGSA" => Some("Matches any Unicode Tangsa character."),
        "TANGUT" => Some("Matches any Unicode Tangut character."),
        "TELUGU" => Some("Matches any Unicode Telugu character."),
        "THAANA" => Some("Matches any Unicode Thaana character."),
        "THAI" => Some("Matches any Unicode Thai character."),
        "TIBETAN" => Some("Matches any Unicode Tibetan character."),
        "TIFINAGH" => Some("Matches any Unicode Tifinagh character."),
        "TIRHUTA" => Some("Matches any Unicode Tirhuta character."),
        "TOTO" => Some("Matches any Unicode Toto character."),
        "UGARITIC" => Some("Matches any Unicode Ugaritic character."),
        "VAI" => Some("Matches any Unicode Vai character."),
        "VITHKUQI" => Some("Matches any Unicode Vithkuqi character."),
        "WANCHO" => Some("Matches any Unicode Wancho character."),
        "WARANG_CITI" => Some("Matches any Unicode Warang Citi character."),
        "YEZIDI" => Some("Matches any Unicode Yezidi character."),
        "YI" => Some("Matches any Unicode Yi character."),
        "ZANABAZAR_SQUARE" => Some("Matches any Unicode Zanabazar Square character."),
        _ => None,
    }
}
