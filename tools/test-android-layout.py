#!/usr/bin/env python3
"""Test suite for the import-android-layout tool.

Provides unit tests that compile Android XML layout files using the import-android-layout module.
No command-line interface; executed directly via Python's unittest runner.
"""
"""Test suite for import-android-layout module.

Provides unit tests that compile Android XML layout files using the import-android-layout tool.
No command-line interface; the script is executed directly via Python's unittest runner.
"""
import importlib.util,re,tempfile,unittest,struct,zlib
from pathlib import Path
spec=importlib.util.spec_from_file_location('importer',Path(__file__).with_name('import-android-layout.py'))
mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
class ImportTests(unittest.TestCase):
 def run_layout(self,xml,resources=None):
  with tempfile.TemporaryDirectory() as d:
   p=Path(d)/'layout.xml';p.write_text(xml);return mod.compile_layout(p,resources)
 def test_progressbar_tint_colors_and_selector(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'color').mkdir()
   (res/'color'/'progress.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_selected="true" android:color="#123456"/><item android:color="#abcdef"/></selector>')
   xml='<ProgressBar xmlns:android="http://schemas.android.com/apk/res/android" android:progressTint="@color/progress" android:progressBackgroundTint="#80112233" android:secondaryProgressTint="#456" android:indeterminateTint="#789"/>'
   result=self.run_layout(xml,res)
   for value in ('kind:149, anchor:"8,0,18,52,86,255"','kind:149, anchor:"0,0,171,205,239,255"','kind:150, anchor:"0,0,17,34,51,128"','kind:151, anchor:"0,0,68,85,102,255"','kind:152, anchor:"0,0,119,136,153,255"'):self.assertIn(value,result)
   with self.assertRaisesRegex(ValueError,'only on ProgressBar'):self.run_layout(xml.replace('ProgressBar','TextView'),res)
   seek=self.run_layout(xml.replace('ProgressBar','SeekBar').replace(' android:indeterminateTint="#789"','').replace('/>',' android:thumbTint="#f00"/>'),res)
   self.assertIn('kind:153, anchor:"0,0,255,0,0,255"',seek)
   rating=self.run_layout(xml.replace('ProgressBar','RatingBar').replace(' android:indeterminateTint="#789"',''),res)
   self.assertIn('kind:151, anchor:"0,0,68,85,102,255"',rating)
   with self.assertRaisesRegex(ValueError,'only on SeekBar'):self.run_layout(xml.replace('/>',' android:thumbTint="#f00"/>'),res)
 def test_seekbar_tick_mark_shape_and_tint(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'tick.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android" android:shape="oval"><solid android:color="#123456"/><size android:width="6dp" android:height="6dp"/></shape>')
   xml='<SeekBar xmlns:android="http://schemas.android.com/apk/res/android" android:min="2" android:max="6" android:tickMark="@drawable/tick" android:tickMarkTint="#f00"/>'
   result=self.run_layout(xml,res)
   self.assertIn('kind:155, anchor:"6.0,6.0,3.0,18,52,86,255"',result)
   self.assertIn('kind:154, anchor:"0,0,255,0,0,255"',result)
   with self.assertRaisesRegex(ValueError,'range exceeds'):self.run_layout(xml.replace('android:max="6"','android:max="1027"'),res)
   with self.assertRaisesRegex(ValueError,'requires tickMark'):self.run_layout(xml.replace(' android:tickMark="@drawable/tick"',''),res)
   (res/'drawable'/'tick_states.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_pressed="true" android:drawable="@drawable/tick"/><item android:drawable="@null"/></selector>')
   selected=self.run_layout(xml.replace('@drawable/tick','@drawable/tick_states'),res)
   self.assertIn('kind:167, anchor:"2,0,6.0,6.0,3.0,18,52,86,255"',selected)
   self.assertIn('kind:167, anchor:"0,0,0,0,0,0,0,0,0"',selected)
   self.assertIn('kind:167',self.run_layout(xml.replace('@drawable/tick','@null'),res))
 def test_seekbar_split_track(self):
  xml='<SeekBar xmlns:android="http://schemas.android.com/apk/res/android" android:splitTrack="true"/>'
  self.assertIn('kind:163, anchor:""',self.run_layout(xml))
  self.assertNotIn('kind:163',self.run_layout(xml.replace('true','false')))
  with self.assertRaisesRegex(ValueError,'Invalid splitTrack'):self.run_layout(xml.replace('true','maybe'))
  with self.assertRaisesRegex(ValueError,'only on SeekBar'):self.run_layout(xml.replace('SeekBar','ProgressBar'))
 def test_checked_text_checkmark_tint_and_gravity(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'color').mkdir()
   (res/'color'/'mark.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_enabled="false" android:color="#777"/><item android:state_checked="true" android:color="#123456"/></selector>')
   xml='<CheckedTextView xmlns:android="http://schemas.android.com/apk/res/android" android:checkMarkGravity="start" android:checkMarkTint="@color/mark"/>'
   result=self.run_layout(xml,res)
   for expected in ('kind:157, anchor:"start"','kind:156, anchor:"0,1,119,119,119,255"','kind:156, anchor:"64,0,18,52,86,255"'):self.assertIn(expected,result)
   with self.assertRaisesRegex(ValueError,'only on CheckedTextView'):self.run_layout(xml.replace('CheckedTextView','TextView'),res)
   with self.assertRaisesRegex(ValueError,'Unsupported checkMarkGravity'):self.run_layout(xml.replace('start','center'),res)
 def test_checked_text_checkmark_drawable_selector(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'mark.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_checked="true"><shape android:shape="oval"><solid android:color="#123456"/><size android:width="16dp" android:height="16dp"/></shape></item><item android:drawable="@null"/></selector>')
   xml='<CheckedTextView xmlns:android="http://schemas.android.com/apk/res/android" android:checkMark="@drawable/mark"/>'
   result=self.run_layout(xml,res)
   for expected in ('kind:164, anchor:"64,0,16.0,16.0,8.0,18,52,86,255"','kind:164, anchor:"0,0,0,0,0,0,0,0,0"'):self.assertIn(expected,result)
   self.assertIn('kind:164',self.run_layout(xml.replace('@drawable/mark','@null'),res))
   with self.assertRaisesRegex(ValueError,'only on CheckedTextView'):self.run_layout(xml.replace('CheckedTextView','TextView'),res)
   button=xml.replace('CheckedTextView','CheckBox').replace('checkMark=','button=')
   result=self.run_layout(button,res)
   self.assertIn('kind:165, anchor:"64,0,16.0,16.0,8.0,18,52,86,255"',result)
   self.assertIn('kind:165',self.run_layout(button.replace('CheckBox','RadioButton'),res))
   with self.assertRaisesRegex(ValueError,'only on CheckBox'):self.run_layout(button.replace('CheckBox','TextView'),res)
   thumb=xml.replace('CheckedTextView','SeekBar').replace('checkMark=','thumb=')
   with self.assertRaisesRegex(ValueError,'thumb state'):self.run_layout(thumb,res)
   (res/'drawable'/'mark.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_pressed="true"><shape android:shape="oval"><solid android:color="#123456"/><size android:width="16dp" android:height="16dp"/></shape></item><item android:drawable="@null"/></selector>')
   self.assertIn('kind:166, anchor:"2,0,16.0,16.0,8.0,18,52,86,255"',self.run_layout(thumb,res))
 def test_checkbox_button_tint(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'color').mkdir()
   (res/'color'/'button.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_enabled="false" android:color="#777"/><item android:state_checked="true" android:color="#123456"/><item android:color="#654321"/></selector>')
   xml='<CheckBox xmlns:android="http://schemas.android.com/apk/res/android" android:buttonTint="@color/button"/>'
   result=self.run_layout(xml,res)
   for expected in ('kind:158, anchor:"0,1,119,119,119,255"','kind:158, anchor:"64,0,18,52,86,255"','kind:158, anchor:"0,0,101,67,33,255"'):self.assertIn(expected,result)
   self.assertIn('kind:158',self.run_layout(xml.replace('CheckBox','RadioButton'),res))
   with self.assertRaisesRegex(ValueError,'only on CheckBox'):self.run_layout(xml.replace('CheckBox','TextView'),res)
 def test_switch_thumb_and_track_tints(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'color').mkdir()
   (res/'color'/'thumb.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_checked="true" android:color="#123456"/><item android:color="#654321"/></selector>')
   xml='<Switch xmlns:android="http://schemas.android.com/apk/res/android" android:thumbTint="@color/thumb" android:trackTint="#abcdef"/>'
   result=self.run_layout(xml,res)
   for expected in ('kind:159, anchor:"64,0,18,52,86,255"','kind:159, anchor:"0,0,101,67,33,255"','kind:160, anchor:"0,0,171,205,239,255"'):self.assertIn(expected,result)
   with self.assertRaisesRegex(ValueError,'only on Switch'):self.run_layout(xml.replace('Switch','SeekBar'),res)
 def test_switch_state_text_configuration(self):
  xml='<Switch xmlns:android="http://schemas.android.com/apk/res/android" android:textOn="YES" android:textOff="NO"/>'
  result=self.run_layout(xml)
  for expected in ('kind:161, anchor:""','text_on:"YES"','text_off:"NO"'):self.assertIn(expected,result)
  self.assertNotIn('kind:161',self.run_layout(xml.replace('/>',' android:showText="false"/>')))
  self.assertIn('kind:161',self.run_layout('<Switch xmlns:android="http://schemas.android.com/apk/res/android" android:showText="true"/>'))
  with self.assertRaisesRegex(ValueError,'Invalid showText'):self.run_layout(xml.replace('/>',' android:showText="maybe"/>'))
  with self.assertRaisesRegex(ValueError,'only on Switch'):self.run_layout(xml.replace('Switch','CheckBox').replace('/>',' android:showText="true"/>'))
 def test_switch_sizing_attributes(self):
  xml='<Switch xmlns:android="http://schemas.android.com/apk/res/android" android:switchMinWidth="120dp" android:switchPadding="20dp" android:thumbTextPadding="16dp"/>'
  result=self.run_layout(xml)
  for value in ('kind:193, anchor:"120.0"','kind:194, anchor:"20.0"','kind:195, anchor:"16.0"'):self.assertIn(value,result)
  with self.assertRaisesRegex(ValueError,'only on Switch'):self.run_layout(xml.replace('Switch','CheckBox'))
  with self.assertRaises(ValueError):self.run_layout(xml.replace('120dp','-1dp'))
  with self.assertRaises(ValueError):self.run_layout(xml.replace('120dp','4097dp'))
 def test_material_divider_attributes(self):
  xml='<com.google.android.material.divider.MaterialDivider xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" android:layout_width="match_parent" android:layout_height="wrap_content" app:dividerThickness="8dp" app:dividerInsetStart="72dp" app:dividerInsetEnd="16dp" app:dividerColor="#123456"/>'
  result=self.run_layout(xml)
  for value in ('kind:Kind::MaterialDivider','divider_thickness:8.0','divider_color:Some(Color(18,52,86,255))','kind:55, anchor:"72.0"','kind:56, anchor:"16.0"'):self.assertIn(value,result)
  with self.assertRaisesRegex(ValueError,'Unsupported attribute'):self.run_layout(xml.replace('dividerInsetStart','unknown'))
 def test_material_divider_styles_inheritance_and_overrides(self):
  tag='com.google.android.material.divider.MaterialDivider'
  xml='<'+tag+' xmlns:app="http://schemas.android.com/apk/res-auto" style="?attr/materialDividerHeavyStyle" app:dividerColor="?attr/colorOutlineVariant"/>'
  self.assertIn('divider_thickness:8.0',self.run_layout(xml))
  self.assertIn('divider_color:None',self.run_layout(xml))
  self.assertIn('divider_thickness:3.0',self.run_layout(xml.replace('app:dividerColor=', 'app:dividerThickness="3dp" app:dividerColor=')))
  self.assertIn('divider_thickness:1.0',self.run_layout(xml.replace('?attr/materialDividerHeavyStyle','?attr/materialDividerStyle')))
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'values'/'styles.xml').write_text('<resources><style name="AppDivider" parent="Widget.Material3.MaterialDivider.Heavy"><item name="dividerInsetStart">24dp</item><item name="dividerColor">#123456</item></style><style name="AppDivider.Dense"><item name="dividerThickness">4dp</item></style></resources>')
   styled=xml.replace('?attr/materialDividerHeavyStyle','@style/AppDivider.Dense').replace(' app:dividerColor="?attr/colorOutlineVariant"','')
   result=self.run_layout(styled,res)
   for value in ('divider_thickness:4.0','divider_color:Some(Color(18,52,86,255))','kind:55, anchor:"24.0"'):self.assertIn(value,result)
   with self.assertRaisesRegex(ValueError,'MaterialDivider style on unsupported view'):self.run_layout('<TextView style="@style/AppDivider"/>',res)
 def test_material_card_style_and_children(self):
  tag='com.google.android.material.card.MaterialCardView'
  xml='<'+tag+' xmlns:android="http://schemas.android.com/apk/res/android" style="?attr/materialCardViewFilledStyle" android:checkable="true" android:clickable="true"><TextView android:text="Inside"/></'+tag+'>'
  result=self.run_layout(xml)
  for value in ('kind:Kind::MaterialCard','kind:57, anchor:"Filled"','kind:58, anchor:""','text:"Inside"','clickable:true'):self.assertIn(value,result)
  self.assertIn('kind:57, anchor:"Outlined"',self.run_layout(xml.replace('?attr/materialCardViewFilledStyle','?attr/materialCardViewStyle')))
  with self.assertRaisesRegex(ValueError,'Invalid checkable'):self.run_layout(xml.replace('checkable="true"','checkable="maybe"'))
  with self.assertRaisesRegex(ValueError,'MaterialCardView style on unsupported view'):self.run_layout('<TextView style="@style/Widget.Material3.CardView.Filled"/>')
 def test_material_card_checked_icon_attributes(self):
  xml='<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:checkedIconSize="32dp" app:checkedIconMargin="4dp" app:checkedIconGravity="BOTTOM_END" app:checkedIconTint="#5261A8"/>'
  out=self.run_layout(xml)
  for value in ('kind:59, anchor:"32.0"','kind:60, anchor:"4.0"','kind:61, anchor:"BOTTOM_END"','kind:62, anchor:"82,97,168,255"'):self.assertIn(value,out)
  with self.assertRaisesRegex(ValueError,'Unsupported checkedIconGravity'):self.run_layout(xml.replace('BOTTOM_END','bottomEnd'))
  with self.assertRaisesRegex(ValueError,'Expected dp/sp'):self.run_layout(xml.replace('32dp','-1dp'))
 def test_material_card_appearance_style_override(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'values'/'styles.xml').write_text('<resources><style name="CardCustom" parent="Widget.Material3.CardView.Outlined"><item name="cardBackgroundColor">#112233</item><item name="strokeColor">#445566</item><item name="strokeWidth">3dp</item><item name="cardCornerRadius">16dp</item></style></resources>')
   xml='<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" style="@style/CardCustom" app:strokeWidth="2dp"/>'
   out=self.run_layout(xml,res)
   for value in ('kind:63, anchor:"17,34,51,255"','kind:64, anchor:"68,85,102,255"','kind:65, anchor:"2.0"','kind:66, anchor:"16.0"'):self.assertIn(value,out)
   with self.assertRaisesRegex(ValueError,'Expected dp/sp'):self.run_layout(xml.replace('2dp','-2dp'),res)
 def test_material_card_content_padding(self):
  xml='<com.google.android.material.card.MaterialCardView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" android:padding="99dp" app:contentPadding="12dp" app:contentPaddingLeft="0dp" app:contentPaddingBottom="20dp"/>'
  out=self.run_layout(xml)
  for value in ('kind:67, anchor:"0.0"','kind:68, anchor:"12.0"','kind:69, anchor:"12.0"','kind:70, anchor:"20.0"'):self.assertIn(value,out)
  with self.assertRaisesRegex(ValueError,'Expected dp/sp'):self.run_layout(xml.replace('20dp','-1dp'))
 def test_material_card_elevation(self):
  xml='<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:cardElevation="8dp"/>'
  self.assertIn('kind:71, anchor:"8.0"',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Expected dp/sp'):self.run_layout(xml.replace('8dp','-8dp'))
 def test_material_card_compat_padding_attributes(self):
  xml='<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:cardElevation="8dp" app:cardMaxElevation="12dp" app:cardUseCompatPadding="true" app:cardPreventCornerOverlap="false"/>'
  out=self.run_layout(xml)
  for value in ('kind:72, anchor:"12.0"','kind:73, anchor:"true"','kind:74, anchor:"false"'):self.assertIn(value,out)
  with self.assertRaisesRegex(ValueError,'Invalid cardUseCompatPadding'):self.run_layout(xml.replace('"true"','"yes"'))
 def test_material_card_color_state_lists(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:strokeColor="@color/card_state_stroke" app:checkedIconTint="@color/card_state_stroke" app:cardForegroundColor="@color/card_state_foreground"/>'
  out=self.run_layout(xml,res)
  for value in ('kind:77, anchor:"0,1,136,136,136,255"','kind:77, anchor:"64,0,179,38,30,255"','kind:77, anchor:"128,0,0,109,59,255"','kind:75, anchor:"64,0,179,38,30,255"','kind:79, anchor:"64,0,179,38,30,64"'):self.assertIn(value,out)
 def test_material_card_theme_colors_and_selectors(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:cardBackgroundColor="?attr/colorSurfaceContainerHighest" app:strokeColor="@color/card_theme_stroke" app:cardForegroundColor="@color/card_theme_foreground"/>'
  out=self.run_layout(xml,res)
  for value in ('kind:63, anchor:"@colorSurfaceContainerHighest"','kind:77, anchor:"64,0,@colorSecondary,255"','kind:79, anchor:"64,0,@colorSecondary,41"','kind:79, anchor:"128,0,@colorOnSurface,20"','kind:79, anchor:"0,0,0,0,0,0"'):self.assertIn(value,out)
  with self.assertRaisesRegex(ValueError,'Unsupported card theme color'):self.run_layout(xml.replace('?attr/colorSurfaceContainerHighest','?attr/unknown'),res)
 def test_material_card_ripple_selector_checkable_state(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:rippleColor="@color/card_theme_ripple"/>'
  out=self.run_layout(xml,res)
  self.assertIn('kind:81, anchor:"258,64,@colorPrimary,255"',out)
  self.assertIn('kind:81, anchor:"66,0,@colorSecondary,255"',out)
 def test_spinner_entries_and_unsupported_modes(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<Spinner xmlns:android="http://schemas.android.com/apk/res/android" android:entries="@array/workspaces" android:spinnerMode="dropdown"/>'
  out=self.run_layout(xml,res)
  self.assertIn('kind:Kind::Spinner',out);self.assertIn('kind:82, anchor:"Projects"',out)
  self.assertIn('kind:83, anchor:"dialog"',self.run_layout(xml.replace('dropdown','dialog'),res))
  with self.assertRaisesRegex(ValueError,'Unsupported Spinner mode'):self.run_layout(xml.replace('dropdown','unknown'),res)
  with self.assertRaisesRegex(ValueError,'Unresolved Spinner entries'):self.run_layout(xml.replace('workspaces','missing'),res)
 def test_spinner_dropdown_size_and_offsets(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<Spinner xmlns:android="http://schemas.android.com/apk/res/android" android:entries="@array/workspaces" android:dropDownWidth="220dp" android:dropDownHorizontalOffset="12dp" android:dropDownVerticalOffset="-5dp"/>'
  out=self.run_layout(xml,res)
  for value in ('kind:85, anchor:"220.0"','kind:86, anchor:"12.0"','kind:87, anchor:"-5.0"'):self.assertIn(value,out)
  self.assertIn('kind:85, anchor:"match_parent"',self.run_layout(xml.replace('220dp','match_parent'),res))
  with self.assertRaisesRegex(ValueError,'Expected dp/sp'):self.run_layout(xml.replace('220dp','-220dp'),res)
 def test_gallery_xml_examples_are_current(self):
  root=Path('codes/apps/bhoo-web')
  for xml in sorted((root/'android-examples').glob('*.xml')):
   generated=root/'src'/('android_'+xml.stem.replace('-', '_')+'.rs')
   with self.subTest(layout=xml.name):
    self.assertEqual(mod.compile_layout(xml,root/'android-examples/res'),generated.read_text(), 'Regenerate gallery XML layouts')
 def test_textclock_hour_cycles(self):
  xml='<TextClock xmlns:android="http://schemas.android.com/apk/res/android" android:format24Hour="kk K"/>'
  self.assertIn('clock_24:"kk K"',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Unsupported TextClock pattern'):self.run_layout(xml.replace('kk K','kkk K'))
 def test_chronometer_indexed_and_escaped_format(self):
  xml='<Chronometer xmlns:android="http://schemas.android.com/apk/res/android" android:format="%1$s / %1$s (100%%)"/>'
  self.assertIn('chrono_format:"%1$s / %1$s (100%%)"',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Unsupported Chronometer format'):self.run_layout(xml.replace('%1$s / %1$s','%s / %s'))
 def test_image_alpha_xml(self):
  xml='<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:imageAlpha="96"/>'
  self.assertIn('image_alpha:96',self.run_layout(xml))
  for value in ('256','-1','foo'):
   with self.assertRaisesRegex(ValueError,'Invalid imageAlpha'):self.run_layout(xml.replace('96',value))
  with self.assertRaisesRegex(ValueError,'imageAlpha unsupported'):self.run_layout(xml.replace('ImageView','TextView'))
 def test_digital_clock_tag(self):
  out=self.run_layout('<DigitalClock/>')
  self.assertIn('kind:Kind::TextClock',out)
  self.assertIn('clock_12:""',out)
 def test_absolute_layout_coordinates(self):
  xml='<AbsoluteLayout xmlns:android="http://schemas.android.com/apk/res/android"><TextView android:layout_x="12dp" android:layout_y="34dp"/></AbsoluteLayout>'
  out=self.run_layout(xml)
  self.assertIn('kind:Kind::Absolute',out)
  self.assertIn('absolute_position:[12.0,34.0]',out)
  self.assertIn('absolute_position:[-12.0,34.0]',self.run_layout(xml.replace('12dp','-12dp')))
 def test_clip_children_attribute(self):
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:clipChildren="false"/>'
  self.assertIn('clip_children_disabled:true',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Unsupported boolean'):self.run_layout(xml.replace('false','no'))
 def test_view_alpha_attribute(self):
  xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:alpha="0.5"/>'
  self.assertIn('view_alpha:0.5',self.run_layout(xml))
  for value in ('-1','1.2','no'):
   with self.assertRaisesRegex(ValueError,'Invalid alpha'):self.run_layout(xml.replace('0.5',value))
 def test_zoom_button_xml(self):
  xml='<ZoomButton xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/add"/>'
  self.assertIn('kind:Kind::Zoom',self.run_layout(xml))
 def test_solid_foreground(self):
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:foreground="#800000ff"/>'
  self.assertIn('foreground:Some(Color(0,0,255,128))',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Expected #RGB'):self.run_layout(xml.replace('#800000ff','#xyz'))
 def test_shape_drawable_foreground(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:foreground="@drawable/foreground_overlay"/>'
  out=self.run_layout(xml,res)
  self.assertIn('foreground:Some(Color(0,0,255,102)),foreground_radius:24.0',out)
 def test_stroked_shape_drawable(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/stroked_panel"/>'
  out=self.run_layout(xml,res)
  self.assertIn('background_stroke:Some(Color(40,116,72,255)),background_stroke_width:3.0',out)
 def test_oval_shape_stroke(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:foreground="@drawable/oval_overlay"/>'
  out=self.run_layout(xml,res)
  self.assertIn('foreground_radius:-1.0',out)
  self.assertIn('foreground_stroke:Some(Color(122,51,166,255)),foreground_stroke_width:4.0',out)
 def test_shape_without_solid_is_transparent(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:foreground="@drawable/outline_only"/>'
  out=self.run_layout(xml,res)
  self.assertIn('foreground:None,foreground_radius:-1.0',out)
  self.assertIn('foreground_stroke:Some(Color(122,51,166,255)),foreground_stroke_width:4.0',out)
 def test_shape_background_padding(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/padded_panel" android:paddingLeft="0dp"/>'
  out=self.run_layout(xml,res)
  self.assertIn('padding_explicit:[true,false,false,false]',out)
  self.assertIn('background_padding:Some([40.0,12.0,12.0,12.0])',out)
 def test_shape_intrinsic_size(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/sized_panel"/>'
  self.assertIn('background_min_size:[160.0,60.0]',self.run_layout(xml,res))
  self.assertIn('background_min_size:[160.0,60.0]',self.run_layout(xml.replace('<View ', '<Space '),res))
 def test_foreground_gravity_and_padding(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:foreground="@drawable/oval_overlay" android:foregroundGravity="end|bottom" android:foregroundInsidePadding="false"/>'
  out=self.run_layout(xml,res)
  self.assertIn('foreground_size:[120.0,40.0],foreground_padding:None,foreground_gravity:[2,2],foreground_excludes_padding:true',out)
 def test_foreground_padding_resource(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:foreground="@drawable/padded_foreground"/>'
  out=self.run_layout(xml,res)
  self.assertIn('foreground_size:[180.0,70.0],foreground_padding:Some([30.0,14.0,12.0,14.0])',out)
 def test_frame_measure_all_children(self):
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:measureAllChildren="true"><View/></FrameLayout>'
  self.assertIn('measure_all_children:true',self.run_layout(xml))
  self.assertIn('measure_all_children:true',self.run_layout('<ViewAnimator/>'))
  with self.assertRaisesRegex(ValueError,'measureAllChildren unsupported'):self.run_layout(xml.replace('FrameLayout','LinearLayout'))
 def test_image_tint(self):
  xml='<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:tint="#80c80064" android:tintMode="src_in"/>'
  self.assertIn('image_tint:Some(Color(200,0,100,128)),image_tint_states:&[],image_tint_mode:1',self.run_layout(xml))
  self.assertIn('image_tint_mode:0',self.run_layout(xml.replace(' android:tintMode="src_in"','')))
  for mode,value in [('src_over',2),('multiply',3),('screen',4),('add',5)]:
   self.assertIn('image_tint_mode:'+str(value),self.run_layout(xml.replace('src_in',mode)))
  with self.assertRaisesRegex(ValueError,'Unsupported image tintMode'):self.run_layout(xml.replace('src_in','overlay'))
  with self.assertRaisesRegex(ValueError,'image tint unsupported'):self.run_layout(xml.replace('ImageView','TextView'))
 def test_image_tint_color_selector(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);(root/'color').mkdir();(root/'values').mkdir()
   (root/'values'/'colors.xml').write_text('<resources><color name="alias">@color/states</color><color name="accent">#80ff0000</color></resources>')
   selector='<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_enabled="false" android:color="#888888"/><item android:state_pressed="true" android:alpha="0.5" android:color="@color/accent"/><item android:color="#00ff00"/></selector>'
   (root/'color'/'states.xml').write_text(selector)
   xml='<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:tint="@color/alias"/>'
   out=self.run_layout(xml,root)
   for token in ('required:0, forbidden:1, color:Color(136,136,136,255)','required:2, forbidden:0, color:Color(255,0,0,64)','required:0, forbidden:0, color:Color(0,255,0,255)'):self.assertIn(token,out)
   text_xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textColor="@color/alias"/>'
   self.assertIn('text_color_states:&[TintState',self.run_layout(text_xml,root))
   self.assertIn('text_color_states:&[TintState',self.run_layout(text_xml.replace('TextView','Button'),root))
   for tag in ('CheckBox','RadioButton','Switch','ToggleButton'):
    self.assertIn('text_color_states:&[TintState',self.run_layout(text_xml.replace('TextView',tag),root))
   with self.assertRaisesRegex(ValueError,'text color selector unsupported'):self.run_layout(text_xml.replace('TextView','EditText'),root)
   (root/'color'/'states.xml').write_text(selector.replace('state_pressed','state_window_focused'))
   with self.assertRaisesRegex(ValueError,'Unsupported color selector state'):self.run_layout(xml,root)
   (root/'color'/'states.xml').write_text(selector.replace('state_pressed','state_checked'))
   self.assertIn('required:64',self.run_layout(text_xml.replace('TextView','CheckBox'),root))
   (root/'color'/'states.xml').write_text(selector.replace('android:alpha="0.5"','android:alpha="1.5"'))
   with self.assertRaisesRegex(ValueError,'Invalid color selector alpha'):self.run_layout(xml,root)
 def test_image_baseline_attributes(self):
  xml='<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:baseline="18dp" android:baselineAlignBottom="true"/>'
  result=self.run_layout(xml)
  self.assertIn('kind:189, anchor:"18.0"',result);self.assertIn('kind:190, anchor:""',result)
  self.assertNotIn('kind:189',self.run_layout(xml.replace('18dp','-1')))
  self.assertNotIn('kind:190',self.run_layout(xml.replace('true','false')))
  with self.assertRaisesRegex(ValueError,'Invalid baselineAlignBottom'):self.run_layout(xml.replace('true','maybe'))
  with self.assertRaisesRegex(ValueError,'baseline attributes unsupported'):self.run_layout(xml.replace('ImageView','TextView'))
 def test_image_crop_to_padding(self):
  xml='<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:cropToPadding="true"/>'
  self.assertIn('kind:188, anchor:""',self.run_layout(xml))
  self.assertNotIn('kind:188',self.run_layout(xml.replace('true','false')))
  with self.assertRaisesRegex(ValueError,'Invalid cropToPadding'):self.run_layout(xml.replace('true','invalid'))
  with self.assertRaisesRegex(ValueError,'cropToPadding unsupported'):self.run_layout(xml.replace('ImageView','TextView'))
 def test_image_adjust_view_bounds(self):
  xml='<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:adjustViewBounds="true"/>'
  self.assertIn('image_adjust_bounds:true',self.run_layout(xml))
  self.assertIn('image_max_width:Some(80.0),image_max_height:Some(60.0)',self.run_layout(xml.replace('/>',' android:maxWidth="80dp" android:maxHeight="60dp"/>')))
  with self.assertRaisesRegex(ValueError,'image maximum dimensions unsupported'):self.run_layout(xml.replace('ImageView','TextView').replace(' android:adjustViewBounds="true"',' android:maxWidth="80dp"'))
  with self.assertRaisesRegex(ValueError,'Unsupported boolean'):self.run_layout(xml.replace('true','maybe'))
  with self.assertRaisesRegex(ValueError,'adjustViewBounds unsupported'):self.run_layout(xml.replace('ImageView','TextView'))
 def test_relative_layout_rules_and_cycles(self):
  xml='<RelativeLayout xmlns:android="http://schemas.android.com/apk/res/android"><View android:id="@+id/child" android:layout_below="@id/anchor" android:layout_alignParentEnd="true"/><View android:id="@+id/anchor"/></RelativeLayout>'
  out=self.run_layout(xml)
  for token in ('kind:Kind::Relative','RelativeRule { kind:3, anchor:"anchor" }','RelativeRule { kind:17, anchor:"" }'):self.assertIn(token,out)
  with self.assertRaisesRegex(ValueError,'RelativeLayout rule outside'):self.run_layout(xml.replace('RelativeLayout','LinearLayout'))
  with self.assertRaisesRegex(ValueError,'Invalid RelativeLayout anchor'):self.run_layout(xml.replace('@id/anchor','anchor'))
  with self.assertRaisesRegex(ValueError,'Invalid RelativeLayout boolean'):self.run_layout(xml.replace('="true"','="maybe"'))
  cycle=xml.replace('android:id="@+id/anchor"','android:id="@+id/anchor" android:layout_below="@id/child"')
  with self.assertRaisesRegex(ValueError,'Circular RelativeLayout dependencies'):self.run_layout(cycle)
  baseline=xml.replace('layout_below','layout_alignBaseline')
  self.assertIn('RelativeRule { kind:22, anchor:"anchor" }',self.run_layout(baseline))
  cycle=baseline.replace('android:id="@+id/anchor"','android:id="@+id/anchor" android:layout_below="@id/child"')
  with self.assertRaisesRegex(ValueError,'Circular RelativeLayout dependencies'):self.run_layout(cycle)
  separate=xml.replace('android:id="@+id/anchor"','android:id="@+id/anchor" android:layout_toEndOf="@id/child"')
  self.assertIn('kind:Kind::Relative',self.run_layout(separate))
 def test_relative_ignore_gravity_import(self):
  xml='<RelativeLayout xmlns:android="http://schemas.android.com/apk/res/android" android:ignoreGravity="@id/fixed"><View android:id="@+id/fixed"/></RelativeLayout>'
  self.assertIn('RelativeRule { kind:23, anchor:"fixed" }',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Invalid ignoreGravity'):self.run_layout(xml.replace('@id/fixed','fixed'))
  with self.assertRaisesRegex(ValueError,'ignoreGravity unsupported'):self.run_layout(xml.replace('RelativeLayout','LinearLayout'))
 def test_table_column_and_span_attributes(self):
  xml='<TableLayout xmlns:android="http://schemas.android.com/apk/res/android"><TableRow><Button android:layout_column="1" android:layout_span="2"/></TableRow></TableLayout>'
  out=self.run_layout(xml);self.assertIn('RelativeRule { kind:30, anchor:"1" }',out);self.assertIn('RelativeRule { kind:31, anchor:"2" }',out)
  with self.assertRaisesRegex(ValueError,'Invalid layout_span'):self.run_layout(xml.replace('layout_span="2"','layout_span="0"'))
  with self.assertRaisesRegex(ValueError,'layout_column outside TableRow'):self.run_layout(xml.replace('TableRow','LinearLayout'))
 def test_table_stretch_and_shrink_columns(self):
  xml='<TableLayout xmlns:android="http://schemas.android.com/apk/res/android" android:stretchColumns="1, 0, 1" android:shrinkColumns="*"><TableRow/></TableLayout>'
  out=self.run_layout(xml);self.assertIn('RelativeRule { kind:32, anchor:"0,1" }',out);self.assertIn('RelativeRule { kind:33, anchor:"*" }',out)
  with self.assertRaisesRegex(ValueError,'Invalid stretchColumns'):self.run_layout(xml.replace('1, 0, 1','x'))
  with self.assertRaisesRegex(ValueError,'stretchColumns outside TableLayout'):self.run_layout(xml.replace('TableLayout','LinearLayout'))
 def test_table_collapse_columns(self):
  xml='<TableLayout xmlns:android="http://schemas.android.com/apk/res/android" android:collapseColumns="0, 2"><TableRow/></TableLayout>'
  self.assertIn('RelativeRule { kind:34, anchor:"0,2" }',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Invalid collapseColumns'):self.run_layout(xml.replace('0, 2','*'))
 def test_table_horizontal_cell_gravity(self):
  xml='<TableLayout xmlns:android="http://schemas.android.com/apk/res/android"><TableRow><Button android:layout_gravity="end"/></TableRow></TableLayout>'
  self.assertIn('RelativeRule { kind:35, anchor:"" }',self.run_layout(xml))
  self.assertNotIn('RelativeRule { kind:35, anchor:"" }',self.run_layout(xml.replace('end','bottom')))
 def test_table_row_baseline_opt_out(self):
  xml='<TableRow xmlns:android="http://schemas.android.com/apk/res/android" android:baselineAligned="false"/>'
  self.assertIn('RelativeRule { kind:36, anchor:"" }',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Invalid baselineAligned'):self.run_layout(xml.replace('false','maybe'))
 def test_linear_layout_baseline_opt_out(self):
  for tag in ['LinearLayout','RadioGroup']:
   xml=f'<{tag} xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="horizontal" android:baselineAligned="false"/>'
   self.assertIn('RelativeRule { kind:36, anchor:"" }',self.run_layout(xml))
   self.assertNotIn('RelativeRule { kind:36, anchor:"" }',self.run_layout(xml.replace('false','true')))
   with self.assertRaisesRegex(ValueError,'Invalid baselineAligned'):self.run_layout(xml.replace('false','maybe'))
 def test_linear_layout_baseline_child_index(self):
  xml='<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:baselineAlignedChildIndex="1"><TextView/><TextView/></LinearLayout>'
  self.assertIn('RelativeRule { kind:148, anchor:"1" }',self.run_layout(xml))
  self.assertNotIn('kind:148',self.run_layout(xml.replace('="1"','="-1"')))
  with self.assertRaisesRegex(ValueError,'Invalid baselineAlignedChildIndex'):self.run_layout(xml.replace('="1"','="-2"'))
  with self.assertRaisesRegex(ValueError,'baselineAlignedChildIndex unsupported'):self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:baselineAlignedChildIndex="0"/>')
 def test_two_line_list_item_and_android_ids(self):
  xml='<TwoLineListItem xmlns:android="http://schemas.android.com/apk/res/android"><TextView android:id="@android:id/text1"/><TextView android:id="@android:id/text2" android:layout_below="@android:id/text1"/></TwoLineListItem>'
  out=self.run_layout(xml);self.assertIn('kind:Kind::Relative',out);self.assertIn('id:"android:text1"',out);self.assertIn('anchor:"android:text1"',out)
  with self.assertRaisesRegex(ValueError,'Unsupported Android ID'):self.run_layout(xml.replace('@android:id/text2','@android:id/unknown'))
  cycle=xml.replace('android:id="@android:id/text1"','android:id="@android:id/text1" android:layout_below="@android:id/text2"')
  with self.assertRaisesRegex(ValueError,'Circular RelativeLayout dependencies'):self.run_layout(cycle)
 def test_grid_layout_automatic_tracks(self):
  xml='<GridLayout xmlns:android="http://schemas.android.com/apk/res/android" android:columnCount="2"><Button/><Button/></GridLayout>'
  out=self.run_layout(xml);self.assertIn('kind:Kind::Grid',out);self.assertIn('RelativeRule { kind:40, anchor:"2" }',out);self.assertIn('horizontal:true',out)
  with self.assertRaisesRegex(ValueError,'Invalid columnCount'):self.run_layout(xml.replace('"2"','"0"'))
  self.assertIn('layout_gravity:[1,1]',self.run_layout(xml.replace('<Button/>','<Button android:layout_gravity="center"/>',1)))
  with self.assertRaisesRegex(ValueError,'GridLayout layout_weight'):self.run_layout(xml.replace('<Button/>','<Button android:layout_weight="1"/>',1))
 def test_grid_explicit_positions(self):
  xml='<GridLayout xmlns:android="http://schemas.android.com/apk/res/android" android:columnCount="2"><Button android:layout_column="1" android:layout_row="3"/></GridLayout>'
  out=self.run_layout(xml);self.assertIn('RelativeRule { kind:42, anchor:"1" }',out);self.assertIn('RelativeRule { kind:43, anchor:"3" }',out)
  with self.assertRaisesRegex(ValueError,'layout_column exceeds columnCount'):self.run_layout(xml.replace('layout_column="1"','layout_column="2"'))
  with self.assertRaisesRegex(ValueError,'layout_row outside GridLayout'):self.run_layout(xml.replace('GridLayout','LinearLayout').replace(' android:columnCount="2"','').replace(' android:layout_column="1"',''))
 def test_grid_spans(self):
  xml='<GridLayout xmlns:android="http://schemas.android.com/apk/res/android" android:columnCount="2"><Button android:layout_columnSpan="2" android:layout_rowSpan="2"/></GridLayout>'
  out=self.run_layout(xml);self.assertIn('RelativeRule { kind:44, anchor:"2" }',out);self.assertIn('RelativeRule { kind:45, anchor:"2" }',out)
  with self.assertRaisesRegex(ValueError,'exceeds columnCount'):self.run_layout(xml.replace('columnSpan="2"','columnSpan="3"'))
  with self.assertRaisesRegex(ValueError,'Invalid layout_rowSpan'):self.run_layout(xml.replace('rowSpan="2"','rowSpan="0"'))
 def test_grid_axis_weights(self):
  xml='<GridLayout xmlns:android="http://schemas.android.com/apk/res/android"><Button android:layout_columnWeight="2" android:layout_rowWeight="0.5"/></GridLayout>'
  out=self.run_layout(xml);self.assertIn('RelativeRule { kind:46, anchor:"2" }',out);self.assertIn('RelativeRule { kind:47, anchor:"0.5" }',out)
  with self.assertRaisesRegex(ValueError,'Invalid layout_rowWeight'):self.run_layout(xml.replace('0.5','-1'))
  with self.assertRaisesRegex(ValueError,'layout_columnWeight outside GridLayout'):self.run_layout(xml.replace('GridLayout','LinearLayout'))
 def test_grid_baseline_gravity(self):
  xml='<GridLayout xmlns:android="http://schemas.android.com/apk/res/android"><TextView android:layout_gravity="end|baseline"/></GridLayout>'
  out=self.run_layout(xml);self.assertIn('layout_gravity:[2,6]',out);self.assertIn('RelativeRule { kind:49, anchor:"" }',out)
  with self.assertRaisesRegex(ValueError,'Conflicting foregroundGravity'):self.run_layout(xml.replace('end|baseline','bottom|baseline'))
 def test_grid_default_margins(self):
  xml='<GridLayout xmlns:android="http://schemas.android.com/apk/res/android" android:useDefaultMargins="true"><Button android:layout_marginLeft="0dp"/></GridLayout>'
  out=self.run_layout(xml);self.assertIn('RelativeRule { kind:53, anchor:"" }',out);self.assertIn('RelativeRule { kind:50, anchor:"1" }',out)
  with self.assertRaisesRegex(ValueError,'Invalid useDefaultMargins'):self.run_layout(xml.replace('true','maybe'))
 def test_grid_alignment_mode(self):
  xml='<GridLayout xmlns:android="http://schemas.android.com/apk/res/android" android:alignmentMode="alignBounds"/>'
  self.assertIn('RelativeRule { kind:54, anchor:"" }',self.run_layout(xml))
  self.assertNotIn('RelativeRule { kind:54, anchor:"" }',self.run_layout(xml.replace('alignBounds','alignMargins')))
  with self.assertRaisesRegex(ValueError,'Invalid alignmentMode'):self.run_layout(xml.replace('alignBounds','wrong'))
 def test_zoom_controls_import(self):
  xml='<ZoomControls xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/zoom"/>'
  self.assertIn('Kind::ZoomControls',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Leaf view cannot have children'):self.run_layout(xml.replace('/>','><Button/></ZoomControls>'))
 def test_textview_min_lines(self):
  xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:minLines="3"/>'
  self.assertIn('min_lines:3',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Invalid minLines'):self.run_layout(xml.replace('3','0'))
 def test_textview_exact_lines(self):
  xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:lines="2"/>'
  self.assertIn('max_lines:2,min_lines:2',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Invalid lines'):self.run_layout(xml.replace('"2"','"0"'))
  self.assertIn('max_lines:2,min_lines:1',self.run_layout(xml.replace('/>',' android:minLines="1"/>')))
  self.assertIn('max_lines:2,min_lines:2',self.run_layout(xml.replace('/>',' android:maxLines="4"/>')))
  self.assertIn('max_lines:2,min_lines:3',self.run_layout(xml.replace('/>',' android:minLines="3" android:maxLines="4"/>')))
 def test_textview_line_spacing(self):
  xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:lineSpacingExtra="6dp" android:lineSpacingMultiplier="1.5"/>'
  self.assertIn('line_spacing_extra:6.0,line_spacing_multiplier:1.5',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Invalid lineSpacingMultiplier'):self.run_layout(xml.replace('1.5','0'))
 def test_textview_letter_spacing(self):
  xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:letterSpacing="-0.05"/>'
  self.assertIn('letter_spacing_em:-0.05',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Invalid letterSpacing'):self.run_layout(xml.replace('-0.05','no'))
 def test_real_demo_layouts(self):
  for p in Path('codes/apps/bhoo-web/android-demo').glob('*.xml'):
   s=mod.compile_layout(p,p.parent/"res");self.assertIn('Kind::Scroll',s);self.assertIn('pub static LAYOUT',s)
 def test_horizontal_scroll_requires_one_child(self):
  out=self.run_layout('<HorizontalScrollView><LinearLayout/></HorizontalScrollView>')
  self.assertIn('Kind::HorizontalScroll',out)
  with self.assertRaisesRegex(ValueError,'HorizontalScrollView requires exactly one child'):
   self.run_layout('<HorizontalScrollView><View/><View/></HorizontalScrollView>')
 def test_padding_precedence_and_relative_edges(self):
  xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:paddingLeft="1dp" android:paddingHorizontal="2dp" android:paddingTop="3dp" android:paddingVertical="4dp" android:paddingStart="7dp"/>'
  out=self.run_layout(xml)
  self.assertIn('padding_edges:[2.0,4.0,2.0,4.0]',out);self.assertIn('padding_relative:[Some(7.0),None]',out)
  out=self.run_layout(xml.replace('android:paddingStart=', 'android:padding="6dp" android:paddingStart='))
  self.assertIn('padding_edges:[6.0,6.0,6.0,6.0]',out);self.assertIn('padding_relative:[Some(7.0),None]',out)
 def test_space_tag(self):
  xml='<Space xmlns:android="http://schemas.android.com/apk/res/android" android:layout_width="20dp" android:layout_height="40dp"/>'
  out=self.run_layout(xml)
  self.assertIn('Kind::Space',out);self.assertIn('width:Size::Dp(20.0)',out);self.assertIn('height:Size::Dp(40.0)',out)
 def test_minimum_view_dimensions(self):
  xml='<Space xmlns:android="http://schemas.android.com/apk/res/android" android:minWidth="12dp" android:minHeight="36dp"/>'
  out=self.run_layout(xml)
  self.assertIn('min_width:12.0',out);self.assertIn('min_height:36.0',out)
  with self.assertRaisesRegex(ValueError,'Expected dp/sp'):self.run_layout(xml.replace('36dp','-1dp'))
 def test_checked_text_view(self):
  xml='<CheckedTextView xmlns:android="http://schemas.android.com/apk/res/android" android:checked="true" android:text="Selected" android:textStyle="bold"/>'
  out=self.run_layout(xml)
  for token in ('Kind::CheckedText','checked:true','text_bold:true'):self.assertIn(token,out)
 def test_android_hex_color_formats(self):
  for color,expected in [('#123','Color(17,34,51,255)'),('#8123','Color(17,34,51,136)'),('#123456','Color(18,52,86,255)'),('#80123456','Color(18,52,86,128)')]:
   xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textColor="'+color+'" android:background="'+color+'"/>'
   self.assertEqual(self.run_layout(xml).count(expected),2)
  with self.assertRaisesRegex(ValueError,'Expected #RGB'):self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="#12"/>')
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'drawable').mkdir()
   (res/'values'/'colors.xml').write_text('<resources><color name="accent">#8123</color></resources>')
   (res/'drawable'/'panel.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="@color/accent"/></shape>')
   self.assertIn('Color(17,34,51,136)',self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/panel"/>',res))
 def test_edit_text_separates_hint_and_initial_value(self):
  xml='<EditText xmlns:android="http://schemas.android.com/apk/res/android" android:text="Initial" android:hint="Your name"/>'
  out=self.run_layout(xml)
  self.assertIn('text:"Initial",hint:"Your name"',out)
  self.assertIn('text:"",hint:"Your name"',self.run_layout(xml.replace('android:text="Initial" ','')))
 def test_edit_text_input_types(self):
  for kind,code in [('text',0),('textEmailAddress',1),('phone',2),('number',3),('textPassword',4),('numberDecimal',5)]:
   xml='<EditText xmlns:android="http://schemas.android.com/apk/res/android" android:inputType="'+kind+'"/>'
   self.assertIn('input_type:'+str(code),self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Unsupported inputType'):self.run_layout(xml.replace('numberDecimal','invalidType'))
  with self.assertRaisesRegex(ValueError,'unsupported on View'):self.run_layout(xml.replace('EditText','View'))
 def test_view_switcher_child_limit(self):
  self.assertIn('Kind::ViewSwitcher',self.run_layout('<ViewSwitcher><TextView/><TextView/></ViewSwitcher>'))
  self.assertIn('Kind::ViewAnimator',self.run_layout('<ViewAnimator><View/><View/><View/></ViewAnimator>'))
  with self.assertRaisesRegex(ValueError,'at most two'):self.run_layout('<ViewSwitcher><View/><View/><View/></ViewSwitcher>')
 def test_view_switcher_alpha_animation_resource(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'anim').mkdir()
   (res/'anim'/'fade.xml').write_text('<alpha xmlns:android="http://schemas.android.com/apk/res/android" android:fromAlpha="0" android:toAlpha="1" android:duration="300" android:interpolator="@android:anim/linear_interpolator"/>')
   xml='<ViewSwitcher xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@anim/fade"><View/></ViewSwitcher>'
   self.assertIn('animation_in:Some(AlphaAnimation { from:0.0, to:1.0, duration:0.3, start_offset:0.0, linear:true })',self.run_layout(xml,res))
   (res/'anim'/'fade.xml').write_text('<unknownAnim/>')
   with self.assertRaisesRegex(ValueError,'Unsupported animation resource'):self.run_layout(xml,res)
 def test_view_switcher_translate_animation_resource(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'anim').mkdir()
   (res/'anim'/'slide.xml').write_text('<translate xmlns:android="http://schemas.android.com/apk/res/android" android:fromXDelta="100%p" android:toXDelta="-12dp" android:fromYDelta="50%" android:duration="300"/>')
   xml='<ViewSwitcher xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@anim/slide"><View/></ViewSwitcher>'
   out=self.run_layout(xml,res)
   for part in ('TranslateAnimation','value:1.0, mode:2','value:-12.0, mode:0','value:0.5, mode:1','duration:0.3'):self.assertIn(part,out)
   (res/'anim'/'slide.xml').write_text('<translate xmlns:android="http://schemas.android.com/apk/res/android" android:fromXDelta="bad"/>')
   with self.assertRaisesRegex(ValueError,'Unsupported animation delta'):self.run_layout(xml,res)
 def test_view_switcher_combined_animation_set(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'anim').mkdir()
   (res/'anim'/'both.xml').write_text('<set xmlns:android="http://schemas.android.com/apk/res/android" android:duration="300" android:interpolator="@android:anim/linear_interpolator"><alpha android:fromAlpha="0" android:toAlpha="1"/><translate android:fromXDelta="100%p" android:toXDelta="0"/></set>')
   xml='<ViewSwitcher xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@anim/both"><View/></ViewSwitcher>'
   out=self.run_layout(xml,res)
   for part in ('animation_in:Some(AlphaAnimation','translate_in:Some(TranslateAnimation','duration:0.3, start_offset:0.0, linear:true','value:1.0, mode:2'):self.assertIn(part,out)
   (res/'anim'/'both.xml').write_text('<set><alpha/><alpha/></set>')
   with self.assertRaisesRegex(ValueError,'Duplicate animation'):self.run_layout(xml,res)
 def test_view_switcher_animation_start_offsets(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'anim').mkdir()
   path=res/'anim'/'delayed.xml'
   path.write_text('<set xmlns:android="http://schemas.android.com/apk/res/android" android:startOffset="200"><alpha android:fromAlpha="0" android:toAlpha="1" android:startOffset="100" android:duration="300"/><translate android:fromXDelta="10" android:startOffset="400" android:duration="300"/></set>')
   xml='<ViewSwitcher xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@anim/delayed"><View/></ViewSwitcher>'
   out=self.run_layout(xml,res)
   self.assertIn('duration:0.3, start_offset:0.3',out)
   self.assertIn('duration:0.3, start_offset:0.6',out)
   path.write_text('<alpha xmlns:android="http://schemas.android.com/apk/res/android" android:startOffset="-1"/>')
   with self.assertRaisesRegex(ValueError,'Invalid animation startOffset'):self.run_layout(xml,res)
 def test_view_flipper_xml(self):
  xml='<ViewFlipper xmlns:android="http://schemas.android.com/apk/res/android" android:autoStart="true" android:flipInterval="1500"><View/><View/><View/></ViewFlipper>'
  out=self.run_layout(xml)
  for token in ('Kind::ViewFlipper','auto_start:true','flip_interval:1.5'):self.assertIn(token,out)
  with self.assertRaisesRegex(ValueError,'Invalid flipInterval'):self.run_layout(xml.replace('1500','0'))
  with self.assertRaisesRegex(ValueError,'unsupported on View'):self.run_layout(xml.replace('ViewFlipper','View'))
 def test_text_switcher_requires_text_children(self):
  self.assertIn('Kind::TextSwitcher',self.run_layout('<TextSwitcher><TextView/><TextView/></TextSwitcher>'))
  with self.assertRaisesRegex(ValueError,'must be TextView'):self.run_layout('<TextSwitcher><Button/></TextSwitcher>')
  with self.assertRaisesRegex(ValueError,'at most two'):self.run_layout('<TextSwitcher><TextView/><TextView/><TextView/></TextSwitcher>')
 def test_image_switcher_requires_image_children(self):
  self.assertIn('Kind::ImageSwitcher',self.run_layout('<ImageSwitcher><ImageView/><ImageView/></ImageSwitcher>'))
  with self.assertRaisesRegex(ValueError,'must be ImageView'):self.run_layout('<ImageSwitcher><TextView/></ImageSwitcher>')
  with self.assertRaisesRegex(ValueError,'at most two'):self.run_layout('<ImageSwitcher><ImageView/><ImageView/><ImageView/></ImageSwitcher>')
 def test_chronometer_xml(self):
  xml='<Chronometer xmlns:android="http://schemas.android.com/apk/res/android" android:format="Elapsed %s" android:countDown="true"/>'
  out=self.run_layout(xml)
  for token in ('Kind::Chronometer','chrono_format:"Elapsed %s"','count_down:true'):self.assertIn(token,out)
  with self.assertRaisesRegex(ValueError,'Unsupported Chronometer format'):self.run_layout(xml.replace('%s','%d'))
  with self.assertRaisesRegex(ValueError,'unsupported on View'):self.run_layout(xml.replace('Chronometer','View'))
 def test_text_clock_patterns(self):
  xml='<TextClock xmlns:android="http://schemas.android.com/apk/res/android" android:format12Hour="h:mm a" android:format24Hour="HH:mm:ss"/>'
  out=self.run_layout(xml)
  for token in ('Kind::TextClock','clock_12:"h:mm a"','clock_24:"HH:mm:ss"'):self.assertIn(token,out)
  self.assertIn('clock_24:"yyyy-MM-dd HH:mm:ss"',self.run_layout(xml.replace('HH:mm:ss','yyyy-MM-dd HH:mm:ss')))
  with self.assertRaisesRegex(ValueError,'Unsupported TextClock pattern'):self.run_layout(xml.replace('h:mm a','EEE, MMM d'))
 def test_analog_clock_xml(self):
  self.assertIn('Kind::AnalogClock',self.run_layout('<AnalogClock/>'))
 def test_linear_layout_dividers(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'divider.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#123"/><size android:width="3dp" android:height="2dp"/></shape>')
   xml='<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:divider="@drawable/divider" android:dividerPadding="8dp" android:showDividers="beginning|middle|end"><View/></LinearLayout>'
   out=self.run_layout(xml,res)
   for token in ('divider_color:Some(Color(17,34,51,255))','divider_thickness:2.0','divider_flags:7','divider_padding:8.0'):self.assertIn(token,out)
   self.assertIn('divider_thickness:3.0',self.run_layout(xml.replace('<LinearLayout ','<LinearLayout android:orientation="horizontal" '),res))
   with self.assertRaisesRegex(ValueError,'Invalid showDividers'):self.run_layout(xml.replace('beginning|middle|end','none|middle'),res)
 def test_linear_layout_weight_sum(self):
  xml='<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:weightSum="4"><View android:layout_weight="1"/></LinearLayout>'
  self.assertIn('weight_sum:4.0',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'unsupported on View'):self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:weightSum="2"/>')
 def test_single_line_ellipsize_modes(self):
  xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:singleLine="true" android:ellipsize="middle"/>'
  out=self.run_layout(xml)
  for token in ('single_line:true','max_lines:1','ellipsize_side:2'):self.assertIn(token,out)
  self.assertIn('ellipsize_side:1',self.run_layout(xml.replace('middle','start')))
  with self.assertRaisesRegex(ValueError,'requires one line'):self.run_layout(xml.replace('android:singleLine="true" ',''))
  marquee=self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:ellipsize="marquee"/>')
  for token in ('single_line:true','max_lines:1','min_lines:1','ellipsize_end:true','ellipsize_side:0'):self.assertIn(token,marquee)
  with self.assertRaisesRegex(ValueError,'Unsupported ellipsize'):self.run_layout(xml.replace('middle','invalid'))
 def test_checkbox_attributes(self):
  xml='<CheckBox xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/check" android:checked="true" android:enabled="false"/>'
  out=self.run_layout(xml)
  self.assertIn('Kind::CheckBox',out);self.assertIn('checked:true',out);self.assertIn('enabled:false',out)
  with self.assertRaisesRegex(ValueError,'Unsupported boolean'):
   self.run_layout('<CheckBox xmlns:android="http://schemas.android.com/apk/res/android" android:checked="perhaps"/>')
 def test_switch_attributes(self):
  out=self.run_layout('<Switch xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/wifi" android:checked="true"/>')
  self.assertIn('Kind::Switch',out);self.assertIn('checked:true',out)
 def test_seekbar_values(self):
  out=self.run_layout('<SeekBar xmlns:android="http://schemas.android.com/apk/res/android" android:min="10" android:max="90" android:progress="120"/>')
  for token in ['Kind::SeekBar','min:10.0','max:90.0','progress:90.0']:self.assertIn(token,out)
  with self.assertRaisesRegex(ValueError,'minimum exceeds maximum'):
   self.run_layout('<SeekBar xmlns:android="http://schemas.android.com/apk/res/android" android:min="9" android:max="3"/>')
  with self.assertRaisesRegex(ValueError,'Expected integer'):
   self.run_layout('<SeekBar xmlns:android="http://schemas.android.com/apk/res/android" android:progress="half"/>')
 def test_progress_bar_defaults_and_horizontal_style(self):
  circular=self.run_layout('<ProgressBar/>')
  self.assertIn('Kind::ProgressBar',circular);self.assertIn('horizontal:false',circular);self.assertIn('indeterminate:true',circular)
  linear=self.run_layout('<ProgressBar xmlns:android="http://schemas.android.com/apk/res/android" style="?android:attr/progressBarStyleHorizontal" android:progress="45"/>')
  for token in ['horizontal:true','indeterminate:false','progress:45.0']:self.assertIn(token,linear)
  with self.assertRaisesRegex(ValueError,'Unsupported style reference'):
   self.run_layout('<ProgressBar style="?android:attr/unknown"/>')
 def test_progress_bar_secondary_progress(self):
  out=self.run_layout('<ProgressBar xmlns:android="http://schemas.android.com/apk/res/android" style="?android:attr/progressBarStyleHorizontal" android:progress="20" android:secondaryProgress="130"/>')
  self.assertIn('progress:20.0',out);self.assertIn('secondary_progress:100.0',out)
 def test_radio_group_import(self):
  xml='<RadioGroup xmlns:android="http://schemas.android.com/apk/res/android"><RadioButton android:id="@+id/one" android:checked="true"/><RadioButton android:id="@+id/two"/></RadioGroup>'
  out=self.run_layout(xml)
  self.assertIn('Kind::RadioGroup',out);self.assertEqual(out.count('Kind::RadioButton'),2);self.assertIn('checked:true',out)
 def test_radio_group_checked_button_reference(self):
  xml='<RadioGroup xmlns:android="http://schemas.android.com/apk/res/android" android:checkedButton="@id/two"><RadioButton android:id="@+id/one"/><RadioButton android:id="@+id/two"/></RadioGroup>'
  self.assertIn('checked_button:"two"',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'uniquely identified RadioButton descendant'):
   self.run_layout(xml.replace('@id/two','@id/missing'))
  with self.assertRaisesRegex(ValueError,'Invalid checkedButton'):
   self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:checkedButton="@id/foo"/>')
 def test_radio_group_checked_button_nested(self):
  xml='<RadioGroup xmlns:android="http://schemas.android.com/apk/res/android" android:checkedButton="@id/nested"><LinearLayout><RadioButton android:id="@+id/nested"/></LinearLayout><RadioButton android:id="@+id/direct"/></RadioGroup>'
  self.assertIn('checked_button:"nested"',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'uniquely identified RadioButton'):self.run_layout(xml.replace('RadioButton android:id="@+id/nested"','TextView android:id="@+id/nested"'))
  with self.assertRaisesRegex(ValueError,'uniquely identified RadioButton'):self.run_layout(xml.replace('<RadioButton android:id="@+id/direct"/>','<RadioButton android:id="@+id/nested"/>'))
 def test_rating_bar_attributes(self):
  xml='<RatingBar xmlns:android="http://schemas.android.com/apk/res/android" android:numStars="5" android:rating="2.5" android:stepSize="0.5" android:isIndicator="true"/>'
  out=self.run_layout(xml)
  for token in ['Kind::RatingBar','num_stars:5','rating:2.5','step_size:0.5','indicator:true']:self.assertIn(token,out)
  with self.assertRaisesRegex(ValueError,'outside supported range'):self.run_layout(xml.replace('0.5','0'))
 def test_rating_bar_mirror_for_rtl(self):
  xml='<RatingBar xmlns:android="http://schemas.android.com/apk/res/android" android:mirrorForRtl="true"/>'
  self.assertIn('mirror_for_rtl:true',self.run_layout(xml))
  with self.assertRaisesRegex(ValueError,'Unsupported boolean'):self.run_layout(xml.replace('true','perhaps'))
  for tag in ('SeekBar','ProgressBar'):self.assertIn('mirror_for_rtl:true',self.run_layout(xml.replace('RatingBar',tag)))
  with self.assertRaisesRegex(ValueError,'unsupported on View'):self.run_layout(xml.replace('RatingBar','View'))
 def test_toggle_button_labels(self):
  xml='<ToggleButton xmlns:android="http://schemas.android.com/apk/res/android" android:textOn="Enabled" android:textOff="Paused" android:checked="true"/>'
  out=self.run_layout(xml)
  for token in ['Kind::ToggleButton','text_on:"Enabled"','text_off:"Paused"','checked:true']:self.assertIn(token,out)
  styled=xml.replace('/>',' android:disabledAlpha="0.25" android:textAllCaps="true" android:textStyle="bold"/>')
  result=self.run_layout(styled)
  for token in ('kind:162, anchor:"0.25"','text_all_caps:true','text_bold:true'):self.assertIn(token,result)
  with self.assertRaisesRegex(ValueError,'Invalid disabledAlpha'):self.run_layout(styled.replace('0.25','1.5'))
  with self.assertRaisesRegex(ValueError,'only on ToggleButton'):self.run_layout(styled.replace('ToggleButton','Button'))
 def test_text_appearance_inheritance_and_direct_override(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'color').mkdir()
   (res/'values'/'styles.xml').write_text('<resources><style name="Base"><item name="android:textSize">22sp</item><item name="android:textColor">@color/text</item><item name="android:textStyle">bold</item></style><style name="Base.Large"><item name="android:letterSpacing">0.1</item><item name="android:textAllCaps">true</item></style><style name="ViewStyle"><item name="android:textAppearance">@style/Base.Large</item><item name="android:textSize">18sp</item></style><style name="Bad"><item name="android:padding">4dp</item></style></resources>')
   (res/'color'/'text.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_enabled="false" android:color="#777"/><item android:color="#123456"/></selector>')
   xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" style="@style/ViewStyle" android:textSize="16sp" android:text="Hello"/>'
   result=self.run_layout(xml,res)
   for expected in ('text_size:16.0','text_bold:true','text_all_caps:true','letter_spacing_em:0.1','color:Color(18,52,86,255)'):self.assertIn(expected,result)
   self.assertNotIn('text_size:22.0',result)
   with self.assertRaisesRegex(ValueError,'Unsupported textAppearance attribute'):self.run_layout(xml.replace('style="@style/ViewStyle"','android:textAppearance="@style/Bad"'),res)
   with self.assertRaisesRegex(ValueError,'textAppearance unsupported'):self.run_layout(xml.replace('TextView','ImageView'),res)
   for tag in ('CheckBox','RadioButton','Switch'):
    control=self.run_layout(xml.replace('TextView',tag),res)
    self.assertIn('text_bold:true',control);self.assertIn('text_all_caps:true',control);self.assertIn('text_size:16.0',control)
 def test_textview_compound_shape_drawables(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'shape.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android" android:shape="rectangle"><solid android:color="#123456"/><size android:width="20dp" android:height="12dp"/><corners android:radius="2dp"/></shape>')
   xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:drawableStart="@drawable/shape" android:drawableEnd="@null" android:drawablePadding="4dp"/>'
   result=self.run_layout(xml,res)
   self.assertIn('kind:173',self.run_layout(xml.replace('TextView','CheckedTextView'),res))
   for token in ('kind:173, anchor:"20.0,12.0,2.0,18,52,86,255"','kind:174, anchor:"0,0,0,0,0,0,0"','kind:175, anchor:"4.0"'):self.assertIn(token,result)
   self.assertIn('kind:173',self.run_layout(xml.replace('TextView','Button'),res))
   with self.assertRaisesRegex(ValueError,'only on TextView'):self.run_layout(xml.replace('TextView','ImageView'),res)
   (res/'color').mkdir();(res/'color'/'tint.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_pressed="true" android:color="#ff0000"/><item android:color="#123456"/></selector>')
   tinted=self.run_layout(xml.replace('/>',' android:drawableTint="@color/tint"/>'),res)
   self.assertIn('kind:176',self.run_layout(xml.replace('TextView','Button').replace('/>',' android:drawableTint="@color/tint" android:drawableTintMode="src_in"/>'),res))
   self.assertIn('kind:176, anchor:"2,0,255,0,0,255"',tinted)
   self.assertIn('kind:176, anchor:"2,0,255,0,0,255"',self.run_layout(xml.replace('TextView','CheckedTextView').replace('/>',' android:drawableTint="@color/tint"/>'),res))
   (res/'color'/'tint.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_checked="true" android:color="#ff0000"/><item android:color="#123456"/></selector>')
   self.assertIn('kind:176, anchor:"64,0,255,0,0,255"',self.run_layout(xml.replace('TextView','CheckedTextView').replace('/>',' android:drawableTint="@color/tint"/>'),res))
   with self.assertRaisesRegex(ValueError,'Unsupported drawableTint selector'):self.run_layout(xml.replace('/>',' android:drawableTint="@color/tint"/>'),res)
   with self.assertRaisesRegex(ValueError,'drawableTint supported only on TextView'):self.run_layout('<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:drawableTint="#123456"/>',res)
   for mode,num in [('src_atop',0),('src_in',1),('src_over',2),('multiply',3),('screen',4),('add',5)]:
    self.assertIn('kind:177, anchor:"'+str(num)+'"',self.run_layout(xml.replace('/>',' android:drawableTintMode="'+mode+'"/>'),res))
   with self.assertRaisesRegex(ValueError,'Unsupported drawableTintMode'):self.run_layout(xml.replace('/>',' android:drawableTintMode="invalid"/>'),res)
   with self.assertRaisesRegex(ValueError,'drawableTintMode supported only'):self.run_layout('<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:drawableTintMode="src_in"/>',res)
 def test_compound_drawable_selector(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'shape.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#123456"/><size android:width="20dp" android:height="12dp"/></shape>')
   selector='<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_pressed="true" android:drawable="@null"/><item android:drawable="@drawable/shape"/></selector>'
   (res/'drawable'/'states.xml').write_text(selector)
   xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:drawableStart="@drawable/states"/>'
   out=self.run_layout(xml,res)
   self.assertIn('kind:182, anchor:"2,0,0,0,0,0,0,0,0"',out)
   self.assertIn('kind:182, anchor:"0,0,20.0,12.0,0.0,18,52,86,255"',out)
   (res/'drawable'/'states.xml').write_text(selector.replace('state_pressed','state_checked'))
   with self.assertRaisesRegex(ValueError,'Unsupported compound drawable selector state'):self.run_layout(xml,res)
   self.assertIn('kind:182, anchor:"64,0,',self.run_layout(xml.replace('TextView','CheckedTextView'),res))
   (res/'drawable'/'small.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#abcdef"/><size android:width="10dp" android:height="8dp"/></shape>')
   varying=selector.replace('android:drawable="@null"','android:drawable="@drawable/small"')
   (res/'drawable'/'states.xml').write_text(varying)
   out=self.run_layout(xml,res)
   self.assertIn('kind:173, anchor:"20.0,12.0,0.0,18,52,86,255"',out)
   self.assertIn('kind:182, anchor:"2,0,10.0,8.0,0.0,171,205,239,255"',out)
   (res/'drawable'/'states.xml').write_text(varying.replace('<selector ', '<selector android:constantSize="true" '))
   self.assertIn('kind:173, anchor:"20.0,12.0,0,0,0,0,0"',self.run_layout(xml,res))
   (res/'drawable'/'states.xml').write_text(selector.replace('state_pressed="true"','state_enabled="false"'))
   with self.assertRaisesRegex(ValueError,'initial intrinsic shape'):self.run_layout(xml,res)

 def test_compound_button_tint_modes(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'shape.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#123456"/><size android:width="20dp" android:height="20dp"/></shape>')
   for tag in ('CheckBox','RadioButton'):
    for mode,num in [('src_atop',0),('src_in',1),('src_over',2),('multiply',3),('screen',4),('add',5)]:
     xml=f'<{tag} xmlns:android="http://schemas.android.com/apk/res/android" android:button="@drawable/shape" android:buttonTintMode="{mode}"/>'
     self.assertIn('kind:184, anchor:"'+str(num)+'"',self.run_layout(xml,res))
    with self.assertRaisesRegex(ValueError,'Unsupported buttonTintMode'):self.run_layout(xml.replace('add','invalid'),res)
    with self.assertRaisesRegex(ValueError,'requires a custom'):self.run_layout(xml.replace(' android:button="@drawable/shape"',''),res)
 def test_checked_text_mark_tint_modes(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'shape.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#123456"/><size android:width="20dp" android:height="20dp"/></shape>')
   for mode,num in [('src_atop',0),('src_in',1),('src_over',2),('multiply',3),('screen',4),('add',5)]:
    xml=f'<CheckedTextView xmlns:android="http://schemas.android.com/apk/res/android" android:checkMark="@drawable/shape" android:checkMarkTintMode="{mode}"/>'
    self.assertIn('kind:185, anchor:"'+str(num)+'"',self.run_layout(xml,res))
   with self.assertRaisesRegex(ValueError,'Unsupported checkMarkTintMode'):self.run_layout(xml.replace('add','invalid'),res)
   with self.assertRaisesRegex(ValueError,'requires a custom'):self.run_layout(xml.replace(' android:checkMark="@drawable/shape"',''),res)
 def test_seekbar_drawable_tint_modes(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'shape.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#123456"/><size android:width="20dp" android:height="20dp"/></shape>')
   for attr,shape,kind in [('thumbTintMode','thumb',186),('tickMarkTintMode','tickMark',187)]:
    for mode,num in [('src_atop',0),('src_in',1),('src_over',2),('multiply',3),('screen',4),('add',5)]:
     xml=f'<SeekBar xmlns:android="http://schemas.android.com/apk/res/android" android:{shape}="@drawable/shape" android:{attr}="{mode}"/>'
     self.assertIn('kind:'+str(kind)+', anchor:"'+str(num)+'"',self.run_layout(xml,res))
    with self.assertRaisesRegex(ValueError,'Unsupported '+attr):self.run_layout(xml.replace('add','invalid'),res)
    with self.assertRaisesRegex(ValueError,'requires a custom'):self.run_layout(xml.replace(' android:'+shape+'="@drawable/shape"',''),res)
 def test_frame_layout_clip_to_padding(self):
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:clipToPadding="false"/>'
  self.assertIn('clip_to_padding_disabled:true',self.run_layout(xml))
  self.assertIn('clip_to_padding_disabled:false',self.run_layout(xml.replace('false','true')))
  with self.assertRaisesRegex(ValueError,'Unsupported boolean'):self.run_layout(xml.replace('false','invalid'))
 def test_frame_layout_gravity(self):
  xml='<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android"><Button android:layout_gravity="end|bottom"/></FrameLayout>'
  self.assertIn('layout_gravity:[2,2]',self.run_layout(xml))
  self.assertIn('layout_gravity:[1,1]',self.run_layout(xml.replace('end|bottom','center')))
  self.assertIn('layout_gravity:[3,2]',self.run_layout(xml.replace('end|bottom','left|bottom')))
  self.assertIn('layout_gravity:[4,2]',self.run_layout(xml.replace('end|bottom','right|bottom')))
  with self.assertRaisesRegex(ValueError,'Conflicting horizontal'):self.run_layout(xml.replace('end|bottom','start|end'))
  with self.assertRaisesRegex(ValueError,'Unsupported layout_gravity'):self.run_layout(xml.replace('end|bottom','fill'))
 def test_linear_layout_content_gravity(self):
  xml='<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:gravity="center_horizontal|bottom"><Button android:layout_gravity="center_vertical"/></LinearLayout>'
  out=self.run_layout(xml)
  self.assertIn('content_gravity:[1,2]',out);self.assertIn('layout_gravity:[0,1],absolute_position:[0.0,0.0],relative_rules:&[],layout_gravity_explicit:true',out)
 def test_button_gravity_is_tracked_when_explicit(self):
  xml='<Button xmlns:android="http://schemas.android.com/apk/res/android" android:text="Item"/>'
  self.assertNotIn('kind:168',self.run_layout(xml))
  self.assertIn('kind:168',self.run_layout(xml.replace('/>',' android:gravity="start|center_vertical"/>')))
 def test_layout_direction_values(self):
  xml='<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:layoutDirection="rtl"/>'
  self.assertIn('layout_direction:2',self.run_layout(xml));self.assertIn('layout_direction:1',self.run_layout(xml.replace('rtl','ltr')))
  with self.assertRaisesRegex(ValueError,'Unsupported layoutDirection'):self.run_layout(xml.replace('rtl','locale'))
 def test_image_view_drawable_and_scale(self):
  xml='<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/photo" android:scaleType="centerCrop"/>'
  out=self.run_layout(xml)
  for token in ['Kind::Image','image_src:"photo"','image_scale:1']:self.assertIn(token,out)
  for mode,index in [('fitXY',2),('center',3),('centerInside',4),('fitStart',5),('fitEnd',6),('matrix',7)]:self.assertIn('image_scale:'+str(index),self.run_layout(xml.replace('centerCrop',mode)))
  with self.assertRaisesRegex(ValueError,'Unsupported ImageView scaleType'):self.run_layout(xml.replace('centerCrop','invalid'))
  with self.assertRaisesRegex(ValueError,'Unresolved resource'):self.run_layout(xml.replace('@drawable/photo','@mipmap/photo'))
 def test_image_view_bundles_validated_png(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><ImageView android:src="@drawable/sample"/><ImageView android:src="@drawable/sample"/><ImageView android:src="@drawable/switcher_day"/></LinearLayout>'
  out=self.run_layout(xml,res)
  self.assertIn('pub fn install_drawables',out)
  self.assertIn('bhoo::Image::rgba(64,40,vec![',out)
  self.assertEqual(out.count('Image::rgba('),2)
  with tempfile.TemporaryDirectory() as d:
   path=Path(d);(path/'drawable').mkdir();(path/'drawable'/'sample.png').write_bytes(b'bad')
   with self.assertRaisesRegex(ValueError,'Invalid PNG'):self.run_layout(xml,path)
 def test_indexed_and_grayscale_png_decoding(self):
  def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
  def png(width,depth,color,raw,palette=b'',transparency=b''):
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,1,depth,color,0,0,0))+(chunk(b'PLTE',palette) if palette else b'')+(chunk(b'tRNS',transparency) if transparency else b'')+chunk(b'IDAT',zlib.compress(b'\0'+raw))+chunk(b'IEND',b'')
  with tempfile.TemporaryDirectory() as d:
   path=Path(d)/'image.png'
   path.write_bytes(png(4,2,3,bytes([0x1b]),bytes([255,0,0,0,255,0,0,0,255,255,255,255]),bytes([255,128,0,255])))
   width,height,pixels=mod.decode_png(path)
   self.assertEqual((width,height),(4,1));self.assertEqual(list(pixels),[255,0,0,255,0,255,0,128,0,0,255,0,255,255,255,255])
   path.write_bytes(png(3,1,0,bytes([0x40])))
   self.assertEqual(list(mod.decode_png(path)[2]),[0,0,0,255,255,255,255,255,0,0,0,255])
   path.write_bytes(png(2,8,4,bytes([24,128,200,255])))
   self.assertEqual(list(mod.decode_png(path)[2]),[24,24,24,128,200,200,200,255])
   path.write_bytes(png(1,8,2,bytes([1,2,3]),transparency=struct.pack('>HHH',1,2,3)))
   self.assertEqual(list(mod.decode_png(path)[2]),[1,2,3,0])
   path.write_bytes(png(1,16,6,struct.pack('>HHHH',0xffff,0x8080,0x0101,0x4040)))
   self.assertEqual(list(mod.decode_png(path)[2]),[255,128,1,64])
   path.write_bytes(png(1,16,0,struct.pack('>H',0x8080),transparency=struct.pack('>H',0x8080)))
   self.assertEqual(list(mod.decode_png(path)[2]),[128,128,128,0])
   path.write_bytes(png(1,16,2,struct.pack('>HHH',0x1234,0x2345,0x3456),transparency=struct.pack('>HHH',0x1234,0x2345,0x3456)))
   self.assertEqual(list(mod.decode_png(path)[2]),[18,35,52,0])
 def test_image_button_drawable_import(self):
  res=Path('codes/apps/bhoo-web/android-examples/res')
  xml='<ImageButton xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/sample" android:scaleType="centerCrop"/>'
  out=self.run_layout(xml,res)
  for token in ['Kind::Icon','image_src:"sample"','image_scale:1','image_builtin:false','pub fn install_drawables']:self.assertIn(token,out)
  builtin=self.run_layout(xml.replace('@drawable/sample','@drawable/menu'))
  self.assertIn('image_builtin:true',builtin)
 def test_density_qualified_drawable_selection(self):
  sample=Path('codes/apps/bhoo-web/android-examples/res/drawable-xhdpi/sample.png').read_bytes()
  xml='<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/sample"/>'
  with tempfile.TemporaryDirectory() as d:
   res=Path(d); (res/'drawable-hdpi').mkdir();(res/'drawable-xhdpi').mkdir()
   (res/'drawable-hdpi'/'sample.png').write_bytes(sample);(res/'drawable-xhdpi'/'sample.png').write_bytes(sample)
   layout=res/'screen.xml';layout.write_text(xml)
   self.assertIn('state.image_scales.insert("sample".into(),0.6666666666666666)',mod.compile_layout(layout,res,160))
   self.assertIn('state.image_scales.insert("sample".into(),0.5)',mod.compile_layout(layout,res,320))
   with self.assertRaisesRegex(ValueError,'Invalid target density'):mod.compile_layout(layout,res,0)
 def test_scroll_view_fill_viewport(self):
  xml='<HorizontalScrollView xmlns:android="http://schemas.android.com/apk/res/android" android:fillViewport="true"><View/></HorizontalScrollView>'
  self.assertIn('fill_viewport:true',self.run_layout(xml))
  self.assertIn('fill_viewport:false',self.run_layout(xml.replace('true','false')))
  with self.assertRaisesRegex(ValueError,'Unsupported boolean'):self.run_layout(xml.replace('true','yes'))
 def test_text_view_max_lines_and_ellipsize(self):
  xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:maxLines="2" android:ellipsize="end"/>'
  out=self.run_layout(xml);self.assertIn('max_lines:2',out);self.assertIn('ellipsize_end:true',out)
  with self.assertRaisesRegex(ValueError,'Invalid maxLines'):self.run_layout(xml.replace('"2"','"0"'))
  with self.assertRaisesRegex(ValueError,'requires one line'):self.run_layout(xml.replace('"end"','"middle"'))
 def test_text_view_bold_and_all_caps(self):
  xml='<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textStyle="bold" android:textAllCaps="true"/>'
  out=self.run_layout(xml);self.assertIn('text_bold:true',out);self.assertIn('text_all_caps:true',out)
  self.assertIn('text_bold:false',self.run_layout(xml.replace('bold','italic')))
  self.assertIn('text_bold:true',self.run_layout(xml.replace('bold','bold|italic')))
  with self.assertRaisesRegex(ValueError,'Unsupported textStyle'):self.run_layout(xml.replace('bold','underline'))
 def test_unsupported_custom_view_is_an_error(self):
  with self.assertRaisesRegex(ValueError,'Unsupported Android view'):
   self.run_layout('<org.thoughtcrime.securesms.components.AvatarView/>')
 def test_constraint_not_silently_ignored(self):
  with self.assertRaisesRegex(ValueError,'RelativeLayout rule outside RelativeLayout'):
   self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:layout_below="@id/foo"/>')
 def test_missing_resource_and_duplicate_id(self):
  with self.assertRaisesRegex(ValueError,'Unresolved resource'):
   self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="@string/missing"/>')
  with self.assertRaisesRegex(ValueError,'Duplicate id'):
   self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><TextView android:id="@+id/a"/><Button android:id="@id/a"/></LinearLayout>')
 def test_generic_workspace_example(self):
  res=Path('codes/libs/ui/libbhoo/examples/android_xml/res')
  out=mod.compile_layout(res/'layout'/'screen.xml',res)
  self.assertEqual(out,(res.parent/'screen.rs').read_text(), 'Regenerate the checked-in sample after importer changes')
  for name in ['header','description','confirm','status']:self.assertIn('id:"'+name+'"',out)
  self.assertIn('visibility:Visibility::Gone',out)
 def test_includes_merge_styles_and_tools_attributes(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'layout').mkdir()
   (res/'values'/'values.xml').write_text('''<resources>
     <string name="caption">From resources</string><dimen name="space">12dp</dimen>
     <style name="BaseLabel"><item name="android:textSize">18sp</item><item name="android:padding">@dimen/space</item></style>
     <style name="BaseLabel.Emphasis"><item name="android:textColor">#123456</item></style>
   </resources>''')
   (res/'layout'/'header.xml').write_text('''<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/title" style="@style/BaseLabel.Emphasis" android:text="@string/caption" android:layout_width="wrap_content" android:layout_height="wrap_content"/>''')
   (res/'layout'/'actions.xml').write_text('''<merge xmlns:android="http://schemas.android.com/apk/res/android"><Button android:id="@+id/confirm" android:text="Confirm"/><Button android:id="@+id/hidden" android:visibility="gone"/></merge>''')
   root=res/'layout'/'screen.xml';root.write_text('''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:tools="http://schemas.android.com/tools" tools:context=".Example" android:orientation="vertical"><include layout="@layout/header" android:id="@+id/heading" android:layout_width="match_parent" android:layout_height="40dp"/><include layout="@layout/actions"/></LinearLayout>''')
   out=mod.compile_layout(root,res)
   for expected in ['id:"heading"','text:"From resources"','text_size:18.0','padding_edges:[12.0,12.0,12.0,12.0]','Color(18,52,86,255)','height:Size::Dp(40.0)','id:"confirm"','visibility:Visibility::Gone']:
    self.assertIn(expected,out)
   self.assertNotIn('id:"title"',out)
 def test_include_and_resource_cycles_are_errors(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'layout').mkdir();(res/'values').mkdir()
   (res/'layout'/'a.xml').write_text('<LinearLayout><include layout="@layout/b"/></LinearLayout>')
   (res/'layout'/'b.xml').write_text('<LinearLayout><include layout="@layout/a"/></LinearLayout>')
   with self.assertRaisesRegex(ValueError,'Cyclic include'):mod.compile_layout(res/'layout'/'a.xml',res)
   (res/'values'/'values.xml').write_text('<resources><string name="a">@string/b</string><string name="b">@string/a</string><style name="A" parent="B"/><style name="B" parent="A"/></resources>')
   with self.assertRaisesRegex(ValueError,'Cyclic resource reference'):self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="@string/a"/>',res)
   with self.assertRaisesRegex(ValueError,'Cyclic style inheritance'):self.run_layout('<TextView style="@style/A"/>',res)
 def test_merge_and_include_errors_are_explicit(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'layout').mkdir();(res/'layout'/'part.xml').write_text('<merge><View/></merge>')
   with self.assertRaisesRegex(ValueError,'merge must be included'):self.run_layout('<merge><View/></merge>',res)
   with self.assertRaisesRegex(ValueError,'cannot override'):self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><include layout="@layout/part" android:id="@+id/x"/></LinearLayout>',res)
   with self.assertRaisesRegex(ValueError,'both width and height'):self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><include layout="@layout/part" android:layout_width="match_parent"/></LinearLayout>',res)
   with self.assertRaisesRegex(ValueError,'Invalid include layout'):self.run_layout('<LinearLayout><include layout="@layout/../../outside"/></LinearLayout>',res)
 def test_margins_precedence_and_negative_dp(self):
  xml='''<View xmlns:android="http://schemas.android.com/apk/res/android" android:layout_margin="4dp" android:layout_marginHorizontal="6dp" android:layout_marginLeft="8dp" android:layout_marginStart="10dp" android:layout_marginBottom="-2dp"/>'''
  out=self.run_layout(xml);self.assertIn('margin:[8.0,4.0,6.0,-2.0]',out);self.assertIn('margin_relative:[Some(10.0),None]',out)
  sp_out=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:layout_margin="40sp" android:layout_marginTop="-4px" android:padding="12px" android:layout_width="fill_parent"/>')
  self.assertIn('margin:[40.0,-4.0,40.0,40.0]',sp_out);self.assertIn('padding_edges:[12.0,12.0,12.0,12.0]',sp_out);self.assertIn('width:Size::Match',sp_out)
  with self.assertRaisesRegex(ValueError,'Expected margin in dp'):self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:layout_margin="2em"/>')
 def test_visibility_values_are_checked(self):
  with self.assertRaisesRegex(ValueError,'Unsupported visibility'):self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:visibility="hidden"/>')
 def test_listview_xml_entries_choices_and_dividers(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'drawable').mkdir()
   (res/'values'/'items.xml').write_text('<resources><string-array name="items"><item>Alpha</item><item>Beta</item></string-array></resources>')
   (res/'drawable'/'line.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#123456"/><size android:height="2dp"/></shape>')
   xml='<ListView xmlns:android="http://schemas.android.com/apk/res/android" android:entries="@array/items" android:choiceMode="multipleChoice" android:divider="@drawable/line" android:dividerHeight="3dp"/>'
   out=self.run_layout(xml,res)
   for expected in ('Kind::ListView','kind:82, anchor:"Alpha"','kind:82, anchor:"Beta"','kind:88, anchor:"multipleChoice"','divider_thickness:3.0','divider_color:Some(Color(18,52,86,255))'):self.assertIn(expected,out)
   disabled=xml.replace('/>', ' android:headerDividersEnabled="false" android:footerDividersEnabled="false"/>')
   for expected in ('kind:89, anchor:"false"','kind:90, anchor:"false"'):self.assertIn(expected,self.run_layout(disabled,res))
   with self.assertRaisesRegex(ValueError,'Invalid headerDividersEnabled'):self.run_layout(disabled.replace('headerDividersEnabled="false"','headerDividersEnabled="maybe"'),res)
   with self.assertRaisesRegex(ValueError,'Unsupported ListView choiceMode'):self.run_layout(xml.replace('multipleChoice','modal'),res)
   with self.assertRaisesRegex(ValueError,'ListView attribute on unsupported view'):self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:choiceMode="singleChoice"/>')
 def test_expandable_divider_attributes(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   for name,color in [('group','#336a50'),('child','#a1c7b1')]:
    (res/'drawable'/f'{name}.xml').write_text(f'<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="{color}"/><size android:height="2dp"/></shape>')
   xml='<ExpandableListView xmlns:android="http://schemas.android.com/apk/res/android" android:divider="@drawable/group" android:childDivider="@drawable/child" android:dividerHeight="3dp"/>'
   out=self.run_layout(xml,res)
   for expected in ('divider_thickness:3.0','divider_color:Some(Color(51,106,80,255))','kind:110, anchor:"Some(Color(161,199,177,255))"'):self.assertIn(expected,out)
   with self.assertRaisesRegex(ValueError,'childDivider unsupported'):self.run_layout(xml.replace('ExpandableListView','ListView'),res)
 def test_framework_toolbar_xml(self):
  xml='<Toolbar xmlns:android="http://schemas.android.com/apk/res/android" android:title="Inbox" android:subtitle="Account" android:navigationIcon="@drawable/back" android:navigationContentDescription="Back"/>'
  out=self.run_layout(xml)
  for expected in ('Kind::Toolbar','text:"Inbox"','hint:"Account"','image_src:"back"','kind:111, anchor:"Back"'):self.assertIn(expected,out)
  self.assertIn('kind:Kind::Text',self.run_layout(xml.replace('/>','><TextView/></Toolbar>')))
  for expected in ('kind:112, anchor:"32.0"','kind:116, anchor:"80.0"'):
   self.assertIn(expected,self.run_layout(xml.replace('/>',' android:contentInsetStart="32dp" android:contentInsetStartWithNavigation="80dp"/>')))
  for expected in ('kind:118, anchor:"4.0"','kind:119, anchor:"8.0"'):
   self.assertIn(expected,self.run_layout(xml.replace('/>',' android:titleMargin="4dp" android:titleMarginStart="8dp"/>')))
  for expected in ('kind:123, anchor:"24,61,46,255"','kind:124, anchor:"76,103,87,255"'):
   self.assertIn(expected,self.run_layout(xml.replace('/>',' android:titleTextColor="#183D2E" android:subtitleTextColor="#4C6757"/>')))
  with self.assertRaisesRegex(ValueError,'Toolbar attribute on unsupported view'):self.run_layout(xml.replace('Toolbar','TextView'))
 def test_toolbar_text_appearance_inheritance_and_errors(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();styles=res/'values'/'styles.xml'
   styles.write_text('<resources><style name="Base"><item name="android:textSize">18sp</item><item name="android:textStyle">bold</item></style><style name="Title" parent="Base"><item name="android:textColor">#123456</item></style></resources>')
   xml='<Toolbar xmlns:android="http://schemas.android.com/apk/res/android" android:title="Inbox" android:titleTextAppearance="@style/Title"/>'
   out=self.run_layout(xml,res)
   for expected in ('kind:125, anchor:"18.0"','kind:126, anchor:"bold"','kind:129, anchor:"18,52,86,255"'):self.assertIn(expected,out)
   styles.write_text('<resources><style name="Title"><item name="android:gravity">center</item></style></resources>')
   with self.assertRaisesRegex(ValueError,'Unsupported Toolbar text appearance'):self.run_layout(xml,res)
 def test_adapter_view_flipper_xml(self):
  xml='<AdapterViewFlipper xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/f" android:autoStart="true" android:flipInterval="2500"/>'
  out=self.run_layout(xml)
  for expected in ('Kind::AdapterViewFlipper','auto_start:true','flip_interval:2.5'):self.assertIn(expected,out)
  self.assertIn('flip_interval:10.0',self.run_layout(xml.replace(' android:flipInterval="2500"','')))
  with self.assertRaisesRegex(ValueError,'Leaf view cannot have children'):self.run_layout(xml.replace('/>','><TextView/></AdapterViewFlipper>'))
 def test_adapter_flipper_alpha_object_animator(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'animator').mkdir();anim=res/'animator'/'fade.xml'
   anim.write_text('<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="alpha" android:valueFrom="0" android:valueTo="1" android:duration="800" android:interpolator="@android:anim/linear_interpolator"/>')
   xml='<AdapterViewFlipper xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@animator/fade"/>'
   out=self.run_layout(xml,res)
   self.assertIn('duration:0.8, start_offset:0.0, linear:true',out)
   anim.write_text(anim.read_text().replace('alpha','backgroundColor'))
   with self.assertRaisesRegex(ValueError,'Unsupported objectAnimator resource'):self.run_layout(xml,res)
 def test_expandable_indicator_selector(self):
  import shutil
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();shutil.copy(Path(__file__).resolve().parents[1]/'codes/apps/bhoo-web/android-examples/res/drawable/toolbar_back.png',res/'drawable'/'icon.png')
   selector=res/'drawable'/'group.xml';selector.write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_expanded="true" android:drawable="@drawable/icon"/><item android:drawable="@null"/></selector>')
   xml='<ExpandableListView xmlns:android="http://schemas.android.com/apk/res/android" android:groupIndicator="@drawable/group" android:childIndicator="@null"/>'
   out=self.run_layout(xml,res)
   for expected in ('kind:131, anchor:"1,0,icon"','kind:131, anchor:"0,0,"','kind:132, anchor:"0,0,"','Image::rgba('):self.assertIn(expected,out)
   selector.write_text(selector.read_text().replace('state_expanded','state_pressed'))
   with self.assertRaisesRegex(ValueError,'Unsupported indicator selector item'):self.run_layout(xml,res)
 def test_expandable_vector_indicator_and_tint(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();vector=res/'drawable'/'indicator.xml'
   vector.write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24" android:tint="?attr/colorControlNormal"><path android:fillColor="#ffffff" android:pathData="M0 0h24v24h-24z"/></vector>')
   xml='<ExpandableListView xmlns:android="http://schemas.android.com/apk/res/android" android:groupIndicator="@drawable/indicator"/>'
   out=self.run_layout(xml,res)
   for expected in ('kind:131, anchor:"0,0,indicator"','Image::rgba(24,24','AndroidDrawableTint::ControlNormal'):self.assertIn(expected,out)
   vector.write_text(vector.read_text().replace('?attr/colorControlNormal','?attr/unknown'))
   with self.assertRaisesRegex(ValueError,'Unsupported vector tint'):self.run_layout(xml,res)
 def test_adapter_flipper_object_animator_set(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'animator').mkdir();anim=res/'animator'/'slide.xml'
   anim.write_text('<set xmlns:android="http://schemas.android.com/apk/res/android" android:ordering="together"><objectAnimator android:propertyName="translationX" android:valueFrom="48dp" android:valueTo="0dp" android:duration="800"/><objectAnimator android:propertyName="alpha" android:valueFrom="0" android:valueTo="1" android:duration="800"/></set>')
   xml='<AdapterViewFlipper xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@animator/slide"/>'
   out=self.run_layout(xml,res);self.assertIn('value:48.0, mode:0',out);self.assertIn('duration:0.8',out)
   anim.write_text(anim.read_text().replace('together','interleaved'))
   with self.assertRaisesRegex(ValueError,'Unsupported AnimatorSet ordering'):self.run_layout(xml,res)
 def test_adapter_flipper_two_translation_axes(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'animator').mkdir();anim=res/'animator'/'diagonal.xml'
   x='<objectAnimator android:propertyName="translationX" android:valueFrom="48dp" android:valueTo="0dp" android:duration="800"/>'
   y='<objectAnimator android:propertyName="translationY" android:valueFrom="32dp" android:valueTo="0dp" android:duration="800"/>'
   wrap=lambda children:'<set xmlns:android="http://schemas.android.com/apk/res/android">'+children+'</set>'
   xml='<AdapterViewFlipper xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@animator/diagonal"/>'
   anim.write_text(wrap(x+y));out=self.run_layout(xml,res)
   self.assertIn('value:48.0, mode:0',out);self.assertIn('value:32.0, mode:0',out)
   anim.write_text(wrap(x+y.replace('800','400')))
   self.assertIn('y_timing:Some((0.4,false,0.0))',self.run_layout(xml,res))
   anim.write_text(wrap(x.replace('duration="800"','duration="800" android:startOffset="200"')+y.replace('duration="800"','duration="800" android:startOffset="500"')))
   out=self.run_layout(xml,res);self.assertIn('start_offset:0.2',out);self.assertIn('y_timing:Some((0.8,false,0.5))',out)
   anim.write_text(wrap(x+x))
   with self.assertRaisesRegex(ValueError,'Duplicate animator property'):self.run_layout(xml,res)
 def test_gridview_column_attributes(self):
  xml='<GridView xmlns:android="http://schemas.android.com/apk/res/android" android:numColumns="auto_fit" android:columnWidth="96dp" android:horizontalSpacing="8dp" android:verticalSpacing="12dp" android:stretchMode="spacingWidthUniform" android:choiceMode="multipleChoice"/>'
  out=self.run_layout(xml)
  for expected in ('Kind::GridView','kind:91, anchor:"auto_fit"','kind:92, anchor:"96.0"','kind:93, anchor:"8.0"','kind:94, anchor:"12.0"','kind:95, anchor:"spacingWidthUniform"','kind:88, anchor:"multipleChoice"'):self.assertIn(expected,out)
  with self.assertRaisesRegex(ValueError,'Invalid GridView numColumns'):self.run_layout(xml.replace('auto_fit','0'))
  with self.assertRaisesRegex(ValueError,'Unsupported GridView stretchMode'):self.run_layout(xml.replace('spacingWidthUniform','invalid'))
  with self.assertRaisesRegex(ValueError,'GridView attribute on unsupported view'):self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:numColumns="2"/>')
 def test_background_color_state_selector(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'color').mkdir();(res/'color'/'row.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_pressed="true" android:color="#123456"/><item android:state_activated="true" android:color="#abcdef"/><item android:color="#ffffff"/></selector>')
   out=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@color/row"/>',res)
   for expected in ('kind:96, anchor:"2,0,18,52,86,255"','kind:96, anchor:"16,0,171,205,239,255"','kind:96, anchor:"0,0,255,255,255,255"'):self.assertIn(expected,out)
 def test_autocomplete_xml_threshold_and_dropdown_attributes(self):
  xml='<AutoCompleteTextView xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/city" android:hint="City" android:inputType="text" android:completionThreshold="3" android:dropDownWidth="200dp" android:dropDownHeight="160dp" android:dropDownHorizontalOffset="-4dp" android:dropDownVerticalOffset="6dp"/>'
  out=self.run_layout(xml)
  for expected in ('Kind::AutoComplete','hint:"City"','kind:97, anchor:"3"','kind:85, anchor:"200.0"','kind:98, anchor:"160.0"','kind:86, anchor:"-4.0"','kind:87, anchor:"6.0"'):self.assertIn(expected,out)
  with self.assertRaisesRegex(ValueError,'Invalid completionThreshold'):self.run_layout(xml.replace('completionThreshold="3"','completionThreshold="-1"'))
  with self.assertRaisesRegex(ValueError,'AutoComplete attribute on unsupported view'):self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:completionThreshold="2"/>')
 def test_calendar_view_xml_range_and_week_start(self):
  xml='<CalendarView xmlns:android="http://schemas.android.com/apk/res/android" android:minDate="02/29/2024" android:maxDate="12/31/2025" android:firstDayOfWeek="2"/>'
  out=self.run_layout(xml)
  for expected in ('Kind::CalendarView','kind:99, anchor:"2024-02-29"','kind:100, anchor:"2025-12-31"','kind:101, anchor:"2"'):self.assertIn(expected,out)
  with self.assertRaisesRegex(ValueError,'Invalid CalendarView minDate'):self.run_layout(xml.replace('02/29/2024','02/29/2023'))
  with self.assertRaisesRegex(ValueError,'maxDate must be'):self.run_layout(xml.replace('12/31/2025','01/01/2024'))
  with self.assertRaisesRegex(ValueError,'Invalid firstDayOfWeek'):self.run_layout(xml.replace('firstDayOfWeek="2"','firstDayOfWeek="8"'))
  with self.assertRaisesRegex(ValueError,'CalendarView attribute on unsupported view'):self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:firstDayOfWeek="1"/>')
 def test_date_picker_calendar_mode_is_explicit(self):
  xml='<DatePicker xmlns:android="http://schemas.android.com/apk/res/android" android:datePickerMode="calendar" android:minDate="02/29/2024" android:firstDayOfWeek="2"/>'
  out=self.run_layout(xml)
  for expected in ('Kind::DatePicker','kind:99, anchor:"2024-02-29"','kind:101, anchor:"2"'):self.assertIn(expected,out)
  self.assertIn('anchor:"spinner"',self.run_layout(xml.replace('datePickerMode="calendar"','datePickerMode="spinner"')))
  with self.assertRaisesRegex(ValueError,'Invalid DatePicker mode'):self.run_layout(xml.replace('datePickerMode="calendar"','datePickerMode="unknown"'))
  with self.assertRaisesRegex(ValueError,'explicitly select a mode'):self.run_layout(xml.replace('android:datePickerMode="calendar" ',''))
 def test_number_picker_is_supported_without_invented_attributes(self):
  out=self.run_layout('<NumberPicker xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/quantity"/>')
  self.assertIn('Kind::NumberPicker',out)
  with self.assertRaises(ValueError):self.run_layout('<NumberPicker xmlns:android="http://schemas.android.com/apk/res/android" android:minValue="1"/>')
 def test_date_picker_spinner_visibility_attributes(self):
  xml='<DatePicker xmlns:android="http://schemas.android.com/apk/res/android" android:datePickerMode="spinner" android:spinnersShown="false" android:calendarViewShown="true"/>'
  out=self.run_layout(xml)
  self.assertIn('kind:103, anchor:"false"',out)
  self.assertIn('kind:104, anchor:"true"',out)
  with self.assertRaisesRegex(ValueError,'Invalid spinnersShown'):self.run_layout(xml.replace('spinnersShown="false"','spinnersShown="maybe"'))
  with self.assertRaisesRegex(ValueError,'requires DatePicker spinner mode'):self.run_layout(xml.replace('datePickerMode="spinner"','datePickerMode="calendar"'))
 def test_time_picker_requires_explicit_spinner_mode(self):
  xml='<TimePicker xmlns:android="http://schemas.android.com/apk/res/android" android:timePickerMode="spinner"/>'
  out=self.run_layout(xml);self.assertIn('Kind::TimePicker',out);self.assertIn('kind:105, anchor:"spinner"',out)
  self.assertIn('anchor:"clock"',self.run_layout(xml.replace('timePickerMode="spinner"','timePickerMode="clock"')))
  with self.assertRaisesRegex(ValueError,'explicitly select clock or spinner'):self.run_layout(xml.replace('timePickerMode="spinner"','timePickerMode="invalid"'))
  with self.assertRaisesRegex(ValueError,'TimePicker attribute on unsupported view'):self.run_layout(xml.replace('<TimePicker','<TextView'))
 def test_search_view_iconified_and_hint(self):
  xml='<SearchView xmlns:android="http://schemas.android.com/apk/res/android" android:iconifiedByDefault="false" android:queryHint="Find items"/>'
  out=self.run_layout(xml);self.assertIn('Kind::SearchView',out);self.assertIn('kind:106, anchor:"false"',out);self.assertIn('kind:107, anchor:"Find items"',out)
  with self.assertRaisesRegex(ValueError,'Invalid iconifiedByDefault'):self.run_layout(xml.replace('iconifiedByDefault="false"','iconifiedByDefault="yes"'))
 def test_expandable_list_xml_tag(self):
  out=self.run_layout('<ExpandableListView xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/tree"/>')
  self.assertIn('Kind::ExpandableListView',out)
 def test_action_menu_view_xml_tag(self):
  out=self.run_layout('<ActionMenuView xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/actions" android:layout_height="56dp"/>')
  self.assertIn('Kind::ActionMenuView',out);self.assertIn('id:"actions"',out)
 def test_stack_view_xml_tag(self):
  out=self.run_layout('<StackView xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/stack" android:layout_height="220dp"/>')
  self.assertIn('Kind::StackView',out);self.assertIn('id:"stack"',out)
  looping=self.run_layout('<StackView xmlns:android="http://schemas.android.com/apk/res/android" android:loopViews="true"/>');self.assertIn('kind:137, anchor:"true"',looping)
  with self.assertRaisesRegex(ValueError,'Invalid loopViews'):self.run_layout('<StackView xmlns:android="http://schemas.android.com/apk/res/android" android:loopViews="maybe"/>')
  flipper=self.run_layout('<AdapterViewFlipper xmlns:android="http://schemas.android.com/apk/res/android" android:loopViews="false"/>');self.assertIn('kind:137, anchor:"false"',flipper)
 def test_gallery_attributes(self):
  xml='<Gallery xmlns:android="http://schemas.android.com/apk/res/android" android:spacing="16dp" android:unselectedAlpha="0.3" android:animationDuration="700"/>'
  out=self.run_layout(xml);self.assertIn('Kind::Gallery',out);self.assertIn('kind:144, anchor:"16.0"',out);self.assertIn('kind:145, anchor:"0.3"',out);self.assertIn('kind:146, anchor:"700"',out)
  with self.assertRaisesRegex(ValueError,'Invalid unselectedAlpha'):self.run_layout(xml.replace('0.3','1.5'))
  with self.assertRaisesRegex(ValueError,'Gallery attributes unsupported'):self.run_layout(xml.replace('Gallery','TextView'))
 def test_tab_widget_xml(self):
  xml='<TabWidget xmlns:android="http://schemas.android.com/apk/res/android" android:tabStripEnabled="false"><TextView android:text="One"/></TabWidget>'
  out=self.run_layout(xml);self.assertIn('Kind::TabWidget',out);self.assertIn('kind:147, anchor:"false"',out)
  with self.assertRaisesRegex(ValueError,'Invalid tabStripEnabled'):self.run_layout(xml.replace('false','maybe'))
  with self.assertRaisesRegex(ValueError,'tabStripEnabled unsupported'):self.run_layout(xml.replace('TabWidget','LinearLayout'))
 def test_tab_widget_custom_strips(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();strip=res/'drawable'/'strip.xml'
   strip.write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#123456"/><size android:width="16dp" android:height="6dp"/></shape>')
   xml='<TabWidget xmlns:android="http://schemas.android.com/apk/res/android" android:tabStripLeft="@drawable/strip" android:tabStripRight="@null"><TextView/></TabWidget>'
   out=self.run_layout(xml,res);self.assertIn('kind:191, anchor:"16.0,6.0,18,52,86,255"',out);self.assertIn('kind:192, anchor:"null"',out)
   strip.write_text('<shape/>')
   with self.assertRaisesRegex(ValueError,'sized solid rectangle'):self.run_layout(xml,res)
   with self.assertRaisesRegex(ValueError,'tabStripRight unsupported'):self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:tabStripRight="@null"/>')
 def test_tab_widget_weight_sum(self):
  xml='<TabWidget xmlns:android="http://schemas.android.com/apk/res/android" android:weightSum="5"><TextView android:layout_width="0dp" android:layout_weight="2"/></TabWidget>'
  out=self.run_layout(xml);self.assertIn('weight_sum:5.0',out);self.assertIn('weight:2.0',out)
 def test_sliding_drawer_nested_descendant_ids(self):
  xml='<SlidingDrawer xmlns:android="http://schemas.android.com/apk/res/android" android:handle="@id/h" android:content="@id/c"><FrameLayout><Button android:id="@+id/h"/></FrameLayout><FrameLayout><FrameLayout android:id="@+id/c"/></FrameLayout></SlidingDrawer>'
  out=self.run_layout(xml);self.assertIn('kind:138, anchor:"h"',out);self.assertIn('kind:139, anchor:"c"',out)
  with self.assertRaisesRegex(ValueError,'uniquely identified'):self.run_layout(xml.replace('@+id/c','@+id/missing'))
  with self.assertRaisesRegex(ValueError,'uniquely identified'):self.run_layout(xml.replace('@id/c','@id/h'))
  with self.assertRaisesRegex(ValueError,'uniquely identified'):self.run_layout(xml.replace('<FrameLayout android:id="@+id/c"/>','<FrameLayout android:id="@+id/c"/><View android:id="@+id/c"/>'))
 def test_tab_host_android_ids(self):
  xml='<TabHost xmlns:android="http://schemas.android.com/apk/res/android"><TabWidget android:id="@android:id/tabs"/><FrameLayout android:id="@android:id/tabcontent"/></TabHost>'
  out=self.run_layout(xml);self.assertIn('Kind::TabHost',out);self.assertIn('id:"android:tabs"',out);self.assertIn('id:"android:tabcontent"',out)
 def test_dialer_filter_required_children(self):
  xml='<DialerFilter xmlns:android="http://schemas.android.com/apk/res/android"><EditText android:id="@android:id/primary"/><EditText android:id="@android:id/hint" android:layout_below="@android:id/primary"/></DialerFilter>'
  out=self.run_layout(xml);self.assertIn('Kind::DialerFilter',out);self.assertIn('id:"android:primary"',out)
  with self.assertRaisesRegex(ValueError,'DialerFilter requires EditText'):self.run_layout(xml.replace('@android:id/hint','@+id/wrong'))
 def test_theme_attr_color_and_dimension_resolution(self):
  xml='<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:layout_height="?attr/actionBarSize" android:background="?attr/colorSurface" android:paddingStart="?android:attr/listPreferredItemPaddingStart"><TextView android:textColor="?android:attr/textColorPrimary" android:background="?attr/selectableItemBackground"/><TextView android:textColor="?attr/colorError" android:background="?attr/colorPrimary"/></LinearLayout>'
  out=self.run_layout(xml)
  for expected in ('height:Size::Dp(56.0)','background:Some(Color(254,247,255,255))','padding_relative:[Some(16.0),None]','color:Some(Color(29,27,32,255))','background:Some(Color(0,0,0,0))','color:Some(Color(179,38,30,255))','background:Some(Color(103,80,164,255))'):
   self.assertIn(expected,out)
  with self.assertRaisesRegex(ValueError,'Unresolved theme color'):self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textColor="?attr/unknownColor"/>')
  with self.assertRaisesRegex(ValueError,'Unresolved theme dimension'):self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:layout_height="?attr/unknownDimen"/>')
 def test_appcompat_and_material_widget_tags(self):
  cases=[
   ('com.google.android.material.textview.MaterialTextView','Kind::Text'),
   ('androidx.appcompat.widget.AppCompatTextView','Kind::Text'),
   ('androidx.appcompat.widget.AppCompatImageView','Kind::Image'),
   ('androidx.appcompat.widget.AppCompatEditText','Kind::Edit'),
   ('com.google.android.material.textfield.TextInputEditText','Kind::Edit'),
   ('com.google.android.material.button.MaterialButton','Kind::Button'),
   ('androidx.appcompat.widget.AppCompatButton','Kind::Button'),
   ('com.google.android.material.appbar.MaterialToolbar','Kind::Toolbar'),
   ('androidx.appcompat.widget.Toolbar','Kind::Toolbar'),
   ('androidx.appcompat.widget.AppCompatCheckBox','Kind::CheckBox'),
   ('androidx.appcompat.widget.SwitchCompat','Kind::Switch'),
   ('com.google.android.material.switchmaterial.SwitchMaterial','Kind::Switch'),
   ('com.google.android.material.materialswitch.MaterialSwitch','Kind::Switch'),
   ('androidx.appcompat.widget.AppCompatRadioButton','Kind::RadioButton'),
  ]
  for tag,kind in cases:
   with self.subTest(tag=tag):
    self.assertIn('kind:'+kind,self.run_layout(f'<{tag} xmlns:android="http://schemas.android.com/apk/res/android"/>'))
  fab=self.run_layout('<com.google.android.material.floatingactionbutton.FloatingActionButton xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:srcCompat="@drawable/add"/>')
  self.assertIn('kind:Kind::Icon',fab);self.assertIn('radius:-1.0',fab);self.assertIn('icon:MaterialIcon::Add',fab);self.assertIn('background:Some(Color(234,221,255,255))',fab)
  til=self.run_layout('<com.google.android.material.textfield.TextInputLayout xmlns:android="http://schemas.android.com/apk/res/android" android:hint="Email"><com.google.android.material.textfield.TextInputEditText/></com.google.android.material.textfield.TextInputLayout>')
  self.assertIn('kind:Kind::Linear',til);self.assertIn('kind:Kind::Edit',til);self.assertIn('hint:"Email"',til)
 def test_app_srccompat_tint_and_background_tint(self):
  img=self.run_layout('<androidx.appcompat.widget.AppCompatImageView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:srcCompat="@drawable/search" app:tint="?attr/colorPrimary"/>')
  self.assertIn('icon:MaterialIcon::Search',img);self.assertIn('image_tint:Some(Color(103,80,164,255))',img)
  btn=self.run_layout('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:backgroundTint="?attr/colorSecondary"/>')
  self.assertIn('background:Some(Color(98,91,113,255))',btn)
  android_tint=self.run_layout('<Button xmlns:android="http://schemas.android.com/apk/res/android" android:backgroundTint="#123456"/>')
  self.assertIn('background:Some(Color(18,52,86,255))',android_tint)
  explicit_bg=self.run_layout('<Button xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" android:background="#abcdef" app:backgroundTint="#123456"/>')
  self.assertIn('background:Some(Color(171,205,239,255))',explicit_bg)
  self.assertIn('kind:194, anchor:"18,52,86,255"',explicit_bg)
 def test_elevation_background_tint_and_text_alignment_rules(self):
  out=self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" android:elevation="6dp" app:backgroundTint="?attr/colorPrimary" android:textAlignment="center"/>')
  self.assertIn('kind:193, anchor:"6.0"',out)
  self.assertIn('kind:194, anchor:"103,80,164,255"',out)
  self.assertIn('kind:195, anchor:"center"',out)
  zero_elev=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:elevation="0dp"/>')
  self.assertNotIn('kind:193',zero_elev)
  for align in ('textStart','viewStart','center','textEnd','viewEnd','left','right','gravity'):
   with self.subTest(textAlignment=align):
    aligned=self.run_layout(f'<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textAlignment="{align}"/>')
    self.assertIn(f'kind:195, anchor:"{align}"',aligned)
  with self.assertRaisesRegex(ValueError,'Unsupported textAlignment'):
   self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textAlignment="justify"/>')
 def test_androidx_and_material_containers_and_common_attrs(self):
  rv_list=self.run_layout('<androidx.recyclerview.widget.RecyclerView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:layoutManager="androidx.recyclerview.widget.LinearLayoutManager" android:scrollbars="vertical" android:focusable="true"/>')
  self.assertIn('kind:Kind::ListView',rv_list)
  rv_grid=self.run_layout('<androidx.recyclerview.widget.RecyclerView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:layoutManager="androidx.recyclerview.widget.GridLayoutManager"/>')
  self.assertIn('kind:Kind::GridView',rv_grid)
  coord=self.run_layout('<androidx.coordinatorlayout.widget.CoordinatorLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><com.google.android.material.appbar.AppBarLayout><com.google.android.material.appbar.MaterialToolbar/></com.google.android.material.appbar.AppBarLayout><androidx.fragment.app.FragmentContainerView app:layout_behavior="@string/appbar_scrolling_view_behavior"/><ViewStub/></androidx.coordinatorlayout.widget.CoordinatorLayout>')
  self.assertIn('kind:Kind::Frame',coord)
  self.assertIn('kind:Kind::Linear',coord)
  self.assertIn('horizontal:false',coord)
  self.assertIn('kind:Kind::View',coord)
  self.assertIn('visibility:Visibility::Gone',coord)
  stub_visible=self.run_layout('<ViewStub xmlns:android="http://schemas.android.com/apk/res/android" android:visibility="visible"/>')
  self.assertIn('kind:Kind::View',stub_visible)
  self.assertIn('visibility:Visibility::Visible',stub_visible)
 def test_constraint_layout_and_rules(self):
  xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <View android:id="@+id/anchor" app:layout_constraintStart_toStartOf="parent" app:layout_constraintTop_toTopOf="parent"/>
   <TextView android:id="@+id/target" app:layout_constraintStart_toEndOf="@id/anchor" app:layout_constraintEnd_toEndOf="parent" app:layout_constraintTop_toBottomOf="@+id/anchor" app:layout_constraintBottom_toBottomOf="parent" app:layout_constraintHorizontal_bias="0.25" app:layout_constraintVertical_bias="0.75"/>
   <Button app:layout_constraintLeft_toLeftOf="parent" app:layout_constraintLeft_toRightOf="@id/anchor" app:layout_constraintRight_toRightOf="parent" app:layout_constraintRight_toLeftOf="@id/target" app:layout_constraintEnd_toStartOf="@id/target" app:layout_constraintBottom_toTopOf="@id/target"/>
  </androidx.constraintlayout.widget.ConstraintLayout>'''
  out=self.run_layout(xml)
  self.assertIn('kind:Kind::ConstraintLayout',out)
  for token in (
   'RelativeRule { kind:196, anchor:"parent" }',
   'RelativeRule { kind:200, anchor:"parent" }',
   'RelativeRule { kind:197, anchor:"anchor" }',
   'RelativeRule { kind:198, anchor:"parent" }',
   'RelativeRule { kind:201, anchor:"anchor" }',
   'RelativeRule { kind:202, anchor:"parent" }',
   'RelativeRule { kind:204, anchor:"0.25" }',
   'RelativeRule { kind:205, anchor:"0.75" }',
   'RelativeRule { kind:199, anchor:"target" }',
   'RelativeRule { kind:203, anchor:"target" }',
  ):
   self.assertIn(token,out)
  short=self.run_layout('<ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android"><View/></ConstraintLayout>')
  self.assertIn('kind:Kind::ConstraintLayout',short)
  with self.assertRaisesRegex(ValueError,'Invalid ConstraintLayout anchor'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintStart_toStartOf="bad_id"/></ConstraintLayout>')
  with self.assertRaisesRegex(ValueError,'Invalid ConstraintLayout bias'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintHorizontal_bias="1.5"/></ConstraintLayout>')
 def test_app_local_theme_attr_resolution_and_precedence(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'drawable').mkdir()
   (res/'values'/'colors.xml').write_text('<resources><color name="brand">#112233</color><color name="dark_bg">#000000</color></resources>')
   (res/'values'/'dimens.xml').write_text('<resources><dimen name="toolbar_pad">18dp</dimen></resources>')
   (res/'drawable'/'row_bg.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#445566"/><corners android:radius="6dp"/></shape>')
   (res/'values'/'themes.xml').write_text('''<resources>
    <style name="Base.Theme.App" parent="Theme.Material3.Light">
     <item name="colorPrimary">@color/brand</item>
     <item name="profile_toolbar_background">@drawable/row_bg</item>
     <item name="emoji_text_color">?attr/colorPrimary</item>
     <item name="custom_pad">@dimen/toolbar_pad</item>
     <item name="header_style">@style/HeaderText</item>
    </style>
    <style name="Theme.App" parent="Base.Theme.App">
     <item name="emoji_text_color">#654321</item>
    </style>
    <style name="Theme.App.Dark" parent="Theme.Material3.Dark">
     <item name="colorPrimary">@color/dark_bg</item>
     <item name="android:textColorSecondary">@color/nonexistent_dark_color</item>
     <item name="dark_only_tint">#778899</item>
    </style>
    <style name="HeaderText"><item name="android:textSize">20sp</item></style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"
    android:background="?attr/profile_toolbar_background"
    android:padding="?attr/custom_pad"
    android:layout_height="?attr/actionBarSize">
    <TextView style="?attr/header_style" android:textColor="?emoji_text_color" android:background="?attr/colorPrimary"/>
    <TextView android:textColor="?android:attr/textColorSecondary" android:background="?attr/dark_only_tint"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   for expected in (
    'background:Some(Color(68,85,102,255))',
    'radius:6.0',
    'padding_edges:[18.0,18.0,18.0,18.0]',
    'height:Size::Dp(56.0)',
    'text_size:20.0',
    'color:Some(Color(101,67,33,255))',
    'background:Some(Color(17,34,51,255))',
    'color:Some(Color(73,69,79,255))',
    'background:Some(Color(119,136,153,255))',
   ):
    self.assertIn(expected,out)
   with self.assertRaisesRegex(ValueError,'Unresolved theme color'):
    self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textColor="?attr/still_missing"/>',res)
   (res/'values'/'cycle.xml').write_text('<resources><style name="Theme.App.Cycle" parent="Theme.App"><item name="cyc_a">?attr/cyc_b</item><item name="cyc_b">?attr/cyc_a</item></style></resources>')
   with self.assertRaisesRegex(ValueError,'Cyclic resource reference'):
    self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textColor="?attr/cyc_a"/>',res)
 def test_builtin_text_appearances_and_widget_styles(self):
  cases=[
   ('android:textAppearance="?attr/textAppearanceDisplayLarge"',('text_size:57.0','letter_spacing_em:-0.00438596','text_all_caps:false')),
   ('android:textAppearance="?attr/textAppearanceBodyMedium"',('text_size:14.0','letter_spacing_em:0.01785714')),
   ('android:textAppearance="?attr/textAppearanceLabelSmall"',('text_size:11.0','letter_spacing_em:0.04545455')),
   ('android:textAppearance="?attr/textAppearanceHeadline5"',('text_size:24.0',)),
   ('android:textAppearance="?attr/textAppearanceOverline"',('text_size:10.0','text_all_caps:true')),
   ('android:textAppearance="?attr/textAppearanceListItem"',('text_size:16.0',)),
   ('android:textAppearance="?android:attr/textAppearanceSmall"',('text_size:14.0',)),
   ('android:textAppearance="?android:textAppearanceMedium"',('text_size:18.0',)),
   ('style="?android:textAppearanceLarge"',('text_size:22.0',)),
   ('android:textAppearance="@style/TextAppearance.Material3.TitleLarge"',('text_size:22.0',)),
   ('android:textAppearance="@style/TextAppearance.MaterialComponents.Caption"',('text_size:12.0',)),
   ('android:textAppearance="@style/Base.TextAppearance.AppCompat.Body1"',('text_size:14.0',)),
   ('android:textAppearance="@android:style/TextAppearance.Medium"',('text_size:18.0',)),
  ]
  for attr,tokens in cases:
   with self.subTest(attr=attr):
    out=self.run_layout(f'<TextView xmlns:android="http://schemas.android.com/apk/res/android" {attr}/>')
    for tok in tokens:self.assertIn(tok,out)
  outlined=self.run_layout('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" style="?attr/materialButtonOutlinedStyle"/>')
  self.assertIn('background:Some(Color(0,0,0,0))',outlined);self.assertIn('background_stroke:Some(Color(121,116,126,255))',outlined);self.assertIn('background_stroke_width:1.0',outlined)
  tonal=self.run_layout('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" style="@style/Widget.Material3.Button.TonalButton"/>')
  self.assertIn('background:Some(Color(232,222,248,255))',tonal)
  pb=self.run_layout('<ProgressBar xmlns:android="http://schemas.android.com/apk/res/android" style="?android:attr/progressBarStyleLarge"/>')
  self.assertIn('kind:Kind::ProgressBar',pb);self.assertIn('indeterminate:true',pb)
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('''<resources>
    <style name="CustomBtn" parent="Widget.Material3.Button">
     <item name="drawableTint">#ffffff</item>
     <item name="cornerRadius">12dp</item>
     <item name="android:textColor">#ffffff</item>
     <item name="android:paddingLeft">8dp</item>
    </style>
    <style name="AppTitle" parent="@android:style/TextAppearance.Medium">
     <item name="android:fontFamily">sans-serif-medium</item>
    </style>
    <style name="AppCompatChild" parent="Widget.AppCompat.ActionBar"/>
    <style name="ThemeChild" parent="ThemeOverlay.Material3.Light"/>
    <style name="EmptyParent" parent=""><item name="android:layout_width">match_parent</item></style>
   </resources>''')
   btn=self.run_layout('<Button xmlns:android="http://schemas.android.com/apk/res/android" style="@style/CustomBtn"/>',res)
   self.assertIn('kind:176, anchor:"0,0,255,255,255,255"',btn);self.assertIn('radius:12.0',btn);self.assertIn('padding_edges:[8.0,0.0,0.0,0.0]',btn)
   title=self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" style="@style/AppTitle"/>',res)
   self.assertIn('text_size:18.0',title)
   self.assertIn('kind:Kind::Toolbar',self.run_layout('<Toolbar style="@style/AppCompatChild"/>',res))
   self.assertIn('width:Size::Match',self.run_layout('<View style="@style/EmptyParent"/>',res))
   with self.assertRaisesRegex(ValueError,'Unresolved style'):
    self.run_layout('<View style="@style/Widget.App.MissingCustom"/>',res)
 def test_include_constraints_and_relative_rules_and_databinding_layout(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'layout').mkdir()
   (res/'layout'/'chip_item.xml').write_text('''<layout xmlns:android="http://schemas.android.com/apk/res/android">
    <data><variable name="label" type="String"/></data>
    <TextView android:id="@+id/inner" android:text="Bound"/>
   </layout>''')
   cl_xml='''<layout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <data><variable name="vm" type="Object"/></data>
    <androidx.constraintlayout.widget.ConstraintLayout>
     <View android:id="@+id/top_ref" app:layout_constraintTop_toTopOf="parent"/>
     <include android:id="@+id/bound_child" layout="@layout/chip_item" android:layout_width="0dp" android:layout_height="wrap_content"
      app:layout_constraintStart_toStartOf="parent" app:layout_constraintEnd_toEndOf="parent"
      app:layout_constraintTop_toBottomOf="@id/top_ref" app:layout_constraintHorizontal_bias="0.3"/>
    </androidx.constraintlayout.widget.ConstraintLayout>
   </layout>'''
   out=self.run_layout(cl_xml,res)
   for tok in ('kind:Kind::ConstraintLayout','id:"bound_child"','text:"Bound"','kind:196, anchor:"parent"','kind:198, anchor:"parent"','kind:201, anchor:"top_ref"','kind:204, anchor:"0.3"'):
    self.assertIn(tok,out)
   rl_xml='''<RelativeLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <View android:id="@+id/head"/>
    <include android:id="@+id/body" layout="@layout/chip_item" android:layout_width="match_parent" android:layout_height="wrap_content"
     android:layout_below="@id/head" android:layout_centerHorizontal="true"/>
   </RelativeLayout>'''
   rl_out=self.run_layout(rl_xml,res)
   self.assertIn('kind:3, anchor:"head"',rl_out);self.assertIn('kind:18, anchor:""',rl_out)
   with self.assertRaisesRegex(ValueError,'exactly one root view'):
    self.run_layout('<layout><data/></layout>',res)
 def test_constraintlayout_guideline_barrier_and_group(self):
  xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <androidx.constraintlayout.widget.Guideline android:id="@+id/g_begin" android:orientation="vertical" app:layout_constraintGuide_begin="16dp"/>
   <androidx.constraintlayout.widget.Guideline android:id="@+id/g_end" android:orientation="horizontal" app:layout_constraintGuide_end="24dp"/>
   <androidx.constraintlayout.widget.Guideline android:id="@+id/g_pct" android:orientation="vertical" app:layout_constraintGuide_percent=".65"/>
   <androidx.constraintlayout.widget.Barrier android:id="@+id/bar" app:barrierDirection="end" app:constraint_referenced_ids="g_begin,g_pct" app:barrierMargin="4dp"/>
   <androidx.constraintlayout.widget.Group android:id="@+id/grp" app:constraint_referenced_ids="g_begin"/>
  </androidx.constraintlayout.widget.ConstraintLayout>'''
  out=self.run_layout(xml)
  for tok in (
   'id:"g_begin"','RelativeRule { kind:210, anchor:"vertical" }','RelativeRule { kind:211, anchor:"16.0" }',
   'id:"g_end"','RelativeRule { kind:210, anchor:"horizontal" }','RelativeRule { kind:212, anchor:"24.0" }',
   'id:"g_pct"','RelativeRule { kind:213, anchor:"0.65" }',
   'id:"bar"','id:"grp"','visibility:Visibility::Gone',
  ):
   self.assertIn(tok,out)
  with self.assertRaisesRegex(ValueError,'Invalid Guideline percent'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><androidx.constraintlayout.widget.Guideline app:layout_constraintGuide_percent="1.5"/></ConstraintLayout>')
  with self.assertRaisesRegex(ValueError,'Unsupported Barrier direction'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><androidx.constraintlayout.widget.Barrier app:barrierDirection="diagonal"/></ConstraintLayout>')
 def test_app_compat_drawables_toolbar_and_common_android_attrs(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'dot.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#123456"/><size android:width="10dp" android:height="10dp"/></shape>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"
    android:fitsSystemWindows="true" android:animateLayoutChanges="true" android:overScrollMode="never" android:theme="@style/ThemeOverlay.Material3.Light">
    <androidx.appcompat.widget.Toolbar app:title="Settings" app:subtitle="General" app:navigationIcon="@drawable/arrow_back"
     app:contentInsetStart="16dp" app:contentInsetEnd="8dp" app:titleTextAppearance="?attr/textAppearanceTitleLarge" app:titleTextColor="#112233"/>
    <TextView android:text="Hello" android:fontFamily="sans-serif-medium" android:textIsSelectable="true" android:autoLink="web|email"
     android:longClickable="true" android:accessibilityHeading="true"
     app:drawableStartCompat="@drawable/dot" app:drawableEndCompat="@null" app:drawableTint="#654321"/>
    <EditText android:imeOptions="actionSend|flagNoEnterAction" android:background="@null"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   for tok in ('Kind::Toolbar','text:"Settings"','hint:"General"','icon:MaterialIcon::ArrowBack','kind:112, anchor:"16.0"','kind:113, anchor:"8.0"','kind:125, anchor:"22.0"','kind:123, anchor:"17,34,51,255"','kind:173, anchor:"10.0,10.0,0.0,18,52,86,255"','kind:174, anchor:"0,0,0,0,0,0,0"','kind:176, anchor:"0,0,101,67,33,255"'):
    self.assertIn(tok,out)
   with self.assertRaisesRegex(ValueError,'Unsupported overScrollMode'):
    self.run_layout('<ScrollView xmlns:android="http://schemas.android.com/apk/res/android" android:overScrollMode="bounce"><View/></ScrollView>')
   with self.assertRaisesRegex(ValueError,'Unsupported autoLink'):
    self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:autoLink="ftp"/>')
   with self.assertRaisesRegex(ValueError,'Unsupported imeOptions'):
    self.run_layout('<EditText xmlns:android="http://schemas.android.com/apk/res/android" android:imeOptions="unknownFlag"/>')
 def test_wave1a_widget_tag_aliases_and_defaults(self):
  cases=[
   ('<androidx.core.widget.NestedScrollView><View/></androidx.core.widget.NestedScrollView>',('kind:Kind::Scroll',)),
   ('<androidx.appcompat.widget.LinearLayoutCompat/>',('kind:Kind::Linear',)),
   ('<androidx.appcompat.widget.AppCompatImageButton/>',('kind:Kind::Icon',)),
   ('<androidx.appcompat.widget.AppCompatSpinner/>',('kind:Kind::Spinner',)),
   ('<androidx.appcompat.widget.AppCompatSeekBar/>',('kind:Kind::SeekBar',)),
   ('<androidx.appcompat.widget.AppCompatRatingBar/>',('kind:Kind::RatingBar',)),
   ('<androidx.appcompat.widget.AppCompatToggleButton/>',('kind:Kind::ToggleButton',)),
   ('<androidx.appcompat.widget.AppCompatAutoCompleteTextView/>',('kind:Kind::AutoComplete',)),
   ('<androidx.appcompat.widget.AppCompatCheckedTextView/>',('kind:Kind::CheckedText',)),
   ('<androidx.cardview.widget.CardView xmlns:app="http://schemas.android.com/apk/res-auto" app:cardCornerRadius="12dp"><TextView/></androidx.cardview.widget.CardView>',('kind:Kind::MaterialCard','kind:66, anchor:"12.0"')),
   ('<com.google.android.material.chip.Chip xmlns:android="http://schemas.android.com/apk/res/android" android:text="Tag"/>',('kind:Kind::Button','radius:16.0','background:Some(Color(247,242,250,255))')),
   ('<com.google.android.material.chip.ChipGroup/>',('kind:Kind::Linear','horizontal:true')),
   ('<com.google.android.material.tabs.TabLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:tabMode="fixed"><com.google.android.material.tabs.TabItem xmlns:android="http://schemas.android.com/apk/res/android" android:text="A"/></com.google.android.material.tabs.TabLayout>',('kind:Kind::TabWidget','text:"A"')),
   ('<com.google.android.material.progressindicator.LinearProgressIndicator/>',('kind:Kind::ProgressBar','horizontal:true','indeterminate:false')),
   ('<com.google.android.material.progressindicator.CircularProgressIndicator/>',('kind:Kind::ProgressBar','horizontal:false','indeterminate:true')),
   ('<com.google.android.material.slider.Slider xmlns:android="http://schemas.android.com/apk/res/android" android:valueFrom="10" android:valueTo="50" android:value="25"/>',('kind:Kind::SeekBar','min:10.0','max:50.0','progress:25.0')),
   ('<com.google.android.material.checkbox.MaterialCheckBox/>',('kind:Kind::CheckBox',)),
   ('<com.google.android.material.radiobutton.MaterialRadioButton/>',('kind:Kind::RadioButton',)),
   ('<com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton xmlns:android="http://schemas.android.com/apk/res/android" android:text="Compose"/>',('kind:Kind::Button','radius:16.0','background:Some(Color(234,221,255,255))')),
   ('<com.google.android.material.bottomnavigation.BottomNavigationView/>',('kind:Kind::Linear','horizontal:true')),
   ('<com.google.android.material.navigation.NavigationBarView/>',('kind:Kind::Linear','horizontal:true')),
   ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:title="Header"><View/></com.google.android.material.appbar.CollapsingToolbarLayout>',('kind:Kind::Frame',)),
   ('<androidx.drawerlayout.widget.DrawerLayout><View/></androidx.drawerlayout.widget.DrawerLayout>',('kind:Kind::Frame',)),
   ('<androidx.swiperefreshlayout.widget.SwipeRefreshLayout><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>',('kind:Kind::Frame',)),
   ('<androidx.viewpager2.widget.ViewPager2/>',('kind:Kind::ViewFlipper',)),
   ('<androidx.viewpager.widget.ViewPager/>',('kind:Kind::ViewFlipper',)),
   ('<android.widget.Space/>',('kind:Kind::Space',)),
  ]
  for xml,tokens in cases:
   with self.subTest(xml=xml[:48]):
    out=self.run_layout(xml)
    for tok in tokens:self.assertIn(tok,out)
 def test_constraintlayout_chains_weights_and_extra_attrs(self):
  xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <TextView android:id="@+id/a" app:layout_constraintStart_toStartOf="parent" app:layout_constraintEnd_toStartOf="@id/b"
    app:layout_constraintHorizontal_chainStyle="spread_inside" app:layout_constraintVertical_chainStyle="packed"
    app:layout_constraintHorizontal_weight="2" app:layout_constraintVertical_weight="1.5"
    app:layout_constraintDimensionRatio="16:9" app:layout_constrainedWidth="true" app:layout_constrainedHeight="false"
    app:layout_constraintWidth_default="percent" app:layout_constraintHeight_default="wrap"
    app:layout_constraintWidth_percent="0.8" app:layout_constraintHeight_percent="0.5"
    app:layout_goneMarginStart="12dp" app:layout_goneMarginBottom="4dp"/>
   <TextView android:id="@+id/b" app:layout_constraintStart_toEndOf="@id/a" app:layout_constraintEnd_toEndOf="parent"
    app:layout_constraintBaseline_toBaselineOf="@id/a" app:layout_constraintHorizontal_weight="1"/>
  </androidx.constraintlayout.widget.ConstraintLayout>'''
  out=self.run_layout(xml)
  for tok in (
   'RelativeRule { kind:214, anchor:"spread_inside" }',
   'RelativeRule { kind:215, anchor:"packed" }',
   'RelativeRule { kind:216, anchor:"2.0" }',
   'RelativeRule { kind:217, anchor:"1.5" }',
   'RelativeRule { kind:216, anchor:"1.0" }',
  ):
   self.assertIn(tok,out)
  with self.assertRaisesRegex(ValueError,'Unsupported ConstraintLayout chainStyle'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintHorizontal_chainStyle="evenly"/></ConstraintLayout>')
  with self.assertRaisesRegex(ValueError,'Invalid ConstraintLayout weight'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintVertical_weight="-1"/></ConstraintLayout>')
 def test_shape_per_corner_radii_and_gradient(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'asym.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android" android:shape="rectangle"><solid android:color="#112233"/><corners android:topLeftRadius="16dp" android:topRightRadius="16dp" android:bottomRightRadius="4dp" android:bottomLeftRadius="16dp"/></shape>')
   (res/'drawable'/'unif.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#112233"/><corners android:topLeftRadius="8dp" android:topRightRadius="8dp" android:bottomRightRadius="8dp" android:bottomLeftRadius="8dp"/></shape>')
   (res/'drawable'/'grad2.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:angle="90" android:startColor="#80000000" android:endColor="#00000000" android:dither="true"/></shape>')
   (res/'drawable'/'grad3.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="linear" android:angle="0" android:startColor="#ff0000" android:centerColor="#00ff00" android:endColor="#0000ff"/></shape>')
   (res/'drawable'/'grad3_pos.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="linear" android:angle="0" android:startColor="#ff0000" android:centerColor="#00ff00" android:endColor="#0000ff" android:centerX="0.25"/></shape>')
   (res/'drawable'/'grad_radial.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="radial" android:gradientRadius="60dp" android:startColor="#fff" android:endColor="#000"/></shape>')
   (res/'drawable'/'grad_bad.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="conic" android:startColor="#fff" android:endColor="#000"/></shape>')
   asym=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/asym"/>',res)
   self.assertIn('RelativeRule { kind:206, anchor:"16.0,16.0,4.0,16.0" }',asym)
   self.assertIn('radius:0.0',asym)
   unif=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/unif"/>',res)
   self.assertIn('radius:8.0',unif);self.assertNotIn('kind:206',unif)
   g2=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/grad2"/>',res)
   self.assertIn('RelativeRule { kind:207, anchor:"90.0,0,0,0,128,0,0,0,0" }',g2)
   g3=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/grad3"/>',res)
   self.assertIn('RelativeRule { kind:207, anchor:"0.0,255,0,0,255,0,255,0,255,0,0,255,255" }',g3)
   g3p=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/grad3_pos"/>',res)
   self.assertIn('RelativeRule { kind:207, anchor:"0.0,255,0,0,255,0,255,0,255,0,0,255,255,0.25" }',g3p)
   gr=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/grad_radial"/>',res)
   self.assertIn('RelativeRule { kind:207, anchor:"radial|0.5,0.5,60.0|255,255,255,255||0,0,0,255" }',gr)
   with self.assertRaisesRegex(ValueError,'Unsupported gradient type'):
    self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/grad_bad"/>',res)
 def test_layer_list_and_inset_drawables(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'base_pill.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#20000000"/><corners android:radius="12dp"/></shape>')
   (res/'drawable'/'layered.xml').write_text('''<layer-list xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:drawable="@drawable/base_pill"/>
    <item android:start="2dp" android:top="1dp" android:end="2dp" android:bottom="3dp">
     <shape android:shape="rectangle">
      <solid android:color="#ffffff"/>
      <stroke android:width="1dp" android:color="#112233"/>
      <corners android:topLeftRadius="10dp" android:topRightRadius="10dp" android:bottomRightRadius="2dp" android:bottomLeftRadius="10dp"/>
      <padding android:left="8dp" android:top="6dp" android:right="8dp" android:bottom="6dp"/>
     </shape>
    </item>
   </layer-list>''')
   (res/'drawable'/'inset_box.xml').write_text('''<inset xmlns:android="http://schemas.android.com/apk/res/android" android:insetLeft="4dp" android:insetTop="2dp" android:insetRight="4dp" android:insetBottom="2dp">
    <shape android:shape="oval"><solid android:color="#336699"/></shape>
   </inset>''')
   out=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/layered"/>',res)
   self.assertIn('RelativeRule { kind:208, anchor:"0.0,0.0,0.0,0.0|12.0|0,0,0,32" }',out)
   self.assertIn('RelativeRule { kind:208, anchor:"2.0,1.0,2.0,3.0|10.0,10.0,2.0,10.0|255,255,255,255|1.0,17,34,51,255" }',out)
   self.assertIn('background_padding:Some([8.0,6.0,8.0,6.0])',out)
   self.assertIn('background:None',out)
   ins=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/inset_box"/>',res)
   self.assertIn('RelativeRule { kind:208, anchor:"4.0,2.0,4.0,2.0|-1.0|51,102,153,255" }',ins)
 def test_ripple_and_drawable_selector_backgrounds(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'rip_unbounded.xml').write_text('<ripple xmlns:android="http://schemas.android.com/apk/res/android" android:color="#33ff0000"/>')
   (res/'drawable'/'rip_bounded.xml').write_text('''<ripple xmlns:android="http://schemas.android.com/apk/res/android" android:color="?attr/colorControlHighlight">
    <item android:id="@android:id/mask" android:drawable="@android:color/white"/>
    <item><shape><solid android:color="#f3edf7"/><corners android:radius="12dp"/></shape></item>
   </ripple>''')
   (res/'drawable'/'rip_sel.xml').write_text('''<ripple xmlns:android="http://schemas.android.com/apk/res/android" android:color="#40112233">
    <item android:id="@android:id/mask" android:drawable="@android:color/black"/>
    <item><selector><item android:state_selected="true" android:drawable="#55112233"/></selector></item>
   </ripple>''')
   (res/'drawable'/'sel_complex.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android" android:exitFadeDuration="200">
    <item android:state_selected="true"><inset android:inset="2dp"><shape><solid android:color="#abcdef"/><stroke android:width="2dp" android:color="#123456"/></shape></inset></item>
    <item><shape><solid android:color="#ffffff"/><corners android:radius="6dp"/><stroke android:width="1dp" android:color="#cccccc"/></shape></item>
   </selector>''')
   unb=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/rip_unbounded"/>',res)
   self.assertIn('RelativeRule { kind:209, anchor:"255,0,0,51|unbounded" }',unb)
   fg_unb=self.run_layout('<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" android:foreground="@drawable/rip_unbounded"/>',res)
   self.assertIn('RelativeRule { kind:209, anchor:"255,0,0,51|unbounded" }',fg_unb)
   bnd=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/rip_bounded"/>',res)
   self.assertIn('RelativeRule { kind:209, anchor:"103,80,164,31" }',bnd)
   self.assertIn('background:Some(Color(243,237,247,255))',bnd)
   self.assertIn('radius:12.0',bnd)
   rsel=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/rip_sel"/>',res)
   self.assertIn('RelativeRule { kind:209, anchor:"17,34,51,64" }',rsel)
   self.assertIn('RelativeRule { kind:96, anchor:"8,0,17,34,51,85" }',rsel)
   self.assertIn('RelativeRule { kind:96, anchor:"0,0,0,0,0,0" }',rsel)
   scplx=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/sel_complex"/>',res)
   self.assertIn('background:Some(Color(255,255,255,255))',scplx)
   self.assertIn('radius:6.0',scplx)
   self.assertIn('background_stroke:Some(Color(204,204,204,255))',scplx)
 def test_vector_drawables_with_theme_and_color_refs_across_views(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();(res/'values').mkdir()
   (res/'values'/'colors.xml').write_text('<resources><color name="brand_fill">#118844</color><color name="brand_tint">#224466</color></resources>')
   (res/'drawable'/'ic_custom.xml').write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24" android:autoMirrored="true" android:tint="?attr/colorControlNormal">
    <path android:fillColor="@color/brand_fill" android:pathData="M2,2h20v20h-20z"/>
   </vector>''')
   (res/'drawable'/'ic_nav.xml').write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24" android:tint="@color/brand_tint">
    <path android:fillColor="?attr/colorOnPrimary" android:pathData="M4,4h16v16h-16z"/>
   </vector>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <Toolbar android:title="Home" android:navigationIcon="@drawable/ic_nav"/>
    <ImageView android:src="@drawable/ic_custom"/>
    <ImageButton app:srcCompat="@drawable/ic_custom"/>
    <com.google.android.material.floatingactionbutton.FloatingActionButton app:srcCompat="@drawable/ic_nav"/>
    <TextView android:text="Star" android:drawableStart="@drawable/ic_custom"/>
    <View android:background="@drawable/ic_custom"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   for tok in (
    'pub fn install_drawables',
    'state.images.entry("ic_custom".into())',
    'state.images.entry("ic_nav".into())',
    'AndroidDrawableTint::ControlNormal',
    'AndroidDrawableTint::Fixed(Color(34,68,102,255))',
    'image_tint:Some(Color(73,69,79,255))',
    'image_tint:Some(Color(34,68,102,255))',
    'kind:173, anchor:"24.0,24.0,0.0,73,69,79,255"',
    'background_min_size:[24.0,24.0]',
   ):
    self.assertIn(tok,out)
 def test_multi_directory_resources_precedence_and_cli(self):
  import subprocess,sys
  with tempfile.TemporaryDirectory() as d:
   primary=Path(d)/'primary_res';lib=Path(d)/'lib_res'
   for r in (primary,lib):
    (r/'values').mkdir(parents=True);(r/'drawable').mkdir();(r/'layout').mkdir()
   (primary/'values'/'strings.xml').write_text('<resources><string name="app_title">Primary</string><dimen name="gap">16dp</dimen></resources>')
   (lib/'values'/'strings.xml').write_text('<resources><string name="app_title">Library</string><string name="lib_only">FromLib</string><color name="lib_bg">#223344</color></resources>')
   (lib/'drawable'/'lib_bg.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="@color/lib_bg"/><corners android:radius="4dp"/></shape>')
   (lib/'layout'/'shared_part.xml').write_text('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="@string/lib_only"/>')
   layout=primary/'layout'/'main.xml'
   layout.write_text('''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:padding="@dimen/gap" android:background="@drawable/lib_bg">
    <TextView android:text="@string/app_title"/>
    <include layout="@layout/shared_part"/>
   </LinearLayout>''')
   out=mod.compile_layout(layout,[primary,lib])
   for tok in ('text:"Primary"','text:"FromLib"','padding_edges:[16.0,16.0,16.0,16.0]','background:Some(Color(34,51,68,255))','radius:4.0'):
    self.assertIn(tok,out)
   self.assertNotIn('text:"Library"',out)
   out_file=Path(d)/'out.rs'
   proc=subprocess.run([sys.executable,str(Path(__file__).with_name('import-android-layout.py')),str(layout),str(out_file),'--resources',str(primary),'--resources',str(lib)],capture_output=True,text=True,check=True)
   self.assertEqual(out_file.read_text(),out)
 def test_aosp_gravity_combinations_and_conflicts(self):
  combos=[
   ('bottom|center','content_gravity:[1,2]'),
   ('center|bottom','content_gravity:[1,2]'),
   ('top|center','content_gravity:[1,0]'),
   ('start|center','content_gravity:[0,1]'),
   ('center|end','content_gravity:[2,1]'),
   ('left|center','content_gravity:[3,1]'),
   ('right|center','content_gravity:[4,1]'),
   ('center|center_vertical','content_gravity:[1,1]'),
   ('left|start|center_vertical','content_gravity:[0,1]'),
  ]
  for grav,expected in combos:
   with self.subTest(gravity=grav):
    self.assertIn(expected,self.run_layout(f'<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:gravity="{grav}"/>'))
  for bad,msg in [('top|bottom','Conflicting vertical'),('left|right','Conflicting horizontal'),('start|end','Conflicting horizontal')]:
   with self.subTest(bad_gravity=bad):
    with self.assertRaisesRegex(ValueError,msg):
     self.run_layout(f'<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:gravity="{bad}"/>')
 def test_wave3a_text_all_caps_rule_218(self):
  btn_false=self.run_layout('<Button xmlns:android="http://schemas.android.com/apk/res/android" android:text="Save" android:textAllCaps="false"/>')
  self.assertIn('RelativeRule { kind:218, anchor:"false" }',btn_false)
  self.assertIn('text_all_caps:false',btn_false)
  btn_true=self.run_layout('<Button xmlns:android="http://schemas.android.com/apk/res/android" android:text="Save" android:textAllCaps="true"/>')
  self.assertIn('RelativeRule { kind:218, anchor:"true" }',btn_true)
  self.assertIn('text_all_caps:true',btn_true)
  btn_default=self.run_layout('<Button xmlns:android="http://schemas.android.com/apk/res/android" android:text="Save"/>')
  self.assertNotIn('kind:218',btn_default)
  btn_m3_ta=self.run_layout('<Button xmlns:android="http://schemas.android.com/apk/res/android" android:text="Save" android:textAppearance="?attr/textAppearanceLabelLarge"/>')
  self.assertIn('RelativeRule { kind:218, anchor:"false" }',btn_m3_ta)
  tv_overline=self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="Tag" android:textAppearance="?attr/textAppearanceOverline"/>')
  self.assertIn('RelativeRule { kind:218, anchor:"true" }',tv_overline)
  edit_m3=self.run_layout('<EditText xmlns:android="http://schemas.android.com/apk/res/android" android:textAppearance="?attr/textAppearanceBodyMedium"/>')
  self.assertIn('RelativeRule { kind:218, anchor:"false" }',edit_m3)
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('<resources><bool name="use_caps">false</bool><style name="NoCapsBtn"><item name="textAllCaps">@bool/use_caps</item></style></resources>')
   styled=self.run_layout('<Button xmlns:android="http://schemas.android.com/apk/res/android" style="@style/NoCapsBtn" android:text="Mixed"/>',res)
   self.assertIn('RelativeRule { kind:218, anchor:"false" }',styled)
 def test_wave3a_constraintlayout_rules_219_to_225(self):
  xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <TextView android:id="@+id/a" app:layout_constraintStart_toStartOf="parent" app:layout_constraintDimensionRatio="16:9"
    app:layout_constraintWidth_percent="0.75" app:layout_constraintHeight_percent="0.4"
    app:layout_goneMarginStart="12dp" app:layout_goneMarginBottom="4dp"/>
   <TextView android:id="@+id/b" app:layout_constraintDimensionRatio="H,4:3" app:layout_constraintBaseline_toBaselineOf="@id/a"
    app:layout_goneMarginLeft="1dp" app:layout_goneMarginTop="2dp" app:layout_goneMarginRight="3dp"
    app:layout_goneMarginBottom="4dp" app:layout_goneMarginStart="5dp" app:layout_goneMarginEnd="6dp"/>
   <View android:id="@+id/c" app:layout_constraintDimensionRatio="W,1.5"/>
   <View android:id="@+id/d" app:layout_constraintDimensionRatio="2.0"/>
   <androidx.constraintlayout.widget.Barrier android:id="@+id/bar" app:barrierDirection="end" app:constraint_referenced_ids="a, b"/>
   <androidx.constraintlayout.widget.Barrier android:id="@+id/bar_default" app:constraint_referenced_ids="@id/a,@+id/c"/>
   <androidx.constraintlayout.widget.Group android:id="@+id/grp" android:visibility="visible" app:constraint_referenced_ids="a, b, c"/>
  </androidx.constraintlayout.widget.ConstraintLayout>'''
  out=self.run_layout(xml)
  for tok in (
   'RelativeRule { kind:219, anchor:"16:9" }',
   'RelativeRule { kind:219, anchor:"H,4:3" }',
   'RelativeRule { kind:219, anchor:"W,1.5" }',
   'RelativeRule { kind:219, anchor:"2.0" }',
   'RelativeRule { kind:220, anchor:"0.75" }',
   'RelativeRule { kind:221, anchor:"0.4" }',
   'RelativeRule { kind:222, anchor:"a" }',
   'RelativeRule { kind:223, anchor:"-1,-1,-1,4.0,12.0,-1" }',
   'RelativeRule { kind:223, anchor:"1.0,2.0,3.0,4.0,5.0,6.0" }',
   'RelativeRule { kind:224, anchor:"end:a,b" }',
   'RelativeRule { kind:224, anchor:"left:a,c" }',
   'RelativeRule { kind:225, anchor:"a,b,c" }',
  ):
   self.assertIn(tok,out)
  for bad_ratio in ('0:9','X,16:9','abc','-1.5','16:0'):
   with self.subTest(bad_ratio=bad_ratio):
    with self.assertRaisesRegex(ValueError,'Invalid layout_constraintDimensionRatio'):
     self.run_layout(f'<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintDimensionRatio="{bad_ratio}"/></ConstraintLayout>')
  with self.assertRaisesRegex(ValueError,'Invalid layout_constraintWidth_percent'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintWidth_percent="1.5"/></ConstraintLayout>')
  with self.assertRaisesRegex(ValueError,'Invalid ConstraintLayout anchor'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintBaseline_toBaselineOf="bad"/></ConstraintLayout>')
 def test_wave3a_resource_qualifiers_and_cli(self):
  import subprocess,sys
  with tempfile.TemporaryDirectory() as d:
   res=Path(d)
   for sub in ('values','values-v21','values-night','values-notnight','values-land','values-port','values-sw600dp','values-night-sw600dp','values-fr','color','color-night','drawable','drawable-night','drawable-anydpi-v26','layout','layout-land'):
    (res/sub).mkdir()
   (res/'values'/'vals.xml').write_text('<resources><string name="lbl">Base</string><string name="only_base">BaseOnly</string><color name="bg">#111111</color><dimen name="pad">8dp</dimen><style name="BoxStyle"><item name="android:padding">4dp</item></style></resources>')
   (res/'values-v21'/'vals.xml').write_text('<resources><dimen name="pad">10dp</dimen></resources>')
   (res/'values-notnight'/'vals.xml').write_text('<resources><color name="bg">#eeeeee</color></resources>')
   (res/'values-night'/'vals.xml').write_text('<resources><string name="lbl">Night</string><color name="bg">#222222</color><style name="BoxStyle"><item name="android:padding">20dp</item></style></resources>')
   (res/'values-land'/'vals.xml').write_text('<resources><string name="lbl">Land</string><dimen name="pad">16dp</dimen></resources>')
   (res/'values-sw600dp'/'vals.xml').write_text('<resources><string name="lbl">Tablet</string><dimen name="pad">24dp</dimen></resources>')
   (res/'values-night-sw600dp'/'vals.xml').write_text('<resources><string name="lbl">NightTablet</string></resources>')
   (res/'values-fr'/'vals.xml').write_text('<resources><string name="lbl">Bonjour</string></resources>')
   (res/'color'/'tint.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:color="#010203"/></selector>')
   (res/'color-night'/'tint.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:color="#040506"/></selector>')
   (res/'drawable'/'panel.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#aa0000"/></shape>')
   (res/'drawable-night'/'panel.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#0000bb"/></shape>')
   (res/'drawable'/'badge.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#101010"/></shape>')
   (res/'drawable-anydpi-v26'/'badge.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#202020"/></shape>')
   (res/'layout'/'part.xml').write_text('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="PortPart"/>')
   (res/'layout-land'/'part.xml').write_text('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="LandPart"/>')
   main=res/'layout'/'main.xml'
   main.write_text('''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="vertical" android:padding="@dimen/pad" android:background="@drawable/panel">
    <TextView style="@style/BoxStyle" android:text="@string/lbl" android:textColor="@color/tint" android:background="@color/bg"/>
    <View android:background="@drawable/badge"/>
    <include layout="@layout/part"/>
   </LinearLayout>''')
   (res/'layout-land'/'main.xml').write_text('''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="horizontal" android:padding="@dimen/pad" android:background="@drawable/panel">
    <TextView android:text="@string/lbl"/>
    <include layout="@layout/part"/>
   </LinearLayout>''')
   # Default: port, not night, sw=0 -> ignores night, land, sw600dp, fr; applies v21, notnight, anydpi-v26
   out_def=mod.compile_layout(main,res)
   for tok in ('horizontal:false','text:"Base"','padding_edges:[10.0,10.0,10.0,10.0]','Color(238,238,238,255)','Color(1,2,3,255)','Color(170,0,0,255)','Color(32,32,32,255)','text:"PortPart"','padding_edges:[4.0,4.0,4.0,4.0]'):
    self.assertIn(tok,out_def)
   # Night mode
   out_night=mod.compile_layout(main,res,night=True)
   for tok in ('text:"Night"','Color(34,34,34,255)','Color(4,5,6,255)','Color(0,0,187,255)','padding_edges:[20.0,20.0,20.0,20.0]'):
    self.assertIn(tok,out_night)
   # Landscape mode -> selects layout-land/main.xml and layout-land/part.xml and values-land
   out_land=mod.compile_layout(main,res,orientation='land')
   for tok in ('horizontal:true','text:"Land"','padding_edges:[16.0,16.0,16.0,16.0]','text:"LandPart"'):
    self.assertIn(tok,out_land)
   # Smallest width 600dp
   out_sw=mod.compile_layout(main,res,sw=600)
   self.assertIn('text:"Tablet"',out_sw);self.assertIn('padding_edges:[24.0,24.0,24.0,24.0]',out_sw)
   # Combined night + sw600dp -> values-night-sw600dp beats both values-night and values-sw600dp
   out_both=mod.compile_layout(main,res,night=True,sw=600)
   self.assertIn('text:"NightTablet"',out_both);self.assertIn('padding_edges:[24.0,24.0,24.0,24.0]',out_both)
   # CLI flags
   out_file=res/'out.rs'
   subprocess.run([sys.executable,str(Path(__file__).with_name('import-android-layout.py')),str(main),str(out_file),'--resources',str(res),'--night','--orientation','land','--sw','600'],capture_output=True,text=True,check=True)
   cli_out=out_file.read_text()
   self.assertIn('horizontal:true',cli_out);self.assertIn('text:"NightTablet"',cli_out);self.assertIn('Color(0,0,187,255)',cli_out)
   with self.assertRaisesRegex(ValueError,'Invalid orientation'):mod.compile_layout(main,res,orientation='diagonal')
   with self.assertRaisesRegex(ValueError,'Invalid smallest width'):mod.compile_layout(main,res,sw=-1)
 def test_wave3a_full_input_type_flags(self):
  combos=[
   ('none',0),('text',0),('textCapCharacters',0),('textCapWords',0),('textCapSentences',0),
   ('textAutoCorrect',0),('textAutoComplete',0),('textMultiLine',0),('textImeMultiLine',0),
   ('textNoSuggestions',0),('textUri',0),('textEmailAddress',1),('textEmailSubject',0),
   ('textShortMessage',0),('textLongMessage',0),('textPersonName',0),('textPostalAddress',0),
   ('textPassword',4),('textVisiblePassword',0),('textWebEditText',0),('textFilter',0),
   ('textPhonetic',0),('textWebEmailAddress',1),('textWebPassword',4),
   ('number',3),('numberSigned',3),('numberDecimal',5),('numberPassword',4),
   ('phone',2),('datetime',0),('date',0),('time',0),
   ('text|textNoSuggestions',0),('textCapSentences|textMultiLine',0),
   ('textImeMultiLine|textNoSuggestions|textMultiLine',0),
   ('textWebEmailAddress|textNoSuggestions',1),('textPassword|textNoSuggestions',4),
   ('number|numberDecimal',5),('number|numberSigned',3),('number|numberPassword',4),
  ]
  for spec,code in combos:
   with self.subTest(inputType=spec):
    out=self.run_layout(f'<EditText xmlns:android="http://schemas.android.com/apk/res/android" android:inputType="{spec}"/>')
    self.assertIn(f'input_type:{code}',out)
  for bad in ('','bogus','text|bogus'):
   with self.subTest(bad_inputType=bad):
    with self.assertRaisesRegex(ValueError,'Unsupported inputType'):
     self.run_layout(f'<EditText xmlns:android="http://schemas.android.com/apk/res/android" android:inputType="{bad}"/>')
 def test_wave3a_standard_list_choice_indicators(self):
  for ref in ('?android:attr/listChoiceIndicatorSingle','?android:attr/listChoiceIndicatorMultiple','?attr/listChoiceIndicatorSingle'):
   with self.subTest(ref=ref):
    ctv=self.run_layout(f'<CheckedTextView xmlns:android="http://schemas.android.com/apk/res/android" android:checkMark="{ref}"/>')
    self.assertIn('kind:Kind::CheckedText',ctv)
    cb=self.run_layout(f'<CheckBox xmlns:android="http://schemas.android.com/apk/res/android" android:button="{ref}"/>')
    self.assertIn('kind:Kind::CheckBox',cb)
    rb=self.run_layout(f'<RadioButton xmlns:android="http://schemas.android.com/apk/res/android" android:button="{ref}"/>')
    self.assertIn('kind:Kind::RadioButton',rb)
    comp=self.run_layout(f'<CheckedTextView xmlns:android="http://schemas.android.com/apk/res/android" android:drawableStart="{ref}"/>')
    self.assertIn('kind:173',comp)
  with self.assertRaisesRegex(ValueError,'checkMark needs local drawable'):
   self.run_layout('<SeekBar xmlns:android="http://schemas.android.com/apk/res/android" android:thumb="?android:attr/listChoiceIndicatorSingle"/>')
 def test_wave3a_included_root_foreign_layout_params(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'layout').mkdir()
   sb=res/'layout'/'status_bar.xml'
   sb.write_text('<View xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/sb" android:layout_width="match_parent" android:layout_height="0dp" android:layout_alignParentTop="true" android:layout_toEndOf="@id/other"/>')
   (res/'layout'/'plain.xml').write_text('<View xmlns:android="http://schemas.android.com/apk/res/android" android:id="@+id/p"/>')
   # Included into LinearLayout -> foreign RelativeLayout rules stripped
   in_lin=self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><include layout="@layout/status_bar"/></LinearLayout>',res)
   self.assertIn('id:"sb"',in_lin);self.assertNotIn('kind:14',in_lin);self.assertNotIn('kind:9',in_lin)
   # Included into RelativeLayout -> rules preserved
   in_rel=self.run_layout('<RelativeLayout xmlns:android="http://schemas.android.com/apk/res/android"><View android:id="@+id/other"/><include layout="@layout/status_bar"/></RelativeLayout>',res)
   self.assertIn('RelativeRule { kind:14, anchor:"" }',in_rel);self.assertIn('RelativeRule { kind:9, anchor:"other" }',in_rel)
   # Standalone direct layout -> still strictly rejected
   with self.assertRaisesRegex(ValueError,'RelativeLayout rule outside RelativeLayout'):
    mod.compile_layout(sb,res)
   # Direct child in LinearLayout -> still strictly rejected
   with self.assertRaisesRegex(ValueError,'RelativeLayout rule outside RelativeLayout'):
    self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><View android:layout_alignParentTop="true"/></LinearLayout>',res)
   # Explicit foreign override on <include> tag itself in LinearLayout -> still rejected
   with self.assertRaisesRegex(ValueError,'RelativeLayout rule outside RelativeLayout'):
    self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><include layout="@layout/plain" android:layout_width="match_parent" android:layout_height="wrap_content" android:layout_below="@id/other"/></LinearLayout>',res)
 def test_wave3a_merge_parent_option_and_cli(self):
  import subprocess,sys
  merge_xml='<merge xmlns:android="http://schemas.android.com/apk/res/android" xmlns:tools="http://schemas.android.com/tools" android:orientation="horizontal" android:layout_width="wrap_content" android:layout_height="wrap_content" tools:parentTag="LinearLayout"><TextView android:id="@+id/a" android:text="One"/><TextView android:id="@+id/b" android:text="Two"/></merge>'
  with tempfile.TemporaryDirectory() as d:
   p=Path(d)/'part.xml';p.write_text(merge_xml)
   with self.assertRaisesRegex(ValueError,'merge must be included'):
    mod.compile_layout(p)
   framed=mod.compile_layout(p,merge_parent='FrameLayout')
   for tok in ('kind:Kind::Frame','id:"a"','id:"b"','text:"One"','text:"Two"'):self.assertIn(tok,framed)
   linear=mod.compile_layout(p,merge_parent='LinearLayout')
   for tok in ('kind:Kind::Linear','horizontal:true','width:Size::Wrap','id:"a"','id:"b"'):self.assertIn(tok,linear)
   with self.assertRaisesRegex(ValueError,'Unsupported merge parent'):
    mod.compile_layout(p,merge_parent='TextView')
   out_file=Path(d)/'out.rs'
   subprocess.run([sys.executable,str(Path(__file__).with_name('import-android-layout.py')),str(p),str(out_file),'--merge-parent','FrameLayout'],capture_output=True,text=True,check=True)
   self.assertEqual(out_file.read_text(),framed)
 def test_wave4a_coordinator_scroll_behavior_and_anchor_rules_226_227(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'strings.xml').write_text('<resources><string name="my_scroll">com.google.android.material.appbar.AppBarLayout$ScrollingViewBehavior</string></resources>')
   xml='''<androidx.coordinatorlayout.widget.CoordinatorLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/appbar"/>
    <androidx.core.widget.NestedScrollView android:id="@+id/body" app:layout_behavior="@string/appbar_scrolling_view_behavior"><View/></androidx.core.widget.NestedScrollView>
    <View android:id="@+id/explicit_scroll" app:layout_behavior="com.google.android.material.appbar.AppBarLayout$ScrollingViewBehavior"/>
    <View android:id="@+id/res_scroll" app:layout_behavior="@string/my_scroll"/>
    <View android:id="@+id/other_beh" app:layout_behavior="com.example.CustomBehavior"/>
    <com.google.android.material.floatingactionbutton.FloatingActionButton android:id="@+id/fab1" app:layout_anchor="@id/appbar" app:layout_anchorGravity="bottom|end"/>
    <View android:id="@+id/badge" app:layout_anchor="@+id/appbar" app:layout_anchorGravity=" top | center_horizontal | top "/>
    <View android:id="@+id/default_grav" app:layout_anchor="@id/appbar"/>
    <View android:id="@+id/null_anc" app:layout_anchor="@null"/>
   </androidx.coordinatorlayout.widget.CoordinatorLayout>'''
   out=self.run_layout(xml,res)
   self.assertEqual(out.count('RelativeRule { kind:226, anchor:"" }'),3)
   self.assertIn('RelativeRule { kind:227, anchor:"appbar:bottom|end" }',out)
   self.assertIn('RelativeRule { kind:227, anchor:"appbar:top|center_horizontal" }',out)
   self.assertIn('RelativeRule { kind:227, anchor:"appbar:center" }',out)
   # Short tag alias and merge_parent=CoordinatorLayout
   short=self.run_layout('<CoordinatorLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><View android:id="@+id/bar"/><View app:layout_behavior="@string/appbar_scrolling_view_behavior" app:layout_anchor="@id/bar" app:layout_anchorGravity="start|bottom"/></CoordinatorLayout>')
   self.assertIn('RelativeRule { kind:226, anchor:"" }',short)
   self.assertIn('RelativeRule { kind:227, anchor:"bar:start|bottom" }',short)
   mp=Path(d)/'coord_merge.xml'
   mp.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><View android:id="@+id/bar"/><View app:layout_behavior="@string/appbar_scrolling_view_behavior" app:layout_anchor="@id/bar" app:layout_anchorGravity="end|top"/></merge>')
   merged=mod.compile_layout(mp,merge_parent='CoordinatorLayout')
   self.assertIn('RelativeRule { kind:226, anchor:"" }',merged)
   self.assertIn('RelativeRule { kind:227, anchor:"bar:end|top" }',merged)
   # Non-CoordinatorLayout parent must not emit 226 or 227
   non_coord=self.run_layout('<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/appbar_scrolling_view_behavior" app:layout_anchor="@id/bar" app:layout_anchorGravity="bottom|end"/></FrameLayout>')
   self.assertNotIn('kind:226',non_coord);self.assertNotIn('kind:227',non_coord)
   with self.assertRaisesRegex(ValueError,'Invalid CoordinatorLayout anchor'):
    self.run_layout('<CoordinatorLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_anchor="bad_id"/></CoordinatorLayout>')
   with self.assertRaisesRegex(ValueError,'Unsupported layout_anchorGravity'):
    self.run_layout('<CoordinatorLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_anchor="@id/bar" app:layout_anchorGravity="bottom|diagonal"/></CoordinatorLayout>')
 def test_wave4a_text_input_box_rule_228(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('''<resources>
    <style name="CustomOutlinedInput" parent="Widget.MaterialComponents.TextInputLayout.OutlinedBox">
     <item name="boxStrokeColor">#445566</item>
     <item name="helperText">From style</item>
    </style>
   </resources>''')
   # Default filled box inheriting hint from child TextInputEditText
   filled=self.run_layout('''<com.google.android.material.textfield.TextInputLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <com.google.android.material.textfield.TextInputEditText android:hint="Email"/>
   </com.google.android.material.textfield.TextInputLayout>''')
   self.assertIn('RelativeRule { kind:228, anchor:"filled|4||Email||0|none" }',filled)
   # Outlined box with all fields + pipe escaping
   outlined=self.run_layout('''<com.google.android.material.textfield.TextInputLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"
    style="?attr/textInputOutlinedStyle" android:hint="First|Last" app:boxCornerRadiusTopStart="12dp" app:boxStrokeColor="#112233"
    app:helperText="Use a|b" app:errorEnabled="true" app:endIconMode="clear_text">
    <com.google.android.material.textfield.TextInputEditText/>
   </com.google.android.material.textfield.TextInputLayout>''')
   self.assertIn('RelativeRule { kind:228, anchor:"outline|12.0|17,34,51,255|First/Last|Use a/b|1|clear_text" }',outlined)
   # ExposedDropdownMenu style -> outline + dropdown_menu
   dd=self.run_layout('<com.google.android.material.textfield.TextInputLayout xmlns:android="http://schemas.android.com/apk/res/android" style="@style/Widget.Material3.TextInputLayout.OutlinedBox.ExposedDropdownMenu" android:hint="Country"><AutoCompleteTextView/></com.google.android.material.textfield.TextInputLayout>')
   self.assertIn('RelativeRule { kind:228, anchor:"outline|4||Country||0|dropdown_menu" }',dd)
   # boxBackgroundMode=none + legacy passwordToggleEnabled
   pw=self.run_layout('<com.google.android.material.textfield.TextInputLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:boxBackgroundMode="none" app:passwordToggleEnabled="true" android:hint="Password"><EditText/></com.google.android.material.textfield.TextInputLayout>')
   self.assertIn('RelativeRule { kind:228, anchor:"none|4||Password||0|password_toggle" }',pw)
   # Explicit endIconMode=password_toggle and custom style inheritance
   styled=self.run_layout('<com.google.android.material.textfield.TextInputLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" style="@style/CustomOutlinedInput" app:endIconMode="password_toggle"><EditText/></com.google.android.material.textfield.TextInputLayout>',res)
   self.assertIn('RelativeRule { kind:228, anchor:"outline|4|68,85,102,255||From style|0|password_toggle" }',styled)
   with self.assertRaisesRegex(ValueError,'Unsupported TextInputLayout boxBackgroundMode'):
    self.run_layout('<com.google.android.material.textfield.TextInputLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:boxBackgroundMode="dashed"/>')
   with self.assertRaisesRegex(ValueError,'Unsupported endIconMode'):
    self.run_layout('<com.google.android.material.textfield.TextInputLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:endIconMode="magic"/>')
   with self.assertRaisesRegex(ValueError,'Invalid errorEnabled'):
    self.run_layout('<com.google.android.material.textfield.TextInputLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:errorEnabled="yes"/>')
 def test_wave4a_button_variant_rule_229_and_material_button_attrs(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'drawable').mkdir()
   (res/'values'/'styles.xml').write_text('<resources><style name="AppOutlined" parent="Widget.MaterialComponents.Button.OutlinedButton"/></resources>')
   (res/'drawable'/'ic_btn.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="18dp" android:height="18dp" android:viewportWidth="24" android:viewportHeight="24"><path android:fillColor="#ffffff" android:pathData="M2,2h20v20h-20z"/></vector>')
   cases=[
    ('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" android:text="Save"/>','filled'),
    ('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" style="?attr/materialButtonOutlinedStyle"/>','outlined'),
    ('<Button xmlns:android="http://schemas.android.com/apk/res/android" style="@style/AppOutlined"/>','outlined'),
    ('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" style="@style/Widget.Material3.Button.TonalButton"/>','tonal'),
    ('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" style="@style/Widget.Material3.Button.TextButton"/>','text'),
    ('<androidx.appcompat.widget.AppCompatButton xmlns:android="http://schemas.android.com/apk/res/android" style="?attr/borderlessButtonStyle"/>','text'),
    ('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" style="@style/Widget.Material3.Button.ElevatedButton"/>','elevated'),
    ('<Button xmlns:android="http://schemas.android.com/apk/res/android" style="?attr/materialButtonElevatedStyle"/>','elevated'),
   ]
   for xml,variant in cases:
    with self.subTest(variant=variant,xml=xml[:60]):
     self.assertIn(f'RelativeRule {{ kind:229, anchor:"{variant}" }}',self.run_layout(xml,res))
   # Plain Button / AppCompatButton without variant style emits no rule 229
   self.assertNotIn('kind:229',self.run_layout('<Button xmlns:android="http://schemas.android.com/apk/res/android" android:text="Plain"/>'))
   self.assertNotIn('kind:229',self.run_layout('<androidx.appcompat.widget.AppCompatButton xmlns:android="http://schemas.android.com/apk/res/android" android:text="Plain"/>'))
   # MaterialButton icon/shape attributes wired cleanly
   full=self.run_layout('''<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"
    android:text="Call" app:icon="@drawable/ic_btn" app:iconTint="#123456" app:iconPadding="8dp" app:iconGravity="textStart"
    app:cornerRadius="20dp" app:strokeColor="#654321" app:strokeWidth="2dp"/>''',res)
   for tok in (
    'RelativeRule { kind:229, anchor:"filled" }',
    'RelativeRule { kind:173, anchor:"18.0,18.0,0.0,255,255,255,255" }',
    'RelativeRule { kind:176, anchor:"0,0,18,52,86,255" }',
    'RelativeRule { kind:175, anchor:"8.0" }',
    'radius:20.0',
    'background_stroke:Some(Color(101,67,33,255))',
    'background_stroke_width:2.0',
    'state.images.entry("ic_btn".into())',
   ):
    self.assertIn(tok,full)
   null_icon=self.run_layout('<com.google.android.material.button.MaterialButton xmlns:app="http://schemas.android.com/apk/res-auto" app:icon="@null" app:iconTint="@null" app:strokeColor="@null"/>')
   self.assertIn('RelativeRule { kind:173, anchor:"0,0,0,0,0,0,0" }',null_icon)
   self.assertNotIn('kind:176',null_icon)
   with self.assertRaisesRegex(ValueError,'Unsupported iconGravity'):
    self.run_layout('<com.google.android.material.button.MaterialButton xmlns:app="http://schemas.android.com/apk/res-auto" app:iconGravity="diagonal"/>')
 def test_wave4a_chip_style_rule_230(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'chip_dot.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android" android:shape="oval"><solid android:color="#224466"/><size android:width="16dp" android:height="16dp"/></shape>')
   assist=self.run_layout('<com.google.android.material.chip.Chip xmlns:android="http://schemas.android.com/apk/res/android" android:text="Help"/>')
   self.assertIn('RelativeRule { kind:230, anchor:"assist|0" }',assist)
   self.assertNotIn('kind:229',assist)
   filt=self.run_layout('<com.google.android.material.chip.Chip xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" style="@style/Widget.Material3.Chip.Filter" android:text="Unread" android:checked="true" app:checkable="true" app:checkedIconVisible="true"/>')
   self.assertIn('RelativeRule { kind:230, anchor:"filter|0" }',filt);self.assertIn('checked:true',filt)
   inp=self.run_layout('<com.google.android.material.chip.Chip xmlns:android="http://schemas.android.com/apk/res/android" style="@style/Widget.Material3.Chip.Input" android:text="Alice"/>')
   self.assertIn('RelativeRule { kind:230, anchor:"input|1" }',inp)
   inp_noclose=self.run_layout('<com.google.android.material.chip.Chip xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" style="@style/Widget.Material3.Chip.Input" app:closeIconVisible="false" android:text="Alice"/>')
   self.assertIn('RelativeRule { kind:230, anchor:"input|0" }',inp_noclose)
   sugg=self.run_layout('<com.google.android.material.chip.Chip xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" style="@style/Widget.Material3.Chip.Suggestion" app:closeIconVisible="true" android:text="Reply"/>')
   self.assertIn('RelativeRule { kind:230, anchor:"suggestion|1" }',sugg)
   styled=self.run_layout('''<com.google.android.material.chip.Chip xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"
    android:text="Tag" android:checkable="true" app:chipIcon="@drawable/chip_dot" app:chipBackgroundColor="#112233"
    app:chipStrokeColor="#445566" app:chipStrokeWidth="2dp" app:chipCornerRadius="8dp" app:closeIconVisible="true"/>''',res)
   for tok in (
    'RelativeRule { kind:230, anchor:"assist|1" }',
    'RelativeRule { kind:173, anchor:"16.0,16.0,-1.0,34,68,102,255" }',
    'background:Some(Color(17,34,51,255))',
    'background_stroke:Some(Color(68,85,102,255))',
    'background_stroke_width:2.0',
    'radius:8.0',
   ):
    self.assertIn(tok,styled)
   with self.assertRaisesRegex(ValueError,'Invalid closeIconVisible'):
    self.run_layout('<com.google.android.material.chip.Chip xmlns:app="http://schemas.android.com/apk/res-auto" app:closeIconVisible="maybe"/>')
   with self.assertRaisesRegex(ValueError,'Invalid checkable'):
    self.run_layout('<com.google.android.material.chip.Chip xmlns:app="http://schemas.android.com/apk/res-auto" app:checkable="maybe"/>')
 def test_wave4a_stateful_drawable_layer_rule_231(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'state_shapes.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:state_enabled="false"><shape><solid android:color="#cccccc"/><corners android:topLeftRadius="12dp" android:topRightRadius="12dp" android:bottomRightRadius="4dp" android:bottomLeftRadius="4dp"/><stroke android:width="1dp" android:color="#999999"/></shape></item>
    <item android:state_pressed="true"><inset android:inset="2dp"><shape><solid android:color="#abcdef"/><corners android:radius="10dp"/><stroke android:width="2dp" android:color="#123456"/></shape></inset></item>
    <item android:state_checked="true" android:drawable="@null"/>
    <item><shape><solid android:color="#ffffff"/><corners android:radius="6dp"/><stroke android:width="1dp" android:color="#333333"/></shape></item>
   </selector>''')
   (res/'drawable'/'radii_diff.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:state_pressed="true"><shape><solid android:color="#ff0000"/><corners android:radius="12dp"/></shape></item>
    <item><shape><solid android:color="#00ff00"/></shape></item>
   </selector>''')
   (res/'drawable'/'rip_stateful.xml').write_text('''<ripple xmlns:android="http://schemas.android.com/apk/res/android" android:color="#40112233">
    <item android:drawable="@drawable/state_shapes"/>
   </ripple>''')
   (res/'drawable'/'inset_stateful.xml').write_text('''<inset xmlns:android="http://schemas.android.com/apk/res/android" android:insetLeft="3dp" android:insetTop="1dp" android:insetRight="3dp" android:insetBottom="1dp" android:drawable="@drawable/radii_diff"/>''')
   out=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/state_shapes"/>',res)
   expected_rules=[
    'RelativeRule { kind:231, anchor:"0,1|0.0,0.0,0.0,0.0|12.0,12.0,4.0,4.0|204,204,204,255|1.0,153,153,153,255" }',
    'RelativeRule { kind:231, anchor:"2,0|2.0,2.0,2.0,2.0|10.0|171,205,239,255|2.0,18,52,86,255" }',
    'RelativeRule { kind:231, anchor:"64,0|0.0,0.0,0.0,0.0|0.0|none" }',
    'RelativeRule { kind:231, anchor:"0,0|0.0,0.0,0.0,0.0|6.0|255,255,255,255|1.0,51,51,51,255" }',
   ]
   positions=[out.index(r) for r in expected_rules]
   self.assertEqual(positions,sorted(positions))
   self.assertIn('background:Some(Color(255,255,255,255))',out)
   self.assertNotIn('kind:208',out)
   rdiff=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/radii_diff"/>',res)
   self.assertIn('RelativeRule { kind:231, anchor:"2,0|0.0,0.0,0.0,0.0|12.0|255,0,0,255" }',rdiff)
   self.assertIn('RelativeRule { kind:231, anchor:"0,0|0.0,0.0,0.0,0.0|0.0|0,255,0,255" }',rdiff)
   self.assertNotIn('kind:96',rdiff)
   rip=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/rip_stateful"/>',res)
   self.assertIn('RelativeRule { kind:209, anchor:"17,34,51,64" }',rip)
   self.assertIn(expected_rules[1],rip)
   ins=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/inset_stateful"/>',res)
   self.assertIn('RelativeRule { kind:231, anchor:"2,0|3.0,1.0,3.0,1.0|12.0|255,0,0,255" }',ins)
   self.assertIn('RelativeRule { kind:231, anchor:"0,0|3.0,1.0,3.0,1.0|0.0|0,255,0,255" }',ins)
 def test_wave4a_nine_patch_decoding_and_padding(self):
  def make_rgba_png(w,h,rgba_grid):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=bytearray()
   for y in range(h):
    rows.append(0)
    for x in range(w):
     rows.extend(rgba_grid(x,y))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(bytes(rows)))+chunk(b'IEND',b'')
  def make_nine_patch(inner_w,inner_h,bottom_span,right_span,fill=(10,20,30,255),red_optical=False):
   w,h=inner_w+2,inner_h+2
   def px(x,y):
    if y==0:return (0,0,0,255) if x==1 else (0,0,0,0)
    if x==0:return (0,0,0,255) if y==1 else (0,0,0,0)
    if y==h-1:
     if 1<=x<=w-2:
      ix=x-1
      if bottom_span is not None and bottom_span[0]<=ix<=bottom_span[1]:return (0,0,0,255)
      if red_optical and ix==0:return (255,0,0,255)
     return (0,0,0,0)
    if x==w-1:
     if 1<=y<=h-2:
      iy=y-1
      if right_span is not None and right_span[0]<=iy<=right_span[1]:return (0,0,0,255)
      if red_optical and iy==0:return (255,0,0,255)
     return (0,0,0,0)
    return fill
   return make_rgba_png(w,h,px)
  # Verify real AOSP .9.png asset decoding
  aosp_9=Path('codes/libs/ui/libbhoo/assets/android-expandable-list/expander_open_mtrl_alpha.9.png')
  aw,ah,apx,apad=mod.decode_nine_patch(aosp_9)
  self.assertEqual((aw,ah),(31,31))
  self.assertEqual(len(apx),31*31*4)
  self.assertEqual(apad,[5,5,5,5])
  with tempfile.TemporaryDirectory() as d:
   res=Path(d)
   for sub in ('drawable','drawable-mdpi','drawable-xhdpi'):(res/sub).mkdir()
   # xhdpi 12x8 inner -> bottom black 2..9 (left=2,right=2), right black 1..6 (top=1,bottom=1), red optical ignored
   p_xhdpi=res/'drawable-xhdpi'/'bubble.9.png'
   p_xhdpi.write_bytes(make_nine_patch(12,8,(2,9),(1,6),fill=(11,22,33,255),red_optical=True))
   w,h,stripped,pad_px=mod.decode_nine_patch(p_xhdpi)
   self.assertEqual((w,h),(12,8))
   self.assertEqual(pad_px,[2,1,2,1])
   self.assertEqual(list(stripped[:4]),[11,22,33,255])
   self.assertEqual(list(stripped[-4:]),[11,22,33,255])
   # Missing bottom/right marks -> [0,0,0,0]
   p_nopad=res/'drawable'/'nopad.9.png'
   p_nopad.write_bytes(make_nine_patch(4,4,None,None))
   self.assertEqual(mod.decode_nine_patch(p_nopad)[3],[0,0,0,0])
   # Too small (<3x3) raises ValueError
   p_tiny=res/'drawable'/'tiny.9.png'
   p_tiny.write_bytes(make_rgba_png(2,2,lambda x,y:(0,0,0,255)))
   with self.assertRaisesRegex(ValueError,'Invalid 9-patch PNG dimensions'):
    mod.decode_nine_patch(p_tiny)
   # Layout compilation with xhdpi .9.png background (density=160 -> scale=0.5)
   layout_bg=res/'bg.xml'
   layout_bg.write_text('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/bubble"/>')
   out_xhdpi=mod.compile_layout(layout_bg,res,density=160)
   for tok in (
    'background_padding:Some([1.0,0.5,1.0,0.5])',
    'background_min_size:[6.0,4.0]',
    'state.image_scales.insert("bubble".into(),0.5)',
    'bhoo::Image::rgba(12,8,vec![11,22,33,255,',
   ):
    self.assertIn(tok,out_xhdpi)
   # Add mdpi .9.png (6x4 inner, bottom 1..4 -> left=1,right=1, right 1..2 -> top=1,bottom=1) -> selected at density=160 (scale=1.0)
   (res/'drawable-mdpi'/'bubble.9.png').write_bytes(make_nine_patch(6,4,(1,4),(1,2),fill=(44,55,66,255)))
   out_mdpi=mod.compile_layout(layout_bg,res,density=160)
   self.assertIn('background_padding:Some([1.0,1.0,1.0,1.0])',out_mdpi)
   self.assertIn('bhoo::Image::rgba(6,4,vec![44,55,66,255,',out_mdpi)
   out_at_320=mod.compile_layout(layout_bg,res,density=320)
   self.assertIn('background_padding:Some([1.0,0.5,1.0,0.5])',out_at_320)
   self.assertIn('bhoo::Image::rgba(12,8,vec![11,22,33,255,',out_at_320)
   # ImageView src and TextView compound drawable with .9.png embed clean stripped image
   img_out=self.run_layout('<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/bubble"/>',res)
   self.assertIn('bhoo::Image::rgba(6,4,vec![44,55,66,255,',img_out)
   comp_out=self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:drawableStart="@drawable/bubble"/>',res)
   self.assertIn('RelativeRule { kind:173, anchor:"6.0,4.0,0.0,0,0,0,255" }',comp_out)
   self.assertIn('bhoo::Image::rgba(6,4,vec![44,55,66,255,',comp_out)
   # Ambiguous .png and .9.png in same drawable folder rejected
   (res/'drawable'/'dup.png').write_bytes(make_rgba_png(4,4,lambda x,y:(1,2,3,255)))
   (res/'drawable'/'dup.9.png').write_bytes(make_nine_patch(4,4,(0,3),(0,3)))
   with self.assertRaisesRegex(ValueError,'Ambiguous PNG drawable'):
    self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/dup"/>',res)
 def test_wave5a_shape_appearance_resolution(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'shapes.xml').write_text('''<resources>
    <dimen name="card_rad">14dp</dimen>
    <style name="Shape.Asym" parent="ShapeAppearance.MaterialComponents">
     <item name="cornerFamily">rounded</item>
     <item name="cornerSizeTopLeft">0dp</item>
     <item name="cornerSizeTopRight">@dimen/card_rad</item>
     <item name="cornerSizeBottomRight">0dp</item>
     <item name="cornerSizeBottomLeft">@dimen/card_rad</item>
    </style>
    <style name="Shape.CutChild" parent="ShapeAppearance.Material3.Corner.Large">
     <item name="cornerFamily">cut</item>
     <item name="cornerSizeBottomRight">4dp</item>
    </style>
    <style name="Shape.Pct10" parent="">
     <item name="cornerSize">10%</item>
    </style>
    <style name="Shape.Pill" parent="">
     <item name="cornerSize">50%</item>
    </style>
    <style name="Shape.BadFamily" parent="">
     <item name="cornerFamily">zigzag</item>
     <item name="cornerSize">8dp</item>
    </style>
    <style name="Shape.BadSize" parent="">
     <item name="cornerSize">150%</item>
    </style>
    <style name="StyledRoundBtn" parent="Widget.Material3.Button">
     <item name="shapeAppearance">@style/ShapeAppearance.Material3.Corner.ExtraLarge</item>
    </style>
   </resources>''')
   # All Material 3 built-in corner styles + MaterialComponents styles + theme refs
   for style_ref,expected_r in [
    ('@style/ShapeAppearance.Material3.Corner.Full','-1.0'),
    ('@style/ShapeAppearance.Material3.Corner.ExtraLarge','28.0'),
    ('@style/ShapeAppearance.Material3.Corner.Large','16.0'),
    ('@style/ShapeAppearance.Material3.Corner.Medium','12.0'),
    ('@style/ShapeAppearance.Material3.Corner.Small','8.0'),
    ('@style/ShapeAppearance.Material3.Corner.ExtraSmall','4.0'),
    ('@style/ShapeAppearance.Material3.Corner.None','0.0'),
    ('@style/ShapeAppearance.Material3.Corner.LargeIncreased','20.0'),
    ('@style/ShapeAppearance.Material3.Corner.ExtraLargeIncreased','32.0'),
    ('@style/ShapeAppearance.Material3.Corner.ExtraExtraLarge','48.0'),
    ('@style/ShapeAppearance.MaterialComponents.SmallComponent','4.0'),
    ('@style/ShapeAppearance.MaterialComponents.MediumComponent','4.0'),
    ('@style/ShapeAppearance.MaterialComponents.LargeComponent','0.0'),
    ('?attr/shapeAppearanceCornerLarge','16.0'),
    ('?attr/shapeAppearanceCornerFull','-1.0'),
   ]:
    with self.subTest(shape=style_ref):
     out=self.run_layout(f'<View xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="{style_ref}"/>',res)
     self.assertIn(f',radius:{expected_r},',out)
     self.assertNotIn('kind:206',out)
   # Base shapeAppearance + ShapeAppearanceOverlay composition -> RULE_CORNER_RADII (206)
   top_only=self.run_layout('<View xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.Large" app:shapeAppearanceOverlay="@style/ShapeAppearanceOverlay.Material3.Corner.Top"/>',res)
   self.assertIn('RelativeRule { kind:206, anchor:"16.0,16.0,0.0,0.0" }',top_only)
   self.assertIn(',radius:0.0,',top_only)
   # App-local asymmetric style (like Sunflower card/button)
   asym=self.run_layout('<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearanceOverlay="@style/Shape.Asym"/>',res)
   self.assertIn('RelativeRule { kind:206, anchor:"0.0,14.0,0.0,14.0" }',asym)
   # Inheriting from builtin M3 style and overriding single corner + cut family
   cut_child=self.run_layout('<View xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/Shape.CutChild"/>',res)
   self.assertIn('RelativeRule { kind:206, anchor:"16.0,16.0,4.0,16.0" }',cut_child)
   # Percentage cornerSize on ShapeableImageView with explicit dp dimensions (10% of 100dp = 10.0)
   pct=self.run_layout('<com.google.android.material.imageview.ShapeableImageView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" android:layout_width="100dp" android:layout_height="120dp" app:shapeAppearanceOverlay="@style/Shape.Pct10" app:strokeColor="#112233" app:strokeWidth="2dp"/>',res)
   self.assertIn('kind:Kind::Image',pct)
   self.assertIn(',radius:10.0,',pct)
   self.assertIn('background_stroke:Some(Color(17,34,51,255)),background_stroke_width:2.0',pct)
   # 50% cornerSize in local style -> pill -1.0; FAB overriding default pill with Medium -> 12.0
   pill=self.run_layout('<View xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearanceOverlay="@style/Shape.Pill"/>',res)
   self.assertIn(',radius:-1.0,',pill)
   fab_med=self.run_layout('<com.google.android.material.floatingactionbutton.FloatingActionButton xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.Medium"/>',res)
   self.assertIn(',radius:12.0,',fab_med)
   # shapeAppearance inherited from widget style, and overridden by explicit app:cornerRadius on MaterialButton
   styled_btn=self.run_layout('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" style="@style/StyledRoundBtn" android:text="Go"/>',res)
   self.assertIn(',radius:28.0,',styled_btn)
   explicit_btn=self.run_layout('<com.google.android.material.button.MaterialButton xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" style="@style/StyledRoundBtn" app:cornerRadius="6dp" android:text="Go"/>',res)
   self.assertIn(',radius:6.0,',explicit_btn)
   # Strict validation errors
   with self.assertRaisesRegex(ValueError,'Unresolved style'):
    self.run_layout('<View xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/DoesNotExist"/>',res)
   with self.assertRaisesRegex(ValueError,'Unsupported cornerFamily'):
    self.run_layout('<View xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/Shape.BadFamily"/>',res)
   with self.assertRaisesRegex(ValueError,'Invalid cornerSize'):
    self.run_layout('<View xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/Shape.BadSize"/>',res)
 def test_wave5a_material3_and_androidx_tags_and_attrs(self):
  xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" android:orientation="vertical" android:clipToOutline="true" android:outlineProvider="bounds" app:elevation="6dp">
   <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab" app:fabAlignmentMode="center" app:fabAnimationMode="slide" app:fabCradleMargin="8dp" app:fabCradleRoundedCornerRadius="12dp" app:hideOnScroll="true" app:navigationIconTint="#112233" app:titleCentered="true"/>
   <com.google.android.material.navigation.NavigationView android:id="@+id/nav" app:menu="@menu/drawer_menu" app:headerLayout="@layout/nav_header"/>
   <com.google.android.material.navigationrail.NavigationRailView android:id="@+id/rail" app:menu="@menu/rail_menu" app:headerLayout="@layout/rail_header" app:menuGravity="top" app:labelVisibilityMode="labeled"/>
   <com.google.android.material.bottomnavigation.BottomNavigationView android:id="@+id/bnav" app:menu="@menu/bottom_menu" app:labelVisibilityMode="selected"/>
   <com.google.android.material.search.SearchBar android:id="@+id/sbar" android:hint="Search messages" app:strokeColor="#445566" app:strokeWidth="1dp"/>
   <com.google.android.material.search.SearchView android:id="@+id/msv" android:hint="Filter mail" app:autoShowKeyboard="true"/>
   <androidx.appcompat.widget.SearchView android:id="@+id/asv" app:iconifiedByDefault="false" app:queryHint="Find contacts"/>
   <com.google.android.material.button.MaterialButtonToggleGroup android:id="@+id/toggle_grp" app:singleSelection="true" app:selectionRequired="false">
    <com.google.android.material.button.MaterialButton android:id="@+id/tb1" android:text="Day"/>
    <com.google.android.material.button.MaterialButton android:id="@+id/tb2" android:text="Week"/>
   </com.google.android.material.button.MaterialButtonToggleGroup>
   <com.google.android.material.tabs.TabLayout android:id="@+id/tabs" app:tabMode="scrollable" app:tabGravity="center" app:tabIndicatorColor="#6750A4" app:tabTextColor="#49454F" app:tabSelectedTextColor="#6750A4">
    <com.google.android.material.tabs.TabItem android:id="@+id/tab1" android:text="Inbox"/>
    <com.google.android.material.tabs.TabItem android:id="@+id/tab2" android:text="Sent" android:layout="@layout/custom_tab"/>
   </com.google.android.material.tabs.TabLayout>
   <com.google.android.material.slider.RangeSlider android:id="@+id/rslider" android:valueFrom="0" android:valueTo="100" android:stepSize="10" app:minSeparation="20dp"/>
   <com.google.android.material.appbar.MaterialToolbar android:id="@+id/mtb" app:title="Mail" app:titleCentered="true" app:navigationIconTint="#223344"/>
   <com.google.android.material.floatingactionbutton.FloatingActionButton android:id="@+id/fab" app:fabSize="mini" app:maxImageSize="24dp" app:elevation="4dp"/>
   <com.google.android.material.progressindicator.LinearProgressIndicator android:id="@+id/lpi" app:indeterminate="true" app:indicatorColor="#6750A4" app:trackColor="#E7E0EC" app:trackCornerRadius="4dp"/>
   <com.google.android.material.progressindicator.CircularProgressIndicator android:id="@+id/cpi" app:indeterminate="false" app:indicatorColor="#B3261E" app:trackColor="#F9DEDC" app:trackCornerRadius="2dp"/>
   <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_grid" app:layoutManager="androidx.recyclerview.widget.GridLayoutManager" app:spanCount="3"/>
   <androidx.preference.PreferenceRecyclerView android:id="@+id/prv"/>
   <androidx.preference.internal.PreferenceImageView android:id="@+id/piv" app:maxWidth="48dp" app:maxHeight="48dp"/>
   <androidx.appcompat.widget.ActionBarContainer android:id="@+id/abc"/>
   <androidx.appcompat.widget.ContentFrameLayout android:id="@+id/cfl"/>
   <androidx.appcompat.widget.FitWindowsLinearLayout android:id="@+id/fwll"/>
   <androidx.appcompat.widget.DialogTitle android:id="@+id/dtitle" android:text="Confirm" app:drawableTintMode="src_in"/>
   <fragment android:id="@+id/frag" android:name="com.example.MainFragment"/>
   <EditText android:id="@+id/ed" android:lines="2" android:saveEnabled="false"><requestFocus/></EditText>
   <androidx.constraintlayout.widget.ConstraintLayout android:id="@+id/cl">
    <TextView android:id="@+id/f1" android:text="One"/>
    <TextView android:id="@+id/f2" android:text="Two"/>
    <androidx.constraintlayout.widget.Placeholder android:id="@+id/ph" app:content="@id/f1" app:placeholder_emptyVisibility="gone"/>
    <androidx.constraintlayout.helper.widget.Flow android:id="@+id/flow" app:constraint_referenced_ids="f1,f2" app:flow_horizontalGap="8dp" app:flow_verticalGap="4dp" app:flow_wrapMode="aligned" app:flow_horizontalStyle="spread_inside" app:flow_verticalStyle="packed" app:flow_horizontalBias="0.25" app:flow_verticalBias="0.5" app:flow_maxElementsWrap="3"/>
   </androidx.constraintlayout.widget.ConstraintLayout>
  </LinearLayout>'''
  out=self.run_layout(xml)
  for tok in (
   'RelativeRule { kind:193, anchor:"6.0" }',
   'kind:Kind::Toolbar,id:"bab"',
   'kind:Kind::Linear,id:"nav",width:Size::Wrap,height:Size::Wrap,min_width:0.0,min_height:0.0,horizontal:false',
   'kind:Kind::Linear,id:"rail",width:Size::Wrap,height:Size::Wrap,min_width:0.0,min_height:0.0,horizontal:false',
   'kind:Kind::Linear,id:"bnav",width:Size::Wrap,height:Size::Wrap,min_width:0.0,min_height:0.0,horizontal:true',
   'kind:Kind::SearchView,id:"sbar"',
   'RelativeRule { kind:107, anchor:"Search messages" }',
   'background_stroke:Some(Color(68,85,102,255)),background_stroke_width:1.0',
   'kind:Kind::Frame,id:"msv"',
   'RelativeRule { kind:302, anchor:"|Filter mail|0|0" }',
   'kind:Kind::SearchView,id:"asv"',
   'RelativeRule { kind:107, anchor:"Find contacts" }',
   'kind:Kind::Linear,id:"toggle_grp",width:Size::Wrap,height:Size::Wrap,min_width:0.0,min_height:0.0,horizontal:true',
   'kind:Kind::Button,id:"tab1"',
   'kind:Kind::Button,id:"tab2"',
   'kind:Kind::SeekBar,id:"rslider"',
   'kind:Kind::Toolbar,id:"mtb"',
   'RelativeRule { kind:193, anchor:"4.0" }',
   'kind:Kind::ProgressBar,id:"lpi"',
   'kind:149, anchor:"0,0,103,80,164,255"',
   'kind:150, anchor:"0,0,231,224,236,255"',
   ',radius:4.0,',
   'kind:Kind::ProgressBar,id:"cpi"',
   ',radius:2.0,',
   'kind:Kind::GridView,id:"rv_grid"',
   'RelativeRule { kind:91, anchor:"3" }',
   'kind:Kind::ListView,id:"prv"',
   'kind:Kind::Image,id:"piv"',
   'image_max_width:Some(48.0),image_max_height:Some(48.0)',
   'kind:Kind::Frame,id:"abc"',
   'kind:Kind::Frame,id:"cfl"',
   'kind:Kind::Linear,id:"fwll"',
   'kind:Kind::Text,id:"dtitle"',
   'RelativeRule { kind:177, anchor:"1" }',
   'kind:Kind::Frame,id:"frag"',
   'kind:Kind::Edit,id:"ed"',
   'kind:Kind::View,id:"ph"',
   'kind:Kind::View,id:"flow"',
  ):
   self.assertIn(tok,out)
  # Verify Flow and Placeholder default to Visibility::Gone and LPI/CPI indeterminate override
  self.assertIn('visibility:Visibility::Gone',out)
  lpi_part=out[out.index('id:"lpi"'):out.index('id:"cpi"')]
  self.assertIn('indeterminate:true',lpi_part)
  cpi_part=out[out.index('id:"cpi"'):out.index('id:"rv_grid"')]
  self.assertIn('indeterminate:false',cpi_part)
  # Strict rejections for invalid enum/bool values
  for bad_xml,err in [
   ('<View xmlns:android="http://schemas.android.com/apk/res/android" android:outlineProvider="fancy"/>','Unsupported outlineProvider'),
   ('<View xmlns:android="http://schemas.android.com/apk/res/android" android:clipToOutline="maybe"/>','Unsupported boolean'),
   ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:fabAlignmentMode="left"/>','Unsupported fabAlignmentMode'),
   ('<com.google.android.material.tabs.TabLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:tabMode="floating"/>','Unsupported tabMode'),
   ('<com.google.android.material.bottomnavigation.BottomNavigationView xmlns:app="http://schemas.android.com/apk/res-auto" app:labelVisibilityMode="always"/>','Unsupported labelVisibilityMode'),
   ('<com.google.android.material.floatingactionbutton.FloatingActionButton xmlns:app="http://schemas.android.com/apk/res-auto" app:fabSize="giant"/>','Unsupported fabSize'),
   ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_wrapMode="spiral"/>','Unsupported flow_wrapMode'),
  ]:
   with self.subTest(err=err):
    with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml)
 def test_wave5a_extra_xml_drawables(self):
  def make_png(w,h,rgba):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=b''.join(b'\x00'+bytes(rgba)*w for _ in range(h))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'sample_px.png').write_bytes(make_png(8,6,(10,20,30,255)))
   (res/'drawable'/'base_box.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#112233"/><corners android:radius="6dp"/></shape>')
   (res/'drawable'/'hairline.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android" android:shape="line"><stroke android:width="2dp" android:color="#445566" android:dashWidth="4dp" android:dashGap="2dp"/></shape>')
   (res/'drawable'/'ring_solid.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android" android:shape="ring" android:thickness="5dp" android:innerRadius="12dp" android:useLevel="false"><solid android:color="#778899"/></shape>')
   (res/'drawable'/'ring_stroke.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android" android:shape="ring"><stroke android:width="3dp" android:color="#aabbcc"/></shape>')
   (res/'drawable'/'rot_ref.xml').write_text('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:fromDegrees="45" android:drawable="@drawable/base_box"/>')
   (res/'drawable'/'scale_inline.xml').write_text('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:scaleWidth="80%"><shape><solid android:color="#223344"/><corners android:radius="4dp"/></shape></scale>')
   (res/'drawable'/'clip_ref.xml').write_text('<clip xmlns:android="http://schemas.android.com/apk/res/android" android:clipOrientation="horizontal" android:gravity="left" android:drawable="@drawable/base_box"/>')
   (res/'drawable'/'levels.xml').write_text('''<level-list xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:minLevel="1" android:maxLevel="2"><shape><solid android:color="#ff0000"/></shape></item>
    <item android:minLevel="0" android:maxLevel="0"><shape><solid android:color="#00aa55"/></shape></item>
   </level-list>''')
   (res/'drawable'/'trans.xml').write_text('''<transition xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:drawable="@drawable/base_box"/>
    <item><shape><solid android:color="#999999"/></shape></item>
   </transition>''')
   (res/'drawable'/'bmp_bg.xml').write_text('<bitmap xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/sample_px" android:tint="#336699" android:gravity="center"/>')
   (res/'drawable'/'rot_img.xml').write_text('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:fromDegrees="90" android:drawable="@drawable/sample_px"/>')
   (res/'drawable'/'lvl_img.xml').write_text('<level-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:minLevel="0" android:maxLevel="1" android:drawable="@drawable/sample_px"/></level-list>')
   (res/'drawable'/'grad_radial3.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="radial" android:gradientRadius="50%" android:startColor="#ff0000" android:centerColor="#00ff00" android:endColor="#0000ff"/></shape>')
   (res/'drawable'/'grad_sweep.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="sweep" android:startColor="#102030" android:endColor="#405060"/></shape>')
   # line & ring shapes
   line_out=self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:divider="@drawable/hairline" android:showDividers="middle"><View android:background="@drawable/hairline"/><ImageView android:src="@drawable/hairline"/></LinearLayout>',res)
   self.assertIn('divider_color:Some(Color(68,85,102,255)),divider_thickness:2.0',line_out)
   self.assertIn('background:None,',line_out)
   self.assertIn('background_stroke:Some(Color(68,85,102,255)),background_stroke_width:2.0',line_out)
   self.assertIn('background_min_size:[0.0,2.0]',line_out)
   self.assertIn('state.images.entry("hairline".into())',line_out)
   ring_out=self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><View android:background="@drawable/ring_solid"/><View android:background="@drawable/ring_stroke"/><ImageView android:src="@drawable/ring_solid"/></LinearLayout>',res)
   self.assertIn('background_stroke:Some(Color(119,136,153,255)),background_stroke_width:5.0',ring_out)
   self.assertIn('background_stroke:Some(Color(170,187,204,255)),background_stroke_width:3.0',ring_out)
   self.assertIn(',radius:-1.0,',ring_out)
   self.assertIn('state.images.entry("ring_solid".into())',ring_out)
   # rotate / scale / clip / level-list / transition wrappers
   for ref,expected in [
    ('@drawable/rot_ref',('background:Some(Color(17,34,51,255))','radius:6.0')),
    ('@drawable/scale_inline',('background:Some(Color(34,51,68,255))','radius:4.0')),
    ('@drawable/clip_ref',('background:Some(Color(17,34,51,255))','radius:6.0')),
    ('@drawable/levels',('background:Some(Color(0,170,85,255))',)),
    ('@drawable/trans',('background:Some(Color(17,34,51,255))','radius:6.0')),
   ]:
    with self.subTest(wrapper=ref):
     out=self.run_layout(f'<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="{ref}"/>',res)
     for tok in expected:self.assertIn(tok,out)
   # ImageView with rotate/level-list/bitmap drawables
   img_wrap=self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><ImageView android:src="@drawable/rot_img"/><ImageView android:src="@drawable/lvl_img"/><ImageView android:src="@drawable/bmp_bg"/></LinearLayout>',res)
   for tok in ('state.images.entry("rot_img".into())','state.images.entry("lvl_img".into())','state.images.entry("bmp_bg".into())','image_tint:Some(Color(51,102,153,255))'):
    self.assertIn(tok,img_wrap)
   # <bitmap> in android:background
   bmp_bg=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/bmp_bg"/>',res)
   for tok in ('background_min_size:[8.0,6.0]','RelativeRule { kind:194, anchor:"51,102,153,255" }','state.images.entry("bmp_bg".into())','AndroidDrawableTint::Fixed(Color(51,102,153,255))'):
    self.assertIn(tok,bmp_bg)
   # Non-linear radial (3-stop) and sweep (2-stop) gradients
   r3=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/grad_radial3"/>',res)
   self.assertIn('RelativeRule { kind:207, anchor:"radial|0.5,0.5,0|255,0,0,255|0,255,0,255|0,0,255,255" }',r3)
   sw=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/grad_sweep"/>',res)
   self.assertIn('RelativeRule { kind:207, anchor:"sweep|0.5,0.5|16,32,48,255||64,80,96,255" }',sw)
   # Strict bitmap validation
   (res/'drawable'/'bmp_bad_grav.xml').write_text('<bitmap xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/sample_px" android:gravity="diagonal"/>')
   with self.assertRaisesRegex(ValueError,'Unsupported bitmap gravity'):
    self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/bmp_bad_grav"/>',res)
 def test_wave5a_custom_view_map_strict_by_default(self):
  import json,subprocess,sys
  xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <org.fossify.commons.views.MyTextView android:id="@+id/title" android:text="Hello" android:textAppearance="?attr/textAppearanceTitleMedium"/>
   <com.google.samples.apps.sunflower.views.MaskedCardView android:id="@+id/card" app:cardCornerRadius="12dp"><TextView android:text="Plant"/></com.google.samples.apps.sunflower.views.MaskedCardView>
  </LinearLayout>'''
  with tempfile.TemporaryDirectory() as d:
   lp=Path(d)/'custom.xml'; lp.write_text(xml)
   # Default strict mode MUST reject unknown views
   with self.assertRaisesRegex(ValueError,'textAppearance unsupported|Unsupported Android view'):
    mod.compile_layout(lp)
   with self.assertRaisesRegex(ValueError,'Unsupported Android view'):
    self.run_layout('<org.fossify.commons.views.MyTextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="Hi"/>')
   # Explicit custom_view_map dict maps custom classes cleanly
   vmap={
    'org.fossify.commons.views.MyTextView':'TextView',
    'com.google.samples.apps.sunflower.views.MaskedCardView':'com.google.android.material.card.MaterialCardView',
   }
   out=mod.compile_layout(lp,custom_view_map=vmap)
   for tok in ('kind:Kind::Text,id:"title"','text:"Hello"','text_size:16.0','kind:Kind::MaterialCard,id:"card"','RelativeRule { kind:66, anchor:"12.0" }','text:"Plant"'):
    self.assertIn(tok,out)
   # CLI --custom-view-map PATH works, and CLI without it still strictly fails
   mp=Path(d)/'map.json'; mp.write_text(json.dumps(vmap))
   out_file=Path(d)/'out.rs'
   strict_proc=subprocess.run([sys.executable,str(Path(__file__).with_name('import-android-layout.py')),str(lp),str(out_file)],capture_output=True,text=True)
   self.assertNotEqual(strict_proc.returncode,0)
   subprocess.run([sys.executable,str(Path(__file__).with_name('import-android-layout.py')),str(lp),str(out_file),'--custom-view-map',str(mp)],capture_output=True,text=True,check=True)
   self.assertEqual(out_file.read_text(),out)
   # Unsupported mapping target rejected
   with self.assertRaisesRegex(ValueError,'Unsupported custom view map target'):
    mod.compile_layout(lp,custom_view_map={'org.fossify.commons.views.MyTextView':'com.example.UnknownBase'})
 def test_wave6a_flow_rule_232(self):
  xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <TextView android:id="@+id/a" android:text="A"/>
   <TextView android:id="@+id/b" android:text="B"/>
   <TextView android:id="@+id/c" android:text="C"/>
   <androidx.constraintlayout.helper.widget.Flow android:id="@+id/flow_default" app:constraint_referenced_ids="a,b"/>
   <androidx.constraintlayout.helper.widget.Flow android:id="@+id/flow_full" android:orientation="vertical" app:flow_wrapMode="aligned" app:flow_horizontalGap="8dp" app:flow_verticalGap="12dp" app:flow_maxElementsWrap="2" app:constraint_referenced_ids="a,@id/b,c"/>
   <androidx.constraintlayout.helper.widget.Flow android:id="@+id/flow_chain2" app:flow_wrapMode="chain2" app:flow_maxElementsWrap="-1" app:constraint_referenced_ids="a,c"/>
  </androidx.constraintlayout.widget.ConstraintLayout>'''
  out=self.run_layout(xml)
  self.assertIn('RelativeRule { kind:232, anchor:"0|none|0|0|0|a,b" }',out)
  self.assertIn('RelativeRule { kind:232, anchor:"1|aligned|8.0|12.0|2|a,b,c" }',out)
  self.assertIn('RelativeRule { kind:232, anchor:"0|chain|0|0|0|a,c" }',out)
  with self.assertRaisesRegex(ValueError,'Unsupported Flow orientation'):
   self.run_layout('<androidx.constraintlayout.helper.widget.Flow xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="diagonal"/>')
  with self.assertRaisesRegex(ValueError,'Invalid flow_maxElementsWrap'):
   self.run_layout('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_maxElementsWrap="-2"/>')
 def test_wave6a_constraint_circle_rule_233(self):
  xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <View android:id="@+id/center"/>
   <View android:id="@+id/orbit_full" app:layout_constraintCircle="@id/center" app:layout_constraintCircleRadius="48dp" app:layout_constraintCircleAngle="135"/>
   <View android:id="@+id/orbit_defaults" app:layout_constraintCircle="@+id/center"/>
   <View android:id="@+id/orbit_parent" app:layout_constraintCircle="parent" app:layout_constraintCircleRadius="20dp" app:layout_constraintCircleAngle="45.5"/>
  </androidx.constraintlayout.widget.ConstraintLayout>'''
  out=self.run_layout(xml)
  self.assertIn('RelativeRule { kind:233, anchor:"center,48.0,135.0" }',out)
  self.assertIn('RelativeRule { kind:233, anchor:"center,0,0" }',out)
  self.assertIn('RelativeRule { kind:233, anchor:"parent,20.0,45.5" }',out)
  with self.assertRaisesRegex(ValueError,'Invalid layout_constraintCircle '):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintCircle="not_an_id"/></ConstraintLayout>')
  with self.assertRaisesRegex(ValueError,'Invalid layout_constraintCircleAngle'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintCircle="@id/center" app:layout_constraintCircleAngle="north"/></ConstraintLayout>')
 def test_wave6a_clip_to_outline_rule_234(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'shapes.xml').write_text('''<resources>
    <style name="AsymCorner" parent="ShapeAppearance.MaterialComponents">
     <item name="cornerSizeTopLeft">0dp</item><item name="cornerSizeTopRight">12dp</item>
     <item name="cornerSizeBottomRight">0dp</item><item name="cornerSizeBottomLeft">12dp</item>
    </style>
   </resources>''')
   # Explicit clipToOutline="true" on generic View
   v_true=self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:clipToOutline="true"/>',res)
   self.assertIn('RelativeRule { kind:234, anchor:"true" }',v_true)
   # ShapeableImageView with non-zero radius / pill / RULE_CORNER_RADII auto-emits 234 unless clipToOutline="false"
   siv_round=self.run_layout('<com.google.android.material.imageview.ShapeableImageView xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.Full"/>',res)
   self.assertIn('RelativeRule { kind:234, anchor:"true" }',siv_round)
   siv_asym=self.run_layout('<com.google.android.material.imageview.ShapeableImageView xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearanceOverlay="@style/AsymCorner"/>',res)
   self.assertIn('RelativeRule { kind:206, anchor:"0.0,12.0,0.0,12.0" }',siv_asym)
   self.assertIn('RelativeRule { kind:234, anchor:"true" }',siv_asym)
   siv_optout=self.run_layout('<com.google.android.material.imageview.ShapeableImageView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.Large" android:clipToOutline="false"/>',res)
   self.assertNotIn('kind:234',siv_optout)
   siv_plain=self.run_layout('<com.google.android.material.imageview.ShapeableImageView/>',res)
   self.assertNotIn('kind:234',siv_plain)
   # MaterialCardView with non-zero shape radius or RULE_CORNER_RADII auto-emits 234 unless clipToOutline="false"
   card_r=self.run_layout('<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.Large"/>',res)
   self.assertIn('RelativeRule { kind:234, anchor:"true" }',card_r)
   card_asym=self.run_layout('<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearanceOverlay="@style/AsymCorner"/>',res)
   self.assertIn('RelativeRule { kind:234, anchor:"true" }',card_asym)
   card_zero=self.run_layout('<com.google.android.material.card.MaterialCardView xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.None"/>',res)
   self.assertNotIn('kind:234',card_zero)
   card_optout=self.run_layout('<com.google.android.material.card.MaterialCardView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.Large" android:clipToOutline="false"/>',res)
   self.assertNotIn('kind:234',card_optout)
   card_explicit=self.run_layout('<com.google.android.material.card.MaterialCardView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:cardCornerRadius="12dp" android:clipToOutline="true"/>',res)
   self.assertIn('RelativeRule { kind:234, anchor:"true" }',card_explicit)
 def test_wave6a_builtin_library_dimens_and_colors_fallback_and_override(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'overrides.xml').write_text('''<resources>
    <dimen name="mtrl_btn_corner_radius">18dp</dimen>
    <color name="design_default_color_primary">#112233</color>
   </resources>''')
   # Without app override -> pinned Material Components / AppCompat AAR fallback values
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"
    android:paddingStart="@dimen/abc_select_dialog_padding_start_material"
    android:paddingEnd="@dimen/abc_list_item_padding_horizontal_material"
    android:minHeight="@dimen/abc_action_bar_default_height_material"
    android:background="@color/design_default_color_primary">
    <com.google.android.material.button.MaterialButton android:id="@+id/b1" app:cornerRadius="@dimen/mtrl_btn_corner_radius" android:textColor="@color/mtrl_btn_text_color_disabled"/>
    <com.google.android.material.card.MaterialCardView android:id="@+id/c1" app:cardCornerRadius="@dimen/mtrl_card_corner_radius" app:cardBackgroundColor="@color/m3_sys_color_light_surface_container"/>
    <View android:id="@+id/v1" android:layout_width="@dimen/design_fab_size_normal" android:layout_height="@dimen/design_bottom_navigation_height" android:minWidth="@dimen/mtrl_fab_min_touch_target" android:background="@color/mtrl_scrim_color"/>
    <View android:id="@+id/v2" android:layout_width="@dimen/m3_sys_shape_corner_value_large" android:layout_height="@dimen/m3_sys_shape_corner_value_extra_large" android:background="@color/material_dynamic_primary70"/>
    <TextView android:id="@+id/t1" android:textColor="@color/abc_primary_text_material_light" android:text="Hi"/>
   </LinearLayout>'''
   fallback=self.run_layout(xml)
   for tok in (
    'padding_relative:[Some(20.0),Some(16.0)]',
    'min_height:56.0',
    'background:Some(Color(98,0,238,255))',
    'radius:4.0',
    'color:Some(Color(0,0,0,97))',
    'RelativeRule { kind:66, anchor:"4.0" }',
    'RelativeRule { kind:63, anchor:"243,237,247,255" }',
    'width:Size::Dp(56.0),height:Size::Dp(56.0),min_width:48.0',
    'background:Some(Color(0,0,0,82))',
    'width:Size::Dp(16.0),height:Size::Dp(28.0)',
    'background:Some(Color(182,157,248,255))',
    'color:Some(Color(0,0,0,222))',
   ):
    self.assertIn(tok,fallback)
   # With app-local res/values override -> app overrides win over library fallbacks (AAPT2 AAR merge semantics)
   overridden=self.run_layout(xml,res)
   self.assertIn('background:Some(Color(17,34,51,255))',overridden)
   self.assertIn('radius:18.0',overridden)
   # Unknown app-local @dimen/foo and @color/bar MUST still fail strictly
   with self.assertRaisesRegex(ValueError,'Unresolved resource @dimen/foo'):
    self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:padding="@dimen/foo"/>',res)
   with self.assertRaisesRegex(ValueError,'Unresolved resource @color/bar'):
    self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@color/bar"/>',res)
 def test_wave6a_typography_attrs_and_font_family_weights(self):
  xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <TextView android:id="@+id/t_med" android:text="Med" android:fontFamily="sans-serif-medium" android:lineHeight="24sp" android:breakStrategy="balanced" android:hyphenationFrequency="normalFast" android:includeFontPadding="false"/>
   <TextView android:id="@+id/t_blk" android:text="Blk" app:fontFamily="sans-serif-black" app:lineHeight="28dp" android:breakStrategy="high_quality" android:hyphenationFrequency="fullFast"/>
   <TextView android:id="@+id/t_reg" android:text="Reg" android:fontFamily="sans-serif" android:lineHeight="20sp" android:breakStrategy="simple" android:hyphenationFrequency="none" android:includeFontPadding="true"/>
   <TextView android:id="@+id/t_lgt" android:text="Lgt" android:fontFamily="sans-serif-light" android:hyphenationFrequency="full"/>
  </LinearLayout>'''
  out=self.run_layout(xml)
  med_s=out[out.index('id:"t_med"'):out.index('id:"t_blk"')]
  blk_s=out[out.index('id:"t_blk"'):out.index('id:"t_reg"')]
  reg_s=out[out.index('id:"t_reg"'):out.index('id:"t_lgt"')]
  lgt_s=out[out.index('id:"t_lgt"'):]
  self.assertIn('text_bold:true',med_s)
  self.assertIn('text_bold:true',blk_s)
  self.assertIn('text_bold:false',reg_s)
  self.assertIn('text_bold:false',lgt_s)
  # Strict rejections for invalid typography attributes
  for bad_xml,err in [
   ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:lineHeight="1.5"/>','Invalid lineHeight'),
   ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:lineHeight="-4sp"/>','Invalid lineHeight'),
   ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:breakStrategy="jagged"/>','Unsupported breakStrategy'),
   ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:hyphenationFrequency="turbo"/>','Unsupported hyphenationFrequency'),
   ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:includeFontPadding="maybe"/>','Unsupported boolean'),
  ]:
   with self.subTest(err=err):
    with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml)
 def test_wave7a_line_height_and_include_font_padding_rules_235_236(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('''<resources>
    <dimen name="body_lh">22.5sp</dimen>
    <style name="BodyAppearance">
     <item name="android:textSize">15sp</item>
     <item name="android:lineHeight">@dimen/body_lh</item>
     <item name="android:includeFontPadding">false</item>
    </style>
    <style name="AppLineHeightAppearance">
     <item name="lineHeight">30dp</item>
     <item name="includeFontPadding">true</item>
    </style>
    <style name="StyledCaption">
     <item name="android:lineHeight">18sp</item>
     <item name="android:includeFontPadding">false</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" android:lineHeight="24sp" android:includeFontPadding="false">
    <TextView android:id="@+id/t_direct" android:lineHeight="24sp" android:includeFontPadding="false"/>
    <Button android:id="@+id/b_app" app:lineHeight="28dp" android:includeFontPadding="true"/>
    <TextView android:id="@+id/t_appear" android:textAppearance="@style/BodyAppearance"/>
    <TextView android:id="@+id/t_appear_app" android:textAppearance="@style/AppLineHeightAppearance"/>
    <TextView android:id="@+id/t_override" android:textAppearance="@style/BodyAppearance" app:lineHeight="32sp" android:includeFontPadding="true"/>
    <EditText android:id="@+id/e_styled" style="@style/StyledCaption"/>
    <TextView android:id="@+id/t_zero" android:lineHeight="0dp"/>
    <TextView android:id="@+id/t_plain" android:text="Plain"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   # Non-text container emits neither rule 235 nor 236
   root_s=out[:out.index('id:"t_direct"')]
   self.assertNotIn('kind:235',root_s)
   self.assertNotIn('kind:236',root_s)
   slices={
    't_direct':out[out.index('id:"t_direct"'):out.index('id:"b_app"')],
    'b_app':out[out.index('id:"b_app"'):out.index('id:"t_appear"')],
    't_appear':out[out.index('id:"t_appear"'):out.index('id:"t_appear_app"')],
    't_appear_app':out[out.index('id:"t_appear_app"'):out.index('id:"t_override"')],
    't_override':out[out.index('id:"t_override"'):out.index('id:"e_styled"')],
    'e_styled':out[out.index('id:"e_styled"'):out.index('id:"t_zero"')],
    't_zero':out[out.index('id:"t_zero"'):out.index('id:"t_plain"')],
    't_plain':out[out.index('id:"t_plain"'):],
   }
   self.assertIn('RelativeRule { kind:235, anchor:"24.0" }',slices['t_direct'])
   self.assertIn('RelativeRule { kind:236, anchor:"false" }',slices['t_direct'])
   self.assertIn('RelativeRule { kind:235, anchor:"28.0" }',slices['b_app'])
   self.assertIn('RelativeRule { kind:236, anchor:"true" }',slices['b_app'])
   self.assertIn('RelativeRule { kind:235, anchor:"22.5" }',slices['t_appear'])
   self.assertIn('RelativeRule { kind:236, anchor:"false" }',slices['t_appear'])
   self.assertIn('RelativeRule { kind:235, anchor:"30.0" }',slices['t_appear_app'])
   self.assertIn('RelativeRule { kind:236, anchor:"true" }',slices['t_appear_app'])
   self.assertIn('RelativeRule { kind:235, anchor:"32.0" }',slices['t_override'])
   self.assertIn('RelativeRule { kind:236, anchor:"true" }',slices['t_override'])
   self.assertIn('RelativeRule { kind:235, anchor:"18.0" }',slices['e_styled'])
   self.assertIn('RelativeRule { kind:236, anchor:"false" }',slices['e_styled'])
   self.assertNotIn('kind:235',slices['t_zero'])
   self.assertNotIn('kind:235',slices['t_plain'])
   self.assertNotIn('kind:236',slices['t_plain'])
 def test_wave7a_nav_bar_and_menu_inflation_rules_237_238(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'menu').mkdir()
   (res/'values'/'strings.xml').write_text('''<resources>
    <string name="nav_home">Home</string>
    <string name="nav_mail">Mail | Inbox</string>
    <string name="nav_chat">Chat</string>
    <string name="nav_profile">Profile</string>
    <string name="nav_settings">Settings</string>
   </resources>''')
   (res/'menu'/'bottom_nav.xml').write_text('''<menu xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/dest_home" android:title="@string/nav_home" android:icon="@drawable/ic_home_24"/>
    <item android:id="@+id/dest_mail" android:title="@string/nav_mail" android:icon="@drawable/ic_outline_mail_24dp"/>
    <item android:id="@+id/dest_chat" android:title="@string/nav_chat" android:icon="@drawable/ic_baseline_chat_bubble_24"/>
    <item android:id="@+id/dest_hidden" android:title="Hidden" android:visible="false"/>
    <item android:id="@+id/dest_custom" android:title="Custom" android:icon="@drawable/custom_unmapped_logo"/>
   </menu>''')
   (res/'menu'/'rail_nav.xml').write_text('''<menu xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/rail_search" android:title="Search" android:icon="@drawable/ic_menu_search" android:orderInCategory="20"/>
    <item android:id="@+id/rail_menu" android:title="Menu" android:icon="@drawable/ic_menu_open_24" android:orderInCategory="10"/>
    <item android:id="@+id/rail_plain" android:title="Plain"/>
   </menu>''')
   (res/'menu'/'drawer_nav.xml').write_text('''<menu xmlns:android="http://schemas.android.com/apk/res/android">
    <group android:checkableBehavior="single">
     <item android:id="@+id/dr_profile" android:title="@string/nav_profile" android:icon="@drawable/ic_account_circle_24"/>
     <item android:id="@+id/dr_star" android:title="Starred" android:icon="@drawable/ic_star_border_24"/>
    </group>
    <item android:title="More">
     <menu>
      <item android:id="@+id/dr_settings" android:title="@string/nav_settings" android:icon="@drawable/ic_nav_settings_black_24dp"/>
     </menu>
    </item>
   </menu>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.bottomnavigation.BottomNavigationView android:id="@+id/bnav" app:menu="@menu/bottom_nav" app:labelVisibilityMode="labeled"/>
    <com.google.android.material.navigation.NavigationBarView android:id="@+id/nbar" app:menu="@menu/missing_menu"/>
    <com.google.android.material.navigationrail.NavigationRailView android:id="@+id/rail" app:menu="@menu/rail_nav" app:labelVisibilityMode="selected"/>
    <com.google.android.material.navigation.NavigationView android:id="@+id/drawer" app:menu="@menu/drawer_nav"/>
    <com.google.android.material.bottomnavigation.BottomNavigationView android:id="@+id/bnav_null" app:menu="@null" app:labelVisibilityMode="unlabeled"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   bnav_s=out[out.index('id:"bnav"'):out.index('id:"nbar"')]
   nbar_s=out[out.index('id:"nbar"'):out.index('id:"rail"')]
   rail_s=out[out.index('id:"rail"'):out.index('id:"drawer"')]
   drawer_s=out[out.index('id:"drawer"'):out.index('id:"bnav_null"')]
   bnav_null_s=out[out.index('id:"bnav_null"'):]
   # BottomNavigationView with inflated menu
   self.assertIn('RelativeRule { kind:237, anchor:"bar|labeled" }',bnav_s)
   for expected in (
    'RelativeRule { kind:238, anchor:"dest_home|Home|home" }',
    'RelativeRule { kind:238, anchor:"dest_mail|Mail / Inbox|mail" }',
    'RelativeRule { kind:238, anchor:"dest_chat|Chat|chat_bubble" }',
    'RelativeRule { kind:238, anchor:"dest_custom|Custom|" }',
   ):
    self.assertIn(expected,bnav_s)
   self.assertNotIn('dest_hidden',bnav_s)
   # NavigationBarView with missing res/menu/missing_menu.xml degrades gracefully
   self.assertIn('RelativeRule { kind:237, anchor:"bar|auto" }',nbar_s)
   self.assertNotIn('kind:238',nbar_s)
   # NavigationRailView with orderInCategory sorting
   self.assertIn('RelativeRule { kind:237, anchor:"rail|selected" }',rail_s)
   rail_items=[
    'RelativeRule { kind:238, anchor:"rail_plain|Plain|" }',
    'RelativeRule { kind:238, anchor:"rail_menu|Menu|menu_open" }',
    'RelativeRule { kind:238, anchor:"rail_search|Search|search" }',
   ]
   for tok in rail_items:self.assertIn(tok,rail_s)
   self.assertEqual([rail_s.index(tok) for tok in rail_items],sorted(rail_s.index(tok) for tok in rail_items))
   # NavigationView drawer with group + submenu
   self.assertIn('RelativeRule { kind:237, anchor:"drawer|labeled" }',drawer_s)
   for expected in (
    'RelativeRule { kind:238, anchor:"dr_profile|Profile|person" }',
    'RelativeRule { kind:238, anchor:"dr_star|Starred|star" }',
    'RelativeRule { kind:238, anchor:"dr_settings|Settings|settings" }',
   ):
    self.assertIn(expected,drawer_s)
   # @null menu emits 237 without 238
   self.assertIn('RelativeRule { kind:237, anchor:"bar|unlabeled" }',bnav_null_s)
   self.assertNotIn('kind:238',bnav_null_s)
   # Without resources at all, valid @menu/... still degrades gracefully
   no_res=self.run_layout('<com.google.android.material.bottomnavigation.BottomNavigationView xmlns:app="http://schemas.android.com/apk/res-auto" app:menu="@menu/bottom_nav"/>')
   self.assertIn('RelativeRule { kind:237, anchor:"bar|auto" }',no_res)
   self.assertNotIn('kind:238',no_res)
   # Strict validation on bad menu ref / bad menu XML
   with self.assertRaisesRegex(ValueError,'Invalid menu reference'):
    self.run_layout('<com.google.android.material.bottomnavigation.BottomNavigationView xmlns:app="http://schemas.android.com/apk/res-auto" app:menu="not_a_ref"/>',res)
   (res/'menu'/'bad_root.xml').write_text('<notmenu/>')
   with self.assertRaisesRegex(ValueError,'Expected a menu root'):
    self.run_layout('<com.google.android.material.navigation.NavigationView xmlns:app="http://schemas.android.com/apk/res-auto" app:menu="@menu/bad_root"/>',res)
   (res/'menu'/'bad_id.xml').write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="plain_bad" android:title="Bad"/></menu>')
   with self.assertRaisesRegex(ValueError,'Invalid menu item id'):
    self.run_layout('<com.google.android.material.navigationrail.NavigationRailView xmlns:app="http://schemas.android.com/apk/res-auto" app:menu="@menu/bad_id"/>',res)
 def test_wave7a_search_bar_rule_239(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'strings.xml').write_text('<resources><string name="sb_hint">Find messages</string></resources>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.search.SearchBar android:id="@+id/sb_android" android:hint="Search mail" style="?attr/materialSearchBarStyle" app:textCentered="true" app:liftOnScroll="true"/>
    <com.google.android.material.search.SearchBar android:id="@+id/sb_app" app:hint="@string/sb_hint"/>
    <com.google.android.material.search.SearchBar android:id="@+id/sb_empty"/>
    <com.google.android.material.search.SearchView android:id="@+id/sv_m3" android:hint="Full search"/>
    <androidx.appcompat.widget.SearchView android:id="@+id/sv_compat" app:queryHint="Compat search"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   s1=out[out.index('id:"sb_android"'):out.index('id:"sb_app"')]
   s2=out[out.index('id:"sb_app"'):out.index('id:"sb_empty"')]
   s3=out[out.index('id:"sb_empty"'):out.index('id:"sv_m3"')]
   s4=out[out.index('id:"sv_m3"'):]
   self.assertIn('RelativeRule { kind:239, anchor:"Search mail" }',s1)
   self.assertIn('RelativeRule { kind:107, anchor:"Search mail" }',s1)
   self.assertIn('RelativeRule { kind:239, anchor:"Find messages" }',s2)
   self.assertIn('RelativeRule { kind:107, anchor:"Find messages" }',s2)
   self.assertIn('RelativeRule { kind:239, anchor:"" }',s3)
   self.assertNotIn('kind:239',s4)
 def test_wave7a_toggle_group_rule_240(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'bools.xml').write_text('''<resources>
    <bool name="tg_single">true</bool>
    <bool name="tg_req">true</bool>
    <style name="StyledToggle">
     <item name="singleSelection">true</item>
     <item name="selectionRequired">false</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.button.MaterialButtonToggleGroup android:id="@+id/tg_def">
     <com.google.android.material.button.MaterialButton android:id="@+id/b1" android:text="A"/>
    </com.google.android.material.button.MaterialButtonToggleGroup>
    <com.google.android.material.button.MaterialButtonToggleGroup android:id="@+id/tg_full" app:singleSelection="@bool/tg_single" app:selectionRequired="@bool/tg_req" app:checkedButton="@id/b2">
     <com.google.android.material.button.MaterialButton android:id="@+id/b2" android:text="Day"/>
     <com.google.android.material.button.MaterialButton android:id="@+id/b3" android:text="Week"/>
    </com.google.android.material.button.MaterialButtonToggleGroup>
    <com.google.android.material.button.MaterialButtonToggleGroup android:id="@+id/tg_styled" style="@style/StyledToggle" app:checkedButton="@null"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   s_def=out[out.index('id:"tg_def"'):out.index('id:"tg_full"')]
   s_full=out[out.index('id:"tg_full"'):out.index('id:"tg_styled"')]
   s_styled=out[out.index('id:"tg_styled"'):]
   self.assertIn('RelativeRule { kind:240, anchor:"0|0|" }',s_def)
   self.assertIn('RelativeRule { kind:240, anchor:"1|1|b2" }',s_full)
   self.assertIn('RelativeRule { kind:240, anchor:"1|0|" }',s_styled)
   for bad_xml,err in [
    ('<com.google.android.material.button.MaterialButtonToggleGroup xmlns:app="http://schemas.android.com/apk/res-auto" app:singleSelection="maybe"/>','Invalid singleSelection'),
    ('<com.google.android.material.button.MaterialButtonToggleGroup xmlns:app="http://schemas.android.com/apk/res-auto" app:selectionRequired="yes"/>','Invalid selectionRequired'),
    ('<com.google.android.material.button.MaterialButtonToggleGroup xmlns:app="http://schemas.android.com/apk/res-auto" app:checkedButton="not_an_id"/>','Invalid checkedButton'),
   ]:
    with self.subTest(err=err):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave7a_font_families_and_italic_text_styles(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('''<resources>
    <style name="MonoItalic">
     <item name="fontFamily">monospace</item>
     <item name="android:textStyle">italic</item>
    </style>
    <style name="MediumItalic">
     <item name="fontFamily">sans-serif-medium</item>
     <item name="android:textStyle">bold|italic</item>
    </style>
    <style name="ToolbarItalicTitle">
     <item name="android:textSize">20sp</item>
     <item name="android:textStyle">italic</item>
    </style>
    <style name="ToolbarBoldItalicTitle">
     <item name="android:textSize">20sp</item>
     <item name="android:textStyle">bold|italic</item>
    </style>
    <style name="ToolbarBadStyle">
     <item name="android:textStyle">underline</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <TextView android:id="@+id/f_mono" android:fontFamily="monospace" android:textStyle="italic"/>
    <TextView android:id="@+id/f_serif" android:fontFamily="serif" android:textStyle="bold|italic"/>
    <TextView android:id="@+id/f_cond" android:fontFamily="sans-serif-condensed" android:textStyle="italic|bold"/>
    <TextView android:id="@+id/f_light" android:fontFamily="sans-serif-light" android:textStyle="normal|italic"/>
    <TextView android:id="@+id/f_med" android:fontFamily="sans-serif-medium" android:textStyle="italic"/>
    <TextView android:id="@+id/f_blk" app:fontFamily="sans-serif-black" android:textStyle="normal"/>
    <TextView android:id="@+id/f_app_mono" android:textAppearance="@style/MonoItalic"/>
    <TextView android:id="@+id/f_app_med" android:textAppearance="@style/MediumItalic"/>
    <Toolbar android:id="@+id/tb_it" android:title="A" android:titleTextAppearance="@style/ToolbarItalicTitle"/>
    <Toolbar android:id="@+id/tb_bi" android:title="B" android:titleTextAppearance="@style/ToolbarBoldItalicTitle"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['f_mono','f_serif','f_cond','f_light','f_med','f_blk','f_app_mono','f_app_med','tb_it','tb_bi']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('text_bold:false',slices['f_mono'])
   self.assertIn('text_bold:true',slices['f_serif'])
   self.assertIn('text_bold:true',slices['f_cond'])
   self.assertIn('text_bold:false',slices['f_light'])
   self.assertIn('text_bold:true',slices['f_med'])
   self.assertIn('text_bold:true',slices['f_blk'])
   self.assertIn('text_bold:false',slices['f_app_mono'])
   self.assertIn('text_bold:true',slices['f_app_med'])
   self.assertIn('RelativeRule { kind:126, anchor:"normal" }',slices['tb_it'])
   self.assertIn('RelativeRule { kind:126, anchor:"bold" }',slices['tb_bi'])
   with self.assertRaisesRegex(ValueError,'Unsupported textStyle'):
    self.run_layout('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textStyle="bold|underline"/>',res)
   with self.assertRaisesRegex(ValueError,'Unsupported Toolbar textStyle'):
    self.run_layout('<Toolbar xmlns:android="http://schemas.android.com/apk/res/android" android:titleTextAppearance="@style/ToolbarBadStyle"/>',res)
 def test_wave8a_extended_fab_rule_242_and_corner_radius(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <bool name="efab_collapsed">false</bool>
    <style name="CollapsedEfab">
     <item name="extended">false</item>
     <item name="collapsedSize">56dp</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_default" android:text="Compose"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_collapsed" android:text="Compose" app:extended="false"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_bool_res" android:text="Compose" app:extended="@bool/efab_collapsed"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_motion_specs" android:text="New" app:extendMotionSpec="@animator/ext_spec" app:shrinkMotionSpec="@animator/shr_spec" app:extendStrategy="wrap_content" app:collapsedSize="56dp" app:elevation="6dp" app:ensureMinTouchTargetSize="true"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_motion_false" android:text="Compact" app:extendMotionSpec="false"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_explicit_wins" android:text="Open" app:extended="true" app:extendMotionSpec="false"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_styled" style="@style/CollapsedEfab" android:text="Styled"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_corner8" android:text="C8" app:cornerRadius="8dp"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_corner0" android:text="C0" app:cornerRadius="0dp"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_pill" android:text="Pill" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.Full"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['ef_default','ef_collapsed','ef_bool_res','ef_motion_specs','ef_motion_false','ef_explicit_wins','ef_styled','ef_corner8','ef_corner0','ef_pill']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:242, anchor:"1" }',slices['ef_default'])
   self.assertIn(',radius:16.0,',slices['ef_default'])
   self.assertIn('background:Some(Color(234,221,255,255))',slices['ef_default'])
   self.assertIn('RelativeRule { kind:242, anchor:"0" }',slices['ef_collapsed'])
   self.assertIn(',radius:16.0,',slices['ef_collapsed'])
   self.assertIn('RelativeRule { kind:242, anchor:"0" }',slices['ef_bool_res'])
   self.assertIn('RelativeRule { kind:242, anchor:"1" }',slices['ef_motion_specs'])
   self.assertIn('RelativeRule { kind:193, anchor:"6.0" }',slices['ef_motion_specs'])
   self.assertIn('RelativeRule { kind:242, anchor:"0" }',slices['ef_motion_false'])
   self.assertIn('RelativeRule { kind:242, anchor:"1" }',slices['ef_explicit_wins'])
   self.assertIn('RelativeRule { kind:242, anchor:"0" }',slices['ef_styled'])
   self.assertIn(',radius:8.0,',slices['ef_corner8'])
   self.assertIn(',radius:0.0,',slices['ef_corner0'])
   self.assertIn(',radius:-1.0,',slices['ef_pill'])
   for bad_xml,err in [
    ('<com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton xmlns:app="http://schemas.android.com/apk/res-auto" app:extended="maybe"/>','Invalid extended'),
    ('<com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton xmlns:app="http://schemas.android.com/apk/res-auto" app:extendStrategy="stretch"/>','Unsupported extendStrategy'),
    ('<com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton xmlns:app="http://schemas.android.com/apk/res-auto" app:collapsedSize="-4dp"/>','Expected dp/sp'),
   ]:
    with self.subTest(err=err):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave8a_bottom_app_bar_rule_243_and_menu_inflation(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'menu').mkdir()
   (res/'values'/'strings.xml').write_text('''<resources>
    <string name="act_search">Search</string>
    <string name="act_archive">Archive</string>
    <style name="EndBab">
     <item name="fabAlignmentMode">end</item>
     <item name="menu">@menu/bab_actions</item>
    </style>
   </resources>''')
   (res/'menu'/'bab_actions.xml').write_text('''<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <item android:id="@+id/act_more" android:title="More" android:icon="@drawable/ic_more_vert_24" android:orderInCategory="30" app:showAsAction="ifRoom"/>
    <item android:id="@+id/act_search" android:title="@string/act_search" android:icon="@drawable/ic_search_24" android:orderInCategory="10" app:showAsAction="always"/>
    <item android:id="@+id/act_archive" android:title="@string/act_archive" android:icon="@drawable/ic_outline_archive_24dp" android:orderInCategory="20" app:badgeText="4"/>
    <item android:id="@+id/act_hidden" android:title="Hidden" android:visible="false"/>
   </menu>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_def"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_end" style="?attr/bottomAppBarStyle" app:fabAlignmentMode="end" app:fabAnchorMode="embed" app:fabAlignmentModeEndMargin="16dp" app:removeEmbeddedFabElevation="true" app:addElevationShadow="false" app:menu="@menu/bab_actions"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_missing" app:menu="@menu/does_not_exist"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_null" app:menu="@null"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_styled" style="@style/EndBab"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['bab_def','bab_end','bab_missing','bab_null','bab_styled']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:243, anchor:"center" }',slices['bab_def'])
   self.assertNotIn('kind:238',slices['bab_def'])
   self.assertIn('RelativeRule { kind:243, anchor:"end" }',slices['bab_end'])
   items=[
    'RelativeRule { kind:238, anchor:"act_search|Search|search" }',
    'RelativeRule { kind:238, anchor:"act_archive|Archive|archive" }',
    'RelativeRule { kind:238, anchor:"act_more|More|more_vert" }',
   ]
   for tok in items:self.assertIn(tok,slices['bab_end'])
   self.assertEqual([slices['bab_end'].index(tok) for tok in items],sorted(slices['bab_end'].index(tok) for tok in items))
   self.assertIn('RelativeRule { kind:244, anchor:"act_archive:4" }',slices['bab_end'])
   self.assertNotIn('act_hidden',slices['bab_end'])
   self.assertIn('RelativeRule { kind:243, anchor:"center" }',slices['bab_missing'])
   self.assertNotIn('kind:238',slices['bab_missing'])
   self.assertIn('RelativeRule { kind:243, anchor:"center" }',slices['bab_null'])
   self.assertNotIn('kind:238',slices['bab_null'])
   self.assertIn('RelativeRule { kind:243, anchor:"end" }',slices['bab_styled'])
   self.assertIn('RelativeRule { kind:238, anchor:"act_search|Search|search" }',slices['bab_styled'])
   for bad_xml,err in [
    ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:fabAlignmentMode="start"/>','Unsupported fabAlignmentMode'),
    ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:menu="invalid_menu"/>','Invalid menu reference'),
    ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:removeEmbeddedFabElevation="yes"/>','Invalid removeEmbeddedFabElevation'),
   ]:
    with self.subTest(err=err):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave8a_nav_badge_rule_244(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'menu').mkdir()
   (res/'values'/'strings.xml').write_text('<resources><string name="unread">99+</string></resources>')
   (res/'menu'/'badged_nav.xml').write_text('''<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <item android:id="@+id/nav_home" android:title="Home" android:icon="@drawable/ic_home_24" app:badgeText=""/>
    <item android:id="@+id/nav_mail" android:title="Mail" android:icon="@drawable/ic_mail_24" app:badgeText="@string/unread"/>
    <item android:id="@+id/nav_chat" android:title="Chat" android:icon="@drawable/ic_chat_24" app:badgeText="3"/>
    <item android:id="@+id/nav_clear" android:title="Clear" app:badgeText="@null"/>
    <item android:id="@+id/nav_hidden" android:title="Hidden" android:visible="false" app:badgeText="9"/>
    <item android:title="NoId" app:badgeText="1"/>
   </menu>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.bottomnavigation.BottomNavigationView android:id="@+id/bn" app:menu="@menu/badged_nav"/>
    <com.google.android.material.navigationrail.NavigationRailView android:id="@+id/nr" app:menu="@menu/badged_nav"/>
    <com.google.android.material.navigation.NavigationView android:id="@+id/nv" app:menu="@menu/badged_nav"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   for start,end in [('bn','nr'),('nr','nv'),('nv',None)]:
    seg=out[out.index(f'id:"{start}"'):(out.index(f'id:"{end}"') if end else len(out))]
    for tok in (
     'RelativeRule { kind:244, anchor:"nav_home:" }',
     'RelativeRule { kind:244, anchor:"nav_mail:99+" }',
     'RelativeRule { kind:244, anchor:"nav_chat:3" }',
    ):
     self.assertIn(tok,seg)
    self.assertNotIn('nav_clear:',seg)
    self.assertNotIn('nav_hidden:',seg)
    self.assertNotIn('anchor:":1"',seg)
 def test_wave8a_swipe_refresh_and_drawer_layout_rules_246_247(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'menu').mkdir()
   (res/'menu'/'drawer_items.xml').write_text('<menu xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/d_inbox" android:title="Inbox" android:icon="@drawable/ic_inbox_24"/></menu>')
   xml='''<androidx.drawerlayout.widget.DrawerLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" xmlns:tools="http://schemas.android.com/tools" android:id="@+id/drawer_root" android:fitsSystemWindows="true" tools:openDrawer="start">
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/swipe_box" app:swipeRefreshLayoutProgressSpinnerBackgroundColor="#FAFAFA">
     <TextView android:id="@+id/main_body" android:text="Content"/>
    </androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <com.google.android.material.navigation.NavigationView android:id="@+id/start_drawer" android:layout_gravity="start" app:menu="@menu/drawer_items"/>
    <LinearLayout android:id="@+id/end_drawer" android:layout_gravity="end"><TextView android:text="Details"/></LinearLayout>
   </androidx.drawerlayout.widget.DrawerLayout>'''
   out=self.run_layout(xml,res)
   drawer_hdr=out[:out.index('id:"swipe_box"')]
   swipe_hdr=out[out.index('id:"swipe_box"'):out.index('id:"main_body"')]
   start_dr=out[out.index('id:"start_drawer"'):out.index('id:"end_drawer"')]
   end_dr=out[out.index('id:"end_drawer"'):]
   self.assertIn('kind:Kind::Frame,id:"drawer_root"',drawer_hdr)
   self.assertIn('RelativeRule { kind:247, anchor:"" }',drawer_hdr)
   self.assertIn('kind:Kind::Frame,id:"swipe_box"',out)
   self.assertIn('RelativeRule { kind:246, anchor:"" }',swipe_hdr)
   self.assertIn('layout_gravity:[0,0]',start_dr)
   self.assertIn('layout_gravity_explicit:true',start_dr)
   self.assertIn('RelativeRule { kind:237, anchor:"drawer|labeled" }',start_dr)
   self.assertIn('RelativeRule { kind:238, anchor:"d_inbox|Inbox|inbox" }',start_dr)
   self.assertIn('layout_gravity:[2,0]',end_dr)
   self.assertIn('layout_gravity_explicit:true',end_dr)
   # merge-parent and custom_view_map also emit rules 246 / 247
   merge_p=Path(d)/'merge.xml'; merge_p.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android"><View/></merge>')
   self.assertIn('RelativeRule { kind:246, anchor:"" }',mod.compile_layout(merge_p,merge_parent='androidx.swiperefreshlayout.widget.SwipeRefreshLayout'))
   self.assertIn('RelativeRule { kind:247, anchor:"" }',mod.compile_layout(merge_p,merge_parent='androidx.drawerlayout.widget.DrawerLayout'))
   custom_p=Path(d)/'custom.xml'; custom_p.write_text('<com.example.MyDrawer xmlns:android="http://schemas.android.com/apk/res/android"><com.example.MyRefresh><View/></com.example.MyRefresh></com.example.MyDrawer>')
   custom_out=mod.compile_layout(custom_p,custom_view_map={'com.example.MyDrawer':'androidx.drawerlayout.widget.DrawerLayout','com.example.MyRefresh':'androidx.swiperefreshlayout.widget.SwipeRefreshLayout'})
   self.assertIn('RelativeRule { kind:247, anchor:"" }',custom_out)
   self.assertIn('RelativeRule { kind:246, anchor:"" }',custom_out)
 def test_wave8a_collapsing_toolbar_and_app_bar_attributes(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'strings.xml').write_text('''<resources>
    <string name="profile_title">Profile</string>
    <integer name="scrim_ms">250</integer>
    <style name="StyledCollapsing" parent="Widget.Material3.CollapsingToolbar">
     <item name="contentScrim">?attr/colorPrimary</item>
     <item name="expandedTitleMarginStart">20dp</item>
    </style>
   </resources>''')
   xml='''<androidx.coordinatorlayout.widget.CoordinatorLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/appbar" app:liftOnScroll="true" app:expanded="true" app:liftOnScrollColor="?attr/colorSurfaceContainer" app:statusBarForeground="?attr/colorSurface" app:elevation="4dp">
     <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl" style="?attr/collapsingToolbarLayoutLargeStyle"
      app:title="@string/profile_title" app:titleEnabled="true"
      app:expandedTitleMargin="16dp" app:expandedTitleMarginStart="24dp" app:expandedTitleMarginEnd="24dp" app:expandedTitleMarginTop="8dp" app:expandedTitleMarginBottom="16dp"
      app:contentScrim="?attr/colorPrimary" app:statusBarScrim="@drawable/scrim_grad"
      app:scrimAnimationDuration="@integer/scrim_ms" app:scrimVisibleHeightTrigger="96dp"
      app:titleCollapseMode="scale" app:collapsedTitleGravity="start|center_vertical" app:expandedTitleGravity="start|bottom"
      app:collapsedTitleTextAppearance="@style/TextAppearance.Material3.TitleLarge" app:expandedTitleTextAppearance="@style/TextAppearance.Material3.HeadlineMedium"
      app:toolbarId="@id/tb" app:titlePositionInterpolator="@android:interpolator/linear"
      app:layout_scrollFlags="scroll|exitUntilCollapsed|snap" app:layout_scrollEffect="compress">
      <ImageView android:id="@+id/hero" app:layout_collapseMode="parallax" app:layout_collapseParallaxMultiplier="0.7"/>
      <com.google.android.material.appbar.MaterialToolbar android:id="@+id/tb" app:layout_collapseMode="pin"/>
     </com.google.android.material.appbar.CollapsingToolbarLayout>
     <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_styled" style="@style/StyledCollapsing" app:layout_scrollFlags="noScroll"/>
    </com.google.android.material.appbar.AppBarLayout>
    <androidx.core.widget.NestedScrollView android:id="@+id/body" app:layout_behavior="@string/appbar_scrolling_view_behavior" app:behavior_overlapTop="32dp">
     <TextView android:text="Feed"/>
    </androidx.core.widget.NestedScrollView>
   </androidx.coordinatorlayout.widget.CoordinatorLayout>'''
   out=self.run_layout(xml,res)
   for tok in ('kind:Kind::Linear,id:"appbar"','kind:Kind::Frame,id:"ctl"','kind:Kind::Image,id:"hero"','kind:Kind::Toolbar,id:"tb"','kind:Kind::Frame,id:"ctl_styled"','RelativeRule { kind:226, anchor:"" }'):
    self.assertIn(tok,out)
   for bad_xml,err in [
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:scrimAnimationDuration="-10"/>','Invalid scrimAnimationDuration'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:expandedTitleMarginStart="-4dp"/>','Expected dp/sp'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:titleCollapseMode="zoom"/>','Unsupported titleCollapseMode'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:layout_scrollFlags="scroll|spin"/>','Unsupported layout_scrollFlags'),
    ('<ImageView xmlns:app="http://schemas.android.com/apk/res-auto" app:layout_collapseMode="float"/>','Unsupported layout_collapseMode'),
    ('<ImageView xmlns:app="http://schemas.android.com/apk/res-auto" app:layout_collapseParallaxMultiplier="1.5"/>','Invalid layout_collapseParallaxMultiplier'),
    ('<com.google.android.material.appbar.AppBarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:liftOnScroll="maybe"/>','Invalid liftOnScroll'),
   ]:
    with self.subTest(err=err):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave10a_collapsing_toolbar_rule_245(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'color').mkdir();(res/'drawable').mkdir()
   (res/'values'/'values.xml').write_text('''<resources>
    <string name="ctl_pipe">Inbox | Primary</string>
    <dimen name="ctl_custom_h">196dp</dimen>
    <style name="CustomMediumCtl" parent="Widget.Material3.CollapsingToolbar.Medium">
     <item name="title">Medium Styled</item>
     <item name="contentScrim">#80112233</item>
    </style>
   </resources>''')
   (res/'color'/'scrim_sel.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_pressed="true" android:color="#ff0000"/><item android:color="#123456"/></selector>')
   (res/'drawable'/'solid_scrim.xml').write_text('<color xmlns:android="http://schemas.android.com/apk/res/android" android:color="#445566"/>')
   (res/'drawable'/'grad_scrim.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:startColor="#000000" android:endColor="#ffffff"/></shape>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_def"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_large" style="?attr/collapsingToolbarLayoutLargeStyle" android:layout_height="match_parent" app:title="@string/ctl_pipe" app:contentScrim="?attr/colorPrimary"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_med" style="?attr/collapsingToolbarLayoutMediumStyle" android:layout_height="wrap_content" app:title="Notes" app:contentScrim="@color/scrim_sel"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_fixed" style="?attr/collapsingToolbarLayoutMediumStyle" android:layout_height="@dimen/ctl_custom_h" app:title="Fixed" app:contentScrim="@drawable/solid_scrim"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_grad" android:layout_height="0dp" style="@style/CustomMediumCtl" app:contentScrim="@drawable/grad_scrim"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_styled_med" style="@style/CustomMediumCtl"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_nulls" app:title="@null" app:contentScrim="@null"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_missing_draw" app:title="Wave 10A" app:contentScrim="@drawable/nonexistent_grad"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['ctl_def','ctl_large','ctl_med','ctl_fixed','ctl_grad','ctl_styled_med','ctl_nulls','ctl_missing_draw']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:245, anchor:"|152|" }',slices['ctl_def'])
   self.assertIn('RelativeRule { kind:245, anchor:"Inbox / Primary|152|103,80,164,255" }',slices['ctl_large'])
   self.assertIn('RelativeRule { kind:245, anchor:"Notes|112|18,52,86,255" }',slices['ctl_med'])
   self.assertIn('RelativeRule { kind:245, anchor:"Fixed|196.0|68,85,102,255" }',slices['ctl_fixed'])
   self.assertIn('RelativeRule { kind:245, anchor:"Medium Styled|112|" }',slices['ctl_grad'])
   self.assertIn('RelativeRule { kind:245, anchor:"Medium Styled|112|17,34,51,128" }',slices['ctl_styled_med'])
   self.assertIn('RelativeRule { kind:245, anchor:"|152|" }',slices['ctl_nulls'])
   self.assertIn('RelativeRule { kind:245, anchor:"Wave 10A|152|" }',slices['ctl_missing_draw'])
   # merge_parent and custom_view_map also emit rule 245
   merge_p=Path(d)/'merge.xml'; merge_p.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android" android:layout_height="180dp"><View/></merge>')
   self.assertIn('RelativeRule { kind:245, anchor:"|180.0|" }',mod.compile_layout(merge_p,res,merge_parent='com.google.android.material.appbar.CollapsingToolbarLayout'))
   custom_p=Path(d)/'custom.xml'; custom_p.write_text('<com.example.MyCtl xmlns:app="http://schemas.android.com/apk/res-auto" app:title="Custom"/>')
   self.assertIn('RelativeRule { kind:245, anchor:"Custom|152|" }',mod.compile_layout(custom_p,res,custom_view_map={'com.example.MyCtl':'com.google.android.material.appbar.CollapsingToolbarLayout'}))
   with self.assertRaisesRegex(ValueError,'Expected #RGB'):
    self.run_layout('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:contentScrim="not_a_color"/>',res)
 def test_wave10a_view_pager_rule_248(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('<resources><style name="VerticalPager"><item name="android:orientation">vertical</item></style></resources>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp2_def"><TextView android:text="P1"/><TextView android:text="P2"/></androidx.viewpager2.widget.ViewPager2>
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp2_h" android:orientation="horizontal"/>
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp2_v" android:orientation="vertical"/>
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp2_styled" style="@style/VerticalPager"/>
    <androidx.viewpager.widget.ViewPager android:id="@+id/vp1_def"><View/></androidx.viewpager.widget.ViewPager>
    <androidx.viewpager.widget.ViewPager android:id="@+id/vp1_v" android:orientation="vertical"/>
    <ViewFlipper android:id="@+id/plain_flipper"><View/></ViewFlipper>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['vp2_def','vp2_h','vp2_v','vp2_styled','vp1_def','vp1_v','plain_flipper']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:248, anchor:"horizontal" }',slices['vp2_def'])
   self.assertIn('horizontal:true',slices['vp2_def'])
   self.assertIn('RelativeRule { kind:248, anchor:"horizontal" }',slices['vp2_h'])
   self.assertIn('RelativeRule { kind:248, anchor:"vertical" }',slices['vp2_v'])
   self.assertIn('horizontal:false',slices['vp2_v'])
   self.assertIn('RelativeRule { kind:248, anchor:"vertical" }',slices['vp2_styled'])
   self.assertIn('RelativeRule { kind:248, anchor:"horizontal" }',slices['vp1_def'])
   self.assertIn('RelativeRule { kind:248, anchor:"vertical" }',slices['vp1_v'])
   self.assertNotIn('kind:248',slices['plain_flipper'])
   # merge_parent and custom_view_map also emit rule 248
   merge_p=Path(d)/'merge.xml'; merge_p.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="vertical"><View/></merge>')
   self.assertIn('RelativeRule { kind:248, anchor:"vertical" }',mod.compile_layout(merge_p,res,merge_parent='androidx.viewpager2.widget.ViewPager2'))
   custom_p=Path(d)/'custom.xml'; custom_p.write_text('<com.example.MyPager xmlns:android="http://schemas.android.com/apk/res/android"/>')
   self.assertIn('RelativeRule { kind:248, anchor:"horizontal" }',mod.compile_layout(custom_p,res,custom_view_map={'com.example.MyPager':'androidx.viewpager.widget.ViewPager'}))
   with self.assertRaisesRegex(ValueError,'Unsupported ViewPager orientation'):
    self.run_layout('<androidx.viewpager2.widget.ViewPager2 xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="diagonal"/>',res)
 def test_wave10a_range_slider_rule_250_and_array_resources(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'values-sw600dp').mkdir()
   (res/'values'/'arrays.xml').write_text('''<resources>
    <integer name="lo_val">18</integer>
    <integer name="hi_val">82</integer>
    <string name="inline_range">[15.5, 65.5]</string>
    <array name="float_range">
     <item>20.0</item>
     <item>70.0</item>
    </array>
    <integer-array name="int_range">
     <item>10</item>
     <item>90</item>
    </integer-array>
    <array name="ref_range">
     <item>@integer/lo_val</item>
     <item>@integer/hi_val</item>
    </array>
    <array name="suffix_range">
     <item>12.5f</item>
     <item>87.5F</item>
    </array>
    <array name="bad_range">
     <item>abc</item>
     <item>50</item>
    </array>
    <style name="StyledRange">
     <item name="values">@array/int_range</item>
    </style>
   </resources>''')
   (res/'values-sw600dp'/'arrays.xml').write_text('<resources><integer-array name="int_range"><item>30</item><item>60</item></integer-array></resources>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_def"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_custom_bounds" android:valueFrom="10" android:valueTo="50"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_neg_bounds" android:valueFrom="-100" android:valueTo="100"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_inline_csv" app:values="20,80"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_inline_bracket" app:values="@string/inline_range"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_clamp_swap" android:valueFrom="0" android:valueTo="100" app:values="120, -5"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_single" android:valueFrom="0" android:valueTo="100" app:values="30"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_arr_float" app:values="@array/float_range"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_arr_int" app:values="@array/int_range"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_arr_ref" app:values="@array/ref_range"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_arr_suffix" app:values="@array/suffix_range"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_arr_missing" android:valueFrom="10" android:valueTo="50" app:values="@array/does_not_exist"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_null" app:values="@null"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/rs_styled" style="@style/StyledRange"/>
    <com.google.android.material.slider.Slider android:id="@+id/single_slider" android:valueFrom="0" android:valueTo="100" android:value="40"/>
    <SeekBar android:id="@+id/plain_seek"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['rs_def','rs_custom_bounds','rs_neg_bounds','rs_inline_csv','rs_inline_bracket','rs_clamp_swap','rs_single','rs_arr_float','rs_arr_int','rs_arr_ref','rs_arr_suffix','rs_arr_missing','rs_null','rs_styled','single_slider','plain_seek']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:250, anchor:"25.0,75.0" }',slices['rs_def'])
   self.assertIn('RelativeRule { kind:250, anchor:"20.0,40.0" }',slices['rs_custom_bounds'])
   self.assertIn('RelativeRule { kind:250, anchor:"-50.0,50.0" }',slices['rs_neg_bounds'])
   self.assertIn('RelativeRule { kind:250, anchor:"20.0,80.0" }',slices['rs_inline_csv'])
   self.assertIn('RelativeRule { kind:250, anchor:"15.5,65.5" }',slices['rs_inline_bracket'])
   self.assertIn('RelativeRule { kind:250, anchor:"0.0,100.0" }',slices['rs_clamp_swap'])
   self.assertIn('RelativeRule { kind:250, anchor:"30.0,100.0" }',slices['rs_single'])
   self.assertIn('RelativeRule { kind:250, anchor:"20.0,70.0" }',slices['rs_arr_float'])
   self.assertIn('RelativeRule { kind:250, anchor:"10.0,90.0" }',slices['rs_arr_int'])
   self.assertIn('RelativeRule { kind:250, anchor:"18.0,82.0" }',slices['rs_arr_ref'])
   self.assertIn('RelativeRule { kind:250, anchor:"12.5,87.5" }',slices['rs_arr_suffix'])
   self.assertIn('RelativeRule { kind:250, anchor:"20.0,40.0" }',slices['rs_arr_missing'])
   self.assertIn('RelativeRule { kind:250, anchor:"25.0,75.0" }',slices['rs_null'])
   self.assertIn('RelativeRule { kind:250, anchor:"10.0,90.0" }',slices['rs_styled'])
   self.assertNotIn('kind:250',slices['single_slider'])
   self.assertNotIn('kind:250',slices['plain_seek'])
   # Qualified values-sw600dp override for <integer-array>
   lp=Path(d)/'sw.xml'; lp.write_text('<com.google.android.material.slider.RangeSlider xmlns:app="http://schemas.android.com/apk/res-auto" app:values="@array/int_range"/>')
   self.assertIn('RelativeRule { kind:250, anchor:"30.0,60.0" }',mod.compile_layout(lp,res,sw=600))
   # Without --resources, @array/... degrades gracefully to default range
   self.assertIn('RelativeRule { kind:250, anchor:"25.0,75.0" }',self.run_layout('<com.google.android.material.slider.RangeSlider xmlns:app="http://schemas.android.com/apk/res-auto" app:values="@array/float_range"/>'))
   # Strict error cases
   for bad_xml,err in [
    ('<com.google.android.material.slider.RangeSlider xmlns:app="http://schemas.android.com/apk/res-auto" app:values="abc,def"/>','Invalid RangeSlider values'),
    ('<com.google.android.material.slider.RangeSlider xmlns:app="http://schemas.android.com/apk/res-auto" app:values="[10, 20"/>','Invalid RangeSlider values'),
    ('<com.google.android.material.slider.RangeSlider xmlns:app="http://schemas.android.com/apk/res-auto" app:values="[]"/>','Invalid RangeSlider values'),
    ('<com.google.android.material.slider.RangeSlider xmlns:app="http://schemas.android.com/apk/res-auto" app:values="10,"/>','Invalid RangeSlider values'),
    ('<com.google.android.material.slider.RangeSlider xmlns:app="http://schemas.android.com/apk/res-auto" app:values="@array/Bad-Name"/>','Invalid RangeSlider values'),
    ('<com.google.android.material.slider.RangeSlider xmlns:app="http://schemas.android.com/apk/res-auto" app:values="@array/bad_range"/>','Invalid RangeSlider values'),
   ]:
    with self.subTest(err=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
   # Duplicate and invalid <array>/<integer-array> checks
   dup_res=Path(d)/'dup_res'; (dup_res/'values').mkdir(parents=True)
   (dup_res/'values'/'dup.xml').write_text('<resources><array name="x"><item>1</item></array><integer-array name="x"><item>2</item></integer-array></resources>')
   with self.assertRaisesRegex(ValueError,'Invalid or duplicate integer-array'):
    self.run_layout('<View/>',dup_res)
   bad_item_res=Path(d)/'bad_item_res'; (bad_item_res/'values').mkdir(parents=True)
   (bad_item_res/'values'/'bad.xml').write_text('<resources><array name="x"><bad>1</bad></array></resources>')
   with self.assertRaisesRegex(ValueError,'Unsupported array item'):
    self.run_layout('<View/>',bad_item_res)
 def test_wave13a_nine_patch_stretch_extraction_and_rules_251_252(self):
  def make_rgba_png(w,h,rgba_grid):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=bytearray()
   for y in range(h):
    rows.append(0)
    for x in range(w):rows.extend(rgba_grid(x,y))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(bytes(rows)))+chunk(b'IEND',b'')
  def make_9(inner_w,inner_h,top_span,left_span,bottom_span=None,right_span=None,fill=(20,40,60,255)):
   w,h=inner_w+2,inner_h+2
   def px(x,y):
    if y==0:
     if 1<=x<=w-2 and top_span is not None and top_span[0]<=x-1<=top_span[1]:return (0,0,0,255)
     return (0,0,0,0)
    if x==0:
     if 1<=y<=h-2 and left_span is not None and left_span[0]<=y-1<=left_span[1]:return (0,0,0,255)
     return (0,0,0,0)
    if y==h-1:
     if 1<=x<=w-2 and bottom_span is not None and bottom_span[0]<=x-1<=bottom_span[1]:return (0,0,0,255)
     return (0,0,0,0)
    if x==w-1:
     if 1<=y<=h-2 and right_span is not None and right_span[0]<=y-1<=right_span[1]:return (0,0,0,255)
     return (0,0,0,0)
    return fill
   return make_rgba_png(w,h,px)
  # Verify real AOSP ZoomControls .9.png stretch ranges match half-open stripped pixel coords
  zoom_down=Path('codes/libs/ui/libbhoo/assets/android-zoomcontrols/btn_zoom_down_normal.9.png')
  zw,zh,_,zpad,zsx,zsy=mod.decode_nine_patch_full(zoom_down)
  self.assertEqual((zw,zh),(79,48))
  self.assertEqual((zsx,zsy),((74,75),(0,1)))
  zoom_up=Path('codes/libs/ui/libbhoo/assets/android-zoomcontrols/btn_zoom_up_normal.9.png')
  _,_,_,_,usx,usy=mod.decode_nine_patch_full(zoom_up)
  self.assertEqual((usx,usy),((4,5),(7,37)))
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   p9=res/'drawable'/'card9.9.png'
   p9.write_bytes(make_9(16,12,(3,11),(2,9),(2,13),(1,10),fill=(10,20,30,255)))
   w,h,stripped,pad_px,sx,sy=mod.decode_nine_patch_full(p9)
   self.assertEqual((w,h),(16,12))
   self.assertEqual(pad_px,[2,1,2,1])
   self.assertEqual((sx,sy),((3,12),(2,10)))
   # Backwards-compatible decode_nine_patch still returns 4-tuple
   self.assertEqual(len(mod.decode_nine_patch(p9)),4)
   # Empty top/left marks default to full inner dimensions
   p9_empty=res/'drawable'/'empty9.9.png'
   p9_empty.write_bytes(make_9(8,6,None,None))
   _,_,_,_,esx,esy=mod.decode_nine_patch_full(p9_empty)
   self.assertEqual((esx,esy),((0,8),(0,6)))
   # Plain PNG, <bitmap>, <vector>, <inset> backgrounds + ImageView/ImageButton .9.png src
   (res/'drawable'/'photo.png').write_bytes(make_rgba_png(10,8,lambda x,y:(90,80,70,255)))
   (res/'drawable'/'bmp_9.xml').write_text('<bitmap xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/card9" android:tint="#445566"/>')
   (res/'drawable'/'vec_bg.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="20dp" android:height="20dp" android:viewportWidth="20" android:viewportHeight="20" android:autoMirrored="true"><path android:fillColor="#112233" android:pathData="M0,0h20v20h-20z"/></vector>')
   (res/'drawable'/'inset_9.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android" android:inset="4dp" android:drawable="@drawable/card9"/>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <View android:id="@+id/bg_png" android:background="@drawable/photo" android:backgroundTint="#ff0000"/>
    <View android:id="@+id/bg_9" android:background="@drawable/card9"/>
    <View android:id="@+id/bg_bmp9" android:background="@drawable/bmp_9"/>
    <View android:id="@+id/bg_vec" android:background="@drawable/vec_bg"/>
    <View android:id="@+id/bg_inset9" android:background="@drawable/inset_9"/>
    <ImageView android:id="@+id/iv_9" android:src="@drawable/card9"/>
    <ImageButton android:id="@+id/ib_9" android:src="@drawable/card9"/>
    <ImageView android:id="@+id/iv_png" android:src="@drawable/photo"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['bg_png','bg_9','bg_bmp9','bg_vec','bg_inset9','iv_9','ib_9','iv_png']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:251, anchor:"photo|" }',slices['bg_png'])
   self.assertIn('background:None,',slices['bg_png'])
   self.assertIn('RelativeRule { kind:194, anchor:"255,0,0,255" }',slices['bg_png'])
   self.assertIn('background_min_size:[10.0,8.0]',slices['bg_png'])
   self.assertIn('state.images.entry("photo".into())',out)
   self.assertIn('RelativeRule { kind:251, anchor:"card9|3,12;2,10" }',slices['bg_9'])
   self.assertIn('RelativeRule { kind:251, anchor:"bmp_9|3,12;2,10" }',slices['bg_bmp9'])
   self.assertIn('RelativeRule { kind:251, anchor:"vec_bg|" }',slices['bg_vec'])
   self.assertIn('RelativeRule { kind:255, anchor:"true" }',slices['bg_vec'])
   self.assertIn('RelativeRule { kind:251, anchor:"card9|3,12;2,10" }',slices['bg_inset9'])
   self.assertIn('RelativeRule { kind:252, anchor:"3,12;2,10" }',slices['iv_9'])
   self.assertIn('RelativeRule { kind:252, anchor:"3,12;2,10" }',slices['ib_9'])
   self.assertNotIn('kind:252',slices['iv_png'])
 def test_wave13a_vector_gradients_trim_path_tint_modes_size_and_automirrored(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <string name="bar_path">M0,5 L20,5</string>
    <color name="grad_start">#ff0000</color>
    <color name="grad_end">#0000ff</color>
   </resources>''')
   # 1. Linear gradient via <aapt:attr><gradient><item>
   lin_p=res/'drawable'/'vec_lin.xml'
   lin_p.write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:width="20dp" android:height="10dp" android:viewportWidth="20" android:viewportHeight="10">
    <path android:pathData="M0,0h20v10h-20z">
     <aapt:attr name="android:fillColor">
      <gradient android:type="linear" android:startX="0" android:startY="0" android:endX="20" android:endY="0">
       <item android:offset="0.0" android:color="#ffff0000"/>
       <item android:offset="1.0" android:color="#ff0000ff"/>
      </gradient>
     </aapt:attr>
    </path>
   </vector>''')
   w,h,px,_=mod.rasterize_vector(lin_p)
   self.assertEqual((w,h),(20,10))
   left_px=tuple(px[(5*w+1)*4:(5*w+1)*4+4])
   right_px=tuple(px[(5*w+18)*4:(5*w+18)*4+4])
   self.assertGreater(left_px[0],200);self.assertLess(left_px[2],60)
   self.assertGreater(right_px[2],200);self.assertLess(right_px[0],60)
   # 2. Radial gradient with startColor/centerColor/endColor
   rad_p=res/'drawable'/'vec_rad.xml'
   rad_p.write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:width="20dp" android:height="20dp" android:viewportWidth="20" android:viewportHeight="20">
    <path android:pathData="M0,0h20v20h-20z">
     <aapt:attr name="android:fillColor">
      <gradient android:type="radial" android:centerX="10" android:centerY="10" android:gradientRadius="10" android:startColor="#ffff0000" android:centerColor="#ff00ff00" android:endColor="#ff0000ff"/>
     </aapt:attr>
    </path>
   </vector>''')
   rw,rh,rpx,_=mod.rasterize_vector(rad_p)
   center_px=tuple(rpx[(10*rw+10)*4:(10*rw+10)*4+4])
   edge_px=tuple(rpx[(0*rw+0)*4:(0*rw+0)*4+4])
   self.assertGreater(center_px[0],200);self.assertGreater(edge_px[2],200)
   # 3. Sweep gradient with startColor/centerColor/endColor
   swp_p=res/'drawable'/'vec_sweep.xml'
   swp_p.write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24">
    <path android:pathData="M0,0h24v24h-24z">
     <aapt:attr name="android:fillColor">
      <gradient android:type="sweep" android:centerX="12" android:centerY="12" android:startColor="#ffff0000" android:centerColor="#ff00ff00" android:endColor="#ff0000ff"/>
     </aapt:attr>
    </path>
   </vector>''')
   sw,sh,spx,_=mod.rasterize_vector(swp_p)
   p3=tuple(spx[(12*sw+21)*4:(12*sw+21)*4+4])
   p9=tuple(spx[(12*sw+2)*4:(12*sw+2)*4+4])
   p12=tuple(spx[(2*sw+12)*4:(2*sw+12)*4+4])
   self.assertGreater(p3[0],180);self.assertEqual(p3[3],255)
   self.assertGreater(p9[1],180);self.assertEqual(p9[3],255)
   self.assertGreater(p12[2],100);self.assertEqual(p12[3],255)
   # 4. trimPathStart / trimPathEnd / trimPathOffset (including wrap-around, empty, curves & arcs)
   trim_p=res/'drawable'/'vec_trim.xml'
   trim_p.write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="20dp" android:height="10dp" android:viewportWidth="20" android:viewportHeight="10">
    <path android:strokeColor="#ffff0000" android:strokeWidth="4" android:pathData="M0,5 L20,5" android:trimPathStart="0.5" android:trimPathEnd="1.0"/>
   </vector>''')
   tw,th,tpx,_=mod.rasterize_vector(trim_p)
   self.assertEqual(tpx[(5*tw+2)*4+3],0)
   self.assertGreater(tpx[(5*tw+15)*4+3],200)
   trim_wrap=res/'drawable'/'vec_trim_wrap.xml'
   trim_wrap.write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="20dp" android:height="10dp" android:viewportWidth="20" android:viewportHeight="10">
    <path android:strokeColor="#ff00ff00" android:strokeWidth="4" android:pathData="M0,5 L20,5" android:trimPathStart="0.25" android:trimPathEnd="0.75" android:trimPathOffset="0.5"/>
   </vector>''')
   ww,wh,wpx,_=mod.rasterize_vector(trim_wrap)
   self.assertGreater(wpx[(5*ww+2)*4+3],200)
   self.assertEqual(wpx[(5*ww+10)*4+3],0)
   self.assertGreater(wpx[(5*ww+18)*4+3],200)
   trim_empty=res/'drawable'/'vec_trim_empty.xml'
   trim_empty.write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="10dp" android:height="10dp" android:viewportWidth="10" android:viewportHeight="10">
    <path android:strokeColor="#ff0000" android:strokeWidth="2" android:pathData="M0,5 L10,5" android:trimPathStart="0.4" android:trimPathEnd="0.4"/>
   </vector>''')
   ew,eh,epx,_=mod.rasterize_vector(trim_empty)
   self.assertEqual(sum(epx[3::4]),0)
   trim_arc=res/'drawable'/'vec_trim_arc.xml'
   trim_arc.write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24">
    <path android:strokeColor="#0000ff" android:strokeWidth="2" android:pathData="M4,12 A8,8 0 1020,12 C20,18 12,20 4,12" android:trimPathStart="0.1" android:trimPathEnd="0.6"/>
   </vector>''')
   aw,ah,apx,_=mod.rasterize_vector(trim_arc)
   self.assertGreater(sum(a>0 for a in apx[3::4]),20)
   # 5. Extended tintMode (all 6 modes) & 1024px raster size
   for tm in ('src_in','src_atop','multiply','screen','add','src_over'):
    tp=res/'drawable'/f'tm_{tm}.xml'
    tp.write_text(f'<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="8dp" android:height="8dp" android:viewportWidth="8" android:viewportHeight="8" android:tint="#ffffff" android:tintMode="{tm}"><path android:fillColor="#000000" android:pathData="M0,0h8v8h-8z"/></vector>')
    self.assertEqual(mod.vector_tint_mode(tp),tm)
    mod.rasterize_vector(tp)
   big_p=res/'drawable'/'big1024.xml'
   big_p.write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="1024dp" android:height="16dp" android:viewportWidth="1024" android:viewportHeight="16"><path android:fillColor="#112233" android:pathData="M0,0h1024v16h-1024z"/></vector>')
   bw,bh,_,_=mod.rasterize_vector(big_p)
   self.assertEqual((bw,bh),(1024,16))
   too_big=res/'drawable'/'big1025.xml'
   too_big.write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="1025dp" android:height="16dp" android:viewportWidth="1025" android:viewportHeight="16"><path android:fillColor="#112233" android:pathData="M0,0h1025v16h-1025z"/></vector>')
   with self.assertRaisesRegex(ValueError,'Vector dimensions out of range'):mod.rasterize_vector(too_big)
   # Strict vector rejections
   for bad_vec,err in [
    ('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="8dp" android:height="8dp" android:viewportWidth="8" android:viewportHeight="8" android:autoMirrored="maybe"><path android:fillColor="#000" android:pathData="M0,0h8v8z"/></vector>','Invalid vector autoMirrored'),
    ('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="8dp" android:height="8dp" android:viewportWidth="8" android:viewportHeight="8"><path android:fillColor="#000" android:pathData="M0,0h8v8z" android:trimPathStart="1.5"/></vector>','Invalid vector trimPath'),
    ('<vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:width="8dp" android:height="8dp" android:viewportWidth="8" android:viewportHeight="8"><path android:pathData="M0,0h8v8z"><aapt:attr name="android:pathData"><gradient android:startColor="#000" android:endColor="#fff"/></aapt:attr></path></vector>','Unsupported vector aapt:attr'),
    ('<vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:width="8dp" android:height="8dp" android:viewportWidth="8" android:viewportHeight="8"><path android:pathData="M0,0h8v8z"><aapt:attr name="android:fillColor"><gradient android:type="linear"/></aapt:attr></path></vector>','Empty vector gradient'),
   ]:
    bp=res/'drawable'/'bad_v.xml';bp.write_text(bad_vec)
    with self.subTest(err=err):
     with self.assertRaisesRegex(ValueError,err):mod.rasterize_vector(bp)
   # 6. @string/ pathData + @color/ in <aapt:attr><gradient> + autoMirrored=true -> RULE_AUTO_MIRRORED (255)
   (res/'drawable'/'vec_str_grad.xml').write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:width="20dp" android:height="10dp" android:viewportWidth="20" android:viewportHeight="10" android:autoMirrored="true" android:tint="#336699" android:tintMode="multiply">
    <path android:strokeWidth="4" android:pathData="@string/bar_path" android:trimPathStart="0.2" android:trimPathEnd="0.9">
     <aapt:attr name="android:strokeColor">
      <gradient android:type="linear" android:startX="0" android:startY="5" android:endX="20" android:endY="5" android:startColor="@color/grad_start" android:endColor="@color/grad_end"/>
     </aapt:attr>
    </path>
   </vector>''')
   (res/'drawable'/'vec_no_mirror.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="12dp" android:height="12dp" android:viewportWidth="12" android:viewportHeight="12" android:autoMirrored="false"><path android:fillColor="#000000" android:pathData="M0,0h12v12h-12z"/></vector>')
   layout_out=self.run_layout('''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <ImageView android:id="@+id/iv_am" android:src="@drawable/vec_str_grad"/>
    <ImageView android:id="@+id/iv_plain" android:src="@drawable/vec_no_mirror"/>
   </LinearLayout>''',res)
   s_am=layout_out[layout_out.index('id:"iv_am"'):layout_out.index('id:"iv_plain"')]
   s_pl=layout_out[layout_out.index('id:"iv_plain"'):]
   self.assertIn('RelativeRule { kind:255, anchor:"true" }',s_am)
   self.assertIn('image_tint_mode:3',s_am)
   self.assertNotIn('kind:255',s_pl)
   self.assertIn('state.images.entry("vec_str_grad".into())',layout_out)
 def test_wave13a_stateful_selector_images_rules_253_254(self):
  def make_rgba_png(w,h,rgba_grid):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=bytearray()
   for y in range(h):
    rows.append(0)
    for x in range(w):rows.extend(rgba_grid(x,y))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(bytes(rows)))+chunk(b'IEND',b'')
  def make_9(inner_w,inner_h,top_span,left_span,fill):
   w,h=inner_w+2,inner_h+2
   def px(x,y):
    if y==0:return (0,0,0,255) if (1<=x<=w-2 and top_span[0]<=x-1<=top_span[1]) else (0,0,0,0)
    if x==0:return (0,0,0,255) if (1<=y<=h-2 and left_span[0]<=y-1<=left_span[1]) else (0,0,0,0)
    if y==h-1 or x==w-1:return (0,0,0,0)
    return fill
   return make_rgba_png(w,h,px)
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'ic_normal.png').write_bytes(make_rgba_png(8,8,lambda x,y:(10,20,30,255)))
   (res/'drawable'/'ic_pressed.9.png').write_bytes(make_9(10,10,(2,7),(3,6),(200,10,10,255)))
   (res/'drawable'/'ic_selected.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="12dp" android:height="12dp" android:viewportWidth="12" android:viewportHeight="12" android:autoMirrored="true"><path android:fillColor="#00ff00" android:pathData="M0,0h12v12h-12z"/></vector>')
   (res/'drawable'/'ic_disabled_bmp.xml').write_text('<bitmap xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/ic_normal" android:tint="#888888"/>')
   (res/'drawable'/'src_sel.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:state_pressed="true" android:drawable="@drawable/ic_pressed"/>
    <item android:state_selected="true" android:drawable="@drawable/ic_selected"/>
    <item android:state_enabled="false" android:drawable="@drawable/ic_disabled_bmp"/>
    <item android:state_checked="true" android:drawable="@null"/>
    <item android:drawable="@drawable/ic_normal"/>
   </selector>''')
   (res/'drawable'/'bg_sel.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:state_pressed="true" android:drawable="@drawable/ic_pressed"/>
    <item android:state_focused="true" android:drawable="@drawable/ic_selected"/>
    <item android:state_enabled="false" android:drawable="@null"/>
    <item android:drawable="@drawable/ic_normal"/>
   </selector>''')
   (res/'drawable'/'bg_inline_sel.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:state_pressed="true"><bitmap android:src="@drawable/ic_pressed"/></item>
    <item android:drawable="@drawable/ic_normal"/>
   </selector>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <ImageView android:id="@+id/iv_sel" android:src="@drawable/src_sel"/>
    <ImageButton android:id="@+id/ib_sel" app:srcCompat="@drawable/src_sel"/>
    <Button android:id="@+id/btn_bg_sel" android:background="@drawable/bg_sel"/>
    <View android:id="@+id/v_inline_sel" android:background="@drawable/bg_inline_sel"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   s_iv=out[out.index('id:"iv_sel"'):out.index('id:"ib_sel"')]
   s_ib=out[out.index('id:"ib_sel"'):out.index('id:"btn_bg_sel"')]
   s_bg=out[out.index('id:"btn_bg_sel"'):out.index('id:"v_inline_sel"')]
   s_in=out[out.index('id:"v_inline_sel"'):]
   expected_253=[
    'RelativeRule { kind:253, anchor:"2,0|ic_pressed|2,8;3,7" }',
    'RelativeRule { kind:253, anchor:"8,0|ic_selected|" }',
    'RelativeRule { kind:253, anchor:"0,1|ic_disabled_bmp|" }',
    'RelativeRule { kind:253, anchor:"64,0||" }',
    'RelativeRule { kind:253, anchor:"0,0|ic_normal|" }',
   ]
   for seg in (s_iv,s_ib):
    for tok in expected_253:self.assertIn(tok,seg)
    self.assertEqual([seg.index(t) for t in expected_253],sorted(seg.index(t) for t in expected_253))
    self.assertIn('RelativeRule { kind:255, anchor:"true" }',seg)
   expected_254=[
    'RelativeRule { kind:254, anchor:"2,0|ic_pressed|2,8;3,7" }',
    'RelativeRule { kind:254, anchor:"4,0|ic_selected|" }',
    'RelativeRule { kind:254, anchor:"0,1||" }',
    'RelativeRule { kind:254, anchor:"0,0|ic_normal|" }',
   ]
   for tok in expected_254:self.assertIn(tok,s_bg)
   self.assertEqual([s_bg.index(t) for t in expected_254],sorted(s_bg.index(t) for t in expected_254))
   self.assertNotIn('kind:251',s_bg)
   self.assertNotIn('kind:96',s_bg)
   self.assertIn('RelativeRule { kind:255, anchor:"true" }',s_bg)
   self.assertIn('RelativeRule { kind:254, anchor:"2,0|bg_inline_sel__s0|2,8;3,7" }',s_in)
   self.assertIn('RelativeRule { kind:254, anchor:"0,0|ic_normal|" }',s_in)
   for key in ('ic_normal','ic_pressed','ic_selected','ic_disabled_bmp','src_sel','bg_inline_sel__s0'):
    self.assertIn(f'state.images.entry("{key}".into())',out)
 def test_wave13a_transforms_text_style_ext_and_tooltip_rules_256_257_258(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <string name="tip_save">Save changes</string>
    <dimen name="shift_x">-12dp</dimen>
    <style name="MonoTipStyle">
     <item name="android:typeface">monospace</item>
     <item name="android:textStyle">italic</item>
     <item name="android:tooltipText">@string/tip_save</item>
     <item name="android:rotation">-30</item>
    </style>
    <style name="SerifAppearance">
     <item name="android:typeface">serif</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <View android:id="@+id/t_rot" android:rotation="45"/>
    <View android:id="@+id/t_all" android:translationX="@dimen/shift_x" android:translationY="8dp" android:scaleX="1.25" android:scaleY="0.75" android:rotation="90" android:transformPivotX="0dp" android:transformPivotY="50%"/>
    <View android:id="@+id/t_none"/>
    <TextView android:id="@+id/tx_it" android:textStyle="italic"/>
    <TextView android:id="@+id/tx_bi_serif" android:textStyle="bold|italic" android:fontFamily="serif"/>
    <TextView android:id="@+id/tx_mono" app:fontFamily="monospace"/>
    <TextView android:id="@+id/tx_tf_serif" android:typeface="serif"/>
    <TextView android:id="@+id/tx_tf_override" android:typeface="serif" android:fontFamily="sans-serif"/>
    <TextView android:id="@+id/tx_plain" android:textStyle="bold" android:fontFamily="sans-serif-medium"/>
    <TextView android:id="@+id/tx_styled" style="@style/MonoTipStyle"/>
    <TextView android:id="@+id/tx_app_serif" android:textAppearance="@style/SerifAppearance"/>
    <Button android:id="@+id/btn_tip_a" android:tooltipText="Primary action"/>
    <ImageButton android:id="@+id/btn_tip_app" app:tooltipText="@string/tip_save"/>
    <View android:id="@+id/btn_tip_null" android:tooltipText="@null"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['t_rot','t_all','t_none','tx_it','tx_bi_serif','tx_mono','tx_tf_serif','tx_tf_override','tx_plain','tx_styled','tx_app_serif','btn_tip_a','btn_tip_app','btn_tip_null']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:256, anchor:"0,0,1,1,45.0,center,center" }',slices['t_rot'])
   self.assertIn('RelativeRule { kind:256, anchor:"-12.0,8.0,1.25,0.75,90.0,0.0,50.0%" }',slices['t_all'])
   self.assertNotIn('kind:256',slices['t_none'])
   self.assertIn('RelativeRule { kind:257, anchor:"1|sans" }',slices['tx_it'])
   self.assertIn('RelativeRule { kind:257, anchor:"1|serif" }',slices['tx_bi_serif'])
   self.assertIn('text_bold:true',slices['tx_bi_serif'])
   self.assertIn('RelativeRule { kind:257, anchor:"0|mono" }',slices['tx_mono'])
   self.assertIn('RelativeRule { kind:257, anchor:"0|serif" }',slices['tx_tf_serif'])
   self.assertNotIn('kind:257',slices['tx_tf_override'])
   self.assertNotIn('kind:257',slices['tx_plain'])
   self.assertIn('RelativeRule { kind:257, anchor:"1|mono" }',slices['tx_styled'])
   self.assertIn('RelativeRule { kind:258, anchor:"Save changes" }',slices['tx_styled'])
   self.assertIn('RelativeRule { kind:256, anchor:"0,0,1,1,-30.0,center,center" }',slices['tx_styled'])
   self.assertIn('RelativeRule { kind:257, anchor:"0|serif" }',slices['tx_app_serif'])
   self.assertIn('RelativeRule { kind:258, anchor:"Primary action" }',slices['btn_tip_a'])
   self.assertIn('RelativeRule { kind:258, anchor:"Save changes" }',slices['btn_tip_app'])
   self.assertNotIn('kind:258',slices['btn_tip_null'])
   for bad_xml,err in [
    ('<View xmlns:android="http://schemas.android.com/apk/res/android" android:rotation="spin"/>','Invalid rotation'),
    ('<View xmlns:android="http://schemas.android.com/apk/res/android" android:scaleX="nan"/>','Invalid scaleX'),
    ('<View xmlns:android="http://schemas.android.com/apk/res/android" android:transformPivotX="bogus"/>','Invalid transformPivotX'),
    ('<View xmlns:android="http://schemas.android.com/apk/res/android" android:transformPivotY="bad%"/>','Invalid transformPivotY'),
    ('<View xmlns:android="http://schemas.android.com/apk/res/android" android:translationX="notdp"/>','Expected margin'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave14c_bottom_sheet_drag_handle_and_side_sheet_rules_259_260_261(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'layout').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <string name="bs_beh">com.google.android.material.bottomsheet.BottomSheetBehavior</string>
    <string name="ss_beh">com.google.android.material.sidesheet.SideSheetBehavior</string>
    <dimen name="peek_h">128dp</dimen>
    <bool name="is_hideable">true</bool>
    <style name="CustomBottomSheet" parent="Widget.Material3.BottomSheet">
     <item name="behavior_peekHeight">96dp</item>
     <item name="behavior_hideable">true</item>
     <item name="behavior_fitToContents">false</item>
     <item name="behavior_halfExpandedRatio">0.6</item>
     <item name="behavior_skipCollapsed">true</item>
    </style>
    <style name="CustomCoplanarSideSheet" parent="Widget.Material3.SideSheet.Coplanar"/>
   </resources>''')
   (res/'layout'/'inc_sheet.xml').write_text('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"><TextView android:text="Included"/></LinearLayout>')
   xml='''<androidx.coordinatorlayout.widget.CoordinatorLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <FrameLayout android:id="@+id/main_pane"/>
    <LinearLayout android:id="@+id/bs_default" app:layout_behavior="@string/bottom_sheet_behavior">
     <com.google.android.material.bottomsheet.BottomSheetDragHandleView android:id="@+id/drag_h"/>
     <com.google.android.material.bottomsheet.BottomSheetDragHandleView android:id="@+id/drag_h_tinted" style="?attr/bottomSheetDragHandleStyle" app:tint="#445566"/>
    </LinearLayout>
    <LinearLayout android:id="@+id/bs_custom" app:layout_behavior="@string/bs_beh" app:behavior_peekHeight="@dimen/peek_h" app:behavior_hideable="@bool/is_hideable" app:behavior_fitToContents="false" app:behavior_halfExpandedRatio="0.65" app:behavior_skipCollapsed="true"/>
    <LinearLayout android:id="@+id/bs_auto" app:layout_behavior="com.google.android.material.bottomsheet.BottomSheetBehavior" app:behavior_peekHeight="auto" app:behavior_hideable="false"/>
    <LinearLayout android:id="@+id/bs_styled" style="@style/CustomBottomSheet" app:layout_behavior="@string/bottom_sheet_behavior"/>
    <include layout="@layout/inc_sheet" android:id="@+id/bs_inc" android:layout_width="match_parent" android:layout_height="wrap_content" app:layout_behavior="@string/bottom_sheet_behavior" app:behavior_peekHeight="72dp"/>
    <FrameLayout android:id="@+id/ss_def" app:layout_behavior="@string/side_sheet_behavior"/>
    <FrameLayout android:id="@+id/ss_start" android:layout_gravity="start" app:layout_behavior="@string/ss_beh"/>
    <FrameLayout android:id="@+id/ss_left" android:layout_gravity="left" app:layout_behavior="com.google.android.material.sidesheet.SideSheetBehavior"/>
    <FrameLayout android:id="@+id/ss_coplanar_attr" android:layout_gravity="end" app:layout_behavior="@string/side_sheet_behavior" app:coplanarSiblingViewId="@id/main_pane"/>
    <FrameLayout android:id="@+id/ss_coplanar_style" style="@style/CustomCoplanarSideSheet" app:layout_behavior="@string/side_sheet_behavior"/>
    <FrameLayout android:id="@+id/ss_modal" style="?attr/sideSheetModalStyle" app:layout_behavior="@string/side_sheet_behavior"/>
   </androidx.coordinatorlayout.widget.CoordinatorLayout>'''
   out=self.run_layout(xml,res)
   names=['bs_default','drag_h','drag_h_tinted','bs_custom','bs_auto','bs_styled','bs_inc','ss_def','ss_start','ss_left','ss_coplanar_attr','ss_coplanar_style','ss_modal']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:259, anchor:"-1|0|1|0.5|0" }',slices['bs_default'])
   self.assertIn('kind:Kind::View,id:"drag_h"',out)
   self.assertIn('RelativeRule { kind:260, anchor:"" }',slices['drag_h'])
   self.assertIn('RelativeRule { kind:260, anchor:"" }',slices['drag_h_tinted'])
   self.assertIn('RelativeRule { kind:194, anchor:"68,85,102,255" }',slices['drag_h_tinted'])
   self.assertIn('RelativeRule { kind:259, anchor:"128.0|1|0|0.65|1" }',slices['bs_custom'])
   self.assertIn('RelativeRule { kind:259, anchor:"-1|0|1|0.5|0" }',slices['bs_auto'])
   self.assertIn('RelativeRule { kind:259, anchor:"96.0|1|0|0.6|1" }',slices['bs_styled'])
   self.assertIn('RelativeRule { kind:259, anchor:"72.0|0|1|0.5|0" }',slices['bs_inc'])
   self.assertIn('RelativeRule { kind:261, anchor:"end|0" }',slices['ss_def'])
   self.assertIn('RelativeRule { kind:261, anchor:"start|0" }',slices['ss_start'])
   self.assertIn('RelativeRule { kind:261, anchor:"start|0" }',slices['ss_left'])
   self.assertIn('RelativeRule { kind:261, anchor:"end|1" }',slices['ss_coplanar_attr'])
   self.assertIn('RelativeRule { kind:261, anchor:"end|1" }',slices['ss_coplanar_style'])
   self.assertIn('RelativeRule { kind:261, anchor:"end|0" }',slices['ss_modal'])
   # Non-CoordinatorLayout parent does not emit 259 or 261
   non_c=self.run_layout('<FrameLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/bottom_sheet_behavior"/><View app:layout_behavior="@string/side_sheet_behavior"/></FrameLayout>')
   self.assertNotIn('kind:259',non_c);self.assertNotIn('kind:261',non_c)
   # merge_parent=CoordinatorLayout emits 259 and 261
   mp=Path(d)/'c_merge.xml'
   mp.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/bottom_sheet_behavior" app:behavior_peekHeight="64dp"/><View android:layout_gravity="start" app:layout_behavior="@string/side_sheet_behavior"/></merge>')
   mout=mod.compile_layout(mp,merge_parent='CoordinatorLayout')
   self.assertIn('RelativeRule { kind:259, anchor:"64.0|0|1|0.5|0" }',mout)
   self.assertIn('RelativeRule { kind:261, anchor:"start|0" }',mout)
   for bad_xml,err in [
    ('<CoordinatorLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/bottom_sheet_behavior" app:behavior_peekHeight="-10dp"/></CoordinatorLayout>','Invalid behavior_peekHeight'),
    ('<CoordinatorLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/bottom_sheet_behavior" app:behavior_hideable="maybe"/></CoordinatorLayout>','Invalid behavior_hideable'),
    ('<CoordinatorLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/bottom_sheet_behavior" app:behavior_fitToContents="maybe"/></CoordinatorLayout>','Invalid behavior_fitToContents'),
    ('<CoordinatorLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/bottom_sheet_behavior" app:behavior_halfExpandedRatio="1.0"/></CoordinatorLayout>','Invalid behavior_halfExpandedRatio'),
    ('<CoordinatorLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/bottom_sheet_behavior" app:behavior_halfExpandedRatio="0"/></CoordinatorLayout>','Invalid behavior_halfExpandedRatio'),
    ('<CoordinatorLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/bottom_sheet_behavior" app:behavior_skipCollapsed="nope"/></CoordinatorLayout>','Invalid behavior_skipCollapsed'),
    ('<CoordinatorLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_behavior="@string/side_sheet_behavior" app:coplanarSiblingViewId="not_an_id"/></CoordinatorLayout>','Invalid coplanarSiblingViewId'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave14c_loading_indicator_and_wavy_progress_rules_262_263(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('''<resources>
    <style name="CustomWavyLinear" parent="Widget.Material3Expressive.LinearProgressIndicator.Wavy">
     <item name="wavelength">36dp</item>
     <item name="waveSpeed">18dp</item>
    </style>
    <style name="CustomContainedLoader" parent="Widget.Material3.LoadingIndicator.Contained"/>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.loadingindicator.LoadingIndicator android:id="@+id/li_plain" app:indicatorColor="#6750A4" app:indicatorSize="38dp"/>
    <com.google.android.material.loadingindicator.LoadingIndicator android:id="@+id/li_contained_style" style="@style/Widget.Material3.LoadingIndicator.Contained"/>
    <com.google.android.material.loadingindicator.LoadingIndicator android:id="@+id/li_contained_theme" style="?attr/loadingIndicatorContainedStyle"/>
    <com.google.android.material.loadingindicator.LoadingIndicator android:id="@+id/li_contained_parent" style="@style/CustomContainedLoader"/>
    <com.google.android.material.loadingindicator.LoadingIndicator android:id="@+id/li_container_color" app:containerColor="#EADDFF"/>
    <com.google.android.material.loadingindicator.LoadingIndicator android:id="@+id/li_transparent" app:containerColor="@android:color/transparent"/>
    <com.google.android.material.progressindicator.LinearProgressIndicator android:id="@+id/lpi_plain"/>
    <com.google.android.material.progressindicator.CircularProgressIndicator android:id="@+id/cpi_plain"/>
    <com.google.android.material.progressindicator.LinearWavyProgressIndicator android:id="@+id/lwpi_tag"/>
    <com.google.android.material.progressindicator.CircularWavyProgressIndicator android:id="@+id/cwpi_tag"/>
    <com.google.android.material.progressindicator.LinearProgressIndicator android:id="@+id/lpi_wavy_style" style="@style/Widget.Material3Expressive.LinearProgressIndicator.Wavy"/>
    <com.google.android.material.progressindicator.LinearProgressIndicator android:id="@+id/lpi_wavy_theme" style="?attr/linearWavyProgressIndicatorStyle"/>
    <com.google.android.material.progressindicator.CircularProgressIndicator android:id="@+id/cpi_wavy_style" style="@style/Widget.Material3Expressive.CircularProgressIndicator.Wavy"/>
    <com.google.android.material.progressindicator.LinearProgressIndicator android:id="@+id/lpi_wavy_attrs" app:wavelength="32dp" app:waveAmplitude="4dp" app:waveSpeed="-24dp" app:trackStopIndicatorSize="6dp" app:indicatorTrackGapSize="8dp"/>
    <com.google.android.material.progressindicator.LinearProgressIndicator android:id="@+id/lpi_gap_only" app:trackStopIndicatorSize="4dp" app:indicatorTrackGapSize="6dp"/>
    <com.google.android.material.progressindicator.LinearProgressIndicator android:id="@+id/lpi_custom_wavy" style="@style/CustomWavyLinear"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['li_plain','li_contained_style','li_contained_theme','li_contained_parent','li_container_color','li_transparent','lpi_plain','cpi_plain','lwpi_tag','cwpi_tag','lpi_wavy_style','lpi_wavy_theme','cpi_wavy_style','lpi_wavy_attrs','lpi_gap_only','lpi_custom_wavy']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('kind:Kind::ProgressBar,id:"li_plain"',out)
   self.assertIn('indeterminate:true',slices['li_plain'])
   self.assertIn('RelativeRule { kind:262, anchor:"0" }',slices['li_plain'])
   self.assertIn('RelativeRule { kind:149, anchor:"0,0,103,80,164,255" }',slices['li_plain'])
   for cid in ('li_contained_style','li_contained_theme','li_contained_parent','li_container_color'):
    self.assertIn('RelativeRule { kind:262, anchor:"1" }',slices[cid])
   self.assertIn('RelativeRule { kind:262, anchor:"0" }',slices['li_transparent'])
   self.assertNotIn('kind:263',slices['lpi_plain'])
   self.assertNotIn('kind:263',slices['cpi_plain'])
   self.assertIn('horizontal:true',slices['lwpi_tag'])
   self.assertIn('RelativeRule { kind:263, anchor:"40.0|3.0|0.0|4.0|4.0" }',slices['lwpi_tag'])
   self.assertIn('horizontal:false',slices['cwpi_tag'])
   self.assertIn('indeterminate:true',slices['cwpi_tag'])
   self.assertIn('RelativeRule { kind:263, anchor:"15.0|1.6|0.0|0.0|4.0" }',slices['cwpi_tag'])
   self.assertIn('RelativeRule { kind:263, anchor:"40.0|3.0|0.0|4.0|4.0" }',slices['lpi_wavy_style'])
   self.assertIn('RelativeRule { kind:263, anchor:"40.0|3.0|0.0|4.0|4.0" }',slices['lpi_wavy_theme'])
   self.assertIn('RelativeRule { kind:263, anchor:"15.0|1.6|0.0|0.0|4.0" }',slices['cpi_wavy_style'])
   self.assertIn('RelativeRule { kind:263, anchor:"32.0|4.0|-24.0|6.0|8.0" }',slices['lpi_wavy_attrs'])
   self.assertIn('RelativeRule { kind:263, anchor:"0.0|0.0|0.0|4.0|6.0" }',slices['lpi_gap_only'])
   self.assertIn('RelativeRule { kind:263, anchor:"36.0|3.0|18.0|4.0|4.0" }',slices['lpi_custom_wavy'])
   for bad_xml,err in [
    ('<com.google.android.material.loadingindicator.LoadingIndicator xmlns:app="http://schemas.android.com/apk/res-auto" app:containerColor="notacolor"/>','Expected #RGB'),
    ('<com.google.android.material.loadingindicator.LoadingIndicator xmlns:app="http://schemas.android.com/apk/res-auto" app:showDelay="-5"/>','Invalid showDelay'),
    ('<com.google.android.material.progressindicator.LinearProgressIndicator xmlns:app="http://schemas.android.com/apk/res-auto" app:wavelength="-4dp"/>','Expected dp/sp'),
    ('<com.google.android.material.progressindicator.CircularProgressIndicator xmlns:app="http://schemas.android.com/apk/res-auto" app:waveAmplitude="bad"/>','Expected dp/sp'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave14c_alert_dialog_snackbar_tab_slider_text_input_ext_chip_group_rules_264_to_269(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'drawable').mkdir()
   (res/'drawable'/'ic_settings_24.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24"><path android:fillColor="#000000" android:pathData="M0,0h24v24h-24z"/></vector>')
   (res/'values'/'vals.xml').write_text('''<resources>
    <integer name="max_bio">140</integer>
    <string name="currency_prefix">$|USD</string>
    <style name="CustomDialog" parent="MaterialAlertDialog.Material3Expressive">
     <item name="icon">@drawable/ic_star_24</item>
    </style>
    <style name="CustomSecondaryTabs" parent="Widget.Material3.TabLayout.Secondary">
     <item name="tabMode">scrollable</item>
     <item name="tabIndicatorColor">#112233</item>
    </style>
    <style name="CustomExpressiveSlider" parent="Widget.Material3Expressive.Slider.Medium">
     <item name="android:stepSize">5</item>
     <item name="labelBehavior">visible</item>
    </style>
    <style name="CustomInputExt" parent="Widget.Material3.TextInputLayout.OutlinedBox">
     <item name="counterEnabled">true</item>
     <item name="counterMaxLength">@integer/max_bio</item>
     <item name="prefixText">@string/currency_prefix</item>
     <item name="suffixText">.00</item>
     <item name="startIconDrawable">@drawable/ic_search_24</item>
    </style>
    <style name="CustomChipGroup" parent="Widget.Material3.ChipGroup">
     <item name="singleLine">true</item>
     <item name="singleSelection">true</item>
     <item name="chipSpacing">12dp</item>
     <item name="chipSpacingVertical">4dp</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <androidx.appcompat.widget.AlertDialogLayout android:id="@+id/dlg_plain"><TextView android:text="Confirm"/></androidx.appcompat.widget.AlertDialogLayout>
    <androidx.appcompat.widget.AlertDialogLayout android:id="@+id/dlg_attr_icon" android:icon="@drawable/ic_delete_24"/>
    <androidx.appcompat.widget.AlertDialogLayout android:id="@+id/dlg_child_icon"><ImageView android:id="@android:id/icon" android:src="@drawable/ic_settings_24"/></androidx.appcompat.widget.AlertDialogLayout>
    <LinearLayout android:id="@+id/dlg_style" style="@style/MaterialAlertDialog.Material3"/>
    <FrameLayout android:id="@+id/dlg_custom" style="@style/CustomDialog"/>
    <com.google.android.material.snackbar.SnackbarContentLayout android:id="@+id/snack" app:animationMode="fade" app:maxActionInlineWidth="128dp">
     <TextView android:text="Saved"/>
     <Button android:text="Undo"/>
    </com.google.android.material.snackbar.SnackbarContentLayout>
    <com.google.android.material.tabs.TabLayout android:id="@+id/tab_def"/>
    <com.google.android.material.tabs.TabLayout android:id="@+id/tab_scroll" app:tabMode="scrollable" app:tabIndicatorColor="#6750A4"/>
    <com.google.android.material.tabs.TabLayout android:id="@+id/tab_sec_theme" style="?attr/tabSecondaryStyle" app:tabMode="auto"/>
    <com.google.android.material.tabs.TabLayout android:id="@+id/tab_sec_custom" style="@style/CustomSecondaryTabs"/>
    <com.google.android.material.slider.Slider android:id="@+id/sl_def"/>
    <com.google.android.material.slider.RangeSlider android:id="@+id/sl_full" android:valueFrom="0" android:valueTo="100" android:stepSize="10" app:minSeparation="20dp" app:labelBehavior="withinBounds" style="@style/Widget.Material3Expressive.Slider.Large"/>
    <com.google.android.material.slider.Slider android:id="@+id/sl_styled" style="@style/CustomExpressiveSlider"/>
    <com.google.android.material.slider.Slider android:id="@+id/sl_gone" app:labelBehavior="gone"/>
    <com.google.android.material.textfield.TextInputLayout android:id="@+id/til_plain"><EditText/></com.google.android.material.textfield.TextInputLayout>
    <com.google.android.material.textfield.TextInputLayout android:id="@+id/til_ext" style="@style/CustomInputExt"><EditText/></com.google.android.material.textfield.TextInputLayout>
    <com.google.android.material.textfield.TextInputLayout android:id="@+id/til_unlim" app:counterEnabled="true" app:counterMaxLength="-1" app:startIconDrawable="@drawable/ic_mail_outline"><EditText/></com.google.android.material.textfield.TextInputLayout>
    <com.google.android.material.chip.ChipGroup android:id="@+id/cg_def"/>
    <com.google.android.material.chip.ChipGroup android:id="@+id/cg_styled" style="@style/CustomChipGroup"/>
    <com.google.android.material.chip.ChipGroup android:id="@+id/cg_zero" app:chipSpacing="0dp" app:chipSpacingHorizontal="6dp"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['dlg_plain','dlg_attr_icon','dlg_child_icon','dlg_style','dlg_custom','snack','tab_def','tab_scroll','tab_sec_theme','tab_sec_custom','sl_def','sl_full','sl_styled','sl_gone','til_plain','til_ext','til_unlim','cg_def','cg_styled','cg_zero']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:264, anchor:"" }',slices['dlg_plain'])
   self.assertIn('RelativeRule { kind:264, anchor:"delete" }',slices['dlg_attr_icon'])
   self.assertIn('RelativeRule { kind:264, anchor:"settings" }',slices['dlg_child_icon'])
   self.assertIn('RelativeRule { kind:264, anchor:"" }',slices['dlg_style'])
   self.assertIn('RelativeRule { kind:264, anchor:"star" }',slices['dlg_custom'])
   self.assertIn('kind:Kind::Linear,id:"snack"',out)
   self.assertIn('horizontal:true',slices['snack'])
   self.assertIn('RelativeRule { kind:265, anchor:"" }',slices['snack'])
   self.assertIn('RelativeRule { kind:266, anchor:"fixed|primary|" }',slices['tab_def'])
   self.assertIn('RelativeRule { kind:266, anchor:"scrollable|primary|103,80,164,255" }',slices['tab_scroll'])
   self.assertIn('RelativeRule { kind:266, anchor:"fixed|secondary|" }',slices['tab_sec_theme'])
   self.assertIn('RelativeRule { kind:266, anchor:"scrollable|secondary|17,34,51,255" }',slices['tab_sec_custom'])
   self.assertIn('RelativeRule { kind:267, anchor:"0.0|0.0|floating|0" }',slices['sl_def'])
   self.assertIn('RelativeRule { kind:267, anchor:"10.0|20.0|withinBounds|1" }',slices['sl_full'])
   self.assertIn('RelativeRule { kind:267, anchor:"5.0|0.0|visible|1" }',slices['sl_styled'])
   self.assertIn('RelativeRule { kind:267, anchor:"0.0|0.0|gone|0" }',slices['sl_gone'])
   self.assertNotIn('kind:268',slices['til_plain'])
   self.assertIn('RelativeRule { kind:268, anchor:"1|140|$/USD|.00|search" }',slices['til_ext'])
   self.assertIn('RelativeRule { kind:268, anchor:"1|0|||mail" }',slices['til_unlim'])
   self.assertIn('RelativeRule { kind:269, anchor:"0|0|8.0|8.0" }',slices['cg_def'])
   self.assertIn('RelativeRule { kind:269, anchor:"1|1|12.0|4.0" }',slices['cg_styled'])
   self.assertIn('RelativeRule { kind:269, anchor:"0|0|6.0|0.0" }',slices['cg_zero'])
   for bad_xml,err in [
    ('<com.google.android.material.snackbar.SnackbarContentLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:animationMode="bounce"/>','Unsupported snackbar animationMode'),
    ('<com.google.android.material.slider.Slider xmlns:app="http://schemas.android.com/apk/res-auto" app:labelBehavior="sticky"/>','Unsupported labelBehavior'),
    ('<com.google.android.material.textfield.TextInputLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:counterEnabled="yes"/>','Invalid counterEnabled'),
    ('<com.google.android.material.textfield.TextInputLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:counterMaxLength="-5"/>','Invalid counterMaxLength'),
    ('<com.google.android.material.textfield.TextInputLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:startIconDrawable="not_a_drawable"/>','Invalid startIconDrawable'),
    ('<com.google.android.material.chip.ChipGroup xmlns:app="http://schemas.android.com/apk/res-auto" app:singleLine="maybe"/>','Invalid singleLine'),
    ('<com.google.android.material.chip.ChipGroup xmlns:app="http://schemas.android.com/apk/res-auto" app:singleSelection="maybe"/>','Invalid singleSelection'),
    ('<com.google.android.material.chip.ChipGroup xmlns:app="http://schemas.android.com/apk/res-auto" app:checkedChip="bad_id"/>','Invalid checkedChip'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave14c_m3_expressive_tags_rules_270_to_273(self):
  with tempfile.TemporaryDirectory() as d:
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.floatingtoolbar.FloatingToolbarLayout android:id="@+id/ftb" style="?attr/floatingToolbarVibrantStyle" app:marginBottomSystemWindowInsets="true">
     <com.google.android.material.button.MaterialButton android:text="Edit"/>
    </com.google.android.material.floatingtoolbar.FloatingToolbarLayout>
    <com.google.android.material.button.MaterialButtonGroup android:id="@+id/bg_def">
     <com.google.android.material.button.MaterialButton android:text="One" app:layout_overflowText="One"/>
    </com.google.android.material.button.MaterialButtonGroup>
    <com.google.android.material.button.MaterialButtonGroup android:id="@+id/bg_conn" style="@style/Widget.Material3Expressive.MaterialButtonGroup.Connected"/>
    <com.google.android.material.button.MaterialButtonGroup android:id="@+id/bg_sp" android:spacing="8dp" app:overflowMode="menu"/>
    <com.google.android.material.button.MaterialButtonGroup android:id="@+id/bg_app_sp" app:spacing="4dp"/>
    <com.google.android.material.button.MaterialSplitButton android:id="@+id/split_btn" style="?attr/materialSplitButtonStyle" android:spacing="2dp">
     <com.google.android.material.button.MaterialButton style="?attr/materialSplitButtonLeadingFilledStyle" android:text="Send"/>
     <com.google.android.material.button.MaterialButton style="?attr/materialSplitButtonIconFilledStyle"/>
    </com.google.android.material.button.MaterialSplitButton>
    <com.google.android.material.carousel.MaskableFrameLayout android:id="@+id/car_plain"><ImageView/></com.google.android.material.carousel.MaskableFrameLayout>
    <com.google.android.material.carousel.MaskableFrameLayout android:id="@+id/car_round" app:shapeAppearance="?attr/shapeAppearanceCornerLarge"><ImageView/></com.google.android.material.carousel.MaskableFrameLayout>
   </LinearLayout>'''
   out=self.run_layout(xml)
   names=['ftb','bg_def','bg_conn','bg_sp','bg_app_sp','split_btn','car_plain','car_round']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('kind:Kind::Linear,id:"ftb"',out)
   self.assertIn('horizontal:true',slices['ftb'])
   self.assertIn('RelativeRule { kind:270, anchor:"" }',slices['ftb'])
   self.assertIn('kind:Kind::Linear,id:"bg_def"',out)
   self.assertIn('horizontal:true',slices['bg_def'])
   self.assertIn('RelativeRule { kind:271, anchor:"12.0" }',slices['bg_def'])
   self.assertIn('RelativeRule { kind:271, anchor:"2.0" }',slices['bg_conn'])
   self.assertIn('RelativeRule { kind:271, anchor:"8.0" }',slices['bg_sp'])
   self.assertIn('RelativeRule { kind:271, anchor:"4.0" }',slices['bg_app_sp'])
   self.assertIn('kind:Kind::Linear,id:"split_btn"',out)
   self.assertIn('horizontal:true',slices['split_btn'])
   self.assertIn('RelativeRule { kind:272, anchor:"" }',slices['split_btn'])
   self.assertIn('kind:Kind::Frame,id:"car_plain"',out)
   self.assertIn('RelativeRule { kind:273, anchor:"" }',slices['car_plain'])
   self.assertIn('RelativeRule { kind:273, anchor:"" }',slices['car_round'])
   self.assertIn(',radius:16.0,',slices['car_round'])
   self.assertIn('RelativeRule { kind:234, anchor:"true" }',slices['car_round'])
   # merge_parent & custom_view_map coverage
   mp=Path(d)/'m.xml';mp.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android"><View/></merge>')
   for parent_cls,expected_rule in [
    ('com.google.android.material.floatingtoolbar.FloatingToolbarLayout','kind:270, anchor:""'),
    ('com.google.android.material.button.MaterialButtonGroup','kind:271, anchor:"12.0"'),
    ('com.google.android.material.button.MaterialSplitButton','kind:272, anchor:""'),
    ('com.google.android.material.carousel.MaskableFrameLayout','kind:273, anchor:""'),
    ('com.google.android.material.snackbar.SnackbarContentLayout','kind:265, anchor:""'),
   ]:
    self.assertIn(expected_rule,mod.compile_layout(mp,merge_parent=parent_cls))
   cp=Path(d)/'c.xml';cp.write_text('<com.example.MySplit xmlns:android="http://schemas.android.com/apk/res/android"/>')
   self.assertIn('kind:272, anchor:""',mod.compile_layout(cp,custom_view_map={'com.example.MySplit':'com.google.android.material.button.MaterialSplitButton'}))
   for bad_xml,err in [
    ('<com.google.android.material.floatingtoolbar.FloatingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:marginBottomSystemWindowInsets="maybe"/>','Invalid marginBottomSystemWindowInsets'),
    ('<com.google.android.material.button.MaterialButtonGroup xmlns:app="http://schemas.android.com/apk/res-auto" app:overflowMode="scroll"/>','Unsupported overflowMode'),
    ('<com.google.android.material.button.MaterialButtonGroup xmlns:android="http://schemas.android.com/apk/res/android" android:spacing="-4dp"/>','Expected dp/sp'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml)
 def test_wave14c_extended_anim_and_animator_resources(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'anim').mkdir();(res/'animator').mkdir();(res/'interpolator').mkdir()
   (res/'interpolator'/'local_lin.xml').write_text('<linearInterpolator xmlns:android="http://schemas.android.com/apk/res/android"/>')
   (res/'interpolator'/'local_path.xml').write_text('<pathInterpolator xmlns:android="http://schemas.android.com/apk/res/android" android:controlX1="0.2" android:controlY1="0" android:controlX2="0" android:controlY2="1"/>')
   # res/anim/: <scale>, <rotate>, combined <set>, and all standard interpolators
   (res/'anim'/'scale_only.xml').write_text('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:fromXScale="0.8" android:toXScale="1.0" android:fromYScale="80%" android:toYScale="100%" android:pivotX="50%" android:pivotY="50%p" android:duration="@android:integer/config_shortAnimTime" android:interpolator="@android:anim/overshoot_interpolator" android:fillAfter="true"/>')
   (res/'anim'/'rot_only.xml').write_text('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:fromDegrees="0" android:toDegrees="360" android:pivotX="12dp" android:pivotY="12dp" android:duration="400" android:interpolator="@android:anim/decelerate_interpolator" android:repeatCount="infinite" android:repeatMode="restart"/>')
   (res/'anim'/'combo.xml').write_text('''<set xmlns:android="http://schemas.android.com/apk/res/android" android:duration="250" android:interpolator="@android:interpolator/fast_out_slow_in">
    <alpha android:fromAlpha="0" android:toAlpha="1"/>
    <translate android:fromYDelta="24dp" android:toYDelta="0dp"/>
    <scale android:fromXScale="0.9" android:toXScale="1.0" android:fromYScale="0.9" android:toYScale="1.0" android:pivotX="50%" android:pivotY="50%"/>
    <rotate android:fromDegrees="-10" android:toDegrees="0" android:pivotX="50%" android:pivotY="50%"/>
   </set>''')
   sw_xml='<ViewSwitcher xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@anim/scale_only" android:outAnimation="@anim/rot_only"><View/></ViewSwitcher>'
   sw_out=self.run_layout(sw_xml,res)
   self.assertIn('animation_in:None,animation_out:None,translate_in:None,translate_out:None',sw_out)
   combo_out=self.run_layout(sw_xml.replace('@anim/scale_only','@anim/combo'),res)
   self.assertIn('animation_in:Some(AlphaAnimation { from:0.0, to:1.0, duration:0.25, start_offset:0.0, linear:false })',combo_out)
   self.assertIn('translate_in:Some(TranslateAnimation',combo_out)
   for interp,exp_linear in [
    ('@android:anim/decelerate_interpolator','false'),
    ('@android:anim/accelerate_interpolator','false'),
    ('@android:anim/overshoot_interpolator','false'),
    ('@android:anim/anticipate_interpolator','false'),
    ('@android:interpolator/fast_out_slow_in','false'),
    ('@android:interpolator/linear','true'),
    ('@interpolator/m3_motion_emphasized','false'),
    ('@interpolator/local_lin','true'),
    ('@interpolator/local_path','false'),
   ]:
    (res/'anim'/'interp_t.xml').write_text(f'<alpha xmlns:android="http://schemas.android.com/apk/res/android" android:fromAlpha="0" android:toAlpha="1" android:duration="200" android:interpolator="{interp}"/>')
    with self.subTest(interp=interp):
     self.assertIn(f'linear:{exp_linear}',self.run_layout('<ViewSwitcher xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@anim/interp_t"><View/></ViewSwitcher>',res))
   # res/animator/: ordering="sequentially", scaleX/scaleY/rotation/elevation/translationZ, repeatCount, repeatMode
   (res/'animator'/'seq.xml').write_text('''<set xmlns:android="http://schemas.android.com/apk/res/android" android:ordering="sequentially">
    <objectAnimator android:propertyName="translationX" android:valueFrom="48dp" android:valueTo="0dp" android:duration="300" android:interpolator="@android:anim/decelerate_interpolator"/>
    <objectAnimator android:propertyName="scaleX" android:valueFrom="0.8" android:valueTo="1.0" android:duration="200" android:repeatCount="2" android:repeatMode="reverse"/>
    <objectAnimator android:propertyName="alpha" android:valueFrom="0" android:valueTo="1" android:duration="400" android:interpolator="@android:interpolator/fast_out_slow_in"/>
   </set>''')
   af_xml='<AdapterViewFlipper xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@animator/seq"/>'
   seq_out=self.run_layout(af_xml,res)
   self.assertIn('animation_in:Some(AlphaAnimation { from:0.0, to:1.0, duration:0.4, start_offset:0.5, linear:false })',seq_out)
   self.assertIn('translate_in:Some(TranslateAnimation { deltas:[AnimationDelta { value:48.0, mode:0 },AnimationDelta { value:0.0, mode:0 },AnimationDelta { value:0.0, mode:0 },AnimationDelta { value:0.0, mode:0 }], duration:0.3, start_offset:0.0, linear:false, y_timing:None })',seq_out)
   for prop,vf,vt in [('scaleX','0.5','1.0'),('scaleY','0.5','1.0'),('rotation','0','180'),('elevation','0dp','6dp'),('translationZ','0dp','4dp')]:
    (res/'animator'/'prop_t.xml').write_text(f'<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="{prop}" android:valueFrom="{vf}" android:valueTo="{vt}" android:duration="250" android:repeatCount="infinite" android:repeatMode="restart" android:interpolator="@interpolator/local_path"/>')
    with self.subTest(prop=prop):
     self.assertIn('animation_in:None',self.run_layout(af_xml.replace('@animator/seq','@animator/prop_t'),res))
   # Strict rejections for anim & animator
   for bad_anim,err in [
    ('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:fromXScale="wide"/>','Invalid scale animation value'),
    ('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:pivotX="center"/>','Invalid animation pivot'),
    ('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:toDegrees="spin"/>','Invalid rotate animation degrees'),
    ('<alpha xmlns:android="http://schemas.android.com/apk/res/android" android:interpolator="@android:anim/bogus_interpolator"/>','Unsupported animation interpolator'),
   ]:
    (res/'anim'/'bad.xml').write_text(bad_anim)
    with self.subTest(bad=bad_anim):
     with self.assertRaisesRegex(ValueError,err):self.run_layout('<ViewSwitcher xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@anim/bad"><View/></ViewSwitcher>',res)
   for bad_animator,err in [
    ('<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="scaleX" android:valueFrom="1" android:valueTo="2" android:repeatCount="many"/>','Invalid animator repeatCount'),
    ('<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="rotation" android:valueFrom="0" android:valueTo="90" android:repeatMode="bounce"/>','Invalid animator repeatMode'),
    ('<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="scaleX" android:valueFrom="1" android:valueTo="2" android:interpolator="@android:anim/bogus"/>','Unsupported animator interpolator'),
   ]:
    (res/'animator'/'bad.xml').write_text(bad_animator)
    with self.subTest(bad=bad_animator):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(af_xml.replace('@animator/seq','@animator/bad'),res)
 def test_wave14c_animation_list_and_animated_selector_drawables(self):
  def make_png(w,h,rgba):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=b''.join(b'\x00'+bytes(rgba)*w for _ in range(h))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'frame1.png').write_bytes(make_png(10,8,(10,20,30,255)))
   (res/'drawable'/'frame2.png').write_bytes(make_png(10,8,(200,10,10,255)))
   (res/'drawable'/'sh1.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#112233"/><size android:width="16dp" android:height="16dp"/></shape>')
   (res/'drawable'/'sh2.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#445566"/><size android:width="16dp" android:height="16dp"/></shape>')
   (res/'drawable'/'anim_list_ref.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="false"><item android:drawable="@drawable/sh1" android:duration="100"/><item android:drawable="@drawable/sh2" android:duration="100"/></animation-list>')
   (res/'drawable'/'anim_list_inline.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="true"><item android:duration="80"><shape><solid android:color="#224466"/><corners android:radius="6dp"/></shape></item><item android:duration="80"><shape><solid android:color="#888888"/></shape></item></animation-list>')
   (res/'drawable'/'anim_list_png.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:drawable="@drawable/frame1" android:duration="60"/><item android:drawable="@drawable/frame2" android:duration="60"/></animation-list>')
   (res/'drawable'/'anim_sel_bg.xml').write_text('''<animated-selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/st_checked" android:state_checked="true" android:drawable="@drawable/sh1"/>
    <item android:id="@+id/st_off" android:drawable="@drawable/sh2"/>
    <transition android:fromId="@id/st_off" android:toId="@id/st_checked" android:drawable="@drawable/anim_list_ref"/>
    <transition android:fromId="@id/st_checked" android:toId="@id/st_off">
     <animation-list><item android:drawable="@drawable/sh2" android:duration="40"/></animation-list>
    </transition>
   </animated-selector>''')
   (res/'drawable'/'anim_sel_img.xml').write_text('''<animated-selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/st_pressed" android:state_pressed="true" android:drawable="@drawable/frame2"/>
    <item android:id="@+id/st_def" android:drawable="@drawable/frame1"/>
    <transition android:fromId="@id/st_def" android:toId="@id/st_pressed" android:drawable="@drawable/anim_list_png"/>
   </animated-selector>''')
   (res/'drawable'/'anim_sel_tick.xml').write_text('''<animated-selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/t_on" android:state_pressed="true" android:drawable="@drawable/sh1"/>
    <item android:id="@+id/t_off" android:drawable="@drawable/sh2"/>
    <transition android:fromId="@id/t_off" android:toId="@id/t_on" android:drawable="@drawable/anim_list_ref"/>
   </animated-selector>''')
   (res/'drawable'/'anim_sel_ind.xml').write_text('''<animated-selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/g_exp" android:state_expanded="true" android:drawable="@drawable/frame2"/>
    <item android:id="@+id/g_col" android:drawable="@drawable/frame1"/>
    <transition android:fromId="@id/g_col" android:toId="@id/g_exp" android:drawable="@drawable/anim_list_png"/>
   </animated-selector>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <View android:id="@+id/v_al_ref" android:background="@drawable/anim_list_ref"/>
    <View android:id="@+id/v_al_in" android:background="@drawable/anim_list_inline"/>
    <ImageView android:id="@+id/iv_al" android:src="@drawable/anim_list_png"/>
    <View android:id="@+id/v_as_bg" android:background="@drawable/anim_sel_bg"/>
    <ImageView android:id="@+id/iv_as" android:src="@drawable/anim_sel_img"/>
    <TextView android:id="@+id/tv_al_cmp" android:drawableStart="@drawable/anim_list_ref"/>
    <CheckedTextView android:id="@+id/ctv_as_cmp" android:drawableStart="@drawable/anim_sel_bg"/>
    <CheckBox android:id="@+id/cb_as_btn" android:button="@drawable/anim_sel_bg"/>
    <CheckBox android:id="@+id/cb_al_btn" android:button="@drawable/anim_list_ref"/>
    <SeekBar android:id="@+id/sb_as_tick" android:tickMark="@drawable/anim_sel_tick"/>
    <SeekBar android:id="@+id/sb_al_tick" android:tickMark="@drawable/anim_list_ref"/>
    <ExpandableListView android:id="@+id/elv_as" android:groupIndicator="@drawable/anim_sel_ind"/>
    <ExpandableListView android:id="@+id/elv_al" android:groupIndicator="@drawable/anim_list_png"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['v_al_ref','v_al_in','iv_al','v_as_bg','iv_as','tv_al_cmp','ctv_as_cmp','cb_as_btn','cb_al_btn','sb_as_tick','sb_al_tick','elv_as','elv_al']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('background:Some(Color(17,34,51,255))',slices['v_al_ref'])
   self.assertIn('background:Some(Color(34,68,102,255))',slices['v_al_in'])
   self.assertIn(',radius:6.0,',slices['v_al_in'])
   self.assertIn('state.images.entry("anim_list_png".into())',out)
   self.assertIn('RelativeRule { kind:96, anchor:"64,0,17,34,51,255" }',slices['v_as_bg'])
   self.assertIn('RelativeRule { kind:96, anchor:"0,0,68,85,102,255" }',slices['v_as_bg'])
   self.assertIn('RelativeRule { kind:253, anchor:"2,0|frame2|" }',slices['iv_as'])
   self.assertIn('RelativeRule { kind:253, anchor:"0,0|frame1|" }',slices['iv_as'])
   self.assertIn('RelativeRule { kind:173, anchor:"16.0,16.0,0.0,17,34,51,255" }',slices['tv_al_cmp'])
   self.assertIn('RelativeRule { kind:182, anchor:"64,0,16.0,16.0,0.0,17,34,51,255" }',slices['ctv_as_cmp'])
   self.assertIn('RelativeRule { kind:182, anchor:"0,0,16.0,16.0,0.0,68,85,102,255" }',slices['ctv_as_cmp'])
   self.assertIn('RelativeRule { kind:165, anchor:"64,0,16.0,16.0,0,17,34,51,255" }',slices['cb_as_btn'])
   self.assertIn('RelativeRule { kind:165, anchor:"0,0,16.0,16.0,0,68,85,102,255" }',slices['cb_as_btn'])
   self.assertIn('RelativeRule { kind:165, anchor:"0,0,16.0,16.0,0,17,34,51,255" }',slices['cb_al_btn'])
   self.assertIn('RelativeRule { kind:167, anchor:"2,0,16.0,16.0,0,17,34,51,255" }',slices['sb_as_tick'])
   self.assertIn('RelativeRule { kind:167, anchor:"0,0,16.0,16.0,0,68,85,102,255" }',slices['sb_as_tick'])
   self.assertIn('RelativeRule { kind:155, anchor:"16.0,16.0,0,17,34,51,255" }',slices['sb_al_tick'])
   self.assertIn('RelativeRule { kind:131, anchor:"1,0,frame2" }',slices['elv_as'])
   self.assertIn('RelativeRule { kind:131, anchor:"0,0,frame1" }',slices['elv_as'])
   self.assertIn('RelativeRule { kind:131, anchor:"0,0,frame1" }',slices['elv_al'])
   for bad_drw,err in [
    ('<animation-list xmlns:android="http://schemas.android.com/apk/res/android"/>','Invalid animation-list drawable'),
    ('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="maybe"><item android:drawable="@drawable/sh1"/></animation-list>','Invalid animation-list boolean'),
    ('<animation-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:drawable="@drawable/sh1" android:duration="-10"/></animation-list>','Invalid animation-list duration'),
    ('<animated-selector xmlns:android="http://schemas.android.com/apk/res/android"><bogus/></animated-selector>','Invalid selector item'),
   ]:
    (res/'drawable'/'bad_d.xml').write_text(bad_drw)
    with self.subTest(bad=bad_drw):
     with self.assertRaisesRegex(ValueError,err):self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/bad_d"/>',res)
 def test_wave16b_scale_and_rotate_transition_rules_274_to_277(self):
  import re as _re
  def verify_rust_anim_anchor(kind, anchor):
   # Mirrors parse_scale_animation / parse_rotate_animation in android_xml.rs
   parts=anchor.split('|')
   self.assertEqual(len(parts),5,f'Expected 5 pipe-delimited parts in {anchor}')
   vals=parts[0].split(',')
   if kind in (274,275):
    self.assertEqual(len(vals),4,f'Expected 4 scale values in {anchor}')
    for v in vals:
     float(v[:-1] if v.endswith('%') else v)
   else:
    self.assertEqual(len(vals),2,f'Expected 2 rotate values in {anchor}')
    for v in vals:float(v)
   self.assertIn(',',parts[1],f'Pivot segment must contain comma in {anchor}')
   for pv in parts[1].split(','):
    s=pv.strip()
    if s=='' or s.lower()=='center':continue
    if s.lower().endswith('%p'):float(s[:-2])
    elif s.endswith('%'):float(s[:-1])
    elif s.endswith('dp'):float(s[:-2])
    elif s.endswith('dip'):float(s[:-3])
    else:float(s)
   self.assertGreaterEqual(float(parts[2]),0.0)
   self.assertGreaterEqual(float(parts[3]),0.0)
   self.assertIn(parts[4],{'0','1'})
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'anim').mkdir();(res/'animator').mkdir();(res/'interpolator').mkdir()
   (res/'interpolator'/'lin.xml').write_text('<linearInterpolator xmlns:android="http://schemas.android.com/apk/res/android"/>')
   (res/'anim'/'scale_in.xml').write_text('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:fromXScale="0.5" android:toXScale="1.0" android:fromYScale="50%" android:toYScale="100%" android:pivotX="50%" android:pivotY="50%p" android:duration="400" android:startOffset="100" android:interpolator="@android:anim/linear_interpolator"/>')
   (res/'anim'/'scale_out.xml').write_text('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:fromXScale="1.0" android:toXScale="0.2" android:fromYScale="100%" android:toYScale="20%" android:pivotX="12dp" android:pivotY="24px" android:duration="@android:integer/config_shortAnimTime" android:interpolator="@android:anim/overshoot_interpolator"/>')
   (res/'anim'/'rot_in.xml').write_text('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:fromDegrees="-90" android:toDegrees="0" android:pivotX="50%" android:pivotY="50%" android:duration="300" android:interpolator="@interpolator/lin"/>')
   (res/'anim'/'rot_out.xml').write_text('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:fromDegrees="0" android:toDegrees="180.5" android:duration="250" android:startOffset="50"/>')
   (res/'anim'/'combo_in.xml').write_text('''<set xmlns:android="http://schemas.android.com/apk/res/android" android:duration="250" android:startOffset="100" android:interpolator="@android:anim/linear_interpolator">
    <alpha android:fromAlpha="0" android:toAlpha="1"/>
    <translate android:fromYDelta="24dp" android:toYDelta="0dp"/>
    <scale android:fromXScale="0.9" android:toXScale="1.0" android:fromYScale="0.9" android:toYScale="1.0" android:pivotX="50%" android:pivotY="50%" android:startOffset="50"/>
    <rotate android:fromDegrees="-15" android:toDegrees="0" android:pivotX="10dp" android:pivotY="5dp"/>
   </set>''')
   (res/'anim'/'combo_noshare.xml').write_text('''<set xmlns:android="http://schemas.android.com/apk/res/android" android:shareInterpolator="false" android:interpolator="@android:anim/linear_interpolator">
    <scale android:fromXScale="0.8" android:toXScale="1.0" android:fromYScale="0.8" android:toYScale="1.0" android:pivotX="50%" android:pivotY="50%" android:duration="200" android:interpolator="@android:interpolator/fast_out_slow_in"/>
    <rotate android:fromDegrees="0" android:toDegrees="45" android:pivotX="50%" android:pivotY="50%" android:duration="300" android:interpolator="@android:anim/linear_interpolator"/>
   </set>''')
   (res/'animator'/'avf_in.xml').write_text('''<set xmlns:android="http://schemas.android.com/apk/res/android" android:ordering="together">
    <objectAnimator android:propertyName="scaleX" android:valueFrom="0.5" android:valueTo="1.0" android:duration="400" android:interpolator="@android:anim/linear_interpolator"/>
    <objectAnimator android:propertyName="scaleY" android:valueFrom="0.5" android:valueTo="1.0" android:duration="400" android:interpolator="@android:anim/linear_interpolator"/>
    <objectAnimator android:propertyName="rotation" android:valueFrom="-45" android:valueTo="0" android:duration="400" android:startOffset="100" android:interpolator="@android:anim/linear_interpolator"/>
   </set>''')
   (res/'animator'/'avf_out.xml').write_text('''<set xmlns:android="http://schemas.android.com/apk/res/android" android:ordering="sequentially">
    <objectAnimator android:propertyName="alpha" android:valueFrom="1" android:valueTo="0" android:duration="150"/>
    <objectAnimator android:propertyName="rotation" android:valueFrom="0" android:valueTo="90" android:duration="200"/>
    <objectAnimator android:propertyName="scaleX" android:valueFrom="1.0" android:valueTo="0.4" android:duration="250"/>
   </set>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <ViewAnimator android:id="@+id/va" android:inAnimation="@anim/scale_in" android:outAnimation="@anim/scale_out"><View/></ViewAnimator>
    <ViewSwitcher android:id="@+id/vs" android:inAnimation="@anim/rot_in" android:outAnimation="@anim/rot_out"><View/></ViewSwitcher>
    <TextSwitcher android:id="@+id/ts" android:inAnimation="@anim/combo_in" android:outAnimation="@anim/combo_noshare"><TextView/></TextSwitcher>
    <ImageSwitcher android:id="@+id/isw" android:inAnimation="@anim/scale_in" android:outAnimation="@anim/rot_out"><ImageView/></ImageSwitcher>
    <ViewFlipper android:id="@+id/vf" android:inAnimation="@anim/combo_in" android:outAnimation="@anim/scale_out"><View/></ViewFlipper>
    <AdapterViewFlipper android:id="@+id/avf" android:inAnimation="@animator/avf_in" android:outAnimation="@animator/avf_out"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['va','vs','ts','isw','vf','avf']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:274, anchor:"0.5,1.0,0.5,1.0|50%,50%p|0.4|0.1|1" }',slices['va'])
   self.assertIn('RelativeRule { kind:275, anchor:"1.0,0.2,1.0,0.2|12dp,24dp|0.2|0.0|0" }',slices['va'])
   self.assertIn('RelativeRule { kind:276, anchor:"-90,0|50%,50%|0.3|0.0|1" }',slices['vs'])
   self.assertIn('RelativeRule { kind:277, anchor:"0,180.5|0dp,0dp|0.25|0.05|0" }',slices['vs'])
   self.assertIn('animation_in:Some(AlphaAnimation { from:0.0, to:1.0, duration:0.25, start_offset:0.1, linear:true })',slices['ts'])
   self.assertIn('translate_in:Some(TranslateAnimation',slices['ts'])
   self.assertIn('RelativeRule { kind:274, anchor:"0.9,1.0,0.9,1.0|50%,50%|0.25|0.15|1" }',slices['ts'])
   self.assertIn('RelativeRule { kind:276, anchor:"-15,0|10dp,5dp|0.25|0.1|1" }',slices['ts'])
   self.assertIn('RelativeRule { kind:275, anchor:"0.8,1.0,0.8,1.0|50%,50%|0.2|0.0|0" }',slices['ts'])
   self.assertIn('RelativeRule { kind:277, anchor:"0,45|50%,50%|0.3|0.0|1" }',slices['ts'])
   self.assertIn('RelativeRule { kind:274, anchor:"0.5,1.0,0.5,1.0|50%,50%p|0.4|0.1|1" }',slices['isw'])
   self.assertIn('RelativeRule { kind:277, anchor:"0,180.5|0dp,0dp|0.25|0.05|0" }',slices['isw'])
   self.assertIn('RelativeRule { kind:274, anchor:"0.9,1.0,0.9,1.0|50%,50%|0.25|0.15|1" }',slices['vf'])
   self.assertIn('RelativeRule { kind:276, anchor:"-15,0|10dp,5dp|0.25|0.1|1" }',slices['vf'])
   self.assertIn('RelativeRule { kind:275, anchor:"1.0,0.2,1.0,0.2|12dp,24dp|0.2|0.0|0" }',slices['vf'])
   self.assertIn('RelativeRule { kind:274, anchor:"0.5,1.0,0.5,1.0|center,center|0.4|0.0|1" }',slices['avf'])
   self.assertIn('RelativeRule { kind:276, anchor:"-45,0|center,center|0.4|0.1|1" }',slices['avf'])
   self.assertIn('RelativeRule { kind:275, anchor:"1.0,0.4,1.0,1.0|center,center|0.25|0.35|0" }',slices['avf'])
   self.assertIn('RelativeRule { kind:277, anchor:"0,90|center,center|0.2|0.15|0" }',slices['avf'])
   # Verify every emitted 274..=277 rule matches android_xml.rs parser
   matched=_re.findall(r'RelativeRule \{ kind:(27[4-7]), anchor:"([^"]+)" \}',out)
   self.assertGreaterEqual(len(matched),15)
   for k_str,anc in matched:
    verify_rust_anim_anchor(int(k_str),anc)
   # Density conversion for px pivots (24px at 320dpi == 12dp)
   lp=Path(d)/'dens.xml';lp.write_text('<ViewSwitcher xmlns:android="http://schemas.android.com/apk/res/android" android:outAnimation="@anim/scale_out"><View/></ViewSwitcher>')
   out_320=mod.compile_layout(lp,res,density=320)
   self.assertIn('RelativeRule { kind:275, anchor:"1.0,0.2,1.0,0.2|12dp,12dp|0.2|0.0|0" }',out_320)
   # Duplicate <scale> or <rotate> in <set> rejected
   for dup_xml in (
    '<set xmlns:android="http://schemas.android.com/apk/res/android"><scale android:fromXScale="0.5" android:toXScale="1"/><scale android:fromXScale="1" android:toXScale="0.5"/></set>',
    '<set xmlns:android="http://schemas.android.com/apk/res/android"><rotate android:fromDegrees="0" android:toDegrees="90"/><rotate android:fromDegrees="90" android:toDegrees="180"/></set>',
   ):
    (res/'anim'/'dup.xml').write_text(dup_xml)
    with self.assertRaisesRegex(ValueError,'Duplicate animation type in set'):
     self.run_layout('<ViewSwitcher xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@anim/dup"><View/></ViewSwitcher>',res)
 def test_wave16b_navigation_view_header_layout(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'layout').mkdir();(res/'layout-night').mkdir();(res/'menu').mkdir();(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <string name="hdr_name">Alex Morgan</string>
    <string name="hdr_email">alex@example.com</string>
    <style name="StyledNavDrawer" parent="Widget.Material3.NavigationView">
     <item name="headerLayout">@layout/nav_header_merge</item>
    </style>
   </resources>''')
   (res/'menu'/'drawer_nav.xml').write_text('''<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <item android:id="@+id/nav_inbox" android:title="Inbox" android:icon="@drawable/ic_inbox_24" app:badgeText="3"/>
    <item android:id="@+id/nav_settings" android:title="Settings" android:icon="@drawable/ic_settings_24"/>
   </menu>''')
   (res/'layout'/'nav_header.xml').write_text('''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"
     android:id="@+id/hdr_root" android:layout_width="match_parent" android:layout_height="160dp" android:orientation="vertical" android:padding="16dp">
    <TextView android:id="@+id/hdr_title" android:text="@string/hdr_name" android:textStyle="bold"/>
    <TextView android:id="@+id/hdr_sub" android:text="@string/hdr_email"/>
   </LinearLayout>''')
   (res/'layout-night'/'nav_header.xml').write_text('''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"
     android:id="@+id/hdr_root" android:layout_width="match_parent" android:layout_height="160dp" android:orientation="vertical">
    <TextView android:id="@+id/hdr_night" android:text="Night Header"/>
   </LinearLayout>''')
   (res/'layout'/'nav_header_merge.xml').write_text('''<merge xmlns:android="http://schemas.android.com/apk/res/android">
    <TextView android:id="@+id/merge_hdr_1" android:text="Workspace"/>
    <TextView android:id="@+id/merge_hdr_2" android:text="Pro Plan"/>
   </merge>''')
   (res/'layout'/'nav_header_db.xml').write_text('''<layout xmlns:android="http://schemas.android.com/apk/res/android">
    <data><variable name="user" type="String"/></data>
    <FrameLayout android:id="@+id/db_hdr_root" android:layout_width="match_parent" android:layout_height="120dp">
     <TextView android:id="@+id/db_hdr_txt" android:text="Bound Header"/>
    </FrameLayout>
   </layout>''')
   xml='''<androidx.drawerlayout.widget.DrawerLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" android:id="@+id/dl">
    <FrameLayout android:id="@+id/content"/>
    <com.google.android.material.navigation.NavigationView android:id="@+id/nav_full" android:layout_width="280dp" android:layout_height="match_parent" android:layout_gravity="start"
     app:headerLayout="@layout/nav_header" app:menu="@menu/drawer_nav">
     <TextView android:id="@+id/nav_footer" android:text="Version 1.0"/>
    </com.google.android.material.navigation.NavigationView>
    <com.google.android.material.navigation.NavigationView android:id="@+id/nav_styled" style="@style/StyledNavDrawer" android:layout_gravity="end"/>
    <com.google.android.material.navigation.NavigationView android:id="@+id/nav_db" app:headerLayout="@layout/nav_header_db" android:layout_gravity="end"/>
    <com.google.android.material.navigation.NavigationView android:id="@+id/nav_null" app:headerLayout="@null" app:menu="@menu/drawer_nav" android:layout_gravity="end"/>
    <com.google.android.material.navigation.NavigationView android:id="@+id/nav_missing" app:headerLayout="@layout/does_not_exist" android:layout_gravity="end"/>
   </androidx.drawerlayout.widget.DrawerLayout>'''
   out=self.run_layout(xml,res)
   names=['nav_full','nav_styled','nav_db','nav_null','nav_missing']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:247, anchor:"" }',out)
   # nav_full preserves menu items, badges, drawer rule, AND inflates header BEFORE inline footer
   self.assertIn('RelativeRule { kind:237, anchor:"drawer|labeled" }',slices['nav_full'])
   self.assertIn('RelativeRule { kind:238, anchor:"nav_inbox|Inbox|inbox" }',slices['nav_full'])
   self.assertIn('RelativeRule { kind:238, anchor:"nav_settings|Settings|settings" }',slices['nav_full'])
   self.assertIn('RelativeRule { kind:244, anchor:"nav_inbox:3" }',slices['nav_full'])
   self.assertIn('id:"hdr_root"',slices['nav_full'])
   self.assertIn('text:"Alex Morgan"',slices['nav_full'])
   self.assertIn('text:"alex@example.com"',slices['nav_full'])
   self.assertLess(slices['nav_full'].index('id:"hdr_root"'),slices['nav_full'].index('id:"nav_footer"'))
   # Style-provided <merge> headerLayout inflates both merge children
   self.assertIn('RelativeRule { kind:237, anchor:"drawer|labeled" }',slices['nav_styled'])
   self.assertIn('id:"merge_hdr_1"',slices['nav_styled'])
   self.assertIn('id:"merge_hdr_2"',slices['nav_styled'])
   # DataBinding <layout> headerLayout inflates cleanly
   self.assertIn('id:"db_hdr_root"',slices['nav_db'])
   self.assertIn('text:"Bound Header"',slices['nav_db'])
   # @null and missing headerLayout degrade cleanly
   self.assertIn('children:&[]',slices['nav_null'])
   self.assertIn('RelativeRule { kind:238, anchor:"nav_inbox|Inbox|inbox" }',slices['nav_null'])
   self.assertIn('children:&[]',slices['nav_missing'])
   # Configuration-qualified night headerLayout
   main_p=Path(d)/'night_nav.xml'
   main_p.write_text('<com.google.android.material.navigation.NavigationView xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:headerLayout="@layout/nav_header"/>')
   night_out=mod.compile_layout(main_p,res,night=True)
   self.assertIn('id:"hdr_night"',night_out)
   self.assertIn('text:"Night Header"',night_out)
   # Strict rejections: invalid ref, cyclic headerLayout, duplicate ID
   with self.assertRaisesRegex(ValueError,'Invalid headerLayout'):
    self.run_layout('<com.google.android.material.navigation.NavigationView xmlns:app="http://schemas.android.com/apk/res-auto" app:headerLayout="not_a_layout"/>',res)
   (res/'layout'/'cyc_hdr.xml').write_text('<com.google.android.material.navigation.NavigationView xmlns:app="http://schemas.android.com/apk/res-auto" app:headerLayout="@layout/cyc_hdr"/>')
   with self.assertRaisesRegex(ValueError,'Cyclic include'):
    self.run_layout('<com.google.android.material.navigation.NavigationView xmlns:app="http://schemas.android.com/apk/res-auto" app:headerLayout="@layout/cyc_hdr"/>',res)
   with self.assertRaisesRegex(ValueError,'Duplicate id hdr_title'):
    self.run_layout('<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><TextView android:id="@+id/hdr_title"/><com.google.android.material.navigation.NavigationView app:headerLayout="@layout/nav_header"/></LinearLayout>',res)
 def test_wave16b_wrapper_and_list_drawables_all_contexts(self):
  def make_png(w,h,rgba):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=b''.join(b'\x00'+bytes(rgba)*w for _ in range(h))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();(res/'values').mkdir()
   (res/'values'/'ints.xml').write_text('<resources><integer name="lvl_max">5</integer></resources>')
   (res/'drawable'/'px_a.png').write_bytes(make_png(8,8,(10,20,30,255)))
   (res/'drawable'/'px_b.png').write_bytes(make_png(8,8,(200,40,50,255)))
   (res/'drawable'/'box_a.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#112233"/><corners android:radius="6dp"/><size android:width="16dp" android:height="16dp"/></shape>')
   (res/'drawable'/'box_b.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#445566"/><corners android:radius="2dp"/><size android:width="16dp" android:height="16dp"/></shape>')
   (res/'drawable'/'vec_a.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24"><path android:fillColor="#2266aa" android:pathData="M0,0h24v24h-24z"/></vector>')
   # All 5 drawables: ref + inline child forms (including inline <vector>)
   (res/'drawable'/'rot_ref.xml').write_text('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:fromDegrees="0" android:toDegrees="90" android:pivotX="50%" android:pivotY="50%" android:autoMirrored="true" android:drawable="@drawable/box_a"/>')
   (res/'drawable'/'rot_inline_vec.xml').write_text('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:fromDegrees="45"><vector android:width="20dp" android:height="20dp" android:viewportWidth="20" android:viewportHeight="20"><path android:fillColor="#cc2233" android:pathData="M0,0h20v20h-20z"/></vector></rotate>')
   (res/'drawable'/'scale_ref.xml').write_text('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:scaleWidth="80%" android:scaleHeight="75%" android:scaleGravity="center" android:drawable="@drawable/vec_a"/>')
   (res/'drawable'/'scale_inline.xml').write_text('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:scaleWidth="60%" android:scaleHeight="60%" android:scaleGravity="start|center_vertical"><shape><solid android:color="#334455"/><corners android:radius="4dp"/></shape></scale>')
   (res/'drawable'/'clip_ref.xml').write_text('<clip xmlns:android="http://schemas.android.com/apk/res/android" android:clipOrientation="vertical" android:gravity="bottom" android:drawable="@drawable/px_a"/>')
   (res/'drawable'/'clip_inline.xml').write_text('<clip xmlns:android="http://schemas.android.com/apk/res/android" android:clipOrientation="horizontal" android:gravity="left"><shape><solid android:color="#556677"/></shape></clip>')
   (res/'drawable'/'lvl_ref.xml').write_text('<level-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:minLevel="1" android:maxLevel="@integer/lvl_max" android:drawable="@drawable/box_a"/><item android:minLevel="0" android:maxLevel="0" android:drawable="@drawable/box_b"/></level-list>')
   (res/'drawable'/'lvl_fallback.xml').write_text('<level-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:minLevel="2" android:maxLevel="4" android:drawable="@drawable/px_b"/><item android:minLevel="5" android:maxLevel="10" android:drawable="@drawable/px_a"/></level-list>')
   (res/'drawable'/'trans_ref.xml').write_text('<transition xmlns:android="http://schemas.android.com/apk/res/android"><item android:drawable="@drawable/vec_a"/><item android:drawable="@drawable/px_b"/></transition>')
   (res/'drawable'/'trans_inline.xml').write_text('<transition xmlns:android="http://schemas.android.com/apk/res/android"><item><shape><solid android:color="#778899"/><corners android:radius="3dp"/></shape></item><item android:drawable="@drawable/box_b"/></transition>')
   # Nested inside <layer-list> (both bg & srcCompat)
   (res/'drawable'/'layer_wrap_bg.xml').write_text('''<layer-list xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:drawable="@drawable/rot_ref"/>
    <item android:left="4dp" android:top="4dp" android:right="4dp" android:bottom="4dp"><scale android:scaleWidth="80%" android:scaleHeight="80%"><shape><solid android:color="#aa1122"/></shape></scale></item>
    <item><clip android:clipOrientation="horizontal" android:gravity="start"><shape><solid android:color="#22aa33"/></shape></clip></item>
   </layer-list>''')
   (res/'drawable'/'layer_wrap_img.xml').write_text('''<layer-list xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:drawable="@drawable/box_a"/>
    <item><rotate android:fromDegrees="180" android:drawable="@drawable/px_a"/></item>
   </layer-list>''')
   # Nested inside <selector> (both bg & src/srcCompat)
   (res/'drawable'/'sel_wrap_bg.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:state_pressed="true" android:drawable="@drawable/lvl_ref"/>
    <item android:state_focused="true"><transition><item android:drawable="@drawable/box_a"/><item android:drawable="@drawable/box_b"/></transition></item>
    <item android:drawable="@drawable/clip_inline"/>
   </selector>''')
   (res/'drawable'/'sel_wrap_img.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:state_pressed="true" android:drawable="@drawable/clip_ref"/>
    <item android:state_selected="true"><scale android:scaleWidth="90%" android:drawable="@drawable/vec_a"/></item>
    <item android:drawable="@drawable/lvl_fallback"/>
   </selector>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <View android:id="@+id/bg_rot_ref" android:background="@drawable/rot_ref"/>
    <View android:id="@+id/bg_rot_vec" android:background="@drawable/rot_inline_vec"/>
    <View android:id="@+id/bg_scale" android:background="@drawable/scale_inline"/>
    <View android:id="@+id/bg_clip" android:background="@drawable/clip_inline"/>
    <View android:id="@+id/bg_lvl" android:background="@drawable/lvl_ref"/>
    <View android:id="@+id/bg_trans" android:background="@drawable/trans_inline"/>
    <View android:id="@+id/bg_layer" android:background="@drawable/layer_wrap_bg"/>
    <View android:id="@+id/bg_sel" android:background="@drawable/sel_wrap_bg"/>
    <ImageView android:id="@+id/src_rot_vec" android:src="@drawable/rot_inline_vec"/>
    <ImageView android:id="@+id/src_scale" app:srcCompat="@drawable/scale_ref"/>
    <ImageView android:id="@+id/src_clip" app:srcCompat="@drawable/clip_ref"/>
    <ImageView android:id="@+id/src_lvl" android:src="@drawable/lvl_fallback"/>
    <ImageView android:id="@+id/src_trans" app:srcCompat="@drawable/trans_ref"/>
    <ImageView android:id="@+id/src_layer" app:srcCompat="@drawable/layer_wrap_img"/>
    <ImageView android:id="@+id/src_sel" app:srcCompat="@drawable/sel_wrap_img"/>
    <TextView android:id="@+id/tv_cmp" android:drawableStart="@drawable/lvl_ref"/>
    <CheckBox android:id="@+id/cb_btn" android:button="@drawable/rot_ref"/>
    <SeekBar android:id="@+id/sb_tick" android:tickMark="@drawable/lvl_ref"/>
    <ExpandableListView android:id="@+id/elv_ind" android:groupIndicator="@drawable/clip_ref"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['bg_rot_ref','bg_rot_vec','bg_scale','bg_clip','bg_lvl','bg_trans','bg_layer','bg_sel','src_rot_vec','src_scale','src_clip','src_lvl','src_trans','src_layer','src_sel','tv_cmp','cb_btn','sb_tick','elv_ind']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('background:Some(Color(17,34,51,255))',slices['bg_rot_ref'])
   self.assertIn('RelativeRule { kind:255, anchor:"true" }',slices['bg_rot_ref'])
   self.assertIn('RelativeRule { kind:251, anchor:"rot_inline_vec|" }',slices['bg_rot_vec'])
   self.assertIn('background:Some(Color(51,68,85,255))',slices['bg_scale'])
   self.assertIn('background:Some(Color(85,102,119,255))',slices['bg_clip'])
   self.assertIn('background:Some(Color(68,85,102,255))',slices['bg_lvl'])
   self.assertIn('background:Some(Color(119,136,153,255))',slices['bg_trans'])
   self.assertIn('RelativeRule { kind:208, anchor:"0.0,0.0,0.0,0.0|6.0|17,34,51,255" }',slices['bg_layer'])
   self.assertIn('RelativeRule { kind:208, anchor:"4.0,4.0,4.0,4.0|0.0|170,17,34,255" }',slices['bg_layer'])
   self.assertIn('RelativeRule { kind:208, anchor:"0.0,0.0,0.0,0.0|0.0|34,170,51,255" }',slices['bg_layer'])
   self.assertIn('RelativeRule { kind:231, anchor:"2,0|0.0,0.0,0.0,0.0|2.0|68,85,102,255" }',slices['bg_sel'])
   self.assertIn('RelativeRule { kind:231, anchor:"4,0|0.0,0.0,0.0,0.0|6.0|17,34,51,255" }',slices['bg_sel'])
   self.assertIn('RelativeRule { kind:231, anchor:"0,0|0.0,0.0,0.0,0.0|0.0|85,102,119,255" }',slices['bg_sel'])
   for img_id,img_key in [('src_rot_vec','rot_inline_vec'),('src_scale','scale_ref'),('src_clip','clip_ref'),('src_lvl','lvl_fallback'),('src_trans','trans_ref'),('src_layer','layer_wrap_img')]:
    self.assertIn(f'image_src:"{img_key}"',slices[img_id])
    self.assertIn(f'state.images.entry("{img_key}".into())',out)
   self.assertIn('RelativeRule { kind:253, anchor:"2,0|clip_ref|" }',slices['src_sel'])
   self.assertIn('RelativeRule { kind:253, anchor:"8,0|sel_wrap_img__s1|" }',slices['src_sel'])
   self.assertIn('RelativeRule { kind:253, anchor:"0,0|lvl_fallback|" }',slices['src_sel'])
   self.assertIn('RelativeRule { kind:173, anchor:"16.0,16.0,2.0,68,85,102,255" }',slices['tv_cmp'])
   self.assertIn('RelativeRule { kind:165, anchor:"0,0,16.0,16.0,6.0,17,34,51,255" }',slices['cb_btn'])
   self.assertIn('RelativeRule { kind:155, anchor:"16.0,16.0,2.0,68,85,102,255" }',slices['sb_tick'])
   self.assertIn('RelativeRule { kind:131, anchor:"0,0,px_a" }',slices['elv_ind'])
   # Strict validation rejections for malformed wrapper/list drawables
   for bad_drw,err in [
    ('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:fromDegrees="spin" android:drawable="@drawable/box_a"/>','Invalid rotate drawable degrees'),
    ('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:pivotX="center" android:drawable="@drawable/box_a"/>','Invalid rotate drawable pivot'),
    ('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:scaleWidth="wide" android:drawable="@drawable/box_a"/>','Invalid scale drawable value'),
    ('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:scaleGravity="diagonal" android:drawable="@drawable/box_a"/>','Unsupported scaleGravity'),
    ('<clip xmlns:android="http://schemas.android.com/apk/res/android" android:clipOrientation="diagonal" android:drawable="@drawable/box_a"/>','Unsupported clipOrientation'),
    ('<clip xmlns:android="http://schemas.android.com/apk/res/android" android:gravity="diagonal" android:drawable="@drawable/box_a"/>','Unsupported clip gravity'),
    ('<level-list xmlns:android="http://schemas.android.com/apk/res/android"/>','Invalid level-list drawable'),
    ('<level-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:minLevel="low" android:drawable="@drawable/box_a"/></level-list>','Invalid level-list level'),
    ('<transition xmlns:android="http://schemas.android.com/apk/res/android"/>','Invalid transition drawable'),
    ('<transition xmlns:android="http://schemas.android.com/apk/res/android"><item android:bogus="1" android:drawable="@drawable/box_a"/></transition>','Unsupported transition item attribute'),
   ]:
    (res/'drawable'/'bad_w.xml').write_text(bad_drw)
    with self.subTest(bad=bad_drw):
     with self.assertRaisesRegex(ValueError,err):
      self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/bad_w"/>',res)
 def test_wave16b_app_bar_layout_lifted_lift_on_scroll_and_scroll_flags(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('''<resources>
    <style name="LiftedAppBar" parent="Widget.Material3.AppBarLayout">
     <item name="android:elevation">0dp</item>
     <item name="lifted">true</item>
     <item name="liftOnScroll">true</item>
     <item name="liftOnScrollTargetViewId">@id/feed_list</item>
    </style>
   </resources>''')
   xml='''<androidx.coordinatorlayout.widget.CoordinatorLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_lifted" app:liftOnScroll="true" app:liftOnScrollTargetViewId="@id/feed_list" app:lifted="true">
     <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_bar" app:title="Inbox" app:layout_scrollFlags="scroll|exitUntilCollapsed|snap">
      <com.google.android.material.appbar.MaterialToolbar android:id="@+id/mtb_child" app:layout_collapseMode="pin" app:layout_scrollFlags="scroll|enterAlways|snapMargins"/>
     </com.google.android.material.appbar.CollapsingToolbarLayout>
     <androidx.appcompat.widget.Toolbar android:id="@+id/atb_child" app:layout_scrollFlags="scroll|enterAlwaysCollapsed"/>
    </com.google.android.material.appbar.AppBarLayout>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_explicit_elev" app:lifted="true" app:elevation="8dp"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_unlifted" app:liftOnScroll="true" app:lifted="false"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_styled" style="@style/LiftedAppBar"/>
    <androidx.core.widget.NestedScrollView android:id="@+id/feed_list" app:layout_behavior="@string/appbar_scrolling_view_behavior">
     <TextView android:text="Messages"/>
    </androidx.core.widget.NestedScrollView>
   </androidx.coordinatorlayout.widget.CoordinatorLayout>'''
   out=self.run_layout(xml,res)
   names=['ab_lifted','ctl_bar','mtb_child','atb_child','ab_explicit_elev','ab_unlifted','ab_styled','feed_list']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:193, anchor:"4.0" }',slices['ab_lifted'])
   self.assertIn('RelativeRule { kind:245, anchor:"Inbox|152|" }',slices['ctl_bar'])
   self.assertIn('kind:Kind::Toolbar,id:"mtb_child"',out)
   self.assertIn('kind:Kind::Toolbar,id:"atb_child"',out)
   self.assertIn('RelativeRule { kind:193, anchor:"8.0" }',slices['ab_explicit_elev'])
   self.assertNotIn('kind:193',slices['ab_unlifted'])
   self.assertIn('RelativeRule { kind:193, anchor:"4.0" }',slices['ab_styled'])
   self.assertIn('RelativeRule { kind:226, anchor:"" }',slices['feed_list'])
   for bad_xml,err in [
    ('<com.google.android.material.appbar.AppBarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:lifted="maybe"/>','Invalid lifted'),
    ('<com.google.android.material.appbar.AppBarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:liftOnScroll="maybe"/>','Invalid liftOnScroll'),
    ('<com.google.android.material.appbar.AppBarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:liftOnScrollTargetViewId="not_an_id"/>','Invalid liftOnScrollTargetViewId'),
    ('<com.google.android.material.appbar.MaterialToolbar xmlns:app="http://schemas.android.com/apk/res-auto" app:layout_scrollFlags="scroll|bogus"/>','Unsupported layout_scrollFlags'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave17b_radial_and_sweep_gradients(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('<resources><dimen name="rad_dim">72dp</dimen><color name="g_mid">#8000ff00</color></resources>')
   (res/'drawable'/'lin_keep.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="linear" android:angle="270" android:startColor="#ff0000" android:centerColor="#00ff00" android:endColor="#0000ff" android:centerY="0.4"/></shape>')
   (res/'drawable'/'rad_def.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="radial" android:startColor="#112233" android:endColor="#445566"/></shape>')
   (res/'drawable'/'rad_dp.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="radial" android:centerX="25%" android:centerY="0.75" android:gradientRadius="@dimen/rad_dim" android:startColor="#ff0000" android:centerColor="@color/g_mid" android:endColor="#0000ff"/></shape>')
   (res/'drawable'/'rad_px.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="radial" android:gradientRadius="48px" android:startColor="#ffffff" android:endColor="#000000"/></shape>')
   (res/'drawable'/'rad_flt.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="radial" android:gradientRadius="45.5" android:startColor="#ffffff" android:endColor="#000000"/></shape>')
   (res/'drawable'/'rad_pctp.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="radial" android:centerX="50%" android:centerY="50%" android:gradientRadius="80%p" android:startColor="#102030" android:endColor="#aabbcc"/></shape>')
   (res/'drawable'/'sw_2.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="sweep" android:startColor="#ff0000" android:endColor="#0000ff"/></shape>')
   (res/'drawable'/'sw_3.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="sweep" android:centerX="0.2" android:centerY="80%" android:startColor="#ff0000" android:centerColor="#00ff00" android:endColor="#0000ff"/></shape>')
   (res/'drawable'/'rad_inset.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android" android:inset="4dp" android:drawable="@drawable/rad_dp"/>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <View android:id="@+id/v_lin" android:background="@drawable/lin_keep"/>
    <View android:id="@+id/v_rdef" android:background="@drawable/rad_def"/>
    <View android:id="@+id/v_rdp" android:background="@drawable/rad_dp"/>
    <View android:id="@+id/v_rpx" android:background="@drawable/rad_px"/>
    <View android:id="@+id/v_rflt" android:background="@drawable/rad_flt"/>
    <View android:id="@+id/v_rpctp" android:background="@drawable/rad_pctp"/>
    <View android:id="@+id/v_sw2" android:background="@drawable/sw_2"/>
    <View android:id="@+id/v_sw3" android:background="@drawable/sw_3"/>
    <View android:id="@+id/v_rins" android:background="@drawable/rad_inset"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['v_lin','v_rdef','v_rdp','v_rpx','v_rflt','v_rpctp','v_sw2','v_sw3','v_rins']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   # Linear format unchanged
   self.assertIn('RelativeRule { kind:207, anchor:"270.0,255,0,0,255,0,255,0,255,0,0,255,255,0.4" }',slices['v_lin'])
   # Radial variants
   self.assertIn('RelativeRule { kind:207, anchor:"radial|0.5,0.5,0|17,34,51,255||68,85,102,255" }',slices['v_rdef'])
   self.assertIn('RelativeRule { kind:207, anchor:"radial|0.25,0.75,72.0|255,0,0,255|0,255,0,128|0,0,255,255" }',slices['v_rdp'])
   self.assertIn('RelativeRule { kind:207, anchor:"radial|0.5,0.5,48.0|255,255,255,255||0,0,0,255" }',slices['v_rpx'])
   self.assertIn('RelativeRule { kind:207, anchor:"radial|0.5,0.5,45.5|255,255,255,255||0,0,0,255" }',slices['v_rflt'])
   self.assertIn('RelativeRule { kind:207, anchor:"radial|0.5,0.5,0|16,32,48,255||170,187,204,255" }',slices['v_rpctp'])
   # Sweep variants
   self.assertIn('RelativeRule { kind:207, anchor:"sweep|0.5,0.5|255,0,0,255||0,0,255,255" }',slices['v_sw2'])
   self.assertIn('RelativeRule { kind:207, anchor:"sweep|0.2,0.8|255,0,0,255|0,255,0,255|0,0,255,255" }',slices['v_sw3'])
   # Wrapped radial inside <inset>
   self.assertIn('RelativeRule { kind:207, anchor:"radial|0.25,0.75,72.0|255,0,0,255|0,255,0,128|0,0,255,255" }',slices['v_rins'])
   # Density scaling of px gradientRadius (48px at 320dpi == 24.0dp)
   lp=Path(d)/'px_dens.xml';lp.write_text('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/rad_px"/>')
   out_320=mod.compile_layout(lp,res,density=320)
   self.assertIn('RelativeRule { kind:207, anchor:"radial|0.5,0.5,24.0|255,255,255,255||0,0,0,255" }',out_320)
   # Strict rejections
   for bad_g,err in [
    ('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="radial" android:centerX="1.5" android:startColor="#fff" android:endColor="#000"/></shape>','Invalid gradient center position'),
    ('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="sweep" android:centerY="120%" android:startColor="#fff" android:endColor="#000"/></shape>','Invalid gradient center position'),
    ('<shape xmlns:android="http://schemas.android.com/apk/res/android"><gradient android:type="radial" android:gradientRadius="-10dp" android:startColor="#fff" android:endColor="#000"/></shape>','Invalid gradientRadius'),
   ]:
    (res/'drawable'/'bad_g.xml').write_text(bad_g)
    with self.subTest(bad=bad_g):
     with self.assertRaisesRegex(ValueError,err):
      self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/bad_g"/>',res)
 def test_wave17b_cut_corner_family_rule_278(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'shapes.xml').write_text('''<resources>
    <style name="Shape.CutAll" parent="">
     <item name="cornerFamily">cut</item>
     <item name="cornerSize">12dp</item>
    </style>
    <style name="Shape.CutEnum" parent="">
     <item name="cornerFamily">1</item>
     <item name="cornerSize">10dp</item>
    </style>
    <style name="Shape.RoundedEnum" parent="">
     <item name="cornerFamily">0</item>
     <item name="cornerSize">10dp</item>
    </style>
    <style name="Shape.CutDiag" parent="ShapeAppearance.Material3.Corner.Large">
     <item name="cornerFamilyTopLeft">cut</item>
     <item name="cornerFamilyBottomRight">cut</item>
    </style>
    <style name="ShapeOverlay.RightRounded" parent="">
     <item name="cornerFamilyTopRight">rounded</item>
     <item name="cornerFamilyBottomRight">rounded</item>
    </style>
    <style name="ShapeOverlay.AllRounded" parent="">
     <item name="cornerFamily">rounded</item>
    </style>
    <style name="ShapeOverlay.OnlyCutFamily" parent="">
     <item name="cornerFamily">cut</item>
    </style>
    <style name="Widget.CutBtnStyle" parent="Widget.Material3.Button">
     <item name="cornerFamily">cut</item>
     <item name="cornerFamilyTopRight">rounded</item>
     <item name="cornerSize">14dp</item>
    </style>
    <style name="Widget.ShapeAppCutBtn" parent="Widget.Material3.Button">
     <item name="shapeAppearance">@style/Shape.CutDiag</item>
    </style>
    <style name="Shape.BadCornerFam" parent="">
     <item name="cornerFamilyTopLeft">bevel</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <View android:id="@+id/v_all_cut" app:shapeAppearance="@style/Shape.CutAll"/>
    <View android:id="@+id/v_enum_cut" app:shapeAppearance="@style/Shape.CutEnum"/>
    <View android:id="@+id/v_enum_rnd" app:shapeAppearance="@style/Shape.RoundedEnum"/>
    <View android:id="@+id/v_m3_rnd" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.Large"/>
    <View android:id="@+id/v_diag_cut" app:shapeAppearance="@style/Shape.CutDiag"/>
    <View android:id="@+id/v_cut_ov_mix" app:shapeAppearance="@style/Shape.CutAll" app:shapeAppearanceOverlay="@style/ShapeOverlay.RightRounded"/>
    <View android:id="@+id/v_cut_ov_rnd" app:shapeAppearance="@style/Shape.CutAll" app:shapeAppearanceOverlay="@style/ShapeOverlay.AllRounded"/>
    <View android:id="@+id/v_direct_all" app:cornerFamily="cut" app:cornerSize="8dp"/>
    <View android:id="@+id/v_direct_mix" app:shapeAppearance="@style/ShapeAppearance.Material3.Corner.Large" app:cornerFamilyTopLeft="cut" app:cornerFamilyBottomLeft="cut"/>
    <com.google.android.material.button.MaterialButton android:id="@+id/btn_styled_direct" style="@style/Widget.CutBtnStyle" android:text="Go"/>
    <com.google.android.material.button.MaterialButton android:id="@+id/btn_styled_sa" style="@style/Widget.ShapeAppCutBtn" android:text="Go"/>
    <com.google.android.material.button.MaterialButton android:id="@+id/btn_exp_rad" app:cornerRadius="6dp" app:shapeAppearanceOverlay="@style/ShapeOverlay.OnlyCutFamily" android:text="Go"/>
    <com.google.android.material.card.MaterialCardView android:id="@+id/card_cut" app:cardCornerRadius="12dp" app:shapeAppearanceOverlay="@style/ShapeOverlay.OnlyCutFamily"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['v_all_cut','v_enum_cut','v_enum_rnd','v_m3_rnd','v_diag_cut','v_cut_ov_mix','v_cut_ov_rnd','v_direct_all','v_direct_mix','btn_styled_direct','btn_styled_sa','btn_exp_rad','card_cut']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn(',radius:12.0,',slices['v_all_cut'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,c,c,c" }',slices['v_all_cut'])
   self.assertIn(',radius:10.0,',slices['v_enum_cut'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,c,c,c" }',slices['v_enum_cut'])
   self.assertIn(',radius:10.0,',slices['v_enum_rnd'])
   self.assertNotIn('kind:278',slices['v_enum_rnd'])
   self.assertNotIn('kind:278',slices['v_m3_rnd'])
   self.assertIn(',radius:16.0,',slices['v_diag_cut'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,r,c,r" }',slices['v_diag_cut'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,r,r,c" }',slices['v_cut_ov_mix'])
   self.assertNotIn('kind:278',slices['v_cut_ov_rnd'])
   self.assertIn(',radius:8.0,',slices['v_direct_all'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,c,c,c" }',slices['v_direct_all'])
   self.assertIn(',radius:16.0,',slices['v_direct_mix'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,r,r,c" }',slices['v_direct_mix'])
   self.assertIn(',radius:14.0,',slices['btn_styled_direct'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,r,c,c" }',slices['btn_styled_direct'])
   self.assertIn(',radius:16.0,',slices['btn_styled_sa'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,r,c,r" }',slices['btn_styled_sa'])
   self.assertIn(',radius:6.0,',slices['btn_exp_rad'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,c,c,c" }',slices['btn_exp_rad'])
   self.assertIn('RelativeRule { kind:66, anchor:"12.0" }',slices['card_cut'])
   self.assertIn('RelativeRule { kind:278, anchor:"c,c,c,c" }',slices['card_cut'])
   with self.assertRaisesRegex(ValueError,'Unsupported cornerFamily'):
    self.run_layout('<View xmlns:app="http://schemas.android.com/apk/res-auto" app:shapeAppearance="@style/Shape.BadCornerFam"/>',res)
   with self.assertRaisesRegex(ValueError,'Unsupported cornerFamily'):
    self.run_layout('<View xmlns:app="http://schemas.android.com/apk/res-auto" app:cornerFamily="bevel"/>',res)
 def test_wave17b_multi_band_nine_patch(self):
  def make_rgba_png(w,h,rgba_grid):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=bytearray()
   for y in range(h):
    rows.append(0)
    for x in range(w):rows.extend(rgba_grid(x,y))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(bytes(rows)))+chunk(b'IEND',b'')
  def make_multi_9(inner_w,inner_h,top_spans,left_spans,bottom_span=None,right_span=None,fill=(20,40,60,255)):
   w,h=inner_w+2,inner_h+2
   def in_spans(v,spans):return any(a<=v<=b for a,b in spans)
   def px(x,y):
    if y==0:
     if 1<=x<=w-2 and in_spans(x-1,top_spans):return (0,0,0,255)
     if x==1:return (255,0,0,255)
     return (0,0,0,0)
    if x==0:
     if 1<=y<=h-2 and in_spans(y-1,left_spans):return (0,0,0,255)
     return (0,0,0,0)
    if y==h-1:
     if 1<=x<=w-2 and bottom_span is not None and bottom_span[0]<=x-1<=bottom_span[1]:return (0,0,0,255)
     return (0,0,0,0)
    if x==w-1:
     if 1<=y<=h-2 and right_span is not None and right_span[0]<=y-1<=right_span[1]:return (0,0,0,255)
     return (0,0,0,0)
    return fill
   return make_rgba_png(w,h,px)
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   p_multi=res/'drawable'/'multi9.9.png'
   p_multi.write_bytes(make_multi_9(16,12,[(2,4),(10,13)],[(1,2),(7,9)],(2,13),(1,10)))
   p_single=res/'drawable'/'single9.9.png'
   p_single.write_bytes(make_multi_9(16,12,[(3,11)],[(2,9)],(2,13),(1,10)))
   p_tri=res/'drawable'/'tri9.9.png'
   p_tri.write_bytes(make_multi_9(16,10,[(1,2),(6,6),(11,13)],[(0,1),(5,5)]))
   p_empty=res/'drawable'/'empty9.9.png'
   p_empty.write_bytes(make_multi_9(8,6,[],[]))
   # Direct android_png.decode_nine_patch_bands verification + backwards compatibility
   w,h,stripped,pad,xb,yb=mod.decode_nine_patch_bands(p_multi)
   self.assertEqual((w,h),(16,12))
   self.assertEqual(len(stripped),16*12*4)
   self.assertEqual(pad,[2,1,2,1])
   self.assertEqual(xb,[(2,5),(10,14)])
   self.assertEqual(yb,[(1,3),(7,10)])
   _,_,_,_,sx,sy=mod.decode_nine_patch_full(p_multi)
   self.assertEqual((sx,sy),((2,14),(1,10)))
   self.assertEqual(len(mod.decode_nine_patch(p_multi)),4)
   _,_,_,_,txb,tyb=mod.decode_nine_patch_bands(p_tri)
   self.assertEqual(txb,[(1,3),(6,7),(11,14)])
   self.assertEqual(tyb,[(0,2),(5,6)])
   _,_,_,_,exb,eyb=mod.decode_nine_patch_bands(p_empty)
   self.assertEqual((exb,eyb),([(0,8)],[(0,6)]))
   # Layout emission: background (251), bitmap bg (251), src (252), selector (253/254), single-band unchanged
   (res/'drawable'/'bmp_multi9.xml').write_text('<bitmap xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/multi9"/>')
   (res/'drawable'/'sel_multi9.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_pressed="true" android:drawable="@drawable/multi9"/><item android:drawable="@drawable/single9"/></selector>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <View android:id="@+id/bg_multi" android:background="@drawable/multi9"/>
    <View android:id="@+id/bg_bmp_multi" android:background="@drawable/bmp_multi9"/>
    <View android:id="@+id/bg_single" android:background="@drawable/single9"/>
    <View android:id="@+id/bg_sel_multi" android:background="@drawable/sel_multi9"/>
    <ImageView android:id="@+id/iv_multi" android:src="@drawable/multi9"/>
    <ImageButton android:id="@+id/ib_tri" app:srcCompat="@drawable/tri9"/>
    <ImageView android:id="@+id/iv_sel_multi" android:src="@drawable/sel_multi9"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['bg_multi','bg_bmp_multi','bg_single','bg_sel_multi','iv_multi','ib_tri','iv_sel_multi']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:251, anchor:"multi9|2,5,10,14;1,3,7,10" }',slices['bg_multi'])
   self.assertIn('RelativeRule { kind:251, anchor:"bmp_multi9|2,5,10,14;1,3,7,10" }',slices['bg_bmp_multi'])
   self.assertIn('RelativeRule { kind:251, anchor:"single9|3,12;2,10" }',slices['bg_single'])
   self.assertIn('RelativeRule { kind:254, anchor:"2,0|multi9|2,5,10,14;1,3,7,10" }',slices['bg_sel_multi'])
   self.assertIn('RelativeRule { kind:254, anchor:"0,0|single9|3,12;2,10" }',slices['bg_sel_multi'])
   self.assertIn('RelativeRule { kind:252, anchor:"2,5,10,14;1,3,7,10" }',slices['iv_multi'])
   self.assertIn('RelativeRule { kind:252, anchor:"1,3,6,7,11,14;0,2,5,6" }',slices['ib_tri'])
   self.assertIn('RelativeRule { kind:253, anchor:"2,0|multi9|2,5,10,14;1,3,7,10" }',slices['iv_sel_multi'])
   self.assertIn('RelativeRule { kind:253, anchor:"0,0|single9|3,12;2,10" }',slices['iv_sel_multi'])
 def test_wave17b_app_bar_lift_on_scroll_rule_279(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <bool name="enable_lift">true</bool>
    <style name="AppBar.LiftOnly" parent="Widget.Material3.AppBarLayout">
     <item name="liftOnScroll">true</item>
    </style>
    <style name="AppBar.LiftTarget" parent="Widget.Material3.AppBarLayout">
     <item name="liftOnScroll">true</item>
     <item name="liftOnScrollTargetViewId">@id/styled_feed</item>
    </style>
   </resources>''')
   xml='''<androidx.coordinatorlayout.widget.CoordinatorLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_on" app:liftOnScroll="true"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_target" app:liftOnScroll="true" app:liftOnScrollTargetViewId="@id/feed_list"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_plus_target" app:liftOnScroll="@bool/enable_lift" app:liftOnScrollTargetViewId="@+id/new_feed" app:lifted="true"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_null_target" app:liftOnScroll="true" app:liftOnScrollTargetViewId="@null"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_off" app:liftOnScroll="false" app:liftOnScrollTargetViewId="@id/feed_list"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_plain"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_style_on" style="@style/AppBar.LiftOnly"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_style_target" style="@style/AppBar.LiftTarget"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_style_override_off" style="@style/AppBar.LiftTarget" app:liftOnScroll="false"/>
    <androidx.core.widget.NestedScrollView android:id="@+id/feed_list"><TextView android:text="Feed"/></androidx.core.widget.NestedScrollView>
    <androidx.core.widget.NestedScrollView android:id="@+id/styled_feed"><TextView android:text="Styled"/></androidx.core.widget.NestedScrollView>
   </androidx.coordinatorlayout.widget.CoordinatorLayout>'''
   out=self.run_layout(xml,res)
   names=['ab_on','ab_target','ab_plus_target','ab_null_target','ab_off','ab_plain','ab_style_on','ab_style_target','ab_style_override_off','feed_list']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:279, anchor:"1" }',slices['ab_on'])
   self.assertIn('RelativeRule { kind:279, anchor:"1|feed_list" }',slices['ab_target'])
   self.assertIn('RelativeRule { kind:279, anchor:"1|new_feed" }',slices['ab_plus_target'])
   self.assertIn('RelativeRule { kind:193, anchor:"4.0" }',slices['ab_plus_target'])
   self.assertIn('RelativeRule { kind:279, anchor:"1" }',slices['ab_null_target'])
   self.assertNotIn('kind:279',slices['ab_off'])
   self.assertNotIn('kind:279',slices['ab_plain'])
   self.assertIn('RelativeRule { kind:279, anchor:"1" }',slices['ab_style_on'])
   self.assertIn('RelativeRule { kind:279, anchor:"1|styled_feed" }',slices['ab_style_target'])
   self.assertNotIn('kind:279',slices['ab_style_override_off'])
 def test_wave18b_locale_and_ldrtl_resource_qualifiers(self):
  import subprocess,sys
  def make_png(w,h,rgba):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=b''.join(b'\x00'+bytes(rgba)*w for _ in range(h))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')
  with tempfile.TemporaryDirectory() as d:
   res=Path(d)
   for sub in (
    'values','values-fr','values-fr-rCA','values-de','values-ar','values-iw','values-ldrtl','values-ldltr',
    'values-fr-ldrtl','values-night','values-fr-ldrtl-night-v29','values-b+zh+Hans+CN','values-b+es+419','values-rCA',
    'color','color-ldrtl','drawable','drawable-xhdpi','drawable-ldrtl-xhdpi','drawable-fr','layout','layout-ldrtl','layout-fr',
   ):
    (res/sub).mkdir()
   (res/'values'/'s.xml').write_text('<resources><string name="greet">Hello</string><string name="fallback">DefaultOnly</string><string name="dir_lbl">DirBase</string><string name="only_fr">BaseOnlyFr</string></resources>')
   (res/'values-fr'/'s.xml').write_text('<resources><string name="greet">Bonjour</string><string name="only_fr">FrenchFallback</string></resources>')
   (res/'values-fr-rCA'/'s.xml').write_text('<resources><string name="greet">Allo</string></resources>')
   (res/'values-de'/'s.xml').write_text('<resources><string name="greet">Hallo</string></resources>')
   (res/'values-ar'/'s.xml').write_text('<resources><string name="greet">Marhaba</string></resources>')
   (res/'values-iw'/'s.xml').write_text('<resources><string name="greet">Shalom</string></resources>')
   (res/'values-ldrtl'/'s.xml').write_text('<resources><string name="dir_lbl">DirRTL</string><string name="greet">GreetRTL</string></resources>')
   (res/'values-ldltr'/'s.xml').write_text('<resources><string name="dir_lbl">DirLTR</string></resources>')
   (res/'values-fr-ldrtl'/'s.xml').write_text('<resources><string name="greet">BonjourRTL</string></resources>')
   (res/'values-night'/'s.xml').write_text('<resources><string name="greet">NightHello</string></resources>')
   (res/'values-fr-ldrtl-night-v29'/'s.xml').write_text('<resources><string name="greet">BonsoirRTL</string></resources>')
   (res/'values-b+zh+Hans+CN'/'s.xml').write_text('<resources><string name="greet">NiHao</string></resources>')
   (res/'values-b+es+419'/'s.xml').write_text('<resources><string name="greet">HolaLatam</string></resources>')
   (res/'values-rCA'/'s.xml').write_text('<resources><string name="greet">InvalidRegionOnly</string></resources>')
   (res/'color'/'ink.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:color="#102030"/></selector>')
   (res/'color-ldrtl'/'ink.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:color="#a0b0c0"/></selector>')
   (res/'drawable-xhdpi'/'arrow.png').write_bytes(make_png(4,4,(10,20,30,255)))
   (res/'drawable-ldrtl-xhdpi'/'arrow.png').write_bytes(make_png(4,4,(200,100,50,255)))
   (res/'drawable'/'flag.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#111111"/></shape>')
   (res/'drawable-fr'/'flag.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#0055aa"/></shape>')
   (res/'layout'/'part.xml').write_text('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="PartBase"/>')
   (res/'layout-ldrtl'/'part.xml').write_text('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="PartRTL"/>')
   (res/'layout-fr'/'part.xml').write_text('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:text="PartFR"/>')
   main=res/'layout'/'main.xml'
   main.write_text('''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/flag">
    <TextView android:id="@+id/t_greet" android:text="@string/greet" android:textColor="@color/ink"/>
    <TextView android:id="@+id/t_dir" android:text="@string/dir_lbl"/>
    <TextView android:id="@+id/t_fr" android:text="@string/only_fr"/>
    <TextView android:id="@+id/t_fb" android:text="@string/fallback"/>
    <ImageView android:id="@+id/iv_arrow" android:src="@drawable/arrow"/>
    <include layout="@layout/part"/>
   </LinearLayout>''')
   (res/'layout-ldrtl'/'main.xml').write_text('''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="horizontal" android:background="@drawable/flag">
    <TextView android:id="@+id/t_greet" android:text="@string/greet" android:textColor="@color/ink"/>
    <TextView android:id="@+id/t_dir" android:text="@string/dir_lbl"/>
    <ImageView android:id="@+id/iv_arrow" android:src="@drawable/arrow"/>
    <include layout="@layout/part"/>
   </LinearLayout>''')
   # Default: no locale, LTR -> ignores fr/ar/de/ldrtl/rCA; picks values-ldltr over values
   out_def=mod.compile_layout(main,res,density=320)
   for tok in ('horizontal:false','text:"Hello"','text:"DirLTR"','text:"BaseOnlyFr"','text:"DefaultOnly"','text:"PartBase"','Color(16,32,48,255)','Color(17,17,17,255)','vec![10,20,30,255'):
    self.assertIn(tok,out_def)
   # Language only: fr -> picks values-fr, drawable-fr, layout-fr; ignores values-fr-rCA
   out_fr=mod.compile_layout(main,res,density=320,locale='fr')
   for tok in ('text:"Bonjour"','text:"FrenchFallback"','text:"DirLTR"','text:"PartFR"','Color(0,85,170,255)'):
    self.assertIn(tok,out_fr)
   # Language + region: fr-rCA (and fr-CA / fr_CA) -> fr-rCA > fr > default
   for loc in ('fr-rCA','fr-CA','fr_CA'):
    out_frca=mod.compile_layout(main,res,locale=loc)
    self.assertIn('text:"Allo"',out_frca)
    self.assertIn('text:"FrenchFallback"',out_frca)
    self.assertIn('text:"DefaultOnly"',out_frca)
   # Explicit ldrtl=True without locale -> picks layout-ldrtl, values-ldrtl, color-ldrtl, drawable-ldrtl-xhdpi
   out_rtl=mod.compile_layout(main,res,density=320,ldrtl=True)
   for tok in ('horizontal:true','text:"GreetRTL"','text:"DirRTL"','text:"PartRTL"','Color(160,176,192,255)','vec![200,100,50,255'):
    self.assertIn(tok,out_rtl)
   # Arabic locale auto-implies ldrtl=True, and values-ar beats values-ldrtl per AOSP precedence
   out_ar=mod.compile_layout(main,res,density=320,locale='ar')
   for tok in ('horizontal:true','text:"Marhaba"','text:"DirRTL"','text:"PartRTL"','vec![200,100,50,255'):
    self.assertIn(tok,out_ar)
   # Hebrew 'he' matches legacy values-iw and auto-implies ldrtl=True
   out_he=mod.compile_layout(main,res,locale='he')
   self.assertIn('text:"Shalom"',out_he)
   self.assertIn('text:"DirRTL"',out_he)
   # Explicit ldrtl=False overrides Arabic auto-RTL
   out_ar_ltr=mod.compile_layout(main,res,density=320,locale='ar',ldrtl=False)
   for tok in ('horizontal:false','text:"Marhaba"','text:"DirLTR"','text:"PartBase"','vec![10,20,30,255'):
    self.assertIn(tok,out_ar_ltr)
   # Combined locale + ldrtl + night -> values-fr-ldrtl-night-v29 wins over all
   out_combo=mod.compile_layout(main,res,locale='fr',ldrtl=True,night=True)
   self.assertIn('text:"BonsoirRTL"',out_combo)
   self.assertIn('text:"DirRTL"',out_combo)
   # BCP-47 script/region qualifiers
   self.assertIn('text:"NiHao"',mod.compile_layout(main,res,locale='zh-Hans-CN'))
   self.assertIn('text:"HolaLatam"',mod.compile_layout(main,res,locale='es-r419'))
   # CLI flags: --locale, --ldrtl, --ldltr, combined with --night
   out_file=res/'cli.rs'
   subprocess.run([sys.executable,str(Path(__file__).with_name('import-android-layout.py')),str(main),str(out_file),'--resources',str(res),'--locale','fr','--ldrtl','--night'],capture_output=True,text=True,check=True)
   self.assertIn('text:"BonsoirRTL"',out_file.read_text())
   subprocess.run([sys.executable,str(Path(__file__).with_name('import-android-layout.py')),str(main),str(out_file),'--resources',str(res),'--locale','ar','--ldltr'],capture_output=True,text=True,check=True)
   self.assertIn('text:"Marhaba"',out_file.read_text())
   self.assertIn('text:"DirLTR"',out_file.read_text())
   # Strict validation errors
   for bad_loc in ('123','english','fr-rBAD','b+12'):
    with self.assertRaisesRegex(ValueError,'Invalid locale'):mod.compile_layout(main,res,locale=bad_loc)
   with self.assertRaisesRegex(ValueError,'Invalid ldrtl'):mod.compile_layout(main,res,ldrtl='yes')
   bad_cli=subprocess.run([sys.executable,str(Path(__file__).with_name('import-android-layout.py')),str(main),str(out_file),'--resources',str(res),'--ldrtl','--ldltr'],capture_output=True,text=True)
   self.assertNotEqual(bad_cli.returncode,0)
 def test_wave18b_enhanced_barrier_rule_224(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('<resources><dimen name="bm">12dp</dimen><bool name="no_gone">false</bool></resources>')
   xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <View android:id="@+id/a"/><View android:id="@+id/b"/>
    <androidx.constraintlayout.widget.Barrier android:id="@+id/b_def" app:barrierDirection="end" app:constraint_referenced_ids="a,b"/>
    <androidx.constraintlayout.widget.Barrier android:id="@+id/b_exp_def" app:barrierDirection="end" app:barrierMargin="0dp" app:barrierAllowsGoneWidgets="true" app:constraint_referenced_ids="a,b"/>
    <androidx.constraintlayout.widget.Barrier android:id="@+id/b_margin" app:barrierDirection="end" app:barrierMargin="@dimen/bm" app:constraint_referenced_ids="a,b"/>
    <androidx.constraintlayout.widget.Barrier android:id="@+id/b_neg" app:barrierDirection="start" app:barrierMargin="-4.5dp" app:constraint_referenced_ids="a,b"/>
    <androidx.constraintlayout.widget.Barrier android:id="@+id/b_nogone" app:barrierDirection="bottom" app:barrierAllowsGoneWidgets="@bool/no_gone" app:constraint_referenced_ids="a,b"/>
    <androidx.constraintlayout.widget.Barrier android:id="@+id/b_both" app:barrierDirection="top" app:barrierMargin="8dp" app:barrierAllowsGoneWidgets="false" app:constraint_referenced_ids="a,b"/>
   </androidx.constraintlayout.widget.ConstraintLayout>'''
   out=self.run_layout(xml,res)
   names=['b_def','b_exp_def','b_margin','b_neg','b_nogone','b_both']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:224, anchor:"end:a,b" }',slices['b_def'])
   self.assertIn('RelativeRule { kind:224, anchor:"end:a,b" }',slices['b_exp_def'])
   self.assertIn('RelativeRule { kind:224, anchor:"end:a,b|12|1" }',slices['b_margin'])
   self.assertIn('RelativeRule { kind:224, anchor:"start:a,b|-4.5|1" }',slices['b_neg'])
   self.assertIn('RelativeRule { kind:224, anchor:"bottom:a,b|0|0" }',slices['b_nogone'])
   self.assertIn('RelativeRule { kind:224, anchor:"top:a,b|8|0" }',slices['b_both'])
   with self.assertRaisesRegex(ValueError,'Invalid barrierAllowsGoneWidgets'):
    self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><androidx.constraintlayout.widget.Barrier app:barrierAllowsGoneWidgets="maybe"/></ConstraintLayout>')
   with self.assertRaisesRegex(ValueError,'Expected margin'):
    self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><androidx.constraintlayout.widget.Barrier app:barrierMargin="wide"/></ConstraintLayout>')
 def test_wave18b_constraint_size_bounds_rule_280(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <dimen name="min_w">48dp</dimen><dimen name="max_w">240dp</dimen><bool name="yes">true</bool>
    <style name="BoundedStyle"><item name="layout_constrainedWidth">true</item><item name="layout_constraintHeight_min">32dp</item></style>
   </resources>''')
   xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <View android:id="@+id/v_plain"/>
    <View android:id="@+id/v_defaults" app:layout_constrainedWidth="false" app:layout_constrainedHeight="false" app:layout_constraintWidth_min="0dp" app:layout_constraintWidth_max="wrap"/>
    <View android:id="@+id/v_cw" app:layout_constrainedWidth="@bool/yes"/>
    <View android:id="@+id/v_ch" app:layout_constrainedHeight="true"/>
    <View android:id="@+id/v_minmax" app:layout_constraintWidth_min="@dimen/min_w" app:layout_constraintWidth_max="@dimen/max_w"/>
    <View android:id="@+id/v_all" app:layout_constrainedWidth="true" app:layout_constrainedHeight="true" app:layout_constraintWidth_min="64dp" app:layout_constraintWidth_max="320dp" app:layout_constraintHeight_min="24dp" app:layout_constraintHeight_max="120.5dp"/>
    <View android:id="@+id/v_styled" style="@style/BoundedStyle"/>
   </androidx.constraintlayout.widget.ConstraintLayout>'''
   out=self.run_layout(xml,res)
   names=['v_plain','v_defaults','v_cw','v_ch','v_minmax','v_all','v_styled']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertNotIn('kind:280',slices['v_plain'])
   self.assertNotIn('kind:280',slices['v_defaults'])
   self.assertIn('RelativeRule { kind:280, anchor:"1,0|0,0,0,0" }',slices['v_cw'])
   self.assertIn('RelativeRule { kind:280, anchor:"0,1|0,0,0,0" }',slices['v_ch'])
   self.assertIn('RelativeRule { kind:280, anchor:"0,0|48,240,0,0" }',slices['v_minmax'])
   self.assertIn('RelativeRule { kind:280, anchor:"1,1|64,320,24,120.5" }',slices['v_all'])
   self.assertIn('RelativeRule { kind:280, anchor:"1,0|0,0,32,0" }',slices['v_styled'])
   with self.assertRaisesRegex(ValueError,'Invalid layout_constrainedWidth'):
    self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constrainedWidth="yes"/></ConstraintLayout>')
   with self.assertRaisesRegex(ValueError,'Expected dp/sp/px'):
    self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintWidth_min="-10dp"/></ConstraintLayout>')
 def test_wave18b_enhanced_flow_rule_232(self):
  xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <View android:id="@+id/a"/><View android:id="@+id/b"/><View android:id="@+id/c"/>
   <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_base" app:flow_wrapMode="chain" app:flow_horizontalGap="8dp" app:flow_maxElementsWrap="2" app:constraint_referenced_ids="a,b,c"/>
   <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_exp_def" app:flow_horizontalStyle="spread" app:flow_verticalStyle="spread" app:flow_horizontalBias="0.5" app:flow_verticalBias="0.5" app:flow_horizontalAlign="center" app:flow_verticalAlign="center" app:constraint_referenced_ids="a,b"/>
   <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_packed" app:flow_horizontalStyle="packed" app:flow_horizontalBias="0.25" app:constraint_referenced_ids="a,b"/>
   <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_full" android:orientation="vertical" app:flow_wrapMode="aligned" app:flow_horizontalGap="6dp" app:flow_verticalGap="4dp" app:flow_horizontalStyle="spread_inside" app:flow_verticalStyle="packed" app:flow_horizontalBias="0.2" app:flow_verticalBias="0.8" app:flow_horizontalAlign="start" app:flow_verticalAlign="baseline" app:flow_maxElementsWrap="3" app:constraint_referenced_ids="a,b,c"/>
   <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_valign_start" app:flow_verticalAlign="start" app:constraint_referenced_ids="a,b"/>
  </androidx.constraintlayout.widget.ConstraintLayout>'''
  out=self.run_layout(xml)
  names=['fl_base','fl_exp_def','fl_packed','fl_full','fl_valign_start']
  slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
  self.assertIn('RelativeRule { kind:232, anchor:"0|chain|8.0|0|2|a,b,c" }',slices['fl_base'])
  self.assertIn('RelativeRule { kind:232, anchor:"0|none|0|0|0|a,b" }',slices['fl_exp_def'])
  self.assertIn('RelativeRule { kind:232, anchor:"0|none|0|0|0|a,b|packed,spread,0.25,0.5,center,center,0" }',slices['fl_packed'])
  self.assertIn('RelativeRule { kind:232, anchor:"1|aligned|6.0|4.0|3|a,b,c|spread_inside,packed,0.2,0.8,start,baseline,3" }',slices['fl_full'])
  self.assertIn('RelativeRule { kind:232, anchor:"0|none|0|0|0|a,b|spread,spread,0.5,0.5,center,start,0" }',slices['fl_valign_start'])
  for bad_xml,err in [
   ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_horizontalStyle="even"/>','Unsupported flow_horizontalStyle'),
   ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_verticalBias="1.5"/>','Invalid flow_verticalBias'),
   ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_horizontalAlign="top"/>','Unsupported flow_horizontalAlign'),
   ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_verticalAlign="diagonal"/>','Unsupported flow_verticalAlign'),
  ]:
   with self.subTest(bad=bad_xml):
    with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml)
 def test_wave18b_constraint_layer_and_placeholder_rules_281_282(self):
  xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
   <TextView android:id="@+id/t1" android:text="One"/>
   <TextView android:id="@+id/t2" android:text="Two"/>
   <androidx.constraintlayout.helper.widget.Layer android:id="@+id/layer_x" app:constraint_referenced_ids="t1,@id/t2" android:rotation="15" android:scaleX="1.2" android:translationY="4dp" android:alpha="0.8" android:elevation="6dp" android:background="#330000ff"/>
   <android.support.constraint.helper.Layer android:id="@+id/layer_legacy" app:constraint_referenced_ids="@id/t1"/>
   <androidx.constraintlayout.widget.Placeholder android:id="@+id/ph_empty"/>
   <androidx.constraintlayout.widget.Placeholder android:id="@+id/ph_inv" app:content="@id/t1" app:emptyVisibility="invisible"/>
   <androidx.constraintlayout.widget.Placeholder android:id="@+id/ph_vis" app:content="@+id/t2" app:placeholder_emptyVisibility="visible"/>
   <androidx.constraintlayout.widget.Placeholder android:id="@+id/ph_null" app:content="@null" app:emptyVisibility="gone"/>
  </androidx.constraintlayout.widget.ConstraintLayout>'''
  out=self.run_layout(xml)
  names=['layer_x','layer_legacy','ph_empty','ph_inv','ph_vis','ph_null']
  slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
  self.assertIn('RelativeRule { kind:281, anchor:"t1,t2" }',slices['layer_x'])
  self.assertIn('RelativeRule { kind:256, anchor:"0,4.0,1.2,1,15.0,center,center" }',slices['layer_x'])
  self.assertIn('RelativeRule { kind:193, anchor:"6.0" }',slices['layer_x'])
  self.assertIn('background:Some(Color(0,0,255,51))',slices['layer_x'])
  self.assertIn('view_alpha:0.8',slices['layer_x'])
  self.assertIn('visibility:Visibility::Visible',slices['layer_x'])
  self.assertIn('RelativeRule { kind:281, anchor:"t1" }',slices['layer_legacy'])
  self.assertIn('RelativeRule { kind:282, anchor:"|gone" }',slices['ph_empty'])
  self.assertIn('visibility:Visibility::Gone',slices['ph_empty'])
  self.assertIn('RelativeRule { kind:282, anchor:"t1|invisible" }',slices['ph_inv'])
  self.assertIn('visibility:Visibility::Invisible',slices['ph_inv'])
  self.assertIn('RelativeRule { kind:282, anchor:"t2|visible" }',slices['ph_vis'])
  self.assertIn('visibility:Visibility::Visible',slices['ph_vis'])
  self.assertIn('RelativeRule { kind:282, anchor:"|gone" }',slices['ph_null'])
  with self.assertRaisesRegex(ValueError,'Invalid Placeholder content'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><androidx.constraintlayout.widget.Placeholder app:content="bad_id"/></ConstraintLayout>')
  with self.assertRaisesRegex(ValueError,'Unsupported emptyVisibility'):
   self.run_layout('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><androidx.constraintlayout.widget.Placeholder app:emptyVisibility="hidden"/></ConstraintLayout>')
 def test_wave18b_app_bar_scroll_flags_rule_284(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('<resources><style name="ScrollBarStyle"><item name="layout_scrollFlags">scroll|enterAlways</item></style></resources>')
   xml='''<com.google.android.material.appbar.AppBarLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.appbar.MaterialToolbar android:id="@+id/tb_plain"/>
    <com.google.android.material.appbar.MaterialToolbar android:id="@+id/tb_ea" app:layout_scrollFlags="scroll|enterAlways"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_exit" app:layout_scrollFlags="scroll|exitUntilCollapsed|snap|scroll"/>
    <androidx.appcompat.widget.Toolbar android:id="@+id/tb_eac" app:layout_scrollFlags="scroll|enterAlwaysCollapsed|snapMargins"/>
    <View android:id="@+id/v_noscroll" app:layout_scrollFlags="noScroll"/>
    <com.google.android.material.appbar.MaterialToolbar android:id="@+id/tb_styled" style="@style/ScrollBarStyle"/>
   </com.google.android.material.appbar.AppBarLayout>'''
   out=self.run_layout(xml,res)
   names=['tb_plain','tb_ea','ctl_exit','tb_eac','v_noscroll','tb_styled']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertNotIn('kind:284',slices['tb_plain'])
   self.assertIn('RelativeRule { kind:284, anchor:"scroll,enterAlways" }',slices['tb_ea'])
   self.assertIn('RelativeRule { kind:284, anchor:"scroll,exitUntilCollapsed,snap" }',slices['ctl_exit'])
   self.assertIn('RelativeRule { kind:284, anchor:"scroll,enterAlwaysCollapsed,snapMargins" }',slices['tb_eac'])
   self.assertIn('RelativeRule { kind:284, anchor:"noScroll" }',slices['v_noscroll'])
   self.assertIn('RelativeRule { kind:284, anchor:"scroll,enterAlways" }',slices['tb_styled'])
   with self.assertRaisesRegex(ValueError,'Unsupported layout_scrollFlags'):
    self.run_layout('<View xmlns:app="http://schemas.android.com/apk/res-auto" app:layout_scrollFlags="scroll|bounce"/>')
 def test_wave18b_animation_list_rule_285(self):
  def make_png(w,h,rgba):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=b''.join(b'\x00'+bytes(rgba)*w for _ in range(h))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir()
   (res/'drawable'/'f0.png').write_bytes(make_png(8,8,(255,0,0,255)))
   (res/'drawable'/'f1.png').write_bytes(make_png(8,8,(0,255,0,255)))
   (res/'drawable'/'f2.png').write_bytes(make_png(8,8,(0,0,255,255)))
   (res/'drawable'/'v0.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="16dp" android:height="16dp" android:viewportWidth="16" android:viewportHeight="16"><path android:fillColor="#ffaa00" android:pathData="M0,0h16v16h-16z"/></vector>')
   (res/'drawable'/'v1.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="16dp" android:height="16dp" android:viewportWidth="16" android:viewportHeight="16"><path android:fillColor="#00aaff" android:pathData="M0,0h16v16h-16z"/></vector>')
   (res/'drawable'/'anim_png.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="false"><item android:drawable="@drawable/f0" android:duration="60"/><item android:drawable="@drawable/f1" android:duration="80"/><item android:drawable="@drawable/f2" android:duration="100"/></animation-list>')
   (res/'drawable'/'anim_vec.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="true"><item android:drawable="@drawable/v0" android:duration="120"/><item android:drawable="@drawable/v1" android:duration="150"/></animation-list>')
   (res/'drawable'/'anim_inline.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="true"><item android:duration="90"><shape><solid android:color="#224466"/><corners android:radius="6dp"/></shape></item><item android:duration="110"><shape><solid android:color="#664422"/></shape></item></animation-list>')
   (res/'drawable'/'anim_inset.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android" android:inset="2dp" android:drawable="@drawable/anim_png"/>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <ImageView android:id="@+id/iv_png" android:src="@drawable/anim_png"/>
    <ImageButton android:id="@+id/ib_vec" app:srcCompat="@drawable/anim_vec"/>
    <View android:id="@+id/v_bg_png" android:background="@drawable/anim_png"/>
    <View android:id="@+id/v_bg_inline" android:background="@drawable/anim_inline"/>
    <View android:id="@+id/v_bg_inset" android:background="@drawable/anim_inset"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['iv_png','ib_vec','v_bg_png','v_bg_inline','v_bg_inset']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   # ImageView src: rule 285 + static fallback anim_png + all 3 frames in image table
   self.assertIn('RelativeRule { kind:285, anchor:"0|f0:60,f1:80,f2:100" }',slices['iv_png'])
   for key in ('anim_png','f0','f1','f2'):
    self.assertIn(f'state.images.entry("{key}".into())',out)
   # ImageButton srcCompat vector frames + oneshot=1
   self.assertIn('RelativeRule { kind:285, anchor:"1|v0:120,v1:150" }',slices['ib_vec'])
   for key in ('anim_vec','v0','v1'):
    self.assertIn(f'state.images.entry("{key}".into())',out)
   # View background PNG animation-list: static fallback kind:251 + kind:285
   self.assertIn('RelativeRule { kind:251, anchor:"f0|" }',slices['v_bg_png'])
   self.assertIn('RelativeRule { kind:285, anchor:"0|f0:60,f1:80,f2:100" }',slices['v_bg_png'])
   # View background inline shape animation-list: static bg/radius fallback + rasterized frames + kind:285
   self.assertIn('background:Some(Color(34,68,102,255))',slices['v_bg_inline'])
   self.assertIn(',radius:6.0,',slices['v_bg_inline'])
   self.assertIn('RelativeRule { kind:285, anchor:"1|anim_inline__f0:90,anim_inline__f1:110" }',slices['v_bg_inline'])
   self.assertIn('state.images.entry("anim_inline__f0".into())',out)
   self.assertIn('state.images.entry("anim_inline__f1".into())',out)
   # Inset wrapper propagates animation-list
   self.assertIn('RelativeRule { kind:285, anchor:"0|f0:60,f1:80,f2:100" }',slices['v_bg_inset'])
   # Cyclic animation-list rejected cleanly
   (res/'drawable'/'anim_cycle.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:drawable="@drawable/anim_cycle" android:duration="50"/></animation-list>')
   with self.assertRaisesRegex(ValueError,'Cyclic drawable reference'):
    self.run_layout('<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/anim_cycle"/>',res)
 def test_wave19b_dynamic_wrapper_drawables_rule_286(self):
  def make_png(w,h,rgba):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=b''.join(b'\x00'+bytes(rgba)*w for _ in range(h))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();(res/'values').mkdir()
   (res/'values'/'ints.xml').write_text('<resources><integer name="max_lvl">10</integer></resources>')
   (res/'drawable'/'px_a.png').write_bytes(make_png(8,8,(10,20,30,255)))
   (res/'drawable'/'px_b.png').write_bytes(make_png(8,8,(200,40,50,255)))
   (res/'drawable'/'sh_a.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#112233"/><corners android:radius="6dp"/><size android:width="16dp" android:height="16dp"/></shape>')
   (res/'drawable'/'sh_b.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#445566"/><corners android:radius="2dp"/><size android:width="16dp" android:height="16dp"/></shape>')
   (res/'drawable'/'vec_a.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="24dp" android:height="24dp" android:viewportWidth="24" android:viewportHeight="24"><path android:fillColor="#2266aa" android:pathData="M0,0h24v24h-24z"/></vector>')
   (res/'drawable'/'lvl_ref.xml').write_text('<level-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:minLevel="0" android:maxLevel="4" android:drawable="@drawable/sh_a"/><item android:minLevel="5" android:maxLevel="@integer/max_lvl" android:drawable="@drawable/sh_b"/></level-list>')
   (res/'drawable'/'lvl_inline.xml').write_text('<level-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:minLevel="-1" android:maxLevel="0"><shape><solid android:color="#aa1122"/></shape></item><item android:maxLevel="2" android:drawable="@drawable/px_a"/></level-list>')
   (res/'drawable'/'clip_h.xml').write_text('<clip xmlns:android="http://schemas.android.com/apk/res/android" android:gravity="end|center_vertical" android:drawable="@drawable/px_a"/>')
   (res/'drawable'/'clip_v.xml').write_text('<clip xmlns:android="http://schemas.android.com/apk/res/android" android:clipOrientation="vertical" android:gravity="bottom"><shape><solid android:color="#334455"/><corners android:radius="4dp"/></shape></clip>')
   (res/'drawable'/'clip_def.xml').write_text('<clip xmlns:android="http://schemas.android.com/apk/res/android" android:drawable="@drawable/sh_a"/>')
   (res/'drawable'/'rot_full.xml').write_text('<rotate xmlns:android="http://schemas.android.com/apk/res/android" android:fromDegrees="-45" android:toDegrees="135" android:pivotX="25%" android:pivotY="75%" android:autoMirrored="true" android:drawable="@drawable/vec_a"/>')
   (res/'drawable'/'rot_def.xml').write_text('<rotate xmlns:android="http://schemas.android.com/apk/res/android"><vector android:width="20dp" android:height="20dp" android:viewportWidth="20" android:viewportHeight="20"><path android:fillColor="#cc2233" android:pathData="M0,0h20v20h-20z"/></vector></rotate>')
   (res/'drawable'/'scale_ref.xml').write_text('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:scaleWidth="80%" android:scaleHeight="50%" android:scaleGravity="center" android:drawable="@drawable/px_b"/>')
   (res/'drawable'/'scale_def.xml').write_text('<scale xmlns:android="http://schemas.android.com/apk/res/android" android:scaleWidth="60%" android:scaleGravity="start|center_vertical"><shape><solid android:color="#556677"/></shape></scale>')
   (res/'drawable'/'trans_ref.xml').write_text('<transition xmlns:android="http://schemas.android.com/apk/res/android"><item android:drawable="@drawable/vec_a"/><item android:drawable="@drawable/px_b"/></transition>')
   (res/'drawable'/'trans_inline.xml').write_text('<transition xmlns:android="http://schemas.android.com/apk/res/android"><item><shape><solid android:color="#778899"/><corners android:radius="3dp"/></shape></item><item android:drawable="@drawable/sh_b"/></transition>')
   (res/'drawable'/'trans_single.xml').write_text('<transition xmlns:android="http://schemas.android.com/apk/res/android"><item android:drawable="@drawable/px_a"/></transition>')
   (res/'drawable'/'inset_clip.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android" android:inset="4dp" android:drawable="@drawable/clip_def"/>')
   (res/'drawable'/'ripple_rot.xml').write_text('<ripple xmlns:android="http://schemas.android.com/apk/res/android" android:color="#33000000"><item android:drawable="@drawable/rot_full"/></ripple>')
   (res/'drawable'/'sel_dyn.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_pressed="true" android:drawable="@drawable/lvl_ref"/><item android:drawable="@drawable/clip_def"/></selector>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <ImageView android:id="@+id/src_lvl" android:src="@drawable/lvl_ref"/>
    <ImageView android:id="@+id/src_clip" app:srcCompat="@drawable/clip_h"/>
    <ImageView android:id="@+id/src_rot" android:src="@drawable/rot_full"/>
    <ImageView android:id="@+id/src_rot_def" app:srcCompat="@drawable/rot_def"/>
    <ImageView android:id="@+id/src_scale" app:srcCompat="@drawable/scale_ref"/>
    <ImageView android:id="@+id/src_trans" android:src="@drawable/trans_ref"/>
    <ImageView android:id="@+id/src_trans1" android:src="@drawable/trans_single"/>
    <ImageView android:id="@+id/both_dyn" android:src="@drawable/rot_full" android:background="@drawable/clip_v"/>
    <View android:id="@+id/bg_lvl" android:background="@drawable/lvl_inline"/>
    <View android:id="@+id/bg_clip_v" android:background="@drawable/clip_v"/>
    <View android:id="@+id/bg_clip_def" android:background="@drawable/clip_def"/>
    <View android:id="@+id/bg_rot" android:background="@drawable/rot_full"/>
    <View android:id="@+id/bg_scale" android:background="@drawable/scale_def"/>
    <View android:id="@+id/bg_trans" android:background="@drawable/trans_inline"/>
    <View android:id="@+id/bg_inset" android:background="@drawable/inset_clip"/>
    <View android:id="@+id/bg_ripple" android:background="@drawable/ripple_rot"/>
    <View android:id="@+id/bg_static" android:background="@drawable/sh_a"/>
    <ImageView android:id="@+id/src_static" android:src="@drawable/vec_a"/>
    <FrameLayout android:id="@+id/fg_dyn" android:foreground="@drawable/clip_def"/>
    <View android:id="@+id/bg_sel" android:background="@drawable/sel_dyn"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['src_lvl','src_clip','src_rot','src_rot_def','src_scale','src_trans','src_trans1','both_dyn','bg_lvl','bg_clip_v','bg_clip_def','bg_rot','bg_scale','bg_trans','bg_inset','bg_ripple','bg_static','src_static','fg_dyn','bg_sel']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   # src dynamic rules + static image_src fallback
   self.assertIn('RelativeRule { kind:286, anchor:"src:level|0:4:sh_a,5:10:sh_b" }',slices['src_lvl'])
   self.assertIn('image_src:"lvl_ref"',slices['src_lvl'])
   self.assertIn('RelativeRule { kind:286, anchor:"src:clip|h|end|px_a" }',slices['src_clip'])
   self.assertIn('RelativeRule { kind:286, anchor:"src:rotate|-45,135|0.25,0.75|vec_a" }',slices['src_rot'])
   self.assertIn('RelativeRule { kind:255, anchor:"true" }',slices['src_rot'])
   self.assertIn('RelativeRule { kind:286, anchor:"src:rotate|0,360|0.5,0.5|rot_def" }',slices['src_rot_def'])
   self.assertIn('RelativeRule { kind:286, anchor:"src:scale|0.8,0.5|center|px_b" }',slices['src_scale'])
   self.assertIn('RelativeRule { kind:286, anchor:"src:transition|250|vec_a,px_b" }',slices['src_trans'])
   self.assertIn('RelativeRule { kind:286, anchor:"src:transition|250|px_a,px_a" }',slices['src_trans1'])
   # Both src and bg dynamic rules on same ImageView
   self.assertIn('RelativeRule { kind:286, anchor:"src:rotate|-45,135|0.25,0.75|vec_a" }',slices['both_dyn'])
   self.assertIn('RelativeRule { kind:286, anchor:"bg:clip|v|bottom|clip_v" }',slices['both_dyn'])
   # bg dynamic rules + static bg/radius fallbacks preserved
   self.assertIn('RelativeRule { kind:286, anchor:"bg:level|0:0:lvl_inline__f0,0:2:px_a" }',slices['bg_lvl'])
   self.assertIn('background:Some(Color(170,17,34,255))',slices['bg_lvl'])
   self.assertIn('RelativeRule { kind:286, anchor:"bg:clip|v|bottom|clip_v" }',slices['bg_clip_v'])
   self.assertIn('background:Some(Color(51,68,85,255))',slices['bg_clip_v'])
   self.assertIn(',radius:4.0,',slices['bg_clip_v'])
   self.assertIn('RelativeRule { kind:286, anchor:"bg:clip|h|start|sh_a" }',slices['bg_clip_def'])
   self.assertIn('RelativeRule { kind:286, anchor:"bg:rotate|-45,135|0.25,0.75|vec_a" }',slices['bg_rot'])
   self.assertIn('RelativeRule { kind:286, anchor:"bg:scale|0.6,1|start|scale_def" }',slices['bg_scale'])
   self.assertIn('RelativeRule { kind:286, anchor:"bg:transition|250|trans_inline__f0,sh_b" }',slices['bg_trans'])
   self.assertIn('background:Some(Color(119,136,153,255))',slices['bg_trans'])
   self.assertIn(',radius:3.0,',slices['bg_trans'])
   # Inset & ripple propagate bg dynamic drawable
   self.assertIn('RelativeRule { kind:286, anchor:"bg:clip|h|start|sh_a" }',slices['bg_inset'])
   self.assertIn('RelativeRule { kind:286, anchor:"bg:rotate|-45,135|0.25,0.75|vec_a" }',slices['bg_ripple'])
   self.assertIn('RelativeRule { kind:209, anchor:"0,0,0,51" }',slices['bg_ripple'])
   # Static shapes/vectors, foreground, and selector do NOT emit kind:286
   for no_dyn in ('bg_static','src_static','fg_dyn','bg_sel'):
    self.assertNotIn('kind:286',slices[no_dyn])
   # All referenced and inline child drawables are bundled in image table
   for key in ('sh_a','sh_b','px_a','px_b','vec_a','rot_def','lvl_inline__f0','clip_v','scale_def','trans_inline__f0'):
    self.assertIn(f'state.images.entry("{key}".into())',out)
   # Cyclic level-list second item rejected
   (res/'drawable'/'lvl_cycle.xml').write_text('<level-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:minLevel="0" android:maxLevel="0" android:drawable="@drawable/px_a"/><item android:minLevel="1" android:maxLevel="2" android:drawable="@drawable/lvl_cycle"/></level-list>')
   with self.assertRaisesRegex(ValueError,'Cyclic drawable reference'):
    self.run_layout('<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/lvl_cycle"/>',res)
 def test_wave19b_constraint_physical_flags_rule_287(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('<resources><style name="PhysPins"><item name="layout_constraintLeft_toLeftOf">parent</item><item name="layout_constraintRight_toRightOf">parent</item></style></resources>')
   xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <View android:id="@+id/v_logical" app:layout_constraintStart_toStartOf="parent" app:layout_constraintEnd_toEndOf="parent"/>
    <View android:id="@+id/v_left_only" app:layout_constraintLeft_toLeftOf="parent"/>
    <View android:id="@+id/v_left_to_right" app:layout_constraintLeft_toRightOf="@id/v_logical"/>
    <View android:id="@+id/v_right_only" app:layout_constraintRight_toRightOf="parent"/>
    <View android:id="@+id/v_right_to_left" app:layout_constraintRight_toLeftOf="@id/v_logical"/>
    <View android:id="@+id/v_both_phys" app:layout_constraintLeft_toLeftOf="parent" app:layout_constraintRight_toRightOf="parent"/>
    <View android:id="@+id/v_phys_l_log_e" app:layout_constraintLeft_toLeftOf="parent" app:layout_constraintEnd_toEndOf="parent"/>
    <View android:id="@+id/v_log_s_phys_r" app:layout_constraintStart_toStartOf="parent" app:layout_constraintRight_toRightOf="parent"/>
    <View android:id="@+id/v_log_overrides_phys" app:layout_constraintStart_toStartOf="parent" app:layout_constraintLeft_toLeftOf="parent" app:layout_constraintEnd_toEndOf="parent" app:layout_constraintRight_toRightOf="parent"/>
    <View android:id="@+id/v_styled_phys" style="@style/PhysPins"/>
   </androidx.constraintlayout.widget.ConstraintLayout>'''
   out=self.run_layout(xml,res)
   names=['v_logical','v_left_only','v_left_to_right','v_right_only','v_right_to_left','v_both_phys','v_phys_l_log_e','v_log_s_phys_r','v_log_overrides_phys','v_styled_phys']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertNotIn('kind:287',slices['v_logical'])
   self.assertIn('RelativeRule { kind:287, anchor:"1" }',slices['v_left_only'])
   self.assertIn('RelativeRule { kind:287, anchor:"1" }',slices['v_left_to_right'])
   self.assertIn('RelativeRule { kind:287, anchor:"2" }',slices['v_right_only'])
   self.assertIn('RelativeRule { kind:287, anchor:"2" }',slices['v_right_to_left'])
   self.assertIn('RelativeRule { kind:287, anchor:"3" }',slices['v_both_phys'])
   self.assertIn('RelativeRule { kind:287, anchor:"1" }',slices['v_phys_l_log_e'])
   self.assertIn('RelativeRule { kind:287, anchor:"2" }',slices['v_log_s_phys_r'])
   self.assertNotIn('kind:287',slices['v_log_overrides_phys'])
   self.assertIn('RelativeRule { kind:287, anchor:"3" }',slices['v_styled_phys'])
 def test_wave19b_elevation_and_translation_z_animators_rules_288_289(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'animator').mkdir();(res/'interpolator').mkdir();(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('<resources><dimen name="elev_hi">8dp</dimen></resources>')
   (res/'interpolator'/'lin.xml').write_text('<linearInterpolator xmlns:android="http://schemas.android.com/apk/res/android"/>')
   (res/'interpolator'/'dec.xml').write_text('<decelerateInterpolator xmlns:android="http://schemas.android.com/apk/res/android"/>')
   (res/'animator'/'elev_in.xml').write_text('<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="elevation" android:valueFrom="1dp" android:valueTo="@dimen/elev_hi" android:duration="@android:integer/config_shortAnimTime" android:startOffset="50" android:interpolator="@android:anim/linear_interpolator"/>')
   (res/'animator'/'tz_out.xml').write_text('<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="translationZ" android:valueFrom="6dp" android:valueTo="0dp" android:duration="150" android:interpolator="@android:anim/decelerate_interpolator"/>')
   (res/'animator'/'elev_def.xml').write_text('<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="elevation" android:valueTo="4.5dp"/>')
   (res/'animator'/'seq_set.xml').write_text('''<set xmlns:android="http://schemas.android.com/apk/res/android" android:ordering="sequentially">
    <objectAnimator android:propertyName="alpha" android:valueFrom="0" android:valueTo="1" android:duration="120"/>
    <objectAnimator android:propertyName="translationZ" android:valueFrom="0dp" android:valueTo="6dp" android:duration="250" android:startOffset="30" android:interpolator="@android:anim/accelerate_interpolator"/>
   </set>''')
   (res/'animator'/'combo_together.xml').write_text('''<set xmlns:android="http://schemas.android.com/apk/res/android" android:ordering="together">
    <objectAnimator android:propertyName="elevation" android:valueFrom="2dp" android:valueTo="6dp" android:duration="300" android:interpolator="@interpolator/lin"/>
    <objectAnimator android:propertyName="translationZ" android:valueFrom="0dp" android:valueTo="2dp" android:duration="300" android:interpolator="@interpolator/lin"/>
    <objectAnimator android:propertyName="rotation" android:valueFrom="0" android:valueTo="15" android:duration="300"/>
   </set>''')
   (res/'animator'/'px_elev.xml').write_text('<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="elevation" android:valueFrom="4px" android:valueTo="16px" android:duration="200" android:interpolator="@interpolator/dec"/>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <AdapterViewFlipper android:id="@+id/avf_basic" android:inAnimation="@animator/elev_in" android:outAnimation="@animator/tz_out"/>
    <AdapterViewFlipper android:id="@+id/avf_def" android:inAnimation="@animator/elev_def"/>
    <AdapterViewFlipper android:id="@+id/avf_seq" android:inAnimation="@animator/seq_set" android:outAnimation="@animator/combo_together"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['avf_basic','avf_def','avf_seq']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertIn('RelativeRule { kind:288, anchor:"1,8|200|50|linear" }',slices['avf_basic'])
   self.assertIn('RelativeRule { kind:289, anchor:"6,0|150|0|decelerate" }',slices['avf_basic'])
   self.assertIn('RelativeRule { kind:288, anchor:"0,4.5|300|0|accelerate_decelerate" }',slices['avf_def'])
   self.assertNotIn('kind:289',slices['avf_def'])
   self.assertIn('animation_in:Some(AlphaAnimation { from:0.0, to:1.0, duration:0.12, start_offset:0.0, linear:false })',slices['avf_seq'])
   self.assertIn('RelativeRule { kind:288, anchor:"0,6|250|150|accelerate" }',slices['avf_seq'])
   self.assertIn('RelativeRule { kind:289, anchor:"2,8|300|0|linear" }',slices['avf_seq'])
   self.assertIn('RelativeRule { kind:277, anchor:"0,15|center,center|0.3|0.0|0" }',slices['avf_seq'])
   # Density conversion for px elevation values (4px,16px at 320dpi -> 2dp,8dp)
   lp=Path(d)/'dens.xml';lp.write_text('<AdapterViewFlipper xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@animator/px_elev"/>')
   out_320=mod.compile_layout(lp,res,density=320)
   self.assertIn('RelativeRule { kind:288, anchor:"2,8|200|0|decelerate" }',out_320)
   # fast_out_slow_in, overshoot, and anticipate interpolators map cleanly to Wave 19A AnimInterp names
   for interp_ref,expected_interp in [
    ('@android:interpolator/fast_out_slow_in','fast_out_slow_in'),
    ('@android:anim/overshoot_interpolator','overshoot'),
    ('@android:anim/anticipate_interpolator','anticipate'),
   ]:
    (res/'animator'/'interp_e.xml').write_text(f'<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android" android:propertyName="elevation" android:valueFrom="0dp" android:valueTo="6dp" android:duration="180" android:interpolator="{interp_ref}"/>')
    with self.subTest(interp=interp_ref):
     self.assertIn(f'RelativeRule {{ kind:288, anchor:"0,6|180|0|{expected_interp}" }}',self.run_layout('<AdapterViewFlipper xmlns:android="http://schemas.android.com/apk/res/android" android:inAnimation="@animator/interp_e"/>',res))
 def test_wave20b_app_bar_ext_and_drawer_lock_mode_rules_290_291(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'color').mkdir();(res/'drawable').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <color name="lift_tint">#123456</color>
    <string name="lock_open_str">locked_open</string>
    <style name="AppBar.LiftColor" parent="Widget.Material3.AppBarLayout">
     <item name="liftOnScroll">true</item>
     <item name="liftOnScrollColor">@color/lift_tint</item>
    </style>
    <style name="CompressChild">
     <item name="layout_scrollEffect">compress</item>
    </style>
    <style name="LockedDrawer">
     <item name="drawerLockMode">locked_closed</item>
    </style>
   </resources>''')
   (res/'color'/'lift_csl.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:state_pressed="true" android:color="#ff0000"/>
    <item android:color="#cc224466"/>
   </selector>''')
   (res/'drawable'/'not_a_color.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#112233"/></shape>')
   xml='''<androidx.coordinatorlayout.widget.CoordinatorLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_plain"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_hex" app:liftOnScrollColor="#80aabbcc">
     <com.google.android.material.appbar.MaterialToolbar android:id="@+id/tb_compress" app:layout_scrollFlags="scroll|enterAlways" app:layout_scrollEffect="compress"/>
     <com.google.android.material.appbar.MaterialToolbar android:id="@+id/tb_none" app:layout_scrollEffect="none"/>
     <com.google.android.material.appbar.MaterialToolbar android:id="@+id/tb_styled_compress" style="@style/CompressChild"/>
    </com.google.android.material.appbar.AppBarLayout>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_color" app:liftOnScroll="true" app:liftOnScrollColor="@color/lift_tint"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_csl" app:liftOnScrollColor="@color/lift_csl"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_attr" app:liftOnScrollColor="?attr/colorSurfaceContainer"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_styled" style="@style/AppBar.LiftColor"/>
    <com.google.android.material.appbar.AppBarLayout android:id="@+id/ab_null_override" style="@style/AppBar.LiftColor" app:liftOnScrollColor="@null"/>
    <androidx.drawerlayout.widget.DrawerLayout android:id="@+id/dl_plain">
     <View android:id="@+id/dl_content"/>
     <com.google.android.material.navigation.NavigationView android:id="@+id/dl_child_plain" android:layout_gravity="start"/>
    </androidx.drawerlayout.widget.DrawerLayout>
    <androidx.drawerlayout.widget.DrawerLayout android:id="@+id/dl_unlocked" app:drawerLockMode="unlocked"/>
    <androidx.drawerlayout.widget.DrawerLayout android:id="@+id/dl_closed" app:drawerLockMode="locked_closed"/>
    <androidx.drawerlayout.widget.DrawerLayout android:id="@+id/dl_open" app:drawerLockMode="locked_open"/>
    <androidx.drawerlayout.widget.DrawerLayout android:id="@+id/dl_num0" app:drawerLockMode="0"/>
    <androidx.drawerlayout.widget.DrawerLayout android:id="@+id/dl_num1" app:drawerLockMode="1"/>
    <androidx.drawerlayout.widget.DrawerLayout android:id="@+id/dl_num2" app:drawerLockMode="2"/>
    <androidx.drawerlayout.widget.DrawerLayout android:id="@+id/dl_styled" style="@style/LockedDrawer">
     <View android:id="@+id/dl_child_main"/>
     <com.google.android.material.navigation.NavigationView android:id="@+id/dl_child_open" android:layout_gravity="start" app:drawerLockMode="@string/lock_open_str"/>
     <LinearLayout android:id="@+id/dl_child_styled" android:layout_gravity="end" style="@style/LockedDrawer"/>
    </androidx.drawerlayout.widget.DrawerLayout>
   </androidx.coordinatorlayout.widget.CoordinatorLayout>'''
   out=self.run_layout(xml,res)
   names=['ab_plain','ab_hex','tb_compress','tb_none','tb_styled_compress','ab_color','ab_csl','ab_attr','ab_styled','ab_null_override','dl_plain','dl_content','dl_child_plain','dl_unlocked','dl_closed','dl_open','dl_num0','dl_num1','dl_num2','dl_styled','dl_child_main','dl_child_open','dl_child_styled']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertNotIn('kind:290',slices['ab_plain'])
   self.assertIn('RelativeRule { kind:290, anchor:"lift_color=170,187,204,128" }',slices['ab_hex'])
   self.assertIn('RelativeRule { kind:284, anchor:"scroll,enterAlways" }',slices['tb_compress'])
   self.assertIn('RelativeRule { kind:290, anchor:"effect=compress" }',slices['tb_compress'])
   self.assertNotIn('kind:290',slices['tb_none'])
   self.assertIn('RelativeRule { kind:290, anchor:"effect=compress" }',slices['tb_styled_compress'])
   self.assertIn('RelativeRule { kind:279, anchor:"1" }',slices['ab_color'])
   self.assertIn('RelativeRule { kind:290, anchor:"lift_color=18,52,86,255" }',slices['ab_color'])
   self.assertIn('RelativeRule { kind:290, anchor:"lift_color=34,68,102,204" }',slices['ab_csl'])
   self.assertIn('RelativeRule { kind:290, anchor:"lift_color=243,237,247,255" }',slices['ab_attr'])
   self.assertIn('RelativeRule { kind:279, anchor:"1" }',slices['ab_styled'])
   self.assertIn('RelativeRule { kind:290, anchor:"lift_color=18,52,86,255" }',slices['ab_styled'])
   self.assertIn('RelativeRule { kind:279, anchor:"1" }',slices['ab_null_override'])
   self.assertNotIn('kind:290',slices['ab_null_override'])
   self.assertNotIn('kind:291',slices['dl_plain'])
   self.assertNotIn('kind:291',slices['dl_child_plain'])
   self.assertIn('RelativeRule { kind:291, anchor:"unlocked" }',slices['dl_unlocked'])
   self.assertIn('RelativeRule { kind:291, anchor:"locked_closed" }',slices['dl_closed'])
   self.assertIn('RelativeRule { kind:291, anchor:"locked_open" }',slices['dl_open'])
   self.assertIn('RelativeRule { kind:291, anchor:"unlocked" }',slices['dl_num0'])
   self.assertIn('RelativeRule { kind:291, anchor:"locked_closed" }',slices['dl_num1'])
   self.assertIn('RelativeRule { kind:291, anchor:"locked_open" }',slices['dl_num2'])
   self.assertIn('RelativeRule { kind:291, anchor:"locked_closed" }',slices['dl_styled'])
   self.assertIn('RelativeRule { kind:291, anchor:"locked_open" }',slices['dl_child_open'])
   self.assertIn('RelativeRule { kind:291, anchor:"locked_closed" }',slices['dl_child_styled'])
   for bad_xml,err in [
    ('<com.google.android.material.appbar.AppBarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:liftOnScrollColor="not_a_color"/>','Expected #RGB'),
    ('<com.google.android.material.appbar.AppBarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:liftOnScrollColor="@drawable/not_a_color"/>','Expected #RGB'),
    ('<com.google.android.material.appbar.AppBarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:liftOnScrollColor="?attr/nonexistentColor"/>','Unresolved theme color'),
    ('<com.google.android.material.appbar.MaterialToolbar xmlns:app="http://schemas.android.com/apk/res-auto" app:layout_scrollEffect="fade"/>','Unsupported layout_scrollEffect'),
    ('<androidx.drawerlayout.widget.DrawerLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:drawerLockMode="locked"/>','Unsupported drawerLockMode'),
    ('<androidx.drawerlayout.widget.DrawerLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:drawerLockMode="3"/>','Unsupported drawerLockMode'),
    ('<androidx.drawerlayout.widget.DrawerLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:drawerLockMode="closed"/></androidx.drawerlayout.widget.DrawerLayout>','Unsupported drawerLockMode'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave20b_flow_first_last_and_gone_margin_baseline_rules_292_223(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <dimen name="gm_base">16dp</dimen>
    <style name="FlowExtStyle">
     <item name="flow_firstHorizontalStyle">packed</item>
     <item name="flow_lastHorizontalStyle">spread_inside</item>
     <item name="flow_firstHorizontalBias">0.15</item>
     <item name="flow_lastVerticalBias">0.9</item>
    </style>
    <style name="GoneBaseStyle">
     <item name="layout_goneMarginStart">10dp</item>
     <item name="layout_goneMarginBaseline">@dimen/gm_base</item>
    </style>
   </resources>''')
   xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <TextView android:id="@+id/a" android:text="A"/>
    <TextView android:id="@+id/b" android:text="B"/>
    <TextView android:id="@+id/c" android:text="C"/>
    <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_plain" app:flow_wrapMode="chain" app:flow_horizontalStyle="packed" app:flow_horizontalBias="0.25" app:constraint_referenced_ids="a,b,c"/>
    <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_partial" app:flow_firstHorizontalStyle="packed" app:flow_lastHorizontalBias="0.25" app:constraint_referenced_ids="a,b,c"/>
    <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_all8" app:flow_wrapMode="chain"
     app:flow_firstHorizontalStyle="spread_inside" app:flow_lastHorizontalStyle="packed"
     app:flow_firstHorizontalBias="0.2" app:flow_lastHorizontalBias="0.8"
     app:flow_firstVerticalStyle="packed" app:flow_lastVerticalStyle="spread"
     app:flow_firstVerticalBias="0.0" app:flow_lastVerticalBias="1.0"
     app:constraint_referenced_ids="a,b,c"/>
    <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_explicit_spread" app:flow_firstHorizontalStyle="spread" app:constraint_referenced_ids="a,b"/>
    <android.support.constraint.helper.Flow android:id="@+id/fl_legacy" app:flow_lastVerticalStyle="spread_inside" app:flow_firstVerticalBias="0.4" app:constraint_referenced_ids="a,b"/>
    <androidx.constraintlayout.helper.widget.Flow android:id="@+id/fl_styled" style="@style/FlowExtStyle" app:constraint_referenced_ids="a,b,c"/>
    <TextView android:id="@+id/gm_none" android:text="N"/>
    <TextView android:id="@+id/gm_legacy6" app:layout_goneMarginStart="12dp" app:layout_goneMarginBottom="4dp"/>
    <TextView android:id="@+id/gm_base_only" app:layout_goneMarginBaseline="12dp"/>
    <TextView android:id="@+id/gm_base_dimen" app:layout_goneMarginStart="8dp" app:layout_goneMarginBaseline="@dimen/gm_base"/>
    <TextView android:id="@+id/gm_base_px" app:layout_goneMarginLeft="1dp" app:layout_goneMarginTop="2dp" app:layout_goneMarginRight="3dp" app:layout_goneMarginBottom="4dp" app:layout_goneMarginStart="5dp" app:layout_goneMarginEnd="6dp" app:layout_goneMarginBaseline="24px"/>
    <TextView android:id="@+id/gm_base_zero" app:layout_goneMarginBaseline="0dp"/>
    <TextView android:id="@+id/gm_styled" style="@style/GoneBaseStyle"/>
   </androidx.constraintlayout.widget.ConstraintLayout>'''
   out=self.run_layout(xml,res)
   names=['fl_plain','fl_partial','fl_all8','fl_explicit_spread','fl_legacy','fl_styled','gm_none','gm_legacy6','gm_base_only','gm_base_dimen','gm_base_px','gm_base_zero','gm_styled']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertNotIn('kind:292',slices['fl_plain'])
   self.assertIn('RelativeRule { kind:232, anchor:"0|chain|0|0|0|a,b,c|packed,spread,0.25,0.5,center,center,0" }',slices['fl_plain'])
   self.assertIn('RelativeRule { kind:292, anchor:"packed,,,0.25,,,," }',slices['fl_partial'])
   self.assertIn('RelativeRule { kind:292, anchor:"spread_inside,packed,0.2,0.8,packed,spread,0,1" }',slices['fl_all8'])
   self.assertIn('RelativeRule { kind:292, anchor:"spread,,,,,,," }',slices['fl_explicit_spread'])
   self.assertIn('RelativeRule { kind:232, anchor:"0|none|0|0|0|a,b" }',slices['fl_legacy'])
   self.assertIn('RelativeRule { kind:292, anchor:",,,,,spread_inside,0.4," }',slices['fl_legacy'])
   self.assertIn('RelativeRule { kind:292, anchor:"packed,spread_inside,0.15,,,,,0.9" }',slices['fl_styled'])
   self.assertNotIn('kind:223',slices['gm_none'])
   self.assertIn('RelativeRule { kind:223, anchor:"-1,-1,-1,4.0,12.0,-1" }',slices['gm_legacy6'])
   self.assertIn('RelativeRule { kind:223, anchor:"-1,-1,-1,-1,-1,-1,12.0" }',slices['gm_base_only'])
   self.assertIn('RelativeRule { kind:223, anchor:"-1,-1,-1,-1,8.0,-1,16.0" }',slices['gm_base_dimen'])
   self.assertIn('RelativeRule { kind:223, anchor:"1.0,2.0,3.0,4.0,5.0,6.0,24.0" }',slices['gm_base_px'])
   self.assertIn('RelativeRule { kind:223, anchor:"-1,-1,-1,-1,-1,-1,0.0" }',slices['gm_base_zero'])
   self.assertIn('RelativeRule { kind:223, anchor:"-1,-1,-1,-1,10.0,-1,16.0" }',slices['gm_styled'])
   for bad_xml,err in [
    ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_firstHorizontalStyle="even"/>','Unsupported flow_firstHorizontalStyle'),
    ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_lastVerticalStyle="stretch"/>','Unsupported flow_lastVerticalStyle'),
    ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_firstHorizontalBias="1.2"/>','Invalid flow_firstHorizontalBias'),
    ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_lastVerticalBias="-0.1"/>','Invalid flow_lastVerticalBias'),
    ('<androidx.constraintlayout.helper.widget.Flow xmlns:app="http://schemas.android.com/apk/res-auto" app:flow_lastHorizontalBias="NaN"/>','Invalid flow_lastHorizontalBias'),
    ('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_goneMarginBaseline="wide"/></ConstraintLayout>','Expected margin'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave20b_animated_selector_transitions_rule_293(self):
  def make_png(w,h,rgba):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=b''.join(b'\x00'+bytes(rgba)*w for _ in range(h))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('<resources><bool name="rev_yes">true</bool><integer name="tr_dur">75</integer></resources>')
   (res/'drawable'/'ic_off.png').write_bytes(make_png(8,8,(20,30,40,255)))
   (res/'drawable'/'ic_on.png').write_bytes(make_png(8,8,(40,180,90,255)))
   (res/'drawable'/'ic_press.png').write_bytes(make_png(8,8,(200,50,60,255)))
   (res/'drawable'/'tr_f0.png').write_bytes(make_png(8,8,(80,80,80,255)))
   (res/'drawable'/'tr_f1.png').write_bytes(make_png(8,8,(160,160,160,255)))
   (res/'drawable'/'tr_vec.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="16dp" android:height="16dp" android:viewportWidth="16" android:viewportHeight="16" android:autoMirrored="true"><path android:fillColor="#0099ff" android:pathData="M0,0h16v16h-16z"/></vector>')
   (res/'drawable'/'sh_a.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#112233"/></shape>')
   (res/'drawable'/'sh_b.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#445566"/></shape>')
   (res/'drawable'/'sh_stroke_a.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#223344"/><stroke android:width="2dp" android:color="#ff0000"/><corners android:radius="4dp"/></shape>')
   (res/'drawable'/'sh_stroke_b.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#556677"/><stroke android:width="2dp" android:color="#00ff00"/><corners android:radius="4dp"/></shape>')
   (res/'drawable'/'al_png.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="true"><item android:drawable="@drawable/tr_f0" android:duration="50"/><item android:drawable="@drawable/tr_f1" android:duration="@integer/tr_dur"/></animation-list>')
   (res/'drawable'/'al_vec.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:drawable="@drawable/tr_vec" android:duration="90"/></animation-list>')
   # src animated-selector: both referenced animation-list (reversible) and inline animation-list, multi-bit state mask, transition before items works
   (res/'drawable'/'asel_src.xml').write_text('''<animated-selector xmlns:android="http://schemas.android.com/apk/res/android">
    <transition android:fromId="@id/st_off" android:toId="@id/st_checked" android:reversible="@bool/rev_yes" android:drawable="@drawable/al_png"/>
    <item android:id="@+id/st_pressed_checked" android:state_pressed="true" android:state_checked="true" android:drawable="@drawable/ic_press"/>
    <item android:id="@+id/st_checked" android:state_checked="true" android:drawable="@drawable/ic_on"/>
    <item android:id="@+id/st_off" android:drawable="@drawable/ic_off"/>
    <transition android:fromId="@id/st_checked" android:toId="@id/st_pressed_checked">
     <animation-list>
      <item android:duration="40"><shape><solid android:color="#ffaa00"/></shape></item>
      <item android:drawable="@drawable/tr_f1" android:duration="60"/>
     </animation-list>
    </transition>
   </animated-selector>''')
   # bg animated-selector (color-list path -> kind:96 + kind:293)
   (res/'drawable'/'asel_bg_color.xml').write_text('''<animated-selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/bg_sel" android:state_selected="true" android:drawable="@drawable/sh_a"/>
    <item android:id="@+id/bg_def" android:drawable="@drawable/sh_b"/>
    <transition android:fromId="@id/bg_def" android:toId="@id/bg_sel" android:reversible="true" android:drawable="@drawable/al_png"/>
   </animated-selector>''')
   # bg animated-selector (stateful layer path -> kind:231 + kind:293 + autoMirrored vector frame)
   (res/'drawable'/'asel_bg_layer.xml').write_text('''<animated-selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/ly_act" android:state_activated="true" android:drawable="@drawable/sh_stroke_a"/>
    <item android:id="@+id/ly_def" android:drawable="@drawable/sh_stroke_b"/>
    <transition android:fromId="@id/ly_def" android:toId="@id/ly_act" android:reversible="false" android:drawable="@drawable/al_vec"/>
   </animated-selector>''')
   # bg animated-selector (stateful bg image path -> kind:254 + kind:293)
   (res/'drawable'/'asel_bg_img.xml').write_text('''<animated-selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/bi_on" android:state_checked="true" android:drawable="@drawable/ic_on"/>
    <item android:id="@+id/bi_off" android:drawable="@drawable/ic_off"/>
    <transition android:fromId="@id/bi_off" android:toId="@id/bi_on">
     <animation-list><item android:drawable="@drawable/tr_f0" android:duration="30"/><item android:duration="45"><shape><solid android:color="#123456"/></shape></item></animation-list>
    </transition>
   </animated-selector>''')
   (res/'drawable'/'asel_no_trans.xml').write_text('''<animated-selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:id="@+id/nt_on" android:state_checked="true" android:drawable="@drawable/ic_on"/>
    <item android:id="@+id/nt_off" android:drawable="@drawable/ic_off"/>
   </animated-selector>''')
   (res/'drawable'/'asel_inset.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android" android:inset="2dp" android:drawable="@drawable/asel_bg_color"/>')
   (res/'drawable'/'asel_ripple.xml').write_text('<ripple xmlns:android="http://schemas.android.com/apk/res/android" android:color="#33000000"><item android:drawable="@drawable/asel_bg_color"/></ripple>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <ImageView android:id="@+id/iv_src" android:src="@drawable/asel_src"/>
    <ImageButton android:id="@+id/ib_compat" app:srcCompat="@drawable/asel_src"/>
    <ImageView android:id="@+id/iv_both" android:src="@drawable/asel_src" android:background="@drawable/asel_bg_color"/>
    <View android:id="@+id/v_bg_color" android:background="@drawable/asel_bg_color"/>
    <View android:id="@+id/v_bg_layer" android:background="@drawable/asel_bg_layer"/>
    <View android:id="@+id/v_bg_img" android:background="@drawable/asel_bg_img"/>
    <View android:id="@+id/v_bg_inset" android:background="@drawable/asel_inset"/>
    <View android:id="@+id/v_bg_ripple" android:background="@drawable/asel_ripple"/>
    <ImageView android:id="@+id/iv_no_trans" android:src="@drawable/asel_no_trans"/>
    <FrameLayout android:id="@+id/fl_fg" android:foreground="@drawable/asel_bg_color"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['iv_src','ib_compat','iv_both','v_bg_color','v_bg_layer','v_bg_img','v_bg_inset','v_bg_ripple','iv_no_trans','fl_fg']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   # ImageView src & ImageButton srcCompat: preserve kind:253 + emit kind:293
   for key in ('iv_src','ib_compat'):
    self.assertIn('RelativeRule { kind:253, anchor:"66,0|ic_press|" }',slices[key])
    self.assertIn('RelativeRule { kind:253, anchor:"64,0|ic_on|" }',slices[key])
    self.assertIn('RelativeRule { kind:253, anchor:"0,0|ic_off|" }',slices[key])
    self.assertIn('RelativeRule { kind:293, anchor:"src:0:64:1|tr_f0:50,tr_f1:75" }',slices[key])
    self.assertIn('RelativeRule { kind:293, anchor:"src:64:66:0|asel_src__t1__f0:40,tr_f1:60" }',slices[key])
   # Both src and bg on same ImageView
   self.assertIn('RelativeRule { kind:293, anchor:"src:0:64:1|tr_f0:50,tr_f1:75" }',slices['iv_both'])
   self.assertIn('RelativeRule { kind:293, anchor:"bg:0:8:1|tr_f0:50,tr_f1:75" }',slices['iv_both'])
   # bg color-list path: preserve kind:96 + emit kind:293
   self.assertIn('RelativeRule { kind:96, anchor:"8,0,17,34,51,255" }',slices['v_bg_color'])
   self.assertIn('RelativeRule { kind:96, anchor:"0,0,68,85,102,255" }',slices['v_bg_color'])
   self.assertIn('RelativeRule { kind:293, anchor:"bg:0:8:1|tr_f0:50,tr_f1:75" }',slices['v_bg_color'])
   # bg stateful-layer path: preserve kind:231 + emit kind:293 + autoMirrored kind:255
   self.assertIn('kind:231, anchor:"16,0|',slices['v_bg_layer'])
   self.assertIn('RelativeRule { kind:293, anchor:"bg:0:16:0|tr_vec:90" }',slices['v_bg_layer'])
   self.assertIn('RelativeRule { kind:255, anchor:"true" }',slices['v_bg_layer'])
   # bg stateful-image path: preserve kind:254 + emit kind:293
   self.assertIn('RelativeRule { kind:254, anchor:"64,0|ic_on|" }',slices['v_bg_img'])
   self.assertIn('RelativeRule { kind:254, anchor:"0,0|ic_off|" }',slices['v_bg_img'])
   self.assertIn('RelativeRule { kind:293, anchor:"bg:0:64:0|tr_f0:30,asel_bg_img__t0__f1:45" }',slices['v_bg_img'])
   # inset & ripple wrappers propagate bg animated-selector transitions
   self.assertIn('RelativeRule { kind:293, anchor:"bg:0:8:1|tr_f0:50,tr_f1:75" }',slices['v_bg_inset'])
   self.assertIn('RelativeRule { kind:293, anchor:"bg:0:8:1|tr_f0:50,tr_f1:75" }',slices['v_bg_ripple'])
   self.assertIn('RelativeRule { kind:209, anchor:"0,0,0,51" }',slices['v_bg_ripple'])
   # no transitions & foreground do NOT emit kind:293
   self.assertNotIn('kind:293',slices['iv_no_trans'])
   self.assertNotIn('kind:293',slices['fl_fg'])
   # All transition frame images bundled
   for img_k in ('tr_f0','tr_f1','tr_vec','asel_src__t1__f0','asel_bg_img__t0__f1'):
    self.assertIn(f'state.images.entry("{img_k}".into())',out)
   # Strict error validation on bad transitions
   for bad_drw,err in [
    ('<animated-selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a" android:drawable="@drawable/sh_a"/><transition android:fromId="@id/a" android:drawable="@drawable/al_png"/></animated-selector>','Invalid animated-selector transition'),
    ('<animated-selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a" android:drawable="@drawable/sh_a"/><transition android:fromId="@id/missing" android:toId="@id/a" android:drawable="@drawable/al_png"/></animated-selector>','Invalid animated-selector transition fromId'),
    ('<animated-selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a" android:drawable="@drawable/sh_a"/><transition android:fromId="@id/a" android:toId="@id/missing" android:drawable="@drawable/al_png"/></animated-selector>','Invalid animated-selector transition toId'),
    ('<animated-selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a" android:drawable="@drawable/sh_a"/><transition android:fromId="@id/a" android:toId="@id/a" android:reversible="maybe" android:drawable="@drawable/al_png"/></animated-selector>','Invalid animated-selector transition reversible'),
    ('<animated-selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a" android:drawable="@drawable/sh_a"/><transition android:fromId="@id/a" android:toId="@id/a" android:drawable="@drawable/sh_a"/></animated-selector>','Unsupported animated-selector transition drawable'),
    ('<animated-selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a" android:drawable="@drawable/sh_a"/><transition android:fromId="@id/a" android:toId="@id/a"><shape><solid android:color="#112233"/></shape></transition></animated-selector>','Unsupported animated-selector transition child'),
    ('<animated-selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:id="@+id/a" android:drawable="@drawable/sh_a"/><transition android:fromId="@id/a" android:toId="@id/a" android:drawable="@drawable/bad_asel"/></animated-selector>','Cyclic drawable reference'),
   ]:
    (res/'drawable'/'bad_asel.xml').write_text(bad_drw)
    with self.subTest(bad=bad_drw):
     with self.assertRaisesRegex(ValueError,err):
      self.run_layout('<View xmlns:android="http://schemas.android.com/apk/res/android" android:background="@drawable/bad_asel"/>',res)
 def test_wave21b_constraint_default_size_rule_294(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('''<resources>
    <string name="def_pct">percent</string>
    <style name="WrapW">
     <item name="layout_constraintWidth_default">wrap</item>
    </style>
    <style name="BothStyle" parent="WrapW">
     <item name="app:layout_constraintHeight_default">@string/def_pct</item>
    </style>
    <style name="BadDefStyle">
     <item name="layout_constraintWidth_default">match</item>
    </style>
   </resources>''')
   xml='''<androidx.constraintlayout.widget.ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <View android:id="@+id/v_omit"/>
    <View android:id="@+id/v_spread" app:layout_constraintWidth_default="spread" app:layout_constraintHeight_default="spread"/>
    <View android:id="@+id/v_w_wrap" app:layout_constraintWidth_default="wrap"/>
    <View android:id="@+id/v_h_pct" app:layout_constraintHeight_default="percent"/>
    <View android:id="@+id/v_both" app:layout_constraintWidth_default="percent" app:layout_constraintHeight_default="wrap"/>
    <View android:id="@+id/v_styled" style="@style/BothStyle"/>
    <View android:id="@+id/v_override_spread" style="@style/WrapW" app:layout_constraintWidth_default="spread"/>
   </androidx.constraintlayout.widget.ConstraintLayout>'''
   out=self.run_layout(xml,res)
   names=['v_omit','v_spread','v_w_wrap','v_h_pct','v_both','v_styled','v_override_spread']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertNotIn('kind:294',slices['v_omit'])
   self.assertNotIn('kind:294',slices['v_spread'])
   self.assertNotIn('kind:294',slices['v_override_spread'])
   self.assertIn('RelativeRule { kind:294, anchor:"wrap,spread" }',slices['v_w_wrap'])
   self.assertIn('RelativeRule { kind:294, anchor:"spread,percent" }',slices['v_h_pct'])
   self.assertIn('RelativeRule { kind:294, anchor:"percent,wrap" }',slices['v_both'])
   self.assertIn('RelativeRule { kind:294, anchor:"wrap,percent" }',slices['v_styled'])
   for bad_xml,err in [
    ('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintWidth_default="fixed"/></ConstraintLayout>','Invalid layout_constraintWidth_default'),
    ('<ConstraintLayout xmlns:app="http://schemas.android.com/apk/res-auto"><View app:layout_constraintHeight_default="match_parent"/></ConstraintLayout>','Invalid layout_constraintHeight_default'),
    ('<ConstraintLayout xmlns:android="http://schemas.android.com/apk/res/android"><View style="@style/BadDefStyle"/></ConstraintLayout>','Invalid layout_constraintWidth_default'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave21b_compound_animation_list_rule_295(self):
  import struct,zlib
  def make_png(w,h,rgba):
   def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
   rows=b''.join(b'\x00'+bytes(rgba)*w for _ in range(h))
   return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();(res/'values').mkdir()
   (res/'values'/'ints.xml').write_text('<resources><integer name="d_fast">55</integer></resources>')
   (res/'drawable'/'p0.png').write_bytes(make_png(16,16,(20,40,60,255)))
   (res/'drawable'/'p1.png').write_bytes(make_png(16,16,(200,40,60,255)))
   (res/'drawable'/'s0.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#112233"/><size android:width="14dp" android:height="14dp"/></shape>')
   (res/'drawable'/'s1.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#445566"/><size android:width="14dp" android:height="14dp"/></shape>')
   (res/'drawable'/'v_mir.xml').write_text('<vector xmlns:android="http://schemas.android.com/apk/res/android" android:width="16dp" android:height="16dp" android:viewportWidth="16" android:viewportHeight="16" android:autoMirrored="true"><path android:fillColor="#00aa88" android:pathData="M0,0 L16,8 L0,16 Z"/></vector>')
   (res/'drawable'/'al_sh.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="false"><item android:drawable="@drawable/s0" android:duration="80"/><item android:drawable="@drawable/s1" android:duration="120"/></animation-list>')
   (res/'drawable'/'al_png.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="true"><item android:drawable="@drawable/p0" android:duration="@integer/d_fast"/><item android:drawable="@drawable/p1" android:duration="90"/></animation-list>')
   (res/'drawable'/'al_vec.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:drawable="@drawable/v_mir" android:duration="70"/></animation-list>')
   (res/'drawable'/'al_in.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android" android:oneshot="true"><item android:duration="45"><shape><solid android:color="#ff0055"/></shape></item><item android:duration="65"><shape><solid android:color="#0055ff"/></shape></item></animation-list>')
   (res/'drawable'/'inset_al_ref.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android" android:inset="2dp" android:drawable="@drawable/al_sh"/>')
   (res/'drawable'/'inset_al_in.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android" android:insetLeft="1dp"><animation-list android:oneshot="true"><item android:drawable="@drawable/p0" android:duration="40"/><item android:drawable="@drawable/p1" android:duration="60"/></animation-list></inset>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <TextView android:id="@+id/tv_se" android:drawableStart="@drawable/al_sh" app:drawableEndCompat="@drawable/al_png"/>
    <TextView android:id="@+id/tv_tb" app:drawableTopCompat="@drawable/inset_al_ref" android:drawableBottom="@drawable/al_in"/>
    <Button android:id="@+id/bt_lr" android:drawableLeft="@drawable/al_vec" app:drawableRightCompat="@drawable/inset_al_in"/>
    <com.google.android.material.button.MaterialButton android:id="@+id/mb_ic" app:icon="@drawable/al_png"/>
    <TextView android:id="@+id/tv_static" android:drawableStart="@drawable/s0"/>
    <CheckBox android:id="@+id/cb_al" android:button="@drawable/al_sh"/>
    <RadioButton android:id="@+id/rb_ins" android:button="@drawable/inset_al_in"/>
    <CheckBox android:id="@+id/cb_static" android:button="@drawable/s0"/>
    <SeekBar android:id="@+id/sb_al" android:tickMark="@drawable/al_sh"/>
    <SeekBar android:id="@+id/sb_ins" android:tickMark="@drawable/inset_al_in"/>
    <SeekBar android:id="@+id/sb_static" android:tickMark="@drawable/s0"/>
    <ExpandableListView android:id="@+id/elv_al" android:groupIndicator="@drawable/al_png"/>
    <ExpandableListView android:id="@+id/elv_ins" android:groupIndicator="@drawable/inset_al_ref"/>
    <ExpandableListView android:id="@+id/elv_static" android:groupIndicator="@drawable/p0"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['tv_se','tv_tb','bt_lr','mb_ic','tv_static','cb_al','rb_ins','cb_static','sb_al','sb_ins','sb_static','elv_al','elv_ins','elv_static']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   # Compound drawables (all 6 slots + compat + MaterialButton icon + inset + inline shape + autoMirrored)
   self.assertIn('RelativeRule { kind:173, anchor:"14.0,14.0,0.0,17,34,51,255" }',slices['tv_se'])
   self.assertIn('RelativeRule { kind:295, anchor:"start:0|s0:80,s1:120" }',slices['tv_se'])
   self.assertIn('RelativeRule { kind:174, anchor:"16.0,16.0,0.0,0,0,0,255" }',slices['tv_se'])
   self.assertIn('RelativeRule { kind:295, anchor:"end:1|p0:55,p1:90" }',slices['tv_se'])
   self.assertIn('RelativeRule { kind:170, anchor:"14.0,14.0,0.0,17,34,51,255" }',slices['tv_tb'])
   self.assertIn('RelativeRule { kind:295, anchor:"top:0|s0:80,s1:120" }',slices['tv_tb'])
   self.assertIn('RelativeRule { kind:172, anchor:"24.0,24.0,0.0,0,0,0,255" }',slices['tv_tb'])
   self.assertIn('RelativeRule { kind:295, anchor:"bottom:1|al_in__f0:45,al_in__f1:65" }',slices['tv_tb'])
   self.assertIn('RelativeRule { kind:295, anchor:"left:0|v_mir:70" }',slices['bt_lr'])
   self.assertIn('RelativeRule { kind:255, anchor:"true" }',slices['bt_lr'])
   self.assertIn('RelativeRule { kind:295, anchor:"right:1|p0:40,p1:60" }',slices['bt_lr'])
   self.assertIn('RelativeRule { kind:295, anchor:"start:1|p0:55,p1:90" }',slices['mb_ic'])
   self.assertNotIn('kind:295',slices['tv_static'])
   # Checkable button (shape + inset PNG)
   self.assertIn('RelativeRule { kind:165, anchor:"0,0,14.0,14.0,0,17,34,51,255" }',slices['cb_al'])
   self.assertIn('RelativeRule { kind:295, anchor:"button:0|s0:80,s1:120" }',slices['cb_al'])
   self.assertIn('RelativeRule { kind:165, anchor:"0,0,16.0,16.0,0,0,0,0,255" }',slices['rb_ins'])
   self.assertIn('RelativeRule { kind:295, anchor:"button:1|p0:40,p1:60" }',slices['rb_ins'])
   self.assertNotIn('kind:295',slices['cb_static'])
   # SeekBar tickMark (shape + inset PNG)
   self.assertIn('RelativeRule { kind:155, anchor:"14.0,14.0,0,17,34,51,255" }',slices['sb_al'])
   self.assertIn('RelativeRule { kind:295, anchor:"tick:0|s0:80,s1:120" }',slices['sb_al'])
   self.assertIn('RelativeRule { kind:155, anchor:"16.0,16.0,0,0,0,0,255" }',slices['sb_ins'])
   self.assertIn('RelativeRule { kind:295, anchor:"tick:1|p0:40,p1:60" }',slices['sb_ins'])
   self.assertNotIn('kind:295',slices['sb_static'])
   # ExpandableListView groupIndicator (PNG + inset shape)
   self.assertIn('RelativeRule { kind:131, anchor:"0,0,p0" }',slices['elv_al'])
   self.assertIn('RelativeRule { kind:295, anchor:"indicator:1|p0:55,p1:90" }',slices['elv_al'])
   self.assertIn('RelativeRule { kind:131, anchor:"0,0,s0" }',slices['elv_ins'])
   self.assertIn('RelativeRule { kind:295, anchor:"indicator:0|s0:80,s1:120" }',slices['elv_ins'])
   self.assertNotIn('kind:295',slices['elv_static'])
   # All frame images bundled
   for img_k in ('p0','p1','s0','s1','v_mir','al_in__f0','al_in__f1'):
    self.assertIn(f'state.images.entry("{img_k}".into())',out)
   # Strict validation on cyclic / malformed animation-list & inset
   (res/'drawable'/'cyc_al.xml').write_text('<animation-list xmlns:android="http://schemas.android.com/apk/res/android"><item android:drawable="@drawable/cyc_al" android:duration="50"/></animation-list>')
   (res/'drawable'/'cyc_ins.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android" android:drawable="@drawable/cyc_ins"/>')
   (res/'drawable'/'bad_ins.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android"/>')
   for xml_bad,err in [
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:drawableStart="@drawable/cyc_al"/>','Cyclic drawable reference'),
    ('<CheckBox xmlns:android="http://schemas.android.com/apk/res/android" android:button="@drawable/cyc_ins"/>','Cyclic drawable reference'),
    ('<SeekBar xmlns:android="http://schemas.android.com/apk/res/android" android:tickMark="@drawable/bad_ins"/>','Invalid inset drawable'),
    ('<ExpandableListView xmlns:android="http://schemas.android.com/apk/res/android" android:groupIndicator="@drawable/cyc_al"/>','Cyclic drawable reference'),
   ]:
    with self.subTest(bad=xml_bad):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(xml_bad,res)
 def test_wave21b_text_layout_ext_rule_296(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'styles.xml').write_text('''<resources>
    <dimen name="as_min">10sp</dimen>
    <dimen name="as_max">28sp</dimen>
    <dimen name="as_step">1.5sp</dimen>
    <style name="JustifiedAppearance">
     <item name="android:breakStrategy">balanced</item>
     <item name="android:hyphenationFrequency">fullFast</item>
     <item name="android:justificationMode">inter_word</item>
    </style>
    <style name="AutoSizeBase">
     <item name="android:autoSizeTextType">uniform</item>
     <item name="app:autoSizeMinTextSize">@dimen/as_min</item>
     <item name="autoSizeMaxTextSize">@dimen/as_max</item>
    </style>
    <style name="AutoSizeChild" parent="AutoSizeBase">
     <item name="android:autoSizeStepGranularity">@dimen/as_step</item>
     <item name="android:breakStrategy">high_quality</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <TextView android:id="@+id/tv_plain" android:text="Plain"/>
    <TextView android:id="@+id/tv_defaults" android:text="Def" android:breakStrategy="simple" android:hyphenationFrequency="none" android:justificationMode="none" app:autoSizeTextType="none"/>
    <TextView android:id="@+id/tv_bh" android:text="BH" android:breakStrategy="balanced" android:hyphenationFrequency="normalFast"/>
    <TextView android:id="@+id/tv_just" android:text="Just" android:justificationMode="inter_word"/>
    <TextView android:id="@+id/tv_as_def" android:text="AutoDef" app:autoSizeTextType="uniform"/>
    <Button android:id="@+id/bt_as_and" android:text="Btn" android:autoSizeTextType="uniform" android:autoSizeMinTextSize="8sp" android:autoSizeMaxTextSize="20dp" android:autoSizeStepGranularity="2sp"/>
    <com.google.android.material.button.MaterialButton android:id="@+id/mb_styled" android:text="Styled" style="@style/AutoSizeChild" android:autoSizeMinTextSize="12sp"/>
    <TextView android:id="@+id/tv_override_none" android:text="Off" style="@style/AutoSizeBase" app:autoSizeTextType="none"/>
    <TextView android:id="@+id/tv_ta" android:text="TA" android:textAppearance="@style/JustifiedAppearance"/>
    <View android:id="@+id/v_nontext" android:justificationMode="inter_word"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['tv_plain','tv_defaults','tv_bh','tv_just','tv_as_def','bt_as_and','mb_styled','tv_override_none','tv_ta','v_nontext']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertNotIn('kind:296',slices['tv_plain'])
   self.assertNotIn('kind:296',slices['tv_defaults'])
   self.assertNotIn('kind:296',slices['tv_override_none'])
   self.assertNotIn('kind:296',slices['v_nontext'])
   self.assertIn('RelativeRule { kind:296, anchor:"balanced|normal|none|" }',slices['tv_bh'])
   self.assertIn('RelativeRule { kind:296, anchor:"simple|none|inter_word|" }',slices['tv_just'])
   self.assertIn('RelativeRule { kind:296, anchor:"simple|none|none|12,112,1" }',slices['tv_as_def'])
   self.assertIn('RelativeRule { kind:296, anchor:"simple|none|none|8,20,2" }',slices['bt_as_and'])
   self.assertIn('RelativeRule { kind:296, anchor:"high_quality|none|none|12,28,1.5" }',slices['mb_styled'])
   self.assertIn('RelativeRule { kind:296, anchor:"balanced|full|inter_word|" }',slices['tv_ta'])
   for bad_xml,err in [
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:breakStrategy="greedy"/>','Unsupported breakStrategy'),
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:hyphenationFrequency="ultra"/>','Unsupported hyphenationFrequency'),
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:justificationMode="inter_char"/>','Unsupported justificationMode'),
    ('<TextView xmlns:app="http://schemas.android.com/apk/res-auto" app:autoSizeTextType="preset"/>','Unsupported autoSizeTextType'),
    ('<TextView xmlns:app="http://schemas.android.com/apk/res-auto" app:autoSizeTextType="uniform" app:autoSizeMinTextSize="0sp"/>','Invalid autoSizeMinTextSize'),
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:autoSizeTextType="uniform" android:autoSizeMinTextSize="-4sp"/>','Invalid autoSizeMinTextSize'),
    ('<TextView xmlns:app="http://schemas.android.com/apk/res-auto" app:autoSizeTextType="uniform" app:autoSizeMaxTextSize="0dp"/>','Invalid autoSizeMaxTextSize'),
    ('<TextView xmlns:app="http://schemas.android.com/apk/res-auto" app:autoSizeTextType="uniform" app:autoSizeStepGranularity="0sp"/>','Invalid autoSizeStepGranularity'),
    ('<TextView xmlns:app="http://schemas.android.com/apk/res-auto" app:autoSizeTextType="uniform" app:autoSizeStepGranularity="2"/>','Invalid autoSizeStepGranularity'),
    ('<TextView xmlns:app="http://schemas.android.com/apk/res-auto" app:autoSizeTextType="uniform" app:autoSizeMinTextSize="30sp" app:autoSizeMaxTextSize="18sp"/>','Invalid autoSizeMinTextSize exceeds autoSizeMaxTextSize'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave21b_material_badge_ext_rule_297(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'menu').mkdir()
   (res/'values'/'dimens.xml').write_text('''<resources>
    <dimen name="bh_off">4dp</dimen>
    <dimen name="bv_off">-2.5dp</dimen>
    <integer name="b_max">3</integer>
    <style name="DirectBadgeStyle">
     <item name="badgeGravity">bottom_start</item>
     <item name="badgeMaxCharacterCount">2</item>
     <item name="badgeHorizontalOffset">6dp</item>
     <item name="badgeVerticalOffset">-1dp</item>
    </style>
   </resources>''')
   (res/'menu'/'nav_plain.xml').write_text('''<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <item android:id="@+id/m_home" android:title="Home" android:icon="@drawable/ic_home_24" app:badgeText="5"/>
    <item android:id="@+id/m_mail" android:title="Mail" android:icon="@drawable/ic_mail_24"/>
   </menu>''')
   (res/'menu'/'nav_ext.xml').write_text('''<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <item android:id="@+id/m_second" android:title="Second" android:orderInCategory="20" app:badgeText="1234" app:badgeGravity="bottom_end" app:badgeMaxCharacterCount="@integer/b_max" app:badgeHorizontalOffset="@dimen/bh_off" app:badgeVerticalOffset="@dimen/bv_off"/>
    <item android:id="@+id/m_hidden" android:title="Hidden" android:visible="false" app:badgeGravity="top_start"/>
    <item android:id="@+id/m_first" android:title="First" android:orderInCategory="10" app:badgeText="1"/>
    <item android:id="@+id/m_third" android:title="Third" android:orderInCategory="30" app:badgeGravity="TOP_START" app:badgeHorizontalOffset="@dimen/m3_badge_horizontal_offset"/>
   </menu>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.bottomnavigation.BottomNavigationView android:id="@+id/bn_plain" app:menu="@menu/nav_plain"/>
    <com.google.android.material.bottomnavigation.BottomNavigationView android:id="@+id/bn_ext" app:menu="@menu/nav_ext"/>
    <com.google.android.material.navigationrail.NavigationRailView android:id="@+id/nr_ext" app:menu="@menu/nav_ext"/>
    <com.google.android.material.navigation.NavigationView android:id="@+id/nv_ext" app:menu="@menu/nav_ext"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_ext" app:menu="@menu/nav_ext"/>
    <com.google.android.material.bottomnavigation.BottomNavigationView android:id="@+id/bn_direct" app:menu="@menu/nav_plain" app:badgeGravity="top_start" app:badgeVerticalOffset="3dp"/>
    <com.google.android.material.navigationrail.NavigationRailView android:id="@+id/nr_nomenu" app:badgeHorizontalOffset="5dp"/>
    <ImageButton android:id="@+id/ib_direct" style="@style/DirectBadgeStyle"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['bn_plain','bn_ext','nr_ext','nv_ext','bab_ext','bn_direct','nr_nomenu','ib_direct']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertNotIn('kind:297',slices['bn_plain'])
   for key in ('bn_ext','nr_ext','nv_ext','bab_ext'):
    self.assertNotIn('kind:297, anchor:"0|',slices[key])
    self.assertIn('RelativeRule { kind:297, anchor:"1|bottom_end|3|4,-2.5" }',slices[key])
    self.assertIn('RelativeRule { kind:297, anchor:"2|top_start|4|1.5,0" }',slices[key])
   self.assertIn('RelativeRule { kind:297, anchor:"0|top_start|4|0,3" }',slices['bn_direct'])
   self.assertIn('RelativeRule { kind:297, anchor:"1|top_start|4|0,3" }',slices['bn_direct'])
   self.assertIn('RelativeRule { kind:297, anchor:"0|top_end|4|5,0" }',slices['nr_nomenu'])
   self.assertIn('RelativeRule { kind:297, anchor:"self|bottom_start|2|6,-1" }',slices['ib_direct'])
   # Strict rejections in menu items and direct attributes
   for bad_item,err in [
    ('<item android:id="@+id/x" app:badgeGravity="center"/>','Unsupported badgeGravity'),
    ('<item android:id="@+id/x" app:badgeMaxCharacterCount="0"/>','Invalid badgeMaxCharacterCount'),
    ('<item android:id="@+id/x" app:badgeMaxCharacterCount="-3"/>','Invalid badgeMaxCharacterCount'),
    ('<item android:id="@+id/x" app:badgeMaxCharacterCount="two"/>','Invalid badgeMaxCharacterCount'),
    ('<item android:id="@+id/x" app:badgeHorizontalOffset="wide"/>','Invalid badgeHorizontalOffset'),
    ('<item android:id="@+id/x" app:badgeVerticalOffset="tall"/>','Invalid badgeVerticalOffset'),
   ]:
    (res/'menu'/'bad_nav.xml').write_text(f'<menu xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">{bad_item}</menu>')
    with self.subTest(bad=bad_item):
     with self.assertRaisesRegex(ValueError,err):
      self.run_layout('<com.google.android.material.bottomnavigation.BottomNavigationView xmlns:app="http://schemas.android.com/apk/res-auto" app:menu="@menu/bad_nav"/>',res)
   with self.assertRaisesRegex(ValueError,'Unsupported badgeGravity'):
    self.run_layout('<ImageButton xmlns:app="http://schemas.android.com/apk/res-auto" app:badgeGravity="middle"/>',res)
 def test_wave22b_collapsing_toolbar_ext_rule_298(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <string name="ctl_sub">Inbox | All</string>
    <integer name="ctl_scrim_ms">450</integer>
    <integer name="ctl_lines">3</integer>
    <item name="hero_mult" format="float" type="dimen">0.75</item>
    <style name="FadeCtlStyle">
     <item name="titleCollapseMode">fade</item>
     <item name="maxLines">@integer/ctl_lines</item>
     <item name="scrimAnimationDuration">@integer/ctl_scrim_ms</item>
     <item name="subtitle">@string/ctl_sub</item>
    </style>
    <style name="ParallaxHeroStyle">
     <item name="layout_collapseMode">parallax</item>
     <item name="layout_collapseParallaxMultiplier">@dimen/hero_mult</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_plain"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_defaults" app:titleCollapseMode="scale" app:maxLines="1" app:scrimAnimationDuration="0" app:subtitle=""/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_null_sub" app:subtitle="@null"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_fade" app:titleCollapseMode="fade"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_lines" app:maxLines="2"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_scrim" app:scrimAnimationDuration="@integer/ctl_scrim_ms"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_sub_only" app:subtitle="@string/ctl_sub"/>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_styled" style="@style/FadeCtlStyle">
     <ImageView android:id="@+id/ch_none" app:layout_collapseMode="none" app:layout_collapseParallaxMultiplier="0.7"/>
     <ImageView android:id="@+id/ch_pin" app:layout_collapseMode="pin"/>
     <ImageView android:id="@+id/ch_par_def" app:layout_collapseMode="parallax"/>
     <ImageView android:id="@+id/ch_par_zero" app:layout_collapseMode="parallax" app:layout_collapseParallaxMultiplier="0.0"/>
     <ImageView android:id="@+id/ch_par_one" app:layout_collapseMode="parallax" app:layout_collapseParallaxMultiplier="1.0"/>
     <ImageView android:id="@+id/ch_par_styled" style="@style/ParallaxHeroStyle"/>
     <FrameLayout android:id="@+id/ch_wrap">
      <ImageView android:id="@+id/grandchild_pin" app:layout_collapseMode="pin"/>
     </FrameLayout>
    </com.google.android.material.appbar.CollapsingToolbarLayout>
    <com.google.android.material.appbar.CollapsingToolbarLayout android:id="@+id/ctl_override_def" style="@style/FadeCtlStyle" app:titleCollapseMode="scale" app:maxLines="1" app:scrimAnimationDuration="0" app:subtitle="@null"/>
    <ImageView android:id="@+id/outside_par" app:layout_collapseMode="parallax" app:layout_collapseParallaxMultiplier="0.6"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['ctl_plain','ctl_defaults','ctl_null_sub','ctl_fade','ctl_lines','ctl_scrim','ctl_sub_only','ctl_styled','ch_none','ch_pin','ch_par_def','ch_par_zero','ch_par_one','ch_par_styled','ch_wrap','grandchild_pin','ctl_override_def','outside_par']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   for omitted in ('ctl_plain','ctl_defaults','ctl_null_sub','ch_none','ch_wrap','grandchild_pin','ctl_override_def','outside_par'):
    self.assertNotIn('kind:298',slices[omitted],omitted)
   self.assertIn('RelativeRule { kind:298, anchor:"fade|1|0|" }',slices['ctl_fade'])
   self.assertIn('RelativeRule { kind:298, anchor:"scale|2|0|" }',slices['ctl_lines'])
   self.assertIn('RelativeRule { kind:298, anchor:"scale|1|450|" }',slices['ctl_scrim'])
   self.assertIn('RelativeRule { kind:298, anchor:"scale|1|0|Inbox / All" }',slices['ctl_sub_only'])
   self.assertIn('RelativeRule { kind:298, anchor:"fade|3|450|Inbox / All" }',slices['ctl_styled'])
   self.assertIn('RelativeRule { kind:298, anchor:"collapse:pin:0.5" }',slices['ch_pin'])
   self.assertIn('RelativeRule { kind:298, anchor:"collapse:parallax:0.5" }',slices['ch_par_def'])
   self.assertIn('RelativeRule { kind:298, anchor:"collapse:parallax:0" }',slices['ch_par_zero'])
   self.assertIn('RelativeRule { kind:298, anchor:"collapse:parallax:1" }',slices['ch_par_one'])
   self.assertIn('RelativeRule { kind:298, anchor:"collapse:parallax:0.75" }',slices['ch_par_styled'])
   # merge_parent CollapsingToolbarLayout direct child also emits collapse rule 298
   merge_p=Path(d)/'merge_ctl.xml'; merge_p.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto"><ImageView android:id="@+id/m_hero" app:layout_collapseMode="parallax"/></merge>')
   merge_out=mod.compile_layout(merge_p,res,merge_parent='com.google.android.material.appbar.CollapsingToolbarLayout')
   self.assertIn('RelativeRule { kind:298, anchor:"collapse:parallax:0.5" }',merge_out)
   for bad_xml,err in [
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:titleCollapseMode="slide"/>','Unsupported titleCollapseMode'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:maxLines="0"/>','Invalid maxLines'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:maxLines="-2"/>','Invalid maxLines'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:scrimAnimationDuration="-5"/>','Invalid scrimAnimationDuration'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:scrimAnimationDuration="fast"/>','Invalid scrimAnimationDuration'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto"><ImageView app:layout_collapseMode="sticky"/></com.google.android.material.appbar.CollapsingToolbarLayout>','Unsupported layout_collapseMode'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto"><ImageView app:layout_collapseMode="parallax" app:layout_collapseParallaxMultiplier="1.2"/></com.google.android.material.appbar.CollapsingToolbarLayout>','Invalid layout_collapseParallaxMultiplier'),
    ('<com.google.android.material.appbar.CollapsingToolbarLayout xmlns:app="http://schemas.android.com/apk/res-auto"><ImageView app:layout_collapseMode="parallax" app:layout_collapseParallaxMultiplier="-0.1"/></com.google.android.material.appbar.CollapsingToolbarLayout>','Invalid layout_collapseParallaxMultiplier'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave22b_extended_fab_ext_rule_299(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <bool name="ef_collapsed">false</bool>
    <dimen name="ef_csize">64.5dp</dimen>
    <style name="CustomEfab">
     <item name="extended">false</item>
     <item name="collapsedSize">44dp</item>
     <item name="extendStrategy">match_parent</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_plain" android:text="New"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_defaults" android:text="New" app:extended="true" app:collapsedSize="56dp" app:extendStrategy="auto"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_null_size" android:text="New" app:collapsedSize="@null"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_collapsed" android:text="New" app:extended="@bool/ef_collapsed"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_csize_dp" android:text="New" app:collapsedSize="48dp"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_csize_res" android:text="New" app:collapsedSize="@dimen/ef_csize"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_wrap" android:text="New" app:extendStrategy="wrap_content"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_match" android:text="New" app:extendStrategy="match_parent"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_styled" style="@style/CustomEfab" android:text="New"/>
    <com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton android:id="@+id/ef_override_def" style="@style/CustomEfab" android:text="New" app:extended="true" app:collapsedSize="56dp" app:extendStrategy="auto"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['ef_plain','ef_defaults','ef_null_size','ef_collapsed','ef_csize_dp','ef_csize_res','ef_wrap','ef_match','ef_styled','ef_override_def']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   for omitted in ('ef_plain','ef_defaults','ef_null_size','ef_override_def'):
    self.assertNotIn('kind:299',slices[omitted],omitted)
    self.assertIn('RelativeRule { kind:242, anchor:"1" }',slices[omitted])
   self.assertIn('RelativeRule { kind:299, anchor:"0|56|auto" }',slices['ef_collapsed'])
   self.assertIn('RelativeRule { kind:299, anchor:"1|48|auto" }',slices['ef_csize_dp'])
   self.assertIn('RelativeRule { kind:299, anchor:"1|64.5|auto" }',slices['ef_csize_res'])
   self.assertIn('RelativeRule { kind:299, anchor:"1|56|wrap_content" }',slices['ef_wrap'])
   self.assertIn('RelativeRule { kind:299, anchor:"1|56|match_parent" }',slices['ef_match'])
   self.assertIn('RelativeRule { kind:299, anchor:"0|44|match_parent" }',slices['ef_styled'])
   for bad_xml,err in [
    ('<com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton xmlns:app="http://schemas.android.com/apk/res-auto" app:extended="yes"/>','Invalid extended'),
    ('<com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton xmlns:app="http://schemas.android.com/apk/res-auto" app:collapsedSize="-8dp"/>','Expected dp/sp'),
    ('<com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton xmlns:app="http://schemas.android.com/apk/res-auto" app:extendStrategy="fill"/>','Unsupported extendStrategy'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave22b_bottom_app_bar_ext_rule_300(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <dimen name="bab_margin">6dp</dimen>
    <dimen name="bab_rad">10dp</dimen>
    <dimen name="bab_voff">2.5dp</dimen>
    <bool name="bab_hide">true</bool>
    <style name="CustomBab">
     <item name="fabAlignmentMode">end</item>
     <item name="fabAnchorMode">embed</item>
     <item name="fabCradleMargin">@dimen/bab_margin</item>
     <item name="fabCradleRoundedCornerRadius">@dimen/bab_rad</item>
     <item name="fabCradleVerticalOffset">@dimen/bab_voff</item>
     <item name="hideOnScroll">@bool/bab_hide</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_plain"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_other" app:fabAnimationMode="slide" app:menuAlignmentMode="start" app:addElevationShadow="false"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_exp_def" app:fabAlignmentMode="center"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_hide_false" app:hideOnScroll="false"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_end" app:fabAlignmentMode="end"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_embed" app:fabAnchorMode="embed"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_cradle" app:fabCradleMargin="8dp" app:fabCradleRoundedCornerRadius="12.5dp" app:fabCradleVerticalOffset="4dp"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_hide_true" app:hideOnScroll="true"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_styled" style="@style/CustomBab"/>
    <com.google.android.material.bottomappbar.BottomAppBar android:id="@+id/bab_override" style="@style/CustomBab" app:fabAlignmentMode="center" app:hideOnScroll="false"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['bab_plain','bab_other','bab_exp_def','bab_hide_false','bab_end','bab_embed','bab_cradle','bab_hide_true','bab_styled','bab_override']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   for omitted in ('bab_plain','bab_other'):
    self.assertNotIn('kind:300',slices[omitted],omitted)
    self.assertIn('RelativeRule { kind:243, anchor:"center" }',slices[omitted])
   self.assertIn('RelativeRule { kind:300, anchor:"center|cradle|5,8,0|0" }',slices['bab_exp_def'])
   self.assertIn('RelativeRule { kind:300, anchor:"center|cradle|5,8,0|0" }',slices['bab_hide_false'])
   self.assertIn('RelativeRule { kind:300, anchor:"end|cradle|5,8,0|0" }',slices['bab_end'])
   self.assertIn('RelativeRule { kind:300, anchor:"center|embed|5,8,0|0" }',slices['bab_embed'])
   self.assertIn('RelativeRule { kind:300, anchor:"center|cradle|8,12.5,4|0" }',slices['bab_cradle'])
   self.assertIn('RelativeRule { kind:300, anchor:"center|cradle|5,8,0|1" }',slices['bab_hide_true'])
   self.assertIn('RelativeRule { kind:300, anchor:"end|embed|6,10,2.5|1" }',slices['bab_styled'])
   self.assertIn('RelativeRule { kind:300, anchor:"center|embed|6,10,2.5|0" }',slices['bab_override'])
   for bad_xml,err in [
    ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:fabAlignmentMode="left"/>','Unsupported fabAlignmentMode'),
    ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:fabAnchorMode="float"/>','Unsupported fabAnchorMode'),
    ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:fabCradleMargin="-2dp"/>','Expected dp/sp'),
    ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:fabCradleRoundedCornerRadius="-1dp"/>','Expected dp/sp'),
    ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:fabCradleVerticalOffset="-3dp"/>','Expected dp/sp'),
    ('<com.google.android.material.bottomappbar.BottomAppBar xmlns:app="http://schemas.android.com/apk/res-auto" app:hideOnScroll="maybe"/>','Invalid hideOnScroll'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave22b_recycler_layout_ext_rule_301(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <string name="rv_stag_mgr">androidx.recyclerview.widget.StaggeredGridLayoutManager</string>
    <integer name="rv_spans">2</integer>
    <bool name="rv_stack">true</bool>
    <style name="StaggeredRvStyle">
     <item name="layoutManager">androidx.recyclerview.widget.StaggeredGridLayoutManager</item>
     <item name="android:orientation">horizontal</item>
     <item name="spanCount">3</item>
     <item name="reverseLayout">true</item>
     <item name="stackFromEnd">true</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_plain"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_lin_v" app:layoutManager="androidx.recyclerview.widget.LinearLayoutManager" android:orientation="vertical" app:reverseLayout="false" app:stackFromEnd="false"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_grid_v" app:layoutManager="androidx.recyclerview.widget.GridLayoutManager" app:spanCount="3"/>
    <android.support.v7.widget.RecyclerView android:id="@+id/rv_sup_plain"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_stag" app:layoutManager="@string/rv_stag_mgr" app:spanCount="@integer/rv_spans"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_lin_h" app:layoutManager="androidx.recyclerview.widget.LinearLayoutManager" android:orientation="horizontal"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_grid_hr" app:layoutManager="androidx.recyclerview.widget.GridLayoutManager" android:orientation="horizontal" app:spanCount="3" app:reverseLayout="true"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_stack" app:layoutManager="androidx.recyclerview.widget.LinearLayoutManager" app:stackFromEnd="@bool/rv_stack"/>
    <android.support.v7.widget.RecyclerView android:id="@+id/rv_sup_full" app:layoutManager="android.support.v7.widget.StaggeredGridLayoutManager" android:orientation="horizontal" app:spanCount="4" app:reverseLayout="true" app:stackFromEnd="true"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_styled" style="@style/StaggeredRvStyle"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_override_v" style="@style/StaggeredRvStyle" app:layoutManager="androidx.recyclerview.widget.LinearLayoutManager" android:orientation="vertical" app:reverseLayout="false" app:stackFromEnd="false"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['rv_plain','rv_lin_v','rv_grid_v','rv_sup_plain','rv_stag','rv_lin_h','rv_grid_hr','rv_stack','rv_sup_full','rv_styled','rv_override_v']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   for omitted in ('rv_plain','rv_lin_v','rv_grid_v','rv_sup_plain','rv_override_v'):
    self.assertNotIn('kind:301',slices[omitted],omitted)
   self.assertIn('kind:Kind::ListView,id:"rv_sup_plain"',out)
   self.assertIn('kind:Kind::GridView,id:"rv_stag"',out)
   self.assertIn('RelativeRule { kind:91, anchor:"2" }',slices['rv_stag'])
   self.assertIn('RelativeRule { kind:301, anchor:"staggered|v|2|0|0" }',slices['rv_stag'])
   self.assertIn('horizontal:true',slices['rv_lin_h'])
   self.assertIn('RelativeRule { kind:301, anchor:"linear|h|1|0|0" }',slices['rv_lin_h'])
   self.assertIn('RelativeRule { kind:301, anchor:"grid|h|3|1|0" }',slices['rv_grid_hr'])
   self.assertIn('RelativeRule { kind:301, anchor:"linear|v|1|0|1" }',slices['rv_stack'])
   self.assertIn('RelativeRule { kind:301, anchor:"staggered|h|4|1|1" }',slices['rv_sup_full'])
   self.assertIn('RelativeRule { kind:301, anchor:"staggered|h|3|1|1" }',slices['rv_styled'])
   # custom_view_map targeting android.support.v7.widget.RecyclerView
   custom_p=Path(d)/'custom_rv.xml'; custom_p.write_text('<com.example.MyRv xmlns:app="http://schemas.android.com/apk/res-auto" app:reverseLayout="true"/>')
   custom_out=mod.compile_layout(custom_p,res,custom_view_map={'com.example.MyRv':'android.support.v7.widget.RecyclerView'})
   self.assertIn('RelativeRule { kind:301, anchor:"linear|v|1|1|0" }',custom_out)
   for bad_xml,err in [
    ('<androidx.recyclerview.widget.RecyclerView xmlns:app="http://schemas.android.com/apk/res-auto" app:spanCount="0"/>','Invalid spanCount'),
    ('<androidx.recyclerview.widget.RecyclerView xmlns:app="http://schemas.android.com/apk/res-auto" app:spanCount="-1"/>','Invalid spanCount'),
    ('<androidx.recyclerview.widget.RecyclerView xmlns:app="http://schemas.android.com/apk/res-auto" app:spanCount="257"/>','Invalid spanCount'),
    ('<androidx.recyclerview.widget.RecyclerView xmlns:app="http://schemas.android.com/apk/res-auto" app:spanCount="two"/>','Invalid spanCount'),
    ('<androidx.recyclerview.widget.RecyclerView xmlns:app="http://schemas.android.com/apk/res-auto" app:reverseLayout="yes"/>','Invalid reverseLayout'),
    ('<androidx.recyclerview.widget.RecyclerView xmlns:app="http://schemas.android.com/apk/res-auto" app:stackFromEnd="1"/>','Invalid stackFromEnd'),
    ('<androidx.recyclerview.widget.RecyclerView xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="diagonal"/>','Unsupported orientation'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave23b_search_view_ext_rule_302(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <string name="sv_hint">Find | Filter\nMail</string>
    <string name="sv_style_hint">Search archive</string>
    <bool name="sv_hide">true</bool>
    <bool name="sv_drawer">true</bool>
    <color name="sv_bg">#F3EDF7</color>
    <style name="CustomSearchViewStyle">
     <item name="layout_anchor">@id/sb_styled_anchor</item>
     <item name="android:hint">@string/sv_style_hint</item>
     <item name="hideNavigationIcon">true</item>
     <item name="useDrawerArrowDrawable">true</item>
     <item name="autoShowKeyboard">false</item>
     <item name="animateNavigationIcon">true</item>
     <item name="animateMenuItems">false</item>
     <item name="headerLayout">@layout/sv_header</item>
     <item name="backgroundTint">@color/sv_bg</item>
     <item name="elevation">6dp</item>
     <item name="android:text">Drafts</item>
    </style>
   </resources>''')
   xml='''<androidx.coordinatorlayout.widget.CoordinatorLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <com.google.android.material.search.SearchBar android:id="@+id/sb_main" android:hint="Search"/>
    <com.google.android.material.search.SearchView android:id="@+id/sv_plain"/>
    <com.google.android.material.search.SearchView android:id="@+id/sv_full"
     app:layout_anchor="@id/sb_main"
     android:hint="@string/sv_hint"
     app:hideNavigationIcon="@bool/sv_hide"
     app:useDrawerArrowDrawable="@bool/sv_drawer"
     app:autoShowKeyboard="true"
     app:animateNavigationIcon="false"
     app:animateMenuItems="true"
     app:headerLayout="@layout/sv_hdr"
     app:backgroundTint="?attr/colorSurfaceContainerHigh"
     app:elevation="8dp"
     android:text="Inbox">
     <TextView android:id="@+id/sv_child" android:text="Recent query"/>
    </com.google.android.material.search.SearchView>
    <com.google.android.material.search.SearchView android:id="@+id/sv_plus" app:layout_anchor="@+id/bar_plus" android:hint="Quick find" style="?attr/materialSearchViewStyle"/>
    <com.google.android.material.search.SearchView android:id="@+id/sv_nulls" app:layout_anchor="@null" android:hint="@null" app:headerLayout="@null"/>
    <com.google.android.material.search.SearchView android:id="@+id/sv_styled" style="@style/CustomSearchViewStyle"/>
    <com.google.android.material.search.SearchView android:id="@+id/sv_override" style="@style/CustomSearchViewStyle" app:layout_anchor="@null" android:hint="Direct" app:hideNavigationIcon="false" app:useDrawerArrowDrawable="false"/>
   </androidx.coordinatorlayout.widget.CoordinatorLayout>'''
   out=self.run_layout(xml,res)
   names=['sb_main','sv_plain','sv_full','sv_child','sv_plus','sv_nulls','sv_styled','sv_override']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   self.assertNotIn('kind:302',slices['sb_main'])
   self.assertIn('kind:Kind::Frame,id:"sv_plain"',out)
   self.assertIn('RelativeRule { kind:302, anchor:"||0|0" }',slices['sv_plain'])
   self.assertNotIn('kind:106',slices['sv_plain'])
   self.assertNotIn('kind:107',slices['sv_plain'])
   self.assertIn('kind:Kind::Frame,id:"sv_full"',out)
   self.assertIn('RelativeRule { kind:302, anchor:"sb_main|Find / Filter Mail|1|1" }',slices['sv_full'])
   self.assertNotIn('kind:227',slices['sv_full'])
   self.assertIn('RelativeRule { kind:193, anchor:"8.0" }',slices['sv_full'])
   self.assertIn('RelativeRule { kind:194, anchor:"236,230,240,255" }',slices['sv_full'])
   self.assertIn('text:"Inbox"',slices['sv_full'])
   self.assertIn('text:"Recent query"',slices['sv_child'])
   self.assertIn('RelativeRule { kind:302, anchor:"bar_plus|Quick find|0|0" }',slices['sv_plus'])
   self.assertIn('RelativeRule { kind:302, anchor:"||0|0" }',slices['sv_nulls'])
   self.assertIn('RelativeRule { kind:302, anchor:"sb_styled_anchor|Search archive|1|1" }',slices['sv_styled'])
   self.assertIn('RelativeRule { kind:193, anchor:"6.0" }',slices['sv_styled'])
   self.assertIn('RelativeRule { kind:194, anchor:"243,237,247,255" }',slices['sv_styled'])
   self.assertIn('text:"Drafts"',slices['sv_styled'])
   self.assertIn('RelativeRule { kind:302, anchor:"|Direct|0|0" }',slices['sv_override'])
   # merge_parent and custom_view_map
   merge_p=Path(d)/'merge_sv.xml'; merge_p.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android"><TextView android:text="Item"/></merge>')
   merge_out=mod.compile_layout(merge_p,res,merge_parent='com.google.android.material.search.SearchView')
   self.assertIn('kind:Kind::Frame',merge_out)
   self.assertIn('RelativeRule { kind:302, anchor:"||0|0" }',merge_out)
   custom_p=Path(d)/'custom_sv.xml'; custom_p.write_text('<com.example.MySv xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto" app:layout_anchor="@id/top_bar" android:hint="Custom"/>')
   custom_out=mod.compile_layout(custom_p,res,custom_view_map={'com.example.MySv':'com.google.android.material.search.SearchView'})
   self.assertIn('kind:Kind::Frame',custom_out)
   self.assertIn('RelativeRule { kind:302, anchor:"top_bar|Custom|0|0" }',custom_out)
   for bad_xml,err in [
    ('<com.google.android.material.search.SearchView xmlns:app="http://schemas.android.com/apk/res-auto" app:layout_anchor="not_an_id"/>','Invalid layout_anchor'),
    ('<com.google.android.material.search.SearchView xmlns:app="http://schemas.android.com/apk/res-auto" app:hideNavigationIcon="yes"/>','Invalid hideNavigationIcon'),
    ('<com.google.android.material.search.SearchView xmlns:app="http://schemas.android.com/apk/res-auto" app:useDrawerArrowDrawable="1"/>','Invalid useDrawerArrowDrawable'),
    ('<com.google.android.material.search.SearchView xmlns:app="http://schemas.android.com/apk/res-auto" app:autoShowKeyboard="always"/>','Invalid autoShowKeyboard'),
    ('<com.google.android.material.search.SearchView xmlns:app="http://schemas.android.com/apk/res-auto" app:animateNavigationIcon="nope"/>','Invalid animateNavigationIcon'),
    ('<com.google.android.material.search.SearchView xmlns:app="http://schemas.android.com/apk/res-auto" app:animateMenuItems="2"/>','Invalid animateMenuItems'),
    ('<com.google.android.material.search.SearchView xmlns:app="http://schemas.android.com/apk/res-auto" app:headerLayout="bad_ref"/>','Invalid headerLayout'),
    ('<FrameLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:hideNavigationIcon="true"/>','Unsupported attribute on FrameLayout'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave23b_swipe_refresh_ext_rule_303(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <color name="sr_bg">#F7F2FA</color>
    <color name="sr_c1">#6750A4</color>
    <color name="sr_c2">#B3261E</color>
    <string name="sr_scheme_str">@color/sr_c1; ?attr/colorSecondary</string>
    <array name="sr_scheme_arr">
     <item>#112233</item>
     <item>@color/sr_c2</item>
    </array>
    <dimen name="sr_off_start">-24.5dp</dimen>
    <dimen name="sr_off_end">80dp</dimen>
    <style name="CustomSwipeStyle">
     <item name="swipeRefreshLayoutProgressSpinnerBackgroundColor">@color/sr_bg</item>
     <item name="swipeRefreshProgressColor">@color/sr_c1, @color/sr_c2</item>
     <item name="swipeRefreshSize">large</item>
     <item name="swipeRefreshOffsetStart">@dimen/sr_off_start</item>
     <item name="swipeRefreshOffsetEnd">@dimen/sr_off_end</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_plain"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <android.support.v4.widget.SwipeRefreshLayout android:id="@+id/sr_sup_plain"><View/></android.support.v4.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_bg_only" app:swipeRefreshLayoutProgressSpinnerBackgroundColor="#FAFAFA"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_bg_theme" app:swipeRefreshLayoutProgressSpinnerBackgroundColor="?attr/colorSurfaceContainerLow"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_single_col" app:swipeRefreshProgressColor="#6750A4"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_comma_col" app:swipeRefreshProgressColor="@color/sr_c1, #00FF00"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_str_col" app:swipeRefreshProgressColor="@string/sr_scheme_str"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_arr_col" app:swipeRefreshProgressColor="@array/sr_scheme_arr"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_size_only" app:swipeRefreshSize="large"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_offs_only" app:swipeRefreshOffsetStart="-16dp" app:swipeRefreshOffsetEnd="96.5dp"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <android.support.v4.widget.SwipeRefreshLayout android:id="@+id/sr_sup_full" app:swipeRefreshLayoutProgressSpinnerBackgroundColor="#FFFFFF" app:swipeRefreshProgressColor="#112233;#445566" app:swipeRefreshSize="large" app:swipeRefreshOffsetStart="0dp" app:swipeRefreshOffsetEnd="72dp"><View/></android.support.v4.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_styled" style="@style/CustomSwipeStyle"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
    <androidx.swiperefreshlayout.widget.SwipeRefreshLayout android:id="@+id/sr_override" style="@style/CustomSwipeStyle" app:swipeRefreshLayoutProgressSpinnerBackgroundColor="@null" app:swipeRefreshProgressColor="@null" app:swipeRefreshSize="default"><View/></androidx.swiperefreshlayout.widget.SwipeRefreshLayout>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['sr_plain','sr_sup_plain','sr_bg_only','sr_bg_theme','sr_single_col','sr_comma_col','sr_str_col','sr_arr_col','sr_size_only','sr_offs_only','sr_sup_full','sr_styled','sr_override']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   for omitted in ('sr_plain','sr_sup_plain'):
    self.assertIn('RelativeRule { kind:246, anchor:"" }',slices[omitted])
    self.assertNotIn('kind:303',slices[omitted],omitted)
   self.assertIn('kind:Kind::Frame,id:"sr_sup_plain"',out)
   self.assertIn('RelativeRule { kind:303, anchor:"250,250,250,255||default|-40,64" }',slices['sr_bg_only'])
   self.assertIn('RelativeRule { kind:303, anchor:"247,242,250,255||default|-40,64" }',slices['sr_bg_theme'])
   self.assertIn('RelativeRule { kind:303, anchor:"|103,80,164,255|default|-40,64" }',slices['sr_single_col'])
   self.assertIn('RelativeRule { kind:303, anchor:"|103,80,164,255;0,255,0,255|default|-40,64" }',slices['sr_comma_col'])
   self.assertIn('RelativeRule { kind:303, anchor:"|103,80,164,255;98,91,113,255|default|-40,64" }',slices['sr_str_col'])
   self.assertIn('RelativeRule { kind:303, anchor:"|17,34,51,255;179,38,30,255|default|-40,64" }',slices['sr_arr_col'])
   self.assertIn('RelativeRule { kind:303, anchor:"||large|-40,64" }',slices['sr_size_only'])
   self.assertIn('RelativeRule { kind:303, anchor:"||default|-16,96.5" }',slices['sr_offs_only'])
   self.assertIn('RelativeRule { kind:303, anchor:"255,255,255,255|17,34,51,255;68,85,102,255|large|0,72" }',slices['sr_sup_full'])
   self.assertIn('RelativeRule { kind:303, anchor:"247,242,250,255|103,80,164,255;179,38,30,255|large|-24.5,80" }',slices['sr_styled'])
   self.assertIn('RelativeRule { kind:303, anchor:"||default|-24.5,80" }',slices['sr_override'])
   # merge_parent and custom_view_map for support.v4 SwipeRefreshLayout
   merge_p=Path(d)/'merge_sr.xml'; merge_p.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android"><View/></merge>')
   merge_out=mod.compile_layout(merge_p,res,merge_parent='android.support.v4.widget.SwipeRefreshLayout')
   self.assertIn('RelativeRule { kind:246, anchor:"" }',merge_out)
   self.assertNotIn('kind:303',merge_out)
   custom_p=Path(d)/'custom_sr.xml'; custom_p.write_text('<com.example.MySr xmlns:app="http://schemas.android.com/apk/res-auto" app:swipeRefreshSize="large"/>')
   custom_out=mod.compile_layout(custom_p,res,custom_view_map={'com.example.MySr':'android.support.v4.widget.SwipeRefreshLayout'})
   self.assertIn('RelativeRule { kind:303, anchor:"||large|-40,64" }',custom_out)
   for bad_xml,err in [
    ('<androidx.swiperefreshlayout.widget.SwipeRefreshLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:swipeRefreshLayoutProgressSpinnerBackgroundColor="not_a_color"/>','Expected #RGB'),
    ('<androidx.swiperefreshlayout.widget.SwipeRefreshLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:swipeRefreshProgressColor="#112233,,#445566"/>','Invalid swipeRefreshProgressColor'),
    ('<androidx.swiperefreshlayout.widget.SwipeRefreshLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:swipeRefreshProgressColor="bad_color"/>','Expected #RGB'),
    ('<androidx.swiperefreshlayout.widget.SwipeRefreshLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:swipeRefreshSize="mini"/>','Unsupported swipeRefreshSize'),
    ('<androidx.swiperefreshlayout.widget.SwipeRefreshLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:swipeRefreshOffsetStart="far"/>','Expected margin'),
    ('<androidx.swiperefreshlayout.widget.SwipeRefreshLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:swipeRefreshOffsetEnd="abc"/>','Expected margin'),
    ('<FrameLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:swipeRefreshSize="large"/>','Unsupported attribute on FrameLayout'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave23b_carousel_layout_rule_304(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <string name="car_lm">com.google.android.material.carousel.CarouselLayoutManager</string>
    <dimen name="car_gap">12.5dp</dimen>
    <style name="HeroCarouselStyle">
     <item name="layoutManager">com.google.android.material.carousel.CarouselLayoutManager</item>
     <item name="carousel_strategy">hero</item>
     <item name="carousel_alignment">center</item>
     <item name="carousel_itemSpacing">@dimen/car_gap</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_plain" app:layoutManager="androidx.recyclerview.widget.LinearLayoutManager"/>
    <HorizontalScrollView android:id="@+id/hsv_plain"><LinearLayout android:id="@+id/ll_plain"/></HorizontalScrollView>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_car_lm" app:layoutManager="com.google.android.material.carousel.CarouselLayoutManager"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_car_full" app:layoutManager="@string/car_lm" app:carousel_strategy="hero" app:carousel_alignment="center" app:carousel_itemSpacing="@dimen/car_gap"/>
    <android.support.v7.widget.RecyclerView android:id="@+id/rv_sup_car" app:carousel_strategy="full_screen"/>
    <HorizontalScrollView android:id="@+id/hsv_car" app:carousel_strategy="uncontained" app:carousel_itemSpacing="0dp">
     <LinearLayout android:id="@+id/ll_inner" android:orientation="horizontal">
      <com.google.android.material.carousel.MaskableFrameLayout android:id="@+id/mfl_item"/>
     </LinearLayout>
    </HorizontalScrollView>
    <LinearLayout android:id="@+id/ll_car" android:orientation="horizontal" app:carousel_alignment="center" app:carousel_itemSpacing="16dp"/>
    <androidx.appcompat.widget.LinearLayoutCompat android:id="@+id/llc_car" app:carousel_strategy="multi_browse"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_styled" style="@style/HeroCarouselStyle"/>
    <androidx.recyclerview.widget.RecyclerView android:id="@+id/rv_override" style="@style/HeroCarouselStyle" app:carousel_strategy="uncontained" app:carousel_alignment="start" app:carousel_itemSpacing="4dp"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['rv_plain','hsv_plain','ll_plain','rv_car_lm','rv_car_full','rv_sup_car','hsv_car','ll_inner','mfl_item','ll_car','llc_car','rv_styled','rv_override']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   for omitted in ('rv_plain','hsv_plain','ll_plain','ll_inner'):
    self.assertNotIn('kind:304',slices[omitted],omitted)
   self.assertIn('RelativeRule { kind:304, anchor:"multi_browse|8|start" }',slices['rv_car_lm'])
   self.assertIn('RelativeRule { kind:304, anchor:"hero|12.5|center" }',slices['rv_car_full'])
   self.assertIn('RelativeRule { kind:304, anchor:"full_screen|8|start" }',slices['rv_sup_car'])
   self.assertIn('RelativeRule { kind:304, anchor:"uncontained|0|start" }',slices['hsv_car'])
   self.assertIn('RelativeRule { kind:273, anchor:"" }',slices['mfl_item'])
   self.assertIn('RelativeRule { kind:304, anchor:"multi_browse|16|center" }',slices['ll_car'])
   self.assertIn('RelativeRule { kind:304, anchor:"multi_browse|8|start" }',slices['llc_car'])
   self.assertIn('RelativeRule { kind:304, anchor:"hero|12.5|center" }',slices['rv_styled'])
   self.assertIn('RelativeRule { kind:304, anchor:"uncontained|4|start" }',slices['rv_override'])
   custom_p=Path(d)/'custom_car.xml'; custom_p.write_text('<com.example.MyCar xmlns:app="http://schemas.android.com/apk/res-auto" app:layoutManager="com.google.android.material.carousel.CarouselLayoutManager"/>')
   custom_out=mod.compile_layout(custom_p,res,custom_view_map={'com.example.MyCar':'androidx.recyclerview.widget.RecyclerView'})
   self.assertIn('RelativeRule { kind:304, anchor:"multi_browse|8|start" }',custom_out)
   for bad_xml,err in [
    ('<androidx.recyclerview.widget.RecyclerView xmlns:app="http://schemas.android.com/apk/res-auto" app:carousel_strategy="coverflow"/>','Unsupported carousel_strategy'),
    ('<LinearLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:carousel_alignment="end"/>','Unsupported carousel_alignment'),
    ('<LinearLayout xmlns:app="http://schemas.android.com/apk/res-auto" app:carousel_itemSpacing="-4dp"/>','Expected dp/sp'),
    ('<HorizontalScrollView xmlns:app="http://schemas.android.com/apk/res-auto" app:carousel_itemSpacing="wide"><View/></HorizontalScrollView>','Expected dp/sp'),
    ('<TextView xmlns:app="http://schemas.android.com/apk/res-auto" app:carousel_strategy="hero"/>','Unsupported attribute on TextView'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave23b_view_pager_ext_rule_305(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <bool name="vp_input_off">false</bool>
    <dimen name="vp_margin">16.5dp</dimen>
    <string name="vp_trans">depth</string>
    <integer name="vp_limit">2</integer>
    <style name="CustomPagerStyle">
     <item name="android:orientation">vertical</item>
     <item name="userInputEnabled">false</item>
     <item name="pageMargin">12dp</item>
     <item name="pageTransformer">zoom_out</item>
     <item name="offscreenPageLimit">3</item>
    </style>
   </resources>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp_plain"/>
    <androidx.viewpager.widget.ViewPager android:id="@+id/vp_def_explicit" android:orientation="horizontal" app:userInputEnabled="true" app:pageMargin="0dp" app:pageTransformer="none" app:offscreenPageLimit="-1"/>
    <android.support.v4.view.ViewPager android:id="@+id/vp_sup_plain" app:offscreenPageLimit="@integer/vp_limit"/>
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp_vert" android:orientation="vertical"/>
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp_no_input" app:userInputEnabled="@bool/vp_input_off"/>
    <androidx.viewpager.widget.ViewPager android:id="@+id/vp_margin" app:pageMargin="@dimen/vp_margin"/>
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp_zoom" app:pageTransformer="zoom_out"/>
    <android.support.v4.view.ViewPager android:id="@+id/vp_sup_full" android:orientation="vertical" app:userInputEnabled="false" app:pageMargin="8dp" app:pageTransformer="@string/vp_trans" app:offscreenPageLimit="1"/>
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp_styled" style="@style/CustomPagerStyle"/>
    <androidx.viewpager2.widget.ViewPager2 android:id="@+id/vp_override_def" style="@style/CustomPagerStyle" android:orientation="horizontal" app:userInputEnabled="true" app:pageMargin="0dp" app:pageTransformer="none"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['vp_plain','vp_def_explicit','vp_sup_plain','vp_vert','vp_no_input','vp_margin','vp_zoom','vp_sup_full','vp_styled','vp_override_def']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   for omitted in ('vp_plain','vp_def_explicit','vp_sup_plain','vp_override_def'):
    self.assertIn('RelativeRule { kind:248, anchor:"horizontal" }',slices[omitted])
    self.assertNotIn('kind:305',slices[omitted],omitted)
   self.assertIn('kind:Kind::ViewFlipper,id:"vp_sup_plain"',out)
   self.assertIn('RelativeRule { kind:248, anchor:"vertical" }',slices['vp_vert'])
   self.assertIn('RelativeRule { kind:305, anchor:"v|1|0|none" }',slices['vp_vert'])
   self.assertIn('RelativeRule { kind:305, anchor:"h|0|0|none" }',slices['vp_no_input'])
   self.assertIn('RelativeRule { kind:305, anchor:"h|1|16.5|none" }',slices['vp_margin'])
   self.assertIn('RelativeRule { kind:305, anchor:"h|1|0|zoom_out" }',slices['vp_zoom'])
   self.assertIn('RelativeRule { kind:305, anchor:"v|0|8|depth" }',slices['vp_sup_full'])
   self.assertIn('RelativeRule { kind:305, anchor:"v|0|12|zoom_out" }',slices['vp_styled'])
   # merge_parent and custom_view_map for android.support.v4.view.ViewPager
   merge_p=Path(d)/'merge_vp.xml'; merge_p.write_text('<merge xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="vertical"><View/></merge>')
   merge_out=mod.compile_layout(merge_p,res,merge_parent='android.support.v4.view.ViewPager')
   self.assertIn('RelativeRule { kind:248, anchor:"vertical" }',merge_out)
   self.assertIn('RelativeRule { kind:305, anchor:"v|1|0|none" }',merge_out)
   custom_p=Path(d)/'custom_vp.xml'; custom_p.write_text('<com.example.MyVp xmlns:app="http://schemas.android.com/apk/res-auto" app:pageTransformer="depth"/>')
   custom_out=mod.compile_layout(custom_p,res,custom_view_map={'com.example.MyVp':'android.support.v4.view.ViewPager'})
   self.assertIn('RelativeRule { kind:305, anchor:"h|1|0|depth" }',custom_out)
   for bad_xml,err in [
    ('<androidx.viewpager2.widget.ViewPager2 xmlns:android="http://schemas.android.com/apk/res/android" android:orientation="diagonal"/>','Unsupported ViewPager orientation'),
    ('<androidx.viewpager2.widget.ViewPager2 xmlns:app="http://schemas.android.com/apk/res-auto" app:userInputEnabled="yes"/>','Invalid userInputEnabled'),
    ('<androidx.viewpager.widget.ViewPager xmlns:app="http://schemas.android.com/apk/res-auto" app:pageMargin="-4dp"/>','Expected dp/sp'),
    ('<androidx.viewpager2.widget.ViewPager2 xmlns:app="http://schemas.android.com/apk/res-auto" app:pageTransformer="cube"/>','Unsupported pageTransformer'),
    ('<androidx.viewpager2.widget.ViewPager2 xmlns:app="http://schemas.android.com/apk/res-auto" app:offscreenPageLimit="0"/>','Invalid offscreenPageLimit'),
    ('<androidx.viewpager2.widget.ViewPager2 xmlns:app="http://schemas.android.com/apk/res-auto" app:offscreenPageLimit="-2"/>','Invalid offscreenPageLimit'),
    ('<androidx.viewpager2.widget.ViewPager2 xmlns:app="http://schemas.android.com/apk/res-auto" app:offscreenPageLimit="many"/>','Invalid offscreenPageLimit'),
    ('<ViewFlipper xmlns:app="http://schemas.android.com/apk/res-auto" app:pageTransformer="depth"/>','Unsupported attribute on ViewFlipper'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave24b_text_links_spans_rule_306(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'values').mkdir();(res/'color').mkdir()
   (res/'values'/'vals.xml').write_text('''<resources>
    <color name="link_blue">#0055AA</color>
    <string name="link_mask">phone|web</string>
    <bool name="link_click_off">false</bool>
    <style name="LinkTextStyle">
     <item name="android:autoLink">email|web</item>
     <item name="android:textColorLink">@color/link_blue</item>
     <item name="android:linksClickable">false</item>
    </style>
    <style name="LinkAppearance">
     <item name="android:textColorLink">#2266CC</item>
    </style>
   </resources>''')
   (res/'color'/'link_csl.xml').write_text('''<selector xmlns:android="http://schemas.android.com/apk/res/android">
    <item android:state_pressed="true" android:color="#FF0000"/>
    <item android:color="#336699"/>
   </selector>''')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android">
    <TextView android:id="@+id/tv_plain" android:text="No links"/>
    <TextView android:id="@+id/tv_none" android:text="None" android:autoLink="none" android:linksClickable="false"/>
    <EditText android:id="@+id/et_plain" android:text="Plain edit"/>
    <TextView android:id="@+id/tv_web" android:text="https://example.com" android:autoLink="web"/>
    <TextView android:id="@+id/tv_combo" android:text="a@b.com https://a.b" android:autoLink="email | web"/>
    <TextView android:id="@+id/tv_phone_map" android:text="555-1234" android:autoLink="phone|map" android:textColorLink="#1A73E8" android:linksClickable="false"/>
    <TextView android:id="@+id/tv_all_theme" android:text="Everything" android:autoLink="web|all" android:textColorLink="?attr/textColorLink"/>
    <TextView android:id="@+id/tv_none_phone" android:text="Call" android:autoLink="none|phone"/>
    <TextView android:id="@+id/tv_color_only" android:text="Spanned" android:textColorLink="@color/link_csl"/>
    <EditText android:id="@+id/et_links" android:text="https://x.y 555-0100" android:autoLink="@string/link_mask" android:textColorLink="@color/link_blue" android:linksClickable="@bool/link_click_off"/>
    <com.google.android.material.textfield.TextInputEditText android:id="@+id/tiet_email" android:autoLink="email"/>
    <com.google.android.material.textview.MaterialTextView android:id="@+id/tv_styled" style="@style/LinkTextStyle" android:text="Styled"/>
    <TextView android:id="@+id/tv_appearance" android:textAppearance="@style/LinkAppearance" android:text="Appeared"/>
    <TextView android:id="@+id/tv_override_off" style="@style/LinkTextStyle" android:autoLink="none" android:textColorLink="@null" android:text="Cleared"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['tv_plain','tv_none','et_plain','tv_web','tv_combo','tv_phone_map','tv_all_theme','tv_none_phone','tv_color_only','et_links','tiet_email','tv_styled','tv_appearance','tv_override_off']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   for omitted in ('tv_plain','tv_none','et_plain','tv_override_off'):
    self.assertNotIn('kind:306',slices[omitted],omitted)
   self.assertIn('RelativeRule { kind:306, anchor:"web||1" }',slices['tv_web'])
   self.assertIn('RelativeRule { kind:306, anchor:"web,email||1" }',slices['tv_combo'])
   self.assertIn('RelativeRule { kind:306, anchor:"phone,map|26,115,232,255|0" }',slices['tv_phone_map'])
   self.assertIn('RelativeRule { kind:306, anchor:"all|103,80,164,255|1" }',slices['tv_all_theme'])
   self.assertIn('RelativeRule { kind:306, anchor:"phone||1" }',slices['tv_none_phone'])
   self.assertIn('RelativeRule { kind:306, anchor:"none|51,102,153,255|1" }',slices['tv_color_only'])
   self.assertIn('RelativeRule { kind:306, anchor:"web,phone|0,85,170,255|0" }',slices['et_links'])
   self.assertIn('RelativeRule { kind:306, anchor:"email||1" }',slices['tiet_email'])
   self.assertIn('RelativeRule { kind:306, anchor:"web,email|0,85,170,255|0" }',slices['tv_styled'])
   self.assertIn('RelativeRule { kind:306, anchor:"none|34,102,204,255|1" }',slices['tv_appearance'])
   for bad_xml,err in [
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:autoLink="ftp"/>','Unsupported autoLink'),
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:autoLink=""/>','Unsupported autoLink'),
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:autoLink="web|"/>','Unsupported autoLink'),
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textColorLink="not_a_color"/>','Expected #RGB'),
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:textColorLink="?attr/unknownLinkColor"/>','Unresolved theme color'),
    ('<TextView xmlns:android="http://schemas.android.com/apk/res/android" android:autoLink="web" android:linksClickable="yes"/>','Unsupported boolean'),
   ]:
    with self.subTest(bad=bad_xml):
     with self.assertRaisesRegex(ValueError,err):self.run_layout(bad_xml,res)
 def test_wave24b_animated_vector_rule_309(self):
  with tempfile.TemporaryDirectory() as d:
   res=Path(d);(res/'drawable').mkdir();(res/'animator').mkdir()
   (res/'drawable'/'vec_spin.xml').write_text('''<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="16dp" android:height="16dp" android:viewportWidth="16" android:viewportHeight="16">
    <group android:name="rot_g" android:pivotX="8" android:pivotY="8">
     <path android:name="bar" android:fillColor="#FF224488" android:pathData="M2,7 L14,7 L14,9 L2,9 Z"/>
    </group>
   </vector>''')
   (res/'animator'/'spin_rot.xml').write_text('''<objectAnimator xmlns:android="http://schemas.android.com/apk/res/android"
    android:propertyName="rotation" android:valueFrom="0" android:valueTo="180"
    android:duration="600" android:repeatCount="infinite" android:interpolator="@android:anim/linear_interpolator"/>''')
   (res/'drawable'/'avd_ext.xml').write_text('''<animated-vector xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:aapt="http://schemas.android.com/aapt" android:drawable="@drawable/vec_spin">
    <target android:name="rot_g" android:animation="@animator/spin_rot"/>
    <target android:name="rot_g">
     <aapt:attr name="android:animation">
      <set android:ordering="together">
       <objectAnimator android:propertyName="scaleX" android:valueFrom="0.5" android:valueTo="1.0" android:duration="600"/>
       <objectAnimator android:propertyName="scaleY" android:valueFrom="0.5" android:valueTo="1.0" android:duration="600"/>
       <objectAnimator android:propertyName="translateX" android:valueFrom="-2" android:valueTo="2" android:duration="600"/>
       <objectAnimator android:propertyName="translateY" android:valueFrom="0" android:valueTo="1" android:duration="600"/>
      </set>
     </aapt:attr>
    </target>
   </animated-vector>''')
   (res/'drawable'/'avd_inline.xml').write_text('''<animated-vector xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:aapt="http://schemas.android.com/aapt">
    <aapt:attr name="android:drawable">
     <vector android:name="vec_root" android:width="16dp" android:height="16dp"
      android:viewportWidth="16" android:viewportHeight="16" android:autoMirrored="true" android:tint="#FF336699">
      <path android:name="draw_path" android:strokeColor="#FFFFFFFF" android:strokeWidth="2"
       android:trimPathEnd="0" android:pathData="M2,8 L14,8"/>
     </vector>
    </aapt:attr>
    <target android:name="draw_path">
     <aapt:attr name="android:animation">
      <set android:ordering="sequentially">
       <objectAnimator android:propertyName="trimPathEnd" android:valueFrom="0" android:valueTo="1" android:duration="400"/>
       <set android:ordering="together">
        <objectAnimator android:propertyName="trimPathStart" android:valueFrom="0" android:valueTo="0.5" android:duration="400"/>
        <objectAnimator android:propertyName="trimPathOffset" android:valueFrom="0" android:valueTo="0.25" android:duration="400"/>
        <objectAnimator android:propertyName="strokeAlpha" android:valueFrom="1" android:valueTo="0.4" android:duration="400"/>
       </set>
      </set>
     </aapt:attr>
    </target>
    <target android:name="vec_root">
     <aapt:attr name="android:animation">
      <objectAnimator android:propertyName="alpha" android:valueFrom="0.2" android:valueTo="1.0" android:duration="800"/>
     </aapt:attr>
    </target>
   </animated-vector>''')
   (res/'drawable'/'avd_bg.xml').write_text('''<animated-vector xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:aapt="http://schemas.android.com/aapt" android:drawable="@drawable/vec_spin">
    <target android:name="bar">
     <aapt:attr name="android:animation">
      <objectAnimator android:propertyName="fillAlpha" android:valueFrom="0.1" android:valueTo="1.0"
       android:startOffset="100" android:duration="400" android:interpolator="@android:interpolator/fast_out_slow_in"/>
     </aapt:attr>
    </target>
    <target android:name="rot_g">
     <aapt:attr name="android:animation">
      <objectAnimator android:propertyName="alpha" android:valueFrom="0.5" android:valueTo="1.0" android:duration="500"/>
     </aapt:attr>
    </target>
   </animated-vector>''')
   (res/'drawable'/'avd_bg_inset.xml').write_text('<inset xmlns:android="http://schemas.android.com/apk/res/android" android:inset="2dp" android:drawable="@drawable/avd_bg"/>')
   (res/'drawable'/'avd_bg_ripple.xml').write_text('<ripple xmlns:android="http://schemas.android.com/apk/res/android" android:color="#33000000"><item android:drawable="@drawable/avd_bg"/></ripple>')
   (res/'drawable'/'avd_static.xml').write_text('<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" android:drawable="@drawable/vec_spin"/>')
   (res/'drawable'/'avd_sel.xml').write_text('<selector xmlns:android="http://schemas.android.com/apk/res/android"><item android:state_pressed="true" android:drawable="@drawable/avd_ext"/><item android:drawable="@drawable/avd_static"/></selector>')
   xml='''<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android" xmlns:app="http://schemas.android.com/apk/res-auto">
    <ImageView android:id="@+id/iv_ext" android:src="@drawable/avd_ext"/>
    <ImageButton android:id="@+id/ib_inline" app:srcCompat="@drawable/avd_inline"/>
    <View android:id="@+id/v_bg" android:background="@drawable/avd_bg"/>
    <View android:id="@+id/v_bg_inset" android:background="@drawable/avd_bg_inset"/>
    <View android:id="@+id/v_bg_ripple" android:background="@drawable/avd_bg_ripple"/>
    <ImageView android:id="@+id/iv_both" android:src="@drawable/avd_ext" android:background="@drawable/avd_bg"/>
    <ImageView android:id="@+id/iv_static" android:src="@drawable/avd_static"/>
    <FrameLayout android:id="@+id/fl_fg" android:foreground="@drawable/avd_bg"/>
    <ImageView android:id="@+id/iv_sel" android:src="@drawable/avd_sel"/>
   </LinearLayout>'''
   out=self.run_layout(xml,res)
   names=['iv_ext','ib_inline','v_bg','v_bg_inset','v_bg_ripple','iv_both','iv_static','fl_fg','iv_sel']
   slices={names[i]:out[out.index(f'id:"{names[i]}"'):(out.index(f'id:"{names[i+1]}"') if i+1<len(names) else len(out))] for i in range(len(names))}
   ext_keys=','.join(f'avd_ext__avd{i}' for i in range(8))
   inl_keys=','.join(f'avd_inline__avd{i}' for i in range(8))
   bg_keys=','.join(f'avd_bg__avd{i}' for i in range(8))
   self.assertIn(f'RelativeRule {{ kind:309, anchor:"src:1:600|{ext_keys}" }}',slices['iv_ext'])
   self.assertIn('image_src:"avd_ext"',slices['iv_ext'])
   self.assertIn(f'RelativeRule {{ kind:309, anchor:"src:0:800|{inl_keys}" }}',slices['ib_inline'])
   self.assertIn('RelativeRule { kind:255, anchor:"true" }',slices['ib_inline'])
   self.assertIn('image_tint:Some(Color(51,102,153,255))',slices['ib_inline'])
   self.assertIn('RelativeRule { kind:251, anchor:"avd_bg__avd0|" }',slices['v_bg'])
   self.assertIn(f'RelativeRule {{ kind:309, anchor:"bg:0:500|{bg_keys}" }}',slices['v_bg'])
   self.assertIn(f'RelativeRule {{ kind:309, anchor:"bg:0:500|{bg_keys}" }}',slices['v_bg_inset'])
   self.assertIn(f'RelativeRule {{ kind:309, anchor:"bg:0:500|{bg_keys}" }}',slices['v_bg_ripple'])
   self.assertIn('RelativeRule { kind:209, anchor:"0,0,0,51" }',slices['v_bg_ripple'])
   self.assertIn(f'RelativeRule {{ kind:309, anchor:"src:1:600|{ext_keys}" }}',slices['iv_both'])
   self.assertIn(f'RelativeRule {{ kind:309, anchor:"bg:0:500|{bg_keys}" }}',slices['iv_both'])
   for no_avd in ('iv_static','fl_fg','iv_sel'):
    self.assertNotIn('kind:309',slices[no_avd],no_avd)
   for base in ('avd_ext','avd_inline','avd_bg'):
    for idx in range(8):
     self.assertIn(f'state.images.entry("{base}__avd{idx}".into())',out)
   self.assertIn('state.images.entry("avd_ext".into())',out)
   self.assertIn('state.images.entry("avd_inline".into())',out)
   # Verify rasterized keyframes actually differ across sampled timestamps
   def img_vec(key):
    m=re.search(rf'state\.images\.entry\("{re.escape(key)}"\.into\(\)\).*?vec!\[([0-9,]+)\]',out)
    self.assertIsNotNone(m,key)
    return m.group(1)
   self.assertEqual(img_vec('avd_ext'),img_vec('avd_ext__avd0'))
   self.assertNotEqual(img_vec('avd_ext__avd0'),img_vec('avd_ext__avd3'))
   self.assertNotEqual(img_vec('avd_inline__avd0'),img_vec('avd_inline__avd4'))
   self.assertNotEqual(img_vec('avd_bg__avd0'),img_vec('avd_bg__avd7'))
   # Error validation cases
   (res/'drawable'/'not_vec.xml').write_text('<shape xmlns:android="http://schemas.android.com/apk/res/android"><solid android:color="#112233"/></shape>')
   bad_cases=[
    ('avd_cycle','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" android:drawable="@drawable/avd_cycle"/>','Cyclic drawable reference'),
    ('avd_nodraw','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android"/>','Animated-vector requires drawable'),
    ('avd_nonvec','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" android:drawable="@drawable/not_vec"/>','Animated-vector drawable must be a vector'),
    ('avd_badtarget','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" android:drawable="@drawable/vec_spin"><target android:name="missing" android:animation="@animator/spin_rot"/></animated-vector>','Unresolved animated-vector target'),
    ('avd_badprop','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:drawable="@drawable/vec_spin"><target android:name="bar"><aapt:attr name="android:animation"><objectAnimator android:propertyName="pathData" android:valueTo="1"/></aapt:attr></target></animated-vector>','Unsupported animated-vector property'),
    ('avd_incompat_g','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:drawable="@drawable/vec_spin"><target android:name="bar"><aapt:attr name="android:animation"><objectAnimator android:propertyName="rotation" android:valueTo="90"/></aapt:attr></target></animated-vector>','Incompatible animated-vector property'),
    ('avd_incompat_p','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:drawable="@drawable/vec_spin"><target android:name="rot_g"><aapt:attr name="android:animation"><objectAnimator android:propertyName="trimPathEnd" android:valueTo="1"/></aapt:attr></target></animated-vector>','Incompatible animated-vector property'),
    ('avd_badvt','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:drawable="@drawable/vec_spin"><target android:name="bar"><aapt:attr name="android:animation"><objectAnimator android:propertyName="fillAlpha" android:valueType="colorType" android:valueTo="1"/></aapt:attr></target></animated-vector>','Unsupported animated-vector valueType'),
    ('avd_badrange','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:drawable="@drawable/vec_spin"><target android:name="bar"><aapt:attr name="android:animation"><objectAnimator android:propertyName="fillAlpha" android:valueFrom="0" android:valueTo="1.5"/></aapt:attr></target></animated-vector>','Invalid animated-vector fillAlpha'),
    ('avd_badanim','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" android:drawable="@drawable/vec_spin"><target android:name="rot_g" android:animation="@animator/no_such_anim"/></animated-vector>','Unresolved animator'),
    ('avd_badrm','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:drawable="@drawable/vec_spin"><target android:name="rot_g"><aapt:attr name="android:animation"><objectAnimator android:propertyName="rotation" android:valueTo="90" android:repeatMode="bounce"/></aapt:attr></target></animated-vector>','Invalid animator repeatMode'),
    ('avd_badrc','<animated-vector xmlns:android="http://schemas.android.com/apk/res/android" xmlns:aapt="http://schemas.android.com/aapt" android:drawable="@drawable/vec_spin"><target android:name="rot_g"><aapt:attr name="android:animation"><objectAnimator android:propertyName="rotation" android:valueTo="90" android:repeatCount="forever"/></aapt:attr></target></animated-vector>','Invalid animator repeatCount'),
   ]
   for dname,dxml,err in bad_cases:
    (res/'drawable'/f'{dname}.xml').write_text(dxml)
    with self.subTest(bad=dname):
     with self.assertRaisesRegex(ValueError,err):
      self.run_layout(f'<ImageView xmlns:android="http://schemas.android.com/apk/res/android" android:src="@drawable/{dname}"/>',res)
if __name__=='__main__':unittest.main()
