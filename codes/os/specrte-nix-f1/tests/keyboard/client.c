/* Temporary, isolated test client. Its D-Bus API only changes its own widgets. */
#include <gtk/gtk.h>
static GtkWidget *window,*normal,*password;
static const char introspection[]=
 "<node><interface name='org.spectre.KeyboardTest'>"
 "<method name='Reset'/>"
 "<method name='Focus'><arg type='s' direction='in'/></method>"
 "<method name='SetText'><arg type='s' direction='in'/></method>"
 "<method name='SetCursor'><arg type='i' direction='in'/></method>"
 "<method name='GetText'><arg type='s' direction='out'/></method>"
 "<method name='CheckPassword'><arg type='s' direction='in'/><arg type='b' direction='out'/></method>"
 "</interface></node>";
static void changed(GtkEditable *entry,gpointer path) {
  g_file_set_contents(path,gtk_entry_get_text(GTK_ENTRY(entry)),-1,NULL);
}
static void done(GtkWidget *widget,gpointer unused) {gtk_main_quit();}
static void method(GDBusConnection *connection,const char *sender,const char *path,
 const char *interface,const char *name,GVariant *args,GDBusMethodInvocation *call,gpointer data) {
 if (g_str_equal(name,"GetText")) {
   g_dbus_method_invocation_return_value(call,g_variant_new("(s)",gtk_entry_get_text(GTK_ENTRY(normal))));return;
 }
 if (g_str_equal(name,"CheckPassword")) {
   const char *s;g_variant_get(args,"(&s)",&s);
   g_dbus_method_invocation_return_value(call,g_variant_new("(b)",g_str_equal(s,gtk_entry_get_text(GTK_ENTRY(password)))));return;
 }
 if (g_str_equal(name,"Reset")) {gtk_entry_set_text(GTK_ENTRY(normal),"");gtk_entry_set_text(GTK_ENTRY(password),"");gtk_widget_grab_focus(normal);}
 else if (g_str_equal(name,"Focus")) {const char *s;g_variant_get(args,"(&s)",&s);gtk_widget_grab_focus(g_str_equal(s,"password")?password:normal);}
 else if (g_str_equal(name,"SetText")) {const char *s;g_variant_get(args,"(&s)",&s);gtk_entry_set_text(GTK_ENTRY(normal),s);gtk_editable_set_position(GTK_EDITABLE(normal),-1);gtk_widget_grab_focus(normal);}
 else if (g_str_equal(name,"SetCursor")) {gint n;g_variant_get(args,"(i)",&n);gtk_editable_set_position(GTK_EDITABLE(normal),n);}
 gtk_window_present(GTK_WINDOW(window));g_dbus_method_invocation_return_value(call,NULL);
}
static const GDBusInterfaceVTable table={.method_call=method};
int main(int argc,char **argv) {
 gtk_init(&argc,&argv);
 window=gtk_window_new(GTK_WINDOW_TOPLEVEL);GtkWidget *box=gtk_box_new(GTK_ORIENTATION_VERTICAL,12);
 gtk_window_set_title(GTK_WINDOW(window),"Spectre Keyboard Test");gtk_window_set_default_size(GTK_WINDOW(window),400,420);
 gtk_container_set_border_width(GTK_CONTAINER(box),20);gtk_container_add(GTK_CONTAINER(window),box);
 gtk_box_pack_start(GTK_BOX(box),gtk_label_new("Keyboard test — normal text"),FALSE,FALSE,0);
 normal=gtk_entry_new();gtk_entry_set_placeholder_text(GTK_ENTRY(normal),"Type here");
 gtk_entry_set_input_purpose(GTK_ENTRY(normal),GTK_INPUT_PURPOSE_FREE_FORM);gtk_box_pack_start(GTK_BOX(box),normal,FALSE,FALSE,0);
 char *path=g_build_filename(g_get_user_runtime_dir(),"spectre-keyboard-test.txt",NULL);g_file_set_contents(path,"",0,NULL);
 g_signal_connect(normal,"changed",G_CALLBACK(changed),path);
 gtk_box_pack_start(GTK_BOX(box),gtk_label_new("Password field (test text only)"),FALSE,FALSE,0);
 password=gtk_entry_new();gtk_entry_set_visibility(GTK_ENTRY(password),FALSE);gtk_entry_set_input_purpose(GTK_ENTRY(password),GTK_INPUT_PURPOSE_PASSWORD);
 gtk_box_pack_start(GTK_BOX(box),password,FALSE,FALSE,0);g_signal_connect(window,"destroy",G_CALLBACK(done),NULL);
 GDBusConnection *bus=g_bus_get_sync(G_BUS_TYPE_SESSION,NULL,NULL);GDBusNodeInfo *info=g_dbus_node_info_new_for_xml(introspection,NULL);
 g_dbus_connection_register_object(bus,"/org/spectre/KeyboardTest",info->interfaces[0],&table,NULL,NULL,NULL);
 guint owner=g_bus_own_name_on_connection(bus,"org.spectre.KeyboardTest",G_BUS_NAME_OWNER_FLAGS_NONE,NULL,NULL,NULL,NULL);
 gtk_widget_show_all(window);gtk_widget_grab_focus(normal);gtk_main();g_bus_unown_name(owner);g_object_unref(bus);g_dbus_node_info_unref(info);g_free(path);return 0;
}
