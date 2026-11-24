#!/usr/bin/env python3
"""Regenerate the small upstream patch; integration code stays in keyboard/."""
from pathlib import Path
import argparse,difflib
p=argparse.ArgumentParser();p.add_argument('source',type=Path);a=p.parse_args();root=Path(__file__).resolve().parents[1];changes={}
def read(name): return (a.source/name).read_text()
s=read('src/pos-osk-widget.c')
s=s.replace('#include "pos-char-popup.h"','#include "pos-char-popup.h"\n#include "spectre-touch.h"')
s=s.replace('  PosIndicatorPopup   *indicator_popup;','  PosIndicatorPopup   *indicator_popup;\n  GdkPixbuf *reference_art;\n  char typing_prefix[64];\n  gboolean adaptive_typing;')
# Prototypes are after the struct so the integration can use private widget data.
mark='static PosOskKey *\npos_osk_widget_locate_key'
protos='''static gboolean spectre_is_reference(PosOskWidget *self);
static void spectre_assign_boxes(PosOskWidget *self);
static PosOskKey *spectre_locate(PosOskWidget *self,double x,double y);
static gboolean spectre_footer_press(PosOskWidget *self,double x,double y);
static gboolean spectre_draw_reference(PosOskWidget *self,cairo_t *cr);

'''
s=s.replace(mark,protos+mark,1)
s=s.replace('    g_ptr_array_add (self->symbols, (gpointer)pos_osk_key_get_symbol (key));','    if (pos_osk_key_get_symbol (key) && *pos_osk_key_get_symbol (key))\n      g_ptr_array_add (self->symbols, (gpointer)pos_osk_key_get_symbol (key));')
s=s.replace('  add_common_keys_pre (self, row, l, r, max_rows);\n  add_common_keys_post (row, l, r, max_rows);','  if (!g_str_has_prefix(self->layout.name,"spectre-explicit-")) {\n    add_common_keys_pre (self, row, l, r, max_rows);\n    add_common_keys_post (row, l, r, max_rows);\n  }')
s=s.replace('  row_num = (int)((y - off_y) / layer->key_height);','  if (spectre_is_reference(self)) return spectre_locate(self,x,y);\n  if (y < off_y) return NULL;\n  row_num = (int)((y - off_y) / layer->key_height);')
s=s.replace('  PosOskKey *key = NULL;\n\n  key = pos_osk_widget_locate_key (self, x, y);','  PosOskKey *key = NULL;\n\n  if (spectre_footer_press(self,x,y)) return GDK_EVENT_STOP;\n  key = pos_osk_widget_locate_key (self, x, y);')
s=s.replace('  g_return_val_if_fail (key != NULL, GDK_EVENT_PROPAGATE);','  if (!key) return GDK_EVENT_PROPAGATE;')
s=s.replace('  key = pos_osk_widget_locate_key (self, event->x, event->y);\n  g_return_val_if_fail (key != NULL, GDK_EVENT_PROPAGATE);','  key = spectre_is_reference(self) ? self->current : pos_osk_widget_locate_key (self, event->x, event->y);\n  g_return_val_if_fail (key != NULL, GDK_EVENT_PROPAGATE);')
# Match the QML model: choose on press, retain that decision until release.
s=s.replace('  key = pos_osk_widget_locate_key (self, event->x, event->y);\n  if (!key) return GDK_EVENT_PROPAGATE;', '  key = spectre_is_reference(self) ? self->current : pos_osk_widget_locate_key(self,event->x,event->y);\n  if (!key) return GDK_EVENT_PROPAGATE;')
s=s.replace('  } else if (event->type == GDK_TOUCH_UPDATE) {','  } else if (event->type == GDK_TOUCH_UPDATE) {\n    if (spectre_is_reference(self)) return GDK_EVENT_STOP;')
s=s.replace('  if (event->type == GDK_TOUCH_END || event->type == GDK_TOUCH_CANCEL) {','  if (event->type == GDK_TOUCH_CANCEL && spectre_is_reference(self)) {\n    pos_osk_widget_cancel_press(self);\n    return GDK_EVENT_STOP;\n  }\n  if (event->type == GDK_TOUCH_END || event->type == GDK_TOUCH_CANCEL) {')
s=s.replace('  if ((event->state & GDK_BUTTON1_MASK) == 0)','  if (spectre_is_reference(self)) return GDK_EVENT_STOP;\n  if ((event->state & GDK_BUTTON1_MASK) == 0)')
# Language tap switches; hold opens emoji, and hold on space opens languages.
s=s.replace('  case POS_OSK_KEY_USE_MENU:\n    pos_osk_widget_show_menu (self, key);','  case POS_OSK_KEY_USE_MENU:\n    if (spectre_is_reference(self)) {\n      GActionGroup *group=gtk_widget_get_action_group(GTK_WIDGET(self),"win");\n      g_action_group_activate_action(group,"next-layout",NULL);\n    } else pos_osk_widget_show_menu (self, key);')
marker='  symbols = pos_osk_key_get_symbols (key);\n  if (symbols == NULL || symbols[0] == NULL)'
new='''  if (spectre_is_reference(self)) {
    const char *symbol=pos_osk_key_get_symbol(key);
    if (pos_osk_key_get_use(key)==POS_OSK_KEY_USE_MENU || g_strcmp0(symbol,",")==0 || g_strcmp0(symbol,"،")==0) {
      pos_osk_widget_cancel_press(self);
      GActionGroup *group=gtk_widget_get_action_group(GTK_WIDGET(self),"win");
      g_action_group_change_action_state(group,"select-layout",g_variant_new_string("emoji"));
      return;
    }
    if (g_strcmp0(symbol," ")==0) {
      pos_osk_widget_cancel_press(self);pos_osk_widget_show_menu(self,key);return;
    }
  }
'''
s=s.replace(marker,new+marker)
s=s.replace('  scale_h = 100 * MAX','  if (spectre_is_reference(self)) { self->key_scale=150; return; }\n  scale_h = 100 * MAX')
s=s.replace('  /* On key size changes we adjust the font and icon size */','  spectre_assign_boxes(self);\n\n  /* On key size changes we adjust the font and icon size */')
s=s.replace('  cairo_save (cr);\n\n  context = gtk_widget_get_style_context (widget);','  if (spectre_draw_reference(self,cr)) return FALSE;\n\n  cairo_save (cr);\n\n  context = gtk_widget_get_style_context (widget);')
s=s.replace('  g_clear_pointer (&self->layout.name, g_free);','  g_clear_object(&self->reference_art);\n  g_clear_pointer (&self->layout.name, g_free);') if False else s
# clear cached artwork in finalize, alongside the existing owned object.
s=s.replace('  G_OBJECT_CLASS (pos_osk_widget_parent_class)->finalize (object);','  g_clear_object(&self->reference_art);\n  G_OBJECT_CLASS (pos_osk_widget_parent_class)->finalize (object);')
marker='static void\npos_osk_widget_class_init (PosOskWidgetClass *klass)'
extra='''#include "spectre-widget.inc"

static GtkSizeRequestMode spectre_request_mode(GtkWidget *widget) {
  return spectre_is_reference(POS_OSK_WIDGET(widget)) ? GTK_SIZE_REQUEST_HEIGHT_FOR_WIDTH : GTK_SIZE_REQUEST_CONSTANT_SIZE;
}
static void spectre_height_for_width(GtkWidget *widget,gint width,gint *minimum,gint *natural) {
  if (spectre_is_reference(POS_OSK_WIDGET(widget))) *minimum=*natural=(gint)ceil(width*777.0/1080.0);
  else pos_osk_widget_get_preferred_height(widget,minimum,natural);
}

'''
s=s.replace(marker,extra+marker)
s=s.replace('  widget_class->draw = pos_osk_widget_draw;','  widget_class->get_request_mode = spectre_request_mode;\n  widget_class->get_preferred_height_for_width = spectre_height_for_width;\n  widget_class->draw = pos_osk_widget_draw;')
s=s.replace('  *minimum_height = *natural_height = self->key_height * self->layout.n_rows;','  if (spectre_is_reference(self)) {\n    *minimum_height=*natural_height=(gint)ceil(MAX(self->width,MINIMUM_WIDTH)*777.0/1080.0);\n    return;\n  }\n  *minimum_height = *natural_height = self->key_height * self->layout.n_rows;')
# New widgets begin with a neutral, enabled scorer until input-purpose arrives.
s=s.replace('pos_osk_widget_init (PosOskWidget *self)\n{','pos_osk_widget_init (PosOskWidget *self)\n{\n  self->adaptive_typing=TRUE;')
changes['src/pos-osk-widget.c']=s
s=read('src/pos-osk-widget.h');s+='\nvoid pos_osk_widget_set_touch_context(PosOskWidget *self,const char *text,gsize length,gboolean adaptive);\nvoid pos_osk_widget_note_touch_symbol(PosOskWidget *self,const char *symbol);\n';changes['src/pos-osk-widget.h']=s
s=read('src/pos-osk-key.c').replace('1.0, 10.0, 1.0,','0.1, 10.0, 1.0,');changes['src/pos-osk-key.c']=s
s=read('src/pos-input-surface.c')
s=s.replace('#include "pos-input-method.h"','#include "pos-input-method.h"\n#include "spectre-touch.h"')
old="""  if (handled)
    return;

  pos_vk_driver_key_down (self->keyboard_driver, "KEY_BACKSPACE", POS_KEYCODE_MODIFIER_NONE);"""
