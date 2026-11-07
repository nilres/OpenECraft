"""Regenerates the bundled Chinese, Japanese and Korean fonts (fonts/*.ttf).

The fonts are subsets of the Noto Sans SC / JP / KR variable fonts of Google Fonts
(https://github.com/google/fonts, ofl/notosans{sc,jp,kr}, SIL Open Font License 1.1)
at weight 350 with the characters of the translations of their language. Run it after
a translation gained characters (`openecraft/src/glyphs/tests.rs` fails then):

    pip install fonttools
    python subset.py <folder with NotoSansSC[wght].ttf, NotoSansJP[wght].ttf, NotoSansKR[wght].ttf>

The OFL allows subsets as modified versions under two conditions it checks: they keep
the license (`fonts/OFL.txt`, the same text as the `OFL.txt` beside the source fonts if
it is there) and their names use none of the Reserved Font Names of the copyright
notice ('Source' for the Noto Sans CJK fonts derived from Adobe's Source Han Sans).
"""
import glob
import os
import re
import sys

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

HERE = os.path.dirname(os.path.abspath(__file__))
CATALOG = os.path.join(HERE, '..', 'openecraft-i18n', 'src', 'catalog')

# (language, source font, bundled font, language name as shown in the language menu)
FONTS = [
    ('zh', 'NotoSansSC[wght].ttf', 'NotoSansSC-Subset.ttf', '简体中文'),
    ('ja', 'NotoSansJP[wght].ttf', 'NotoSansJP-Subset.ttf', '日本語'),
    ('ko', 'NotoSansKR[wght].ttf', 'NotoSansKR-Subset.ttf', '한국어'),
]

# Punctuation the translations may use beside the catalog's own characters.
EXTRA = '、。，．：；？！「」『』（）【】《》〈〉・ー～…‥―〜％＋－＝／'


def characters(lang, name):
    chars = set(name) | set(EXTRA)
    for path in sorted(glob.glob(os.path.join(CATALOG, '*', lang + '.tsv'))):
        with open(path, encoding='utf-8') as f:
            for line in f:
                if not line.startswith('#'):
                    chars.update(line.rstrip('\r\n').split('\t', 1)[-1])
    return sorted(c for c in chars if ord(c) > 0x7F and not c.isspace())


def reserved_names(font):
    """The Reserved Font Names of the copyright notice of a font."""
    notice = font['name'].getDebugName(0) or ''
    return re.findall(r"Reserved Font Names? ['\"]([^'\"]+)['\"]", notice)


def check_license(font, source):
    """Fails when the subset would break the OFL: a reserved name in its font names, or a
    license text that differs from the one of the source fonts."""
    reserved = reserved_names(font)
    for name_id in (1, 3, 4, 6, 16, 21):
        value = font['name'].getDebugName(name_id) or ''
        for r in reserved:
            if r.lower() in value.lower():
                sys.exit(f'name {name_id} "{value}" uses the Reserved Font Name "{r}"')
    own = os.path.join(HERE, 'fonts', 'OFL.txt')
    theirs = os.path.join(source, 'OFL.txt')
    if not os.path.exists(own):
        sys.exit('fonts/OFL.txt is missing')
    if os.path.exists(theirs):
        text = lambda p: open(p, encoding='utf-8').read().replace('\r\n', '\n').strip()
        if text(own) != text(theirs):
            sys.exit('fonts/OFL.txt differs from the OFL.txt of the source fonts: copy it over')


def main(source):
    for lang, src, dst, name in FONTS:
        font = TTFont(os.path.join(source, src))
        font = instancer.instantiateVariableFont(font, {'wght': 350})
        options = subset.Options()
        options.hinting = False
        options.layout_features = []
        options.name_IDs = ['*']
        options.notdef_outline = True
        sub = subset.Subsetter(options)
        sub.populate(unicodes=[ord(c) for c in characters(lang, name)])
        sub.subset(font)
        check_license(font, source)
        out = os.path.join(HERE, 'fonts', dst)
        font.save(out)
        print(out, os.path.getsize(out), 'bytes')


if __name__ == '__main__':
    main(sys.argv[1])
