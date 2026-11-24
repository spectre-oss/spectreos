#!/usr/bin/env python3
"""Build the Rust demo and the optional pinned Angular Material comparison reference."""
import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess
import zipfile

ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'codes/apps/bhoo-web'
SITE=APP/'site'
TDD=ROOT/'codes/libs/ui/libbhoo/docs'
SCENES={'button':1,'icon-button':2,'checkbox':3,'switch':4,'radio-group':5,'slider':6,'range-slider':7,'tabs':8,'chip':9,'list-and-list-item':10,'virtualized-list':11,'progress-indicator':12,'tooltip':13,'badge':14,'divider':15,'focus-ring':16,'typography-text':17,'scroll-area-viewport':18}
SCENES.update({slug:19+i for i,slug in enumerate((APP/'src/families.txt').read_text().splitlines())})
SNIPPETS={
'button':'if frame.button_with(1, "Summon a ghost", Options {\n    variant: ButtonVariant::Filled,\n    ..Options::default()\n}).clicked {\n    ghosts += 1;\n}',
'icon-button':'frame.icon_button(1, "+", "Add item", Options::default());',
'checkbox':'frame.checkbox_bool(1, "Binary selection", &mut checked, true);\nframe.checkbox(2, "Three states", &mut check_state, true);',
'switch':'frame.switch(1, "Notifications", &mut notifications, true);',
'radio-group':'let choices = [\n    Choice { id: 301, label: "Balanced", enabled: true },\n    Choice { id: 302, label: "Performance", enabled: true },\n];\nframe.radio_group(300, "Power mode", &choices, &mut selected, true);',
'slider':'frame.slider(1, "Volume", &mut volume, 0.0..=100.0, 1.0, true);',
'range-slider':'frame.range_slider([1, 2], "Range", &mut range,\n    0.0..=100.0, 1.0, true);',
'chip':'if frame.chip(1, "Installed", selected, true).clicked {\n    selected = !selected;\n}',
'tabs':'frame.tabs(1, &["Overview", "Activity", "Settings"],\n    &mut selected_tab, true);',
'list-and-list-item':'if frame.list_item(1, "Applications", selected, true).clicked {\n    selected = true;\n}',
'virtualized-list':'frame.virtual_list(1, 280.0, 100_000, &mut list,\n    |frame, index, selected| {\n        frame.list_item(index as u64 + 100, "Application",\n            selected, true)\n    });',
'progress-indicator':'frame.progress(1, "Download", Some(progress));\nframe.progress(2, "Working", None); // indeterminate',
'badge':'let response = frame.icon_button(1, "✉", "Inbox", Options::default());\nframe.badge_at(response.id, icon_bounds, BadgeContent::Label("3"), "3 messages", false);',
'divider':'frame.divider();\nframe.divider_at(bounds, DividerOrientation::Vertical, 0.0, 0.0);',
'tooltip':'let response = frame.button(1, "Hover or focus me");\nframe.tooltip(2, response, "A small, useful detail");',
'focus-ring':'// Focus is shared by all enabled, visible controls.\n// Tab / Shift+Tab navigate. Space / Enter activate.\nframe.button(1, "First action");\nframe.button(2, "Second action");',
'typography-text':'frame.label("Bhoo / interface text");\n// Unchanged labels share bounded Arc<str> cache entries.',
'scroll-area-viewport':'let bounds = frame.reserve(200.);\nframe.scroll_viewport(900, bounds, &mut offset, 112.);\n// Paint content within bounds, translating y by -offset.',
}
def clean(s):return re.sub(r'\[([^\]]+)\]\([^)]+\)',r'\1',s).replace('**','').replace('`','').strip()
def section(text,name):
    m=re.search(r'^## '+re.escape(name)+r'\n(.*?)(?=^## |\Z)',text,re.M|re.S)
    return m.group(1).strip() if m else ''
