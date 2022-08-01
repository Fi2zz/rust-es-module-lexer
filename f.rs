pub fn get_ascii_code(c: char) -> usize {
    match c {
        ' ' => return 32,
        '!' => return 33,
        '"' => return 34,
        '#' => return 35,
        '$' => return 36,
        '%' => return 37,
        '&' => return 38,
        '\'' => return 39,
        '(' => return 40,
        ')' => return 41,
        '*' => return 42,
        '+' => return 43,
        ',' => return 44,
        '-' => return 45,
        '.' => return 46,
        '/' => return 47,
        '0' => return 48,
        '1' => return 49,
        '2' => return 50,
        '3' => return 51,
        '4' => return 52,
        '5' => return 53,
        '6' => return 54,
        '7' => return 55,
        '8' => return 56,
        '9' => return 57,
        ':' => return 58,
        ';' => return 59,
        '<' => return 60,
        '=' => return 61,
        '>' => return 62,
        '?' => return 63,
        '@' => return 64,
        'A' => return 65,
        'B' => return 66,
        'C' => return 67,
        'D' => return 68,
        'E' => return 69,
        'F' => return 70,
        'G' => return 71,
        'H' => return 72,
        'I' => return 73,
        'J' => return 74,
        'K' => return 75,
        'L' => return 76,
        'M' => return 77,
        'N' => return 78,
        'O' => return 79,
        'P' => return 80,
        'Q' => return 81,
        'R' => return 82,
        'S' => return 83,
        'T' => return 84,
        'U' => return 85,
        'V' => return 86,
        'W' => return 87,
        'X' => return 88,
        'Y' => return 89,
        'Z' => return 90,
        '[' => return 91,
        '\\' => return 92,
        ']' => return 93,
        '^' => return 94,
        '_' => return 95,
        '`' => return 96,
        'a' => return 97,
        'b' => return 98,
        'c' => return 99,
        'd' => return 100,
        'e' => return 101,
        'f' => return 102,
        'g' => return 103,
        'h' => return 104,
        'i' => return 105,
        'j' => return 106,
        'k' => return 107,
        'l' => return 108,
        'm' => return 109,
        'n' => return 110,
        'o' => return 111,
        'p' => return 112,
        'q' => return 113,
        'r' => return 114,
        's' => return 115,
        't' => return 116,
        'u' => return 117,
        'v' => return 118,
        'w' => return 119,
        'x' => return 120,
        'y' => return 121,
        'z' => return 122,
        '{' => return 123,
        '|' => return 124,
        '}' => return 125,
        '~' => return 126,
        _ => return 0,
    }
}
pub fn get_char_by_code(code: usize) -> char {
    match code {
        32 => ' ',
        33 => '!',
        34 => '"',
        35 => '#',
        36 => '$',
        37 => '%',
        38 => '&',
        39 => '\'',
        40 => '(',
        41 => ')',
        42 => '*',
        43 => '+',
        44 => ',',
        45 => '-',
        46 => '.',
        47 => '/',
        48 => '0',
        49 => '1',
        50 => '2',
        51 => '3',
        52 => '4',
        53 => '5',
        54 => '6',
        55 => '7',
        56 => '8',
        57 => '9',
        58 => ':',
        59 => ';',
        60 => '<',
        61 => '=',
        62 => '>',
        63 => '?',
        64 => '@',
        65 => 'A',
        66 => 'B',
        67 => 'C',
        68 => 'D',
        69 => 'E',
        70 => 'F',
        71 => 'G',
        72 => 'H',
        73 => 'I',
        74 => 'J',
        75 => 'K',
        76 => 'L',
        77 => 'M',
        78 => 'N',
        79 => 'O',
        80 => 'P',
        81 => 'Q',
        82 => 'R',
        83 => 'S',
        84 => 'T',
        85 => 'U',
        86 => 'V',
        87 => 'W',
        88 => 'X',
        89 => 'Y',
        90 => 'Z',
        91 => '[',
        92 => '\\',
        93 => ']',
        94 => '^',
        95 => '_',
        96 => '`',
        97 => 'a',
        98 => 'b',
        99 => 'c',
        100 => 'd',
        101 => 'e',
        102 => 'f',
        103 => 'g',
        104 => 'h',
        105 => 'i',
        106 => 'j',
        107 => 'k',
        108 => 'l',
        109 => 'm',
        110 => 'n',
        111 => 'o',
        112 => 'p',
        113 => 'q',
        114 => 'r',
        115 => 's',
        116 => 't',
        117 => 'u',
        118 => 'v',
        119 => 'w',
        120 => 'x',
        121 => 'y',
        122 => 'z',
        123 => '{',
        124 => '|',
        125 => '}',
        126 => '~',
        _ => '\n',
    }
}
