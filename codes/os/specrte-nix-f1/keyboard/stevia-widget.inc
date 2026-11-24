/* Native interaction and artwork rendering for the supplied keyboard skins. */
#include "reference-geometry.h"

static gboolean spectre_is_reference(PosOskWidget *self) {
  return self->layout.name && g_str_has_prefix(self->layout.name,"spectre-explicit-");
}
static gboolean spectre_is_persian(PosOskWidget *self) {
  return self->layout.locale && g_str_has_prefix(self->layout.locale,"fa");
}
static void spectre_assign_boxes(PosOskWidget *self) {
  if (!spectre_is_reference(self)) return;
  gboolean fa=spectre_is_persian(self);
  double scale=self->width/1080.0;
  /* A layer surface initially measures before the deck has its final width.
   * Request again after allocation so the 777px artwork is never clipped. */
  gint requested_width,requested_height;
  gtk_widget_get_size_request(GTK_WIDGET(self),&requested_width,&requested_height);
  gint wanted=(gint)ceil(777*scale);
  if (requested_height!=wanted) gtk_widget_set_size_request(GTK_WIDGET(self),-1,wanted);
  for (int l=0;l<=POS_OSK_WIDGET_LAST_LAYER;l++) {
    PosOskWidgetKeyboardLayer *layer=&self->layout.layers[l];
    layer->offset_x=0;
    layer->key_width=self->width/MAX(1.0,layer->width);
    layer->key_height=151*scale;
    for (guint r=0;r<layer->n_rows;r++) {
      PosOskWidgetRow *row=&layer->rows[r];double cursor=row->offset_x*layer->key_width;
      int expected=fa?spectre_persian_counts[r]:spectre_english_counts[r];
      gboolean measured=row->keys->len==(guint)expected;
      for (guint i=0;i<row->keys->len;i++) {
        PosOskKey *key=g_ptr_array_index(row->keys,i);
        GdkRectangle box;
        if (measured) {
          const double *face=fa?spectre_persian_faces[r][i]:spectre_english_faces[r][i];
          box=(GdkRectangle){(int)round((face[0]-7)*scale),
            (int)round((r==3?464:10+r*151)*scale),
            (int)round((face[2]+14)*scale),(int)round(151*scale)};
        } else {
          box=(GdkRectangle){(int)round(cursor),(int)round((10+r*151)*scale),
            (int)round(pos_osk_key_get_width(key)*layer->key_width),(int)round(151*scale)};
        }
        pos_osk_key_set_box(key,&box);
        cursor+=pos_osk_key_get_width(key)*layer->key_width;
      }
    }
  }
}
static PosOskKey *spectre_locate(PosOskWidget *self,double x,double y) {
  PosOskWidgetKeyboardLayer *layer=pos_osk_widget_get_current_layer(self);
  SpectreTouchKey candidates[64];PosOskKey *objects[64];size_t n=0;
  if (x<0 || x>self->width || y<0 || y>self->width*625/1080.0) return NULL;
  for (guint r=0;r<layer->n_rows;r++) {
    PosOskWidgetRow *row=&layer->rows[r];
    for (guint i=0;i<row->keys->len && n<G_N_ELEMENTS(candidates);i++) {
      PosOskKey *key=g_ptr_array_index(row->keys,i);
      const GdkRectangle *b=pos_osk_key_get_box(key);const char *s=pos_osk_key_get_symbol(key);
      gboolean command=pos_osk_key_get_use(key)!=POS_OSK_KEY_USE_KEY ||
        !s || !*s || g_str_has_prefix(s,"KEY_") || g_str_equal(s," ") || g_str_equal(s,"‌");
      candidates[n]=(SpectreTouchKey){b->x,b->y,b->width,b->height,s,command};objects[n++]=key;
    }
  }
  int chosen=spectre_touch_choose(candidates,n,x,y,self->typing_prefix,
    self->adaptive_typing && g_strcmp0(g_getenv("SPECTRE_ADAPTIVE_TYPING"),"0")!=0);
  return chosen<0?NULL:objects[chosen];
}
static gboolean spectre_footer_press(PosOskWidget *self,double x,double y) {
  if (!spectre_is_reference(self)) return FALSE;
  double sx=x*1080.0/self->width,sy=y*1080.0/self->width;
  if (sy<645) return FALSE;
  GActionGroup *group=gtk_widget_get_action_group(GTK_WIDGET(self),"win");
  if (sy<=740 && sx<220)
    g_action_group_activate_action(group,"menu",g_variant_new("(ii)",(int)x,(int)y));
  else if (sy<=740 && sx>870)
    g_action_group_activate_action(group,"hide-keyboard",NULL);
  return TRUE;
}
static gboolean spectre_draw_reference(PosOskWidget *self,cairo_t *cr) {
  if (!spectre_is_reference(self)) return FALSE;
  double scale=self->width/1080.0;
  if (!self->reference_art) {
    g_autoptr(GError) error=NULL;
    const char *path=spectre_is_persian(self)?
      "/mobi/phosh/stevia/reference/persian.png":"/mobi/phosh/stevia/reference/english.png";
    self->reference_art=gdk_pixbuf_new_from_resource(path,&error);
    if (!self->reference_art) {g_warning("Keyboard artwork could not load: %s",error->message);return FALSE;}
  }
  cairo_save(cr);cairo_scale(cr,scale,scale);
  gdk_cairo_set_source_pixbuf(cr,self->reference_art,0,0);
  cairo_pattern_set_filter(cairo_get_source(cr),CAIRO_FILTER_NEAREST);
  cairo_paint(cr);cairo_restore(cr);
  PosOskWidgetKeyboardLayer *layer=pos_osk_widget_get_current_layer(self);
  if (self->layer!=POS_OSK_WIDGET_LAYER_NORMAL) {
    /* Other layers keep the same geometry and footer, with live native labels. */
    cairo_save(cr);cairo_set_source_rgb(cr,42/255.0,42/255.0,44/255.0);
    cairo_rectangle(cr,0,0,self->width,630*scale);cairo_fill(cr);cairo_restore(cr);
    for (guint r=0;r<layer->n_rows;r++)
      for (guint i=0;i<layer->rows[r].keys->len;i++)
        draw_key(self,g_ptr_array_index(layer->rows[r].keys,i),cr);
  } else if (self->current && pos_osk_key_get_pressed(self->current)) {
    const GdkRectangle *b=pos_osk_key_get_box(self->current);
    cairo_save(cr);cairo_set_source_rgba(cr,.65,.74,1,.24);
    cairo_rectangle(cr,b->x+7*scale,b->y+15*scale,MAX(0,b->width-14*scale),MAX(0,b->height-30*scale));
    cairo_fill(cr);cairo_restore(cr);
  }
  return TRUE;
}

void pos_osk_widget_set_touch_context(PosOskWidget *self,const char *text,gsize length,gboolean adaptive) {
  self->adaptive_typing=adaptive;
  spectre_prefix_set(self->typing_prefix,adaptive&&text?text:"",adaptive&&text?MIN(length,strlen(text)):0);
}
void pos_osk_widget_note_touch_symbol(PosOskWidget *self,const char *symbol) {
  if (!self->adaptive_typing) return;
  size_t n=strlen(self->typing_prefix);
  if (g_strcmp0(symbol,"KEY_BACKSPACE")==0) {if(n)self->typing_prefix[n-1]=0;return;}
  if (symbol && strlen(symbol)==1 && g_ascii_isalpha(*symbol) && n<63) {
    self->typing_prefix[n]=g_ascii_tolower(*symbol);self->typing_prefix[n+1]=0;
  } else self->typing_prefix[0]=0;
}
