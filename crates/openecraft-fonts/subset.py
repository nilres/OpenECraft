"""Regenerates the bundled Chinese, Japanese and Korean fonts (fonts/*.ttf).

The fonts are subsets of the Noto Sans SC / JP / KR variable fonts of Google Fonts
(https://github.com/google/fonts, ofl/notosans{sc,jp,kr}, SIL Open Font License 1.1)
at weight 350 with the characters of the translations of their language. Run it after
a translation gained characters (`openecraft/src/glyphs/tests.rs` fails then):

    pip install fonttools
    python subset.py <folder with NotoSansSC[wght].ttf, NotoSansJP[wght].ttf, NotoSansKR[wght].ttf>
"""
import glob
import os
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
        out = os.path.join(HERE, 'fonts', dst)
        font.save(out)
        print(out, os.path.getsize(out), 'bytes')


if __name__ == '__main__':
    main(sys.argv[1])