new="""  if (handled)
    return;

  /* Match the supplied erase() rule for normal IM clients; selections and
   * secure fields keep the client's native Backspace semantics. */
  if (self->bs_mode != POS_BACKSPACE_MODE_WORD &&
      pos_input_method_get_purpose(self->input_method) != POS_INPUT_METHOD_PURPOSE_PASSWORD &&
      pos_input_method_get_purpose(self->input_method) != POS_INPUT_METHOD_PURPOSE_PIN) {
    guint anchor,cursor;
    const char *text=pos_input_method_get_surrounding_text(self->input_method,&anchor,&cursor);
    if (text && cursor && cursor==anchor && cursor<=strlen(text) && g_utf8_validate(text,cursor,NULL)) {
      glong count=0;
      g_autofree gunichar *points=g_utf8_to_ucs4(text,cursor,NULL,&count,NULL);
      if (points && count) {
        gsize start=spectre_erase_start(points,count);
        guint bytes=cursor-(g_utf8_offset_to_pointer(text,start)-text);
        pos_input_method_delete_surrounding_text(self->input_method,bytes,0,TRUE);
        return;
      }
    }
  }
  pos_vk_driver_key_down (self->keyboard_driver, "KEY_BACKSPACE", POS_KEYCODE_MODIFIER_NONE);"""
