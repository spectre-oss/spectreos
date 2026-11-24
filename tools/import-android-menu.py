#!/usr/bin/env python3
"""Compile the supported Android menu XML subset to Bhoo Toolbar items."""
import argparse, json, re, tempfile, xml.etree.ElementTree as ET
from pathlib import Path
from android_png import decode_png
from android_vector import rasterize_vector, vector_tint
A='{http://schemas.android.com/apk/res/android}'
APP='{http://schemas.android.com/apk/res-auto}'
TOOLS='{http://schemas.android.com/tools}'
ICONS={'search':'Search','more_vert':'MoreVert','add':'Add','arrow_back':'ArrowBack','settings':'Settings','close':'Close','home':'Home','star':'Star','archive':'Archive'}
ANDROID_COLORS={
    '@android:color/transparent':'#00000000','@android:color/white':'#FFFFFFFF','@android:color/black':'#FF000000',
    '@android:color/darker_gray':'#FFAAAAAA','@android:color/primary_text_light':'#FF000000','@android:color/primary_text_dark':'#FFFFFFFF',
    '@android:color/secondary_text_light':'#8A000000','@android:color/secondary_text_dark':'#B3FFFFFF',
}
ANDROID_STRINGS={
    '@android:string/ok':'OK','@android:string/cancel':'Cancel','@android:string/yes':'Yes','@android:string/no':'No',
    '@android:string/copy':'Copy','@android:string/paste':'Paste','@android:string/cut':'Cut',
    '@android:string/selectAll':'Select all','@android:string/search_go':'Search',
}
THEME_COLORS={
    'colorPrimary':'#6750A4','colorOnPrimary':'#FFFFFF','colorSecondary':'#625B71','colorOnSecondary':'#FFFFFF',
    'colorSurface':'#FEF7FF','colorOnSurface':'#1D1B20','colorOnSurfaceVariant':'#49454F','colorError':'#B3261E',
    'colorAccent':'#6750A4','colorControlNormal':'#49454F','colorControlActivated':'#6750A4',
    'textColorPrimary':'#1D1B20','textColorSecondary':'#49454F','textColorPrimaryInverse':'#F5EFF7',
}
def compile_menu(path,resources=None,density=160):
    if not 1<=density<=1000:raise ValueError("Invalid target density")
    if resources is None:resource_roots=[]
    elif isinstance(resources,(str,Path)):resource_roots=[Path(resources)]
    else:resource_roots=[Path(r) for r in resources if r is not None]
    pool={};theme_attrs={};images={};tints={}
    for res_dir in resource_roots:
        for file in sorted(res_dir.glob('values/*.xml')):
            try:vroot=ET.parse(file).getroot()
            except ET.ParseError:continue
            for element in vroot:
                if element.tag in {'string','color','bool','integer'} and 'name' in element.attrib:
                    raw_val=''.join(element.itertext())
                    pool.setdefault('@'+element.tag+'/'+element.attrib['name'],raw_val if element.tag=='string' else raw_val.strip())
                elif element.tag=='item' and element.attrib.get('type') in {'string','color','bool','integer'} and 'name' in element.attrib:
                    kind=element.attrib['type'];raw_val=''.join(element.itertext())
                    pool.setdefault('@'+kind+'/'+element.attrib['name'],raw_val if kind=='string' else raw_val.strip())
                elif element.tag=='style':
                    for item in element:
                        if item.tag=='item' and item.attrib.get('name'):
                            theme_attrs.setdefault(item.attrib['name'].strip(),''.join(item.itertext()).strip())
    def strip_tools(attrib):
        return {k:v for k,v in attrib.items() if not k.startswith(TOOLS)}
    def parse_theme_ref(v):
        if v.startswith('?android:attr/'):return ('android',v[len('?android:attr/'):])
        if v.startswith('?attr/'):return ('app',v[len('?attr/'):])
        if v.startswith('?android:') and '/' not in v and len(v)>len('?android:'):return ('android',v[len('?android:'):])
        if v.startswith('?') and '/' not in v and ':' not in v and len(v)>1:return ('app',v[1:])
        return None
    def lookup_theme_attr(v):
        parsed=parse_theme_ref(v)
        if parsed is None:return None
        ns,name=parsed
        if ns=='android':
            return theme_attrs.get('android:'+name,theme_attrs.get(name))
        return theme_attrs.get(name,theme_attrs.get('android:'+name))
    def resolve(value):
        seen=set()
        while True:
            if value in ANDROID_STRINGS:return ANDROID_STRINGS[value]
            if not value.startswith('@string/'):break
            if value in seen:raise ValueError('Cyclic string resource '+value)
            seen.add(value)
            if value not in pool:raise ValueError('Unresolved string resource '+value)
            value=pool[value]
        if value.startswith('@'):raise ValueError('Unsupported resource '+value)
        return value
    def resolve_scalar(value):
        seen=set()
        while value.startswith(('@bool/','@integer/','@string/')):
            if value in seen:raise ValueError('Cyclic resource '+value)
            seen.add(value)
            if value not in pool:raise ValueError('Unresolved resource '+value)
            value=pool[value]
        return value
    def parse_color_hex(value):
        if not re.fullmatch(r'#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})',value):return None
        raw=value[1:]
        if len(raw) in (3,4):raw=''.join(ch*2 for ch in raw)
        if len(raw)==6:raw='ff'+raw
        return '#'+raw.lower()
    def resolve_tint_color(raw,seen=()):
        val=raw.strip()
        if val=='@null':return None
        if val in ('?attr/colorControlNormal','?android:attr/colorControlNormal'):return 'controlNormal'
        if val in seen:raise ValueError('Cyclic iconTint '+raw)
        if val in ANDROID_COLORS:return parse_color_hex(ANDROID_COLORS[val])
        if val.startswith('?'):
            tval=lookup_theme_attr(val)
            if tval is not None:return resolve_tint_color(tval,seen+(val,))
            parsed=parse_theme_ref(val)
            if parsed and parsed[1] in THEME_COLORS:
                if parsed[1]=='colorControlNormal':return 'controlNormal'
                return parse_color_hex(THEME_COLORS[parsed[1]])
            raise ValueError('Unsupported iconTint '+raw)
        if val.startswith('@color/'):
            if val in pool:return resolve_tint_color(pool[val],seen+(val,))
            cname=val.split('/',1)[1]
            for res_dir in resource_roots:
                cfile=res_dir/'color'/(cname+'.xml')
                if cfile.is_file():
                    croot=ET.parse(cfile).getroot()
                    if croot.tag=='selector':
                        chosen=None;first_item=None
                        for ci in croot:
                            if ci.tag!='item' or A+'color' not in ci.attrib:continue
                            if first_item is None:first_item=ci
                            if not [k for k in ci.attrib if k not in {A+'color',A+'alpha'} and not k.startswith(TOOLS)]:
                                chosen=ci;break
                        target=chosen if chosen is not None else first_item
                        if target is not None:
                            base_c=resolve_tint_color(target.attrib[A+'color'],seen+(val,))
                            if base_c and base_c.startswith('#') and A+'alpha' in target.attrib:
                                af=float(resolve_scalar(target.attrib[A+'alpha']))
                                a=int(int(base_c[1:3],16)*af+0.5)
                                base_c=f'#{a:02x}{base_c[3:]}'
                            return base_c
            raise ValueError('Unsupported iconTint '+raw)
        hx=parse_color_hex(val)
        if hx is None:raise ValueError('Unsupported iconTint '+raw)
        return hx
    def parse_show_as_action(raw_action):
        flags=[f.strip() for f in raw_action.split('|')]
        if len(flags)!=len(set(flags)) or any(flag not in {'never','ifRoom','always','withText','collapseActionView'} for flag in flags):
            raise ValueError('Unsupported showAsAction '+raw_action)
        placement=[flag for flag in flags if flag not in {'withText','collapseActionView'}]
        if len(placement)>1:raise ValueError('Conflicting showAsAction '+raw_action)
        return (placement[0] if placement else 'never'),'true' if 'withText' in flags else 'false'
    def rasterize_menu_vector(vector_path):
        try:
            img=rasterize_vector(vector_path,density)
            vt=vector_tint(vector_path)
            return img,vt
        except ValueError as err:
            if not resource_roots:raise
            msg=str(err)
            if not (msg.startswith('Unsupported vector tint') or msg.startswith('Unsupported vector color') or msg.startswith('Unsupported color') or msg.startswith('Invalid color') or msg.startswith('Unsupported vector attribute')):
                raise
            orig_root=ET.parse(vector_path).getroot()
            if orig_root.tag!='vector':raise
            def clone_elem(el):
                c=ET.Element(el.tag,strip_tools(el.attrib))
                for child in el:c.append(clone_elem(child))
                return c
            vroot=clone_elem(orig_root)
            vroot.attrib.pop(A+'autoMirrored',None)
            resolved_vt=None
            if A+'tint' in vroot.attrib:
                rt=resolve_tint_color(vroot.attrib[A+'tint'])
                if rt=='controlNormal':resolved_vt='controlNormal';vroot.attrib[A+'tint']='?attr/colorControlNormal'
                elif rt is not None:resolved_vt=rt;vroot.attrib[A+'tint']=rt
                else:vroot.attrib.pop(A+'tint',None)
            for el in vroot.iter():
                if el is vroot:continue
                for cattr in ('fillColor','strokeColor'):
                    if A+cattr in el.attrib and el.attrib[A+cattr].startswith(('@','?')):
                        rc=resolve_tint_color(el.attrib[A+cattr])
                        if rc=='controlNormal':rc=parse_color_hex(THEME_COLORS['colorControlNormal'])
                        if rc is not None:el.attrib[A+cattr]=rc
            with tempfile.TemporaryDirectory() as td:
                tmp_vec=Path(td)/'vector.xml'
                ET.ElementTree(vroot).write(tmp_vec,encoding='utf-8',xml_declaration=True)
                img=rasterize_vector(tmp_vec,density)
                vt=resolved_vt if resolved_vt is not None else vector_tint(tmp_vec)
            return img,vt
    root=ET.parse(path).getroot()
    if root.tag!='menu' or strip_tools(root.attrib):raise ValueError('Expected a plain menu root')
    def integer(value):
        value=resolve_scalar(value).strip()
        try:
            if not re.fullmatch(r'-?(?:0[xX][0-9a-fA-F]+|[0-9]+)',value):raise ValueError()
            result=int(value,16 if value.lstrip('-').lower().startswith('0x') else 10)
            if not -2147483648<=result<=4294967295:raise ValueError()
            return result
        except ValueError:raise ValueError('Invalid menu integer '+value)
    def category(value):
        value=resolve_scalar(value).strip()
        named={'container':1,'system':2,'secondary':3,'alternative':4}
        index=named[value] if value in named else (integer(value)&0xffff0000)>>16
        if index>=6:raise ValueError('Invalid menu category '+value)
        return index
    items=[];ids=set();entries=[]
    for element in root:
        if element.tag=='group':
            gattrs=strip_tools(element.attrib)
            if set(gattrs)-{A+'id',A+'enabled',A+'visible',A+'menuCategory',A+'orderInCategory',A+'checkableBehavior'}:raise ValueError('Unsupported menu group attribute')
            defaults={key:resolve_scalar(gattrs.get(A+key,'true')).strip() for key in ('enabled','visible')}
            if any(value not in {'true','false'} for value in defaults.values()):raise ValueError('Invalid menu boolean')
            group=gattrs.get(A+'id','')
            match=re.fullmatch(r'@\+?id/([a-zA-Z_][a-zA-Z0-9_]*)',group) if group else None
            if group and not match:raise ValueError('Invalid menu group id '+group)
            defaults['group']='bhoo::android_xml::view_id('+json.dumps(match.group(1))+')' if match else '0'
            defaults['category']=category(gattrs.get(A+'menuCategory','0'))
            defaults['order']=integer(gattrs.get(A+'orderInCategory','0'))&0xffff
            defaults['checkable']=gattrs.get(A+'checkableBehavior','none')
            if defaults['checkable'] not in {'none','all','single'}:raise ValueError('Invalid group checkableBehavior')
            entries.extend((child,defaults) for child in element)
        else:entries.append((element,{'enabled':'true','visible':'true','group':'0','category':0,'order':0,'checkable':'none'}))
    allowed_item_attrs=(
        {A+key for key in ('id','title','titleCondensed','icon','iconTint','iconTintMode','showAsAction','actionViewClass','enabled','visible','menuCategory','orderInCategory','checkable','checked','contentDescription','tooltipText')}
        | {APP+key for key in ('showAsAction','iconTint','iconTintMode','actionViewClass','contentDescription','tooltipText')}
    )
    for item,defaults in entries:
        if item.tag!='item' or len(item):raise ValueError('Only flat menu items are supported')
        iattrs=strip_tools(item.attrib)
        if set(iattrs)-allowed_item_attrs:raise ValueError('Unsupported menu item attribute')
        raw=iattrs.get(A+'id','');match=re.fullmatch(r'@\+?id/([a-zA-Z_][a-zA-Z0-9_]*)',raw)
        if not match:raise ValueError('Each menu item needs an @id')
        name=match.group(1)
        if name in ids:raise ValueError('Duplicate menu item id '+name)
        ids.add(name);title=resolve(iattrs.get(A+'title',''))
        if A+'titleCondensed' in iattrs:resolve(iattrs[A+'titleCondensed'])
        if A+'showAsAction' in iattrs:parse_show_as_action(iattrs[A+'showAsAction'])
        raw_action=iattrs.get(APP+'showAsAction',iattrs.get(A+'showAsAction','never'))
        action,with_text=parse_show_as_action(raw_action)
        for av_key in (APP+'actionViewClass',A+'actionViewClass'):
            if av_key in iattrs:
                avc=resolve(iattrs[av_key]).strip()
                if not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_$.]*',avc):raise ValueError('Invalid actionViewClass '+iattrs[av_key])
        for tm_key in (APP+'iconTintMode',A+'iconTintMode'):
            if tm_key in iattrs:
                tm=iattrs[tm_key].strip()
                if tm not in {'src_over','src_in','src_atop','multiply','screen','add'}:raise ValueError('Unsupported iconTintMode '+tm)
        item_icon_tint=None
        if APP+'iconTint' in iattrs:item_icon_tint=resolve_tint_color(iattrs[APP+'iconTint'])
        elif A+'iconTint' in iattrs:item_icon_tint=resolve_tint_color(iattrs[A+'iconTint'])
        enabled=resolve_scalar(iattrs.get(A+'enabled',defaults['enabled'])).strip()
        visible=resolve_scalar(iattrs.get(A+'visible',defaults['visible'])).strip()
        if enabled not in {'true','false'} or visible not in {'true','false'}:raise ValueError('Invalid menu boolean')
        checkable=resolve_scalar(iattrs.get(A+'checkable',str(defaults['checkable']!='none').lower())).strip()
        checked=resolve_scalar(iattrs.get(A+'checked','false')).strip()
        if checkable not in {'true','false'} or checked not in {'true','false'}:raise ValueError('Invalid menu boolean')
        exclusive=str(defaults['checkable']=='single' and A+'checkable' not in iattrs).lower()
        item_category=category(iattrs.get(A+'menuCategory')) if A+'menuCategory' in iattrs else defaults['category']
        item_order=integer(iattrs.get(A+'orderInCategory'))&0xffff if A+'orderInCategory' in iattrs else defaults['order']
        icon=iattrs.get(A+'icon','');icon_expr='None';drawable_expr='None'
        if icon.startswith('?'):
            ticon=lookup_theme_attr(icon)
            if ticon is not None:icon=ticon.strip()
        if icon:
            match=re.fullmatch(r'@drawable/([a-z][a-z0-9_]*)',icon)
            if not match:raise ValueError('Unsupported menu icon '+icon)
            icon_name=match.group(1);candidates=[];vector=None
            for res_dir in resource_roots:
                local_cands=[]
                for folder,dpi in [('drawable',160),('drawable-mdpi',160),('drawable-hdpi',240),('drawable-xhdpi',320),('drawable-xxhdpi',480),('drawable-xxxhdpi',640)]:
                    file=res_dir/folder/(icon_name+'.png')
                    if file.is_file():local_cands.append((dpi,file))
                if len({dpi for dpi,_ in local_cands})!=len(local_cands):raise ValueError('Ambiguous PNG drawable density '+icon)
                nodpi=res_dir/'drawable-nodpi'/(icon_name+'.png')
                if not local_cands and nodpi.is_file():local_cands.append((160,nodpi))
                local_vec=None
                for vfolder in ('drawable-anydpi-v24','drawable-anydpi-v21','drawable-v24','drawable-v21','drawable'):
                    vf=res_dir/vfolder/(icon_name+'.xml')
                    if vf.is_file():local_vec=vf;break
                if local_cands or local_vec is not None:
                    candidates=local_cands;vector=local_vec;break
            if candidates and vector is not None and vector.parent.name=='drawable':raise ValueError('Ambiguous menu drawable '+icon)
            if vector is not None and (not candidates or vector.parent.name.startswith('drawable-anydpi')):
                img,vt=rasterize_menu_vector(vector)
                images[icon_name]=img
                if vt is not None:tints[icon_name]=vt
                drawable_expr='Some('+json.dumps(icon_name)+'.into())'
            elif candidates:
                higher=sorted((dpi,file) for dpi,file in candidates if dpi>=density)
                source_density,file=higher[0] if higher else max(candidates)
                images[icon_name]=(*decode_png(file),160.0/source_density);drawable_expr='Some('+json.dumps(icon_name)+'.into())'
            elif icon_name in ICONS:icon_expr='Some(bhoo::MaterialIcon::'+ICONS[icon_name]+')'
            else:raise ValueError('Unresolved PNG menu icon '+icon)
            if drawable_expr!='None' and item_icon_tint is not None:
                tints[icon_name]=item_icon_tint
        items.append(((1,4,5,3,2,0)[item_category],item_order,'        Item {id:bhoo::android_xml::view_id('+json.dumps(name)+'),group_id:'+defaults['group']+',label:'+json.dumps(title,ensure_ascii=False)+'.into(),icon:'+icon_expr+',icon_drawable:'+drawable_expr+',show_text_with_icon:'+with_text+',show_as_action:'+'bhoo::android_xml::AndroidShowAsAction::'+{'never':'Never','ifRoom':'IfRoom','always':'Always'}[action]+',enabled:'+enabled+',visible:'+visible+',checkable:'+checkable+',checked:'+checked+',exclusive:'+exclusive+'},'))
    items.sort(key=lambda item:item[:2])
    items=[item[2] for item in items]
    result='// Generated by tools/import-android-menu.py. Edit the XML source.\npub fn items() -> Vec<bhoo::android_xml::AndroidToolbarMenuItem> {\n    use bhoo::android_xml::AndroidToolbarMenuItem as Item;\n    vec![\n'+'\n'.join(items)+'\n    ]\n}\n'
    result+='pub fn install_drawables(state: &mut bhoo::android_xml::LayoutState) {\n'
    for name,tint in tints.items():
        if tint=='controlNormal':value='bhoo::android_xml::AndroidDrawableTint::ControlNormal'
        else:
            alpha,red,green,blue=(int(tint[i:i+2],16) for i in (1,3,5,7))
            value=f'bhoo::android_xml::AndroidDrawableTint::Fixed(bhoo::Color({red},{green},{blue},{alpha}))'
        result+='    state.drawable_tints.insert('+json.dumps(name)+'.into(),'+value+');\n'
    for name,(width,height,pixels,scale) in images.items():
        result+='    state.image_scales.insert('+json.dumps(name)+'.into(),'+str(scale)+');\n'
        result+='    state.images.entry('+json.dumps(name)+'.into()).or_insert_with(|| bhoo::Image::rgba('+str(width)+','+str(height)+',vec!['+','.join(map(str,pixels))+']).expect("validated menu drawable"));\n'
    return result+'}\n'
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('input',type=Path);p.add_argument('output',type=Path);p.add_argument('--resources',type=Path);p.add_argument('--density',type=int,default=160);a=p.parse_args()
    try:a.output.write_text(compile_menu(a.input,a.resources,a.density))
    except (ValueError,ET.ParseError) as e:p.exit(1,str(e)+'\n')
