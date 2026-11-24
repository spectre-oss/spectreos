#!/usr/bin/env python3
"""Compile pinned, audited Android row XML into Bhoo static trees.
Original screen XML is retained as reference. Android controllers are replaced by
local Rust controllers; this does not import Android activities or Compose.
"""
import json, shutil, xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
APP=ROOT/'codes/apps/bhoo-web'
FIX=APP/'android-app-tests'
PROFILES={
'fossify':('item_conversation',['activity_main','activity_thread','layout_thread_send_message_holder']),
'deltachat':('conversation_list_item_view',['conversation_list_activity','conversation_activity','conversation_input_panel']),
'thunderbird':('message_list_item',['message_list','message_list_fragment','message','message_compose']),
'sunflower':('list_item_plant',['fragment_view_pager','fragment_plant_list','fragment_plant_detail','fragment_garden'])}
ALIASES={'activity_margin':'16dp','medium_margin':'8dp','small_margin':'4dp','tiny_margin':'2dp','two_line_list_item_min_height':'72dp','list_icon_size_medium':'48dp','small_icon_size':'24dp','list_primary_text_size':'16sp','list_secondary_text_size':'14sp','normal_text_size':'14sp','list_tertiary_text_size':'12sp'}
OVERRIDES={
'fossify':{'conversation_image':{'kind':'Avatar'},'draft_indicator':{'visibility':'gone'},'conversation_body_short':{'layout_constraintStart_toStartOf':'@id/conversation_address','layout_constraintStart_toEndOf':'','layout_constraintEnd_toEndOf':'parent','layout_constraintEnd_toStartOf':''}},
'deltachat':{'avatar':{'kind':'Avatar'},'from_text':{'textSize':'16sp'},'subject':{'textSize':'14sp'},'date':{'textSize':'12sp'},'request_badge':{'visibility':'gone'},'archived_badge':{'visibility':'gone'},'unread_indicator':{'visibility':'gone'},'divider':{'background':'#eeeeee'}},
# The original chain/barriers depend on runtime icon visibility. Fix the local
# fixture to its no-attachment/no-thread-counter state and record each change.
'thunderbird':{'contact_picture':{'kind':'Avatar'},'status':{'visibility':'gone'},'thread_count':{'visibility':'gone'},'attachment':{'visibility':'gone'},'star':{'visibility':'gone'},'account_color_chip':{'visibility':'gone'},'subject':{'layout_width':'0dp','textSize':'16sp','textStyle':'bold','layout_constraintStart_toStartOf':'@id/start_keyline','layout_constraintStart_toEndOf':'','layout_constraintEnd_toStartOf':'@id/date'},'date':{'textSize':'12sp','layout_constraintTop_toTopOf':'@id/top_guideline'},'preview':{'textSize':'14sp','maxLines':'1','layout_constraintStart_toStartOf':'@id/start_keyline','layout_constraintStart_toEndOf':'','layout_constraintEnd_toEndOf':'parent','layout_constraintEnd_toStartOf':''},'divider':{'background':'#eeeeee'}},
'sunflower':{'plant_item_image':{'kind':'Picture'},'plant_item_title':{'textSize':'16sp'}}}
def prepare(name,row,screens):
 dest=FIX/name
 if not dest.exists():
  src=ROOT/'context/android-app-tests'/name; manifest=json.loads((src/'manifest.json').read_text()); res=src/manifest['resource_root']; dest.mkdir(parents=True)
  shutil.copy2(src/'manifest.json',dest/'manifest.json')
  for f in src.glob('*'):
   if f.is_file() and f.name.lower().startswith(('license','copying','notice')):shutil.copy2(f,dest/f.name)
  for folder in ['values']:
   shutil.copytree(res/folder,dest/folder,dirs_exist_ok=True)
  (dest/'layout').mkdir()
  for file in [row]+screens:
   p=res/'layout'/f'{file}.xml'
   if p.exists():shutil.copy2(p,dest/'layout'/p.name)
 return dest

