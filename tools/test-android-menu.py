#!/usr/bin/env python3
"""Test suite for the import-android-menu module.

Provides unit tests that compile Android XML menu files using the import-android-menu tool.
No command-line interface; executed directly via Python's unittest runner.
"""
import importlib.util, tempfile, unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('menu',Path(__file__).with_name('import-android-menu.py'));mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
class MenuTests(unittest.TestCase):
 def test_checkable_groups_and_item_overrides(self):
  with tempfile.TemporaryDirectory() as tmp:
   xml=Path(tmp)/'menu.xml';xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><group android:id="@+id/g" android:checkableBehavior="single"><item android:id="@+id/a" android:checked="true"/><item android:id="@+id/b" android:checkable="true"/><item android:id="@+id/c" android:checkable="false"/></group></menu>')
   out=mod.compile_menu(xml);lines={name:next(line for line in out.splitlines() if 'view_id("'+name+'")' in line) for name in ('a','b','c')}
   self.assertIn('checkable:true,checked:true,exclusive:true',lines['a']);self.assertIn('checkable:true,checked:false,exclusive:false',lines['b']);self.assertIn('checkable:false,checked:false,exclusive:false',lines['c'])
   xml.write_text(xml.read_text().replace('checkableBehavior="single"','checkableBehavior="invalid"'))
   with self.assertRaisesRegex(ValueError,'Invalid group checkableBehavior'):mod.compile_menu(xml)
 def test_android_category_and_order_sorting_with_group_defaults(self):
  with tempfile.TemporaryDirectory() as tmp:
   xml=Path(tmp)/'menu.xml';xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/system" android:menuCategory="system"/><group android:menuCategory="secondary" android:orderInCategory="20"><item android:id="@+id/late"/><item android:id="@+id/early" android:orderInCategory="2"/><item android:id="@+id/tie" android:orderInCategory="2"/><item android:id="@+id/default_category" android:menuCategory="0"/></group><item android:id="@+id/alternative" android:menuCategory="alternative"/></menu>')
   out=mod.compile_menu(xml);names=['default_category','alternative','early','tie','late','system'];positions=[out.index('view_id("'+name+'")') for name in names];self.assertEqual(positions,sorted(positions))
   xml.write_text(xml.read_text().replace('menuCategory="system"','menuCategory="invalid"'))
   with self.assertRaisesRegex(ValueError,'Invalid menu integer'):mod.compile_menu(xml)
   xml.write_text(xml.read_text().replace('menuCategory="invalid"','menuCategory="0x60000"'))
   with self.assertRaisesRegex(ValueError,'Invalid menu category'):mod.compile_menu(xml)
 def test_group_defaults_and_item_overrides(self):
  with tempfile.TemporaryDirectory() as tmp:
   xml=Path(tmp)/'menu.xml';xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><group android:id="@+id/actions" android:enabled="false" android:visible="false"><item android:id="@+id/inherited"/><item android:id="@+id/override" android:enabled="true" android:visible="true"/></group><item android:id="@+id/independent"/></menu>')
   output=mod.compile_menu(xml);lines={name:next(line for line in output.splitlines() if 'view_id("'+name+'")' in line) for name in ('inherited','override','independent')}
   self.assertIn('enabled:false,visible:false',lines['inherited']);self.assertIn('group_id:bhoo::android_xml::view_id("actions")',lines['inherited']);self.assertIn('group_id:0',lines['independent'])
   for name in ('override','independent'):self.assertIn('enabled:true,visible:true',lines[name])
   xml.write_text(xml.read_text().replace('android:enabled="false"','android:enabled="invalid"',1))
   with self.assertRaisesRegex(ValueError,'Invalid menu boolean'):mod.compile_menu(xml)
   xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><group android:unknown="single"/></menu>')
   with self.assertRaisesRegex(ValueError,'Unsupported menu group attribute'):mod.compile_menu(xml)
 def test_import_resources_and_item_properties(self):
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);(root/'values').mkdir();(root/'values'/'strings.xml').write_text('<resources><string name="search">Search items</string></resources>')
   xml=root/'menu.xml';xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/search" android:title="@string/search" android:icon="@drawable/search" android:showAsAction="ifRoom"/><item android:id="@id/disabled" android:title="Disabled" android:enabled="false"/></menu>')
   result=mod.compile_menu(xml,root)
   for value in ('view_id("search")','label:"Search items"','Some(bhoo::MaterialIcon::Search)','show_as_action:bhoo::android_xml::AndroidShowAsAction::IfRoom','enabled:false'):self.assertIn(value,result)
 def test_png_drawable_resource_is_embedded(self):
  import shutil
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);(root/'drawable').mkdir();shutil.copy(Path(__file__).resolve().parents[1]/'codes/apps/bhoo-web/android-examples/res/drawable/toolbar_search.png',root/'drawable'/'custom.png')
   xml=root/'menu.xml';xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/search" android:title="Search" android:icon="@drawable/custom" android:showAsAction="always"/></menu>')
   output=mod.compile_menu(xml,root)
   for value in ('icon_drawable:Some("custom".into())','Image::rgba(24,24','show_as_action:bhoo::android_xml::AndroidShowAsAction::Always'):self.assertIn(value,output)
   with self.assertRaisesRegex(ValueError,'Unresolved PNG'):mod.compile_menu(xml,None)
 def test_vector_drawable_rasterizes_and_rejects_unsupported_features(self):
  import shutil
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);(root/'drawable').mkdir();source=Path(__file__).resolve().parents[1]/'codes/apps/bhoo-web/android-examples/res/drawable/toolbar_vector_search.xml';shutil.copy(source,root/'drawable'/'custom_vector.xml')
   xml=root/'menu.xml';xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/search" android:title="Search" android:icon="@drawable/custom_vector" android:showAsAction="always"/></menu>')
   output=mod.compile_menu(xml,root,320)
   self.assertIn('icon_drawable:Some("custom_vector".into())',output);self.assertIn('Image::rgba(48,48',output)
   _,_,pixels,_=mod.rasterize_vector(root/'drawable'/'custom_vector.xml',320);self.assertGreater(sum(a>0 for a in pixels[3::4]),100);self.assertEqual(pixels[3],0)
   (root/'drawable'/'custom_vector.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24"><unsupported/></vector>')
   with self.assertRaisesRegex(ValueError,'Unsupported vector element'):mod.compile_menu(xml,root)
 def test_vector_group_transforms_and_composited_alpha(self):
  with tempfile.TemporaryDirectory() as tmp:
   path=Path(tmp)/'v.xml';path.write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="10dp" android:height="10dp" android:viewportWidth="10" android:viewportHeight="10" android:alpha="0.5"><group android:translateX="4" android:translateY="3" android:scaleX="2" android:scaleY="2"><path android:fillColor="#ff0000" android:pathData="M0 0h2v2h-2z"/><path android:fillColor="#ff0000" android:pathData="M0 0h2v2h-2z"/></group></vector>')
   width,height,pixels,_=mod.rasterize_vector(path)
   def alpha(x,y):return pixels[(y*width+x)*4+3]
   self.assertEqual((width,height),(10,10));self.assertEqual(alpha(1,1),0);self.assertTrue(126<=alpha(5,4)<=129);self.assertEqual(alpha(8,4),0)
 def test_vector_clip_scope_and_intersection(self):
  with tempfile.TemporaryDirectory() as tmp:
   path=Path(tmp)/'v.xml';path.write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="10dp" android:height="10dp" android:viewportWidth="10" android:viewportHeight="10"><group><path android:fillColor="#00ff00" android:pathData="M0 0h2v2h-2z"/><clip-path android:pathData="M2 0h6v10h-6z"/><clip-path android:pathData="M0 2h10v6h-10z"/><path android:fillColor="#ff0000" android:pathData="M0 0h10v10h-10z"/></group><path android:fillColor="#0000ff" android:pathData="M8 8h2v2h-2z"/></vector>')
   width,_,pixels,_=mod.rasterize_vector(path)
   def pixel(x,y):return tuple(pixels[(y*width+x)*4:(y*width+x+1)*4])
   self.assertEqual(pixel(0,0),(0,255,0,255));self.assertEqual(pixel(3,3),(255,0,0,255));self.assertEqual(pixel(1,4)[3],0);self.assertEqual(pixel(9,9),(0,0,255,255))
 def test_vector_tint_metadata_and_unsupported_modes(self):
  with tempfile.TemporaryDirectory() as tmp:
   path=Path(tmp)/'v.xml';base='<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="1dp" android:height="1dp" android:viewportWidth="1" android:viewportHeight="1" TINT><path android:fillColor="#ffffff" android:pathData="M0 0h1v1h-1z"/></vector>'
   path.write_text(base.replace('TINT','android:tint="?attr/colorControlNormal"'));self.assertEqual(mod.vector_tint(path),'controlNormal')
   path.write_text(base.replace('TINT','android:tint="#80402010"'));self.assertEqual(mod.vector_tint(path),'#80402010')
   path.write_text(base.replace('TINT','android:tint="?attr/unknown"'))
   with self.assertRaisesRegex(ValueError,'Unsupported vector tint'):mod.rasterize_vector(path)
   path.write_text(base.replace('TINT','android:tint="#ffffff" android:tintMode="multiply"'));self.assertEqual(mod.vector_tint(path),'#ffffffff')
   path.write_text(base.replace('TINT','android:tint="#ffffff" android:tintMode="overlay"'))
   with self.assertRaisesRegex(ValueError,'Unsupported vector tintMode'):mod.rasterize_vector(path)
 def test_unknown_attributes_and_nested_menu_fail(self):
  with tempfile.TemporaryDirectory() as tmp:
   xml=Path(tmp)/'menu.xml';xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a" android:alphabeticShortcut="a"/></menu>')
   with self.assertRaisesRegex(ValueError,'Unsupported menu item attribute'):mod.compile_menu(xml)
   xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a"><menu/></item></menu>')
   with self.assertRaisesRegex(ValueError,'Only flat menu items'):mod.compile_menu(xml)
 def test_show_as_action_with_text_flag(self):
  with tempfile.TemporaryDirectory() as tmp:
   xml=Path(tmp)/'menu.xml';base='<menu xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a" android:title="Search" android:showAsAction="VALUE"/></menu>'
   for flags in ('ifRoom|withText','withText|ifRoom'):
    xml.write_text(base.replace('VALUE',flags));result=mod.compile_menu(xml);self.assertIn('show_text_with_icon:true',result);self.assertIn('AndroidShowAsAction::IfRoom',result)
   xml.write_text(base.replace('VALUE','ifRoom|always'))
   with self.assertRaisesRegex(ValueError,'Conflicting showAsAction'):mod.compile_menu(xml)
 def test_wave6a_app_show_as_action_icon_tint_action_view_class_and_attrs(self):
  import shutil
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);(root/'values').mkdir();(root/'drawable').mkdir()
   (root/'values'/'strings.xml').write_text('<resources><string name="s_find">Find</string><color name="brand_tint">#224466</color><style name="AppTheme"><item name="menu_icon_tint">@color/brand_tint</item></style></resources>')
   shutil.copy(Path(__file__).resolve().parents[1]/'codes/apps/bhoo-web/android-examples/res/drawable/toolbar_search.png',root/'drawable'/'ic_find.png')
   shutil.copy(Path(__file__).resolve().parents[1]/'codes/apps/bhoo-web/android-examples/res/drawable/toolbar_vector_search.xml',root/'drawable'/'ic_vec.xml')
   xml=root/'menu.xml'
   xml.write_text('''<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" xmlns:tools="http://schemas.android.com/tools" tools:ignore="AppCompatResource">
    <item android:id="@+id/menuSearch" android:title="@string/s_find" android:icon="@drawable/ic_find" app:showAsAction="collapseActionView|ifRoom" app:actionViewClass="androidx.appcompat.widget.SearchView" app:iconTint="?attr/menu_icon_tint" android:orderInCategory="20" android:visible="true" android:enabled="true" android:checkable="false"/>
    <item android:id="@+id/menuFilter" android:title="Filter" android:icon="@drawable/ic_vec" app:showAsAction="always|withText" app:iconTint="#80112233" app:iconTintMode="src_in" android:orderInCategory="10" android:visible="false" android:enabled="false" android:checkable="true" android:checked="true"/>
    <item android:id="@+id/menuReset" android:title="Reset" app:showAsAction="collapseActionView|never" android:orderInCategory="30" android:titleCondensed="Rst"/>
   </menu>''')
   out=mod.compile_menu(xml,root)
   # Order sorting: menuFilter (10) -> menuSearch (20) -> menuReset (30)
   pos=[out.index('view_id("'+n+'")') for n in ('menuFilter','menuSearch','menuReset')]
   self.assertEqual(pos,sorted(pos))
   lines={n:next(l for l in out.splitlines() if 'view_id("'+n+'")' in l) for n in ('menuSearch','menuFilter','menuReset')}
   self.assertIn('show_as_action:bhoo::android_xml::AndroidShowAsAction::IfRoom,enabled:true,visible:true,checkable:false',lines['menuSearch'])
   self.assertIn('show_text_with_icon:true,show_as_action:bhoo::android_xml::AndroidShowAsAction::Always,enabled:false,visible:false,checkable:true,checked:true',lines['menuFilter'])
   self.assertIn('show_as_action:bhoo::android_xml::AndroidShowAsAction::Never',lines['menuReset'])
   self.assertIn('state.drawable_tints.insert("ic_find".into(),bhoo::android_xml::AndroidDrawableTint::Fixed(bhoo::Color(34,68,102,255)));',out)
   self.assertIn('state.drawable_tints.insert("ic_vec".into(),bhoo::android_xml::AndroidDrawableTint::Fixed(bhoo::Color(17,34,51,128)));',out)
   # Strict validation rejections
   xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><item android:id="@+id/a" app:iconTint="?attr/no_such_tint"/></menu>')
   with self.assertRaisesRegex(ValueError,'Unsupported iconTint'):mod.compile_menu(xml,root)
   xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><item android:id="@+id/a" app:actionViewClass="not a class!"/></menu>')
   with self.assertRaisesRegex(ValueError,'Invalid actionViewClass'):mod.compile_menu(xml,root)
   xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><item android:id="@+id/a" app:showAsAction="collapseActionView|ifRoom|always"/></menu>')
   with self.assertRaisesRegex(ValueError,'Conflicting showAsAction'):mod.compile_menu(xml,root)
   xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><item android:id="@+id/a" app:iconTintMode="bogus"/></menu>')
   with self.assertRaisesRegex(ValueError,'Unsupported iconTintMode'):mod.compile_menu(xml,root)
   # Vector using theme fillColor (?attr/colorSurface) and theme-attr icon (?menu_ok_icon)
   (root/'drawable'/'ic_themed.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24"><path android:fillColor="?attr/colorSurface" android:pathData="M2,2h20v20h-20z"/></vector>')
   (root/'values'/'themes.xml').write_text('<resources><style name="AppTheme"><item name="menu_ok_icon">@drawable/ic_themed</item></style></resources>')
   xml.write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><item android:id="@+id/ok" android:title="OK" android:icon="?menu_ok_icon" app:showAsAction="always"/></menu>')
   themed_out=mod.compile_menu(xml,root)
   self.assertIn('icon_drawable:Some("ic_themed".into())',themed_out)
   self.assertIn('Image::rgba(24,24',themed_out)
if __name__=='__main__':unittest.main()
