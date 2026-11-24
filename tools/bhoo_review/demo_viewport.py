"""Opt-in native scrolling for long, vertically static review demos.

Do not use this on demos that implement their own vertical drag gestures.
"""


def add_scroll_viewport(html, height):
    if isinstance(height, bool) or not isinstance(height, int) or not 500 <= height <= 4096:
        raise ValueError('demo_height must be an integer between 500 and 4096 CSS pixels')
    text = html.decode('utf-8')
    if 'data-embed="true"' not in text or '</head>' not in text:
        raise ValueError('Scrollable review requires an embedded Bhoo demo')
    if 'id="bhoo-review-scroll"' in text:
        raise ValueError('Demo already has a review scroll viewport')
    style = f'''<style id="bhoo-review-scroll">
html[data-embed=true]{{overflow-y:auto!important;height:100%!important}}
html[data-embed=true] body{{height:{height}px!important;overflow:clip!important;position:relative!important}}
html[data-embed=true] #canvas-wrap{{position:absolute!important;top:0!important;left:0!important;width:100%!important;height:{height}px!important}}
html[data-embed=true] #preview{{height:100%!important;touch-action:pan-y!important}}
</style>'''
    return text.replace('</head>', style + '</head>', 1).encode('utf-8')