assert old in s
s=s.replace(old,new)
s=s.replace('  pos_input_surface_submit_symbol (self, symbol);','  GtkWidget *child=hdy_deck_get_visible_child(self->deck);\n  if (POS_IS_OSK_WIDGET(child)) pos_osk_widget_note_touch_symbol(POS_OSK_WIDGET(child),symbol);\n  pos_input_surface_submit_symbol (self, symbol);',1)
context='''  gboolean adaptive=pos_input_method_get_purpose(im)!=POS_INPUT_METHOD_PURPOSE_PASSWORD &&
    pos_input_method_get_purpose(im)!=POS_INPUT_METHOD_PURPOSE_PIN;
  pos_osk_widget_set_touch_context(POS_OSK_WIDGET(self->osk_terminal),text,cursor,adaptive);
  for (guint i=0;i<self->osks->len;i++)
    pos_osk_widget_set_touch_context(g_ptr_array_index(self->osks,i),text,cursor,adaptive);
'''
s=s.replace('  text = pos_input_method_get_surrounding_text (im, &anchor, &cursor);','  text = pos_input_method_get_surrounding_text (im, &anchor, &cursor);\n'+context)
reset=context.replace('text,cursor','NULL,0')
s=s.replace('  active = pos_input_method_get_active (im);','  active = pos_input_method_get_active (im);\n'+reset)
# Purpose changes reset prediction, even when a client does not supply surrounding text.
s=s.replace('  select_layout_by_im_purpose (self);\n}',reset+'  select_layout_by_im_purpose (self);\n}',1)
extra='''static void spectre_hide_activated(GSimpleAction *action,GVariant *parameter,gpointer data) {
  g_object_set(data,"surface-visible",FALSE,NULL);
}
static void spectre_next_layout(GSimpleAction *action,GVariant *parameter,gpointer data) {
  PosInputSurface *self=data;GtkWidget *child=hdy_deck_get_visible_child(self->deck);
  const char *current=POS_IS_OSK_WIDGET(child)?pos_osk_widget_get_name(POS_OSK_WIDGET(child)):"";
  const char *next=g_strcmp0(current,"ir")==0?"us":"ir";
  g_action_group_change_action_state(G_ACTION_GROUP(self->action_map),"select-layout",g_variant_new_string(next));
}
'''
s=s.replace('static GActionEntry entries[] =',extra+'\nstatic GActionEntry entries[] =').replace('  { .name = "clipboard-paste",','  { .name = "hide-keyboard", .activate=spectre_hide_activated },\n  { .name = "next-layout", .activate=spectre_next_layout },\n  { .name = "clipboard-paste",')
changes['src/pos-input-surface.c']=s
s=read('src/phosh-osk-stevia.gresources.xml').replace('    <!-- from gnome-shell -->','    <file>reference/english.png</file>\n    <file>reference/persian.png</file>\n    <!-- from gnome-shell -->');changes['src/phosh-osk-stevia.gresources.xml']=s
out=''
for name,s in changes.items():out+=''.join(difflib.unified_diff(read(name).splitlines(True),s.splitlines(True),fromfile='a/'+name,tofile='b/'+name))
(root/'patches/stevia-explicit-layout.patch').write_text(out)