def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--rust-only', action='store_true', help='Build the original playground without the Angular reference')
    args=parser.parse_args()
    if not args.rust_only:
        subprocess.run(['node', str(ROOT/'tools/build-bhoo-angular.mjs')], cwd=ROOT, check=True)
    for xml in sorted((APP/'android-examples').glob('*.xml')):
        output = APP/'src'/('android_'+xml.stem.replace('-', '_')+'.rs')
        subprocess.run(['python3', str(ROOT/'tools/import-android-layout.py'), str(xml), str(output), '--resources', str(APP/'android-examples/res')], check=True)
    subprocess.run(['python3',str(ROOT/'tools/import-android-menu.py'),str(APP/'android-examples/res/menu/toolbar_actions.xml'),str(APP/'src/android_toolbar_menu.rs'),'--resources',str(APP/'android-examples/res')],check=True)
    for name in ('chats','conversation'):
        subprocess.run(['python3',str(ROOT/'tools/import-android-layout.py'),str(APP/'android-demo'/f'{name}.xml'),str(APP/'src'/f'android_{name}.rs'),'--resources',str(APP/'android-demo/res')],check=True)
    for name in ('inbox','detail','compose'):
        subprocess.run(['python3',str(ROOT/'tools/import-android-layout.py'),str(APP/'android-gmail/res/layout'/f'gmail_{name}.xml'),str(APP/'src'/f'android_gmail_{name}.rs'),'--resources',str(APP/'android-gmail/res')],check=True)
    for name in ('home','detail','editor','settings'):
        subprocess.run(['python3',str(ROOT/'tools/import-android-layout.py'),str(APP/'android-contacts/res/layout'/f'contacts_{name}.xml'),str(APP/'src'/f'android_contacts_{name}.rs'),'--resources',str(APP/'android-contacts/res')],check=True)
    subprocess.run(['python3',str(ROOT/'tools/import-android-app-tests.py')],check=True)
    targets=subprocess.check_output(['rustup','target','list','--installed'],text=True)
    if 'wasm32-unknown-unknown' not in targets:raise SystemExit('Install the small WASM standard library first: rustup target add wasm32-unknown-unknown')
    subprocess.run(['cargo','build','--locked','--manifest-path',str(APP/'Cargo.toml'),'--release','--target','wasm32-unknown-unknown','--target-dir',str(ROOT/'context/build/bhoo-web')],cwd=ROOT,check=True)
    wasm=ROOT/'context/build/bhoo-web/wasm32-unknown-unknown/release/bhoo_web_demo.wasm'
    shutil.copy2(wasm,SITE/'bhoo.wasm')
    coverage=json.loads((APP/'demo-coverage.json').read_text())
    components=[]
    for path in (TDD/'components').glob('*.md'):
        if path.name=='README.md':continue
        text=path.read_text();name=text.splitlines()[0][2:];slug=path.stem
        cat=re.search(r'\| Category \| ([^|]+) \|',text).group(1).strip()
        scope=section(text,'Scope and variants').split('\n\n')[0]
        refs=[];seen=set()
        for label,url in re.findall(r'\[([^\]]+)\]\((https://[^)]+)\)',text):
            if url in seen:continue
            seen.add(url);refs.append({'label':clean(label),'url':url})
        components.append({'slug':slug,'name':name,'category':cat,'scene':SCENES.get(slug),'summary':clean(scope),
            'preview':coverage.get(slug),
            'status':clean(re.search(r'^Status: (.+)$',text,re.M).group(1)),
            'contract':clean(section(text,'State and interaction contract').split('\n\n')[0]),
            'motion':clean(section(text,'Motion and animation').split('\n\n')[0]),
            'reduced':clean(re.search(r'\*\*Reduced motion:\*\* (.+)',text).group(1)),
            'checks':[clean(x[2:]) for x in section(text,'Acceptance criteria').splitlines() if x.startswith('- ')],
            'references':refs,'code':SNIPPETS.get(slug, ' // Interactive Rust composition in codes/apps/bhoo-web/src/gallery.rs\n // Family: '+slug+'\n gallery.show(frame, '+str(SCENES[slug])+');')})
    assert len(components)==len(SCENES), 'Every catalog entry needs a Rust scene'
    assert {c['slug'] for c in components}==set(SCENES), 'Scene registry differs from catalog'
    # Preserve the researched catalog order instead of filesystem order.
    catalog=(TDD/'components.md').read_text()
    ordering=re.findall(r'\]\(components/([^/)]+)\.md\)',catalog)
    order={slug:i for i,slug in enumerate(ordering)}
    components.sort(key=lambda c:order.get(c['slug'],999))
    overview={'slug':'overview','name':'Component playground','category':'Getting started','scene':0,
        'summary':'A little spirit for every interface. Explore the actual Bhoo Rust core in your browser — with live state, keyboard input, and interruptible motion.',
        'status':f'All {len(SCENES)} entries have Rust previews: 18 initial widget demos and {len(SCENES)-18} experimental compositions. Full family and motion parity remain open.',
        'contract':'Application state stays with the caller. Controls share pointer capture, cancellation, keyboard focus, and activation. Tab moves focus; Space and Enter activate supported controls.',
        'motion':'Normalized effects use the same analytic critically damped springs as the native demo. The browser requests frames only for input, an active effect, or a semantic deadline.',
        'reduced':'Active effects snap to their targets. Indeterminate movement uses a static busy treatment.',
        'checks':['The browser runs compiled Rust, not mock JavaScript widgets.','Experimental compositions and remaining production scope are labeled per family.'],
        'references':[{'label':'CUI — browser component explorer','url':'https://yappologistic.github.io/CUI/'},{'label':'Material Design 3 — components','url':'https://m3.material.io/components'}],
        'code':'frame.switch(1, "Notifications", &mut notifications, true);\nframe.checkbox(2, "Three states", &mut state, true);\nframe.slider(3, "Volume", &mut volume, 0.0..=100.0, 1.0, true);'}
    payload={'components':components,'overview':overview,'live_count':len(SCENES),'wasm_bytes':wasm.stat().st_size}
    (SITE/'catalog.json').write_text(json.dumps(payload,separators=(',',':'))+'\n')
    legal=SITE/'licenses';legal.mkdir(exist_ok=True)
    for src,name in [(ROOT/'LICENSE','GPL-3.0.txt'),(ROOT/'THIRD_PARTY_NOTICES.md','NOTICES.md'),(TDD/'reference-licenses/compose-material3/LICENSE.txt','Apache-2.0.txt'),(TDD/'reference-licenses/unicode-segmentation/LICENSE-MIT.txt','Unicode-Segmentation-MIT.txt'),(TDD/'reference-licenses/unicode-width/LICENSE-MIT.txt','Unicode-Width-MIT.txt'),(TDD/'reference-licenses/unicode-width/Unicode-License.txt','Unicode-License.txt'),(TDD/'reference-licenses/unicode-linebreak/Unicode-DFS-2016.txt','Unicode-DFS-2016.txt')]:shutil.copy2(src,legal/name)
    # Supply the corresponding source for the demo without shipping framework repos or assets.
    source_files=[ROOT/'LICENSE',ROOT/'THIRD_PARTY_NOTICES.md',ROOT/'tools/build-bhoo-web.py',ROOT/'tools/build-bhoo-angular.mjs',ROOT/'tools/serve-bhoo-web.py',ROOT/'tools/test-bhoo-web.mjs',ROOT/'tools/BhooShapes.java',ROOT/'tools/generate-bhoo-loading.py',ROOT/'tools/generate-beercss-shapes.py',ROOT/'tools/generate-material-navigation-icons.py',ROOT/'tools/generate-material-symbols.py',ROOT/'tools/test-bhoo-wayland.py']
    for name in ['generate-android-zoomcontrols.py','generate-bhoo-test-video.py','generate-android-clock-shapes.py','import-android-app-tests.py','build-bhoo-app-tests.py','test-bhoo-app-tests.mjs','import-android-layout.py','import-android-menu.py','android_vector.py','android_png.py','test-android-layout.py','test-bhoo-comparison.mjs','generate-bhoo-media.mjs','bhoo-png-diff.mjs','build-bhoo-webxdc.py']:
        path=ROOT/'tools'/name
        if path.exists():source_files.append(path)
    for folder in [APP,ROOT/'codes/apps/wisp',ROOT/'codes/libs/ui/libbhoo',TDD]:
        for path in folder.rglob('*'):
            relative=path.relative_to(folder)
            if not path.is_file() or any(part in {'target','upstream'} for part in relative.parts):continue
            if folder==TDD and relative.parts[0]=='reviews':continue  # Private, concurrently updated review records.
            if folder==APP and relative.parts[:2]==('site','angular'):continue
            if folder==APP and relative.parts[0]=='angular-reference':
                source_files.append(path)
                continue
            if folder==APP and path.name not in {'families.txt','demo-coverage.json'} and not path.name.startswith(('LICENSE','NOTICE','COPYING')) and path.suffix not in {'.rs','.toml','.lock','.md','.html','.js','.css','.svg','.bin','.txt','.xml','.json','.png'}:continue
            source_files.append(path)
    with zipfile.ZipFile(SITE/'source.zip','w',compression=zipfile.ZIP_DEFLATED) as archive:
        for path in dict.fromkeys(source_files):archive.write(path,path.relative_to(ROOT))
    print(f'Built {len(components)} catalog pages / {len(SCENES)} live previews. WASM: {wasm.stat().st_size:,} bytes. Site: {SITE}',flush=True)
if __name__=='__main__':main()