def compile_tree(name,dest,row):
 resources={}
 for p in sorted((dest/'values').glob('*.xml')):
  for e in ET.parse(p).getroot():
   if e.tag in ['dimen','color','string','integer']:resources[f'@{e.tag}/{e.get("name")}']=''.join(e.itertext())
 if name=='fossify':resources.update({'@dimen/'+k:v for k,v in ALIASES.items()})
 changes=[]; unresolved=set(); classes=set(); ignored=set()
 handled={'layout_width','layout_height','visibility','text','textSize','textStyle','textColor','background','cornerRadius','tint','gravity','maxLines','singleLine','orientation','layout_weight','layout_gravity','minWidth','minHeight','layout_constraintGuide_begin','layout_constraintGuide_end','constraint_referenced_ids','barrierDirection','layout_below','layout_toEndOf','layout_toRightOf','layout_toStartOf','layout_above','layout_alignParentEnd','layout_alignParentBottom','layout_centerHorizontal','layout_centerVertical','layout_centerInParent','layout_constraintHorizontal_bias','layout_constraintVertical_bias','icon','kind'}
 handled.update('layout_constraint'+side+'_to'+end+'Of' for side,end in [('Start','Start'),('Start','End'),('End','Start'),('End','End'),('Top','Top'),('Top','Bottom'),('Bottom','Top'),('Bottom','Bottom')])
 handled.update(prefix+suffix for prefix in ['padding','layout_margin'] for suffix in ['','Horizontal','Vertical','Start','End','Top','Bottom','Left','Right'])
 def resolve(v):
  seen=set()
  while v in resources and v not in seen:seen.add(v);v=resources[v]
  if v.startswith(('@dimen/','@color/','@string/','?')):unresolved.add(v);return ''
  if v.startswith('@{'):return ''
  return v
 def node(e):
  if e.tag=='layout':return node(next(c for c in e if c.tag!='data'))
  tag=e.tag.split('.')[-1];classes.add(e.tag)
  a={k.split('}')[-1]:resolve(v) for k,v in e.attrib.items() if not k.startswith('{http://schemas.android.com/tools}')}
  ident=a.pop('id','').split('/')[-1]
  kind='Text' if 'TextView' in tag else 'Linear' if tag=='LinearLayout' else 'Avatar' if tag=='AvatarView' else tag if tag in ['Guideline','Barrier'] else 'Container'
  if tag=='ImageView':kind='Icon'
  mods=OVERRIDES[name].get(ident,{})
  if mods:changes.append({'id':ident,'overrides':mods});a.update(mods)
  kind=a.pop('kind',kind)
  if tag=='MaskedCardView':a.update(background='#f1f5ed',cornerRadius='12dp')
  if name=='sunflower' and tag=='ConstraintLayout':
   a['layout_height']='match_parent';changes.append({'class':tag,'overrides':{'layout_height':'match_parent'},'reason':'card height allocated by virtual grid instead of Android wrap-content constraint measurement'})
  # Rows expose a single outer target, avoiding nested target stealing on touch.
  a.pop('clickable',None)
  ignored.update(k for k in a if k not in handled)
  children=[node(c) for c in e if c.tag!='data']
  return 'NativeNode { kind: '+json.dumps(kind)+', id: '+json.dumps(ident)+', attrs: &['+','.join('('+json.dumps(k)+','+json.dumps(v)+')' for k,v in a.items() if v)+'], children: &['+','.join(children)+'] }'
 tree=node(ET.parse(dest/'layout'/f'{row}.xml').getroot())
 report={'row':row,'original_classes':sorted(classes),'attributes_not_executed_by_bridge':sorted(ignored),'explicit_adapters':changes,'unresolved_nonlayout_theme_attributes':sorted(unresolved),'limits':['Only row/card XML is executed; screen XML guides the Rust controller.','No Android services, resource qualifiers, bidi constraints, arbitrary custom views or data binding runtime.','Theme/drawable selectors are replaced by Bhoo colors and icons.','Plant photographs replaced by locally drawn placeholder illustrations.'],'dependency_dimension_aliases':ALIASES if name=='fossify' else {}}
 (dest/'ADAPTERS.json').write_text(json.dumps(report,indent=2)+'\n')
 return f'pub static {name.upper()}: NativeNode = {tree};\n'
output='// Generated by tools/import-android-app-tests.py. Original licenses in android-app-tests.\nuse bhoo::android_reference::NativeNode;\n'
for name,(row,screens) in PROFILES.items():output+=compile_tree(name,prepare(name,row,screens),row)
(APP/'src/android_app_rows.rs').write_text(output)
print('Compiled four original XML row templates; explicit adapter reports saved.')
