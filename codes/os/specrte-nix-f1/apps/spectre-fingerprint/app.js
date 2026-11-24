import Gtk from 'gi://Gtk?version=4.0';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Gdk from 'gi://Gdk?version=4.0';

const client = GLib.getenv('FINGERPRINT_CLIENT');
const smoke = ARGV.includes('--smoke-test');
const uiSmoke = ARGV.includes('--ui-smoke-test');
const checkOnStart = smoke || ARGV.includes('--check-on-start');
const app = new Gtk.Application({application_id: 'org.spectre.Fingerprint', flags: (smoke || uiSmoke) ? Gio.ApplicationFlags.NON_UNIQUE : Gio.ApplicationFlags.DEFAULT_FLAGS});
let window, title, description, button, spinner, geometry, latest, details, busy = false, tests = 0;
let exitCode = (smoke || uiSmoke) ? 1 : 0;
const actions = [];
let activeAction = 'test';
function label(text, css) {
    const w = new Gtk.Label({label: text, wrap: true, xalign: 0});
    if (css) w.add_css_class(css);
    return w;
}
function check(action = 'test') {
    if (busy) return;
    busy = true;
    activeAction = action;
    for (const control of actions) control.sensitive = false;
    spinner.start();
    title.label = {test: 'Checking your sensor…', enroll: 'Preparing enrollment…', list: 'Checking saved fingerprints…', verify: 'Preparing verification…'}[action];
    description.label = 'Checking the hardware and required services.';
    const process = new Gio.Subprocess({argv: [client, '--client', action],
        flags: Gio.SubprocessFlags.STDOUT_PIPE | Gio.SubprocessFlags.STDERR_PIPE});
    try {
        process.init(null);
        const stream = new Gio.DataInputStream({base_stream: process.get_stdout_pipe()});
        function readResult() {
            stream.read_line_async(GLib.PRIORITY_DEFAULT, null, (input, res) => {
                try {
                    const [line] = input.read_line_finish_utf8(res);
                    if (line === null) throw new Error('Service ended without a result');
                    const result = JSON.parse(line);
                    if (result.event === 'progress') {
                        title.label = result.title;
                        description.label = result.message;
                        readResult();
                    } else {
                        finish(result);
                        process.wait_async(null, () => {});
                    }
                } catch (e) {
                    finish({ok: false, title: 'Could not finish the check', message: String(e)});
                }
            });
        }
        readResult();
    } catch (e) {
        finish({ok: false, title: 'Could not start the check', message: String(e)});
    }
}
function finish(result) {
    busy = false;
    for (const control of actions) control.sensitive = true;
    spinner.stop();
    title.label = result.title || 'Sensor check failed';
    description.label = result.message || 'See test details below.';
    if (result.rows && result.columns) geometry.label = `Goodix · ${result.rows} × ${result.columns}`;
    const now = GLib.DateTime.new_now_local().format('%H:%M:%S');
    tests++;
    latest.label = `${result.ok ? 'Passed' : 'Failed'} at ${now} · Check ${tests}`;
    latest.remove_css_class(result.ok ? 'failed' : 'passed');
    latest.add_css_class(result.ok ? 'passed' : 'failed');
    details.label = result.details || result.message || '';
    print(JSON.stringify({event: activeAction, ...result}));
    if (smoke) {
        exitCode = result.ok ? 0 : 1;
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 2000, () => {app.quit(); return GLib.SOURCE_REMOVE;});
    }
}
app.connect('activate', () => {
    if (window) { window.present(); return; }
    const css = new Gtk.CssProvider();
    css.load_from_string(`
        window { background: #101b23; color: #edf5f6; }
        headerbar { background: #101b23; border: none; }
        .page { padding: 24px 22px; }
        .eyebrow { color: #8acfc5; font-size: 12px; font-weight: 700; letter-spacing: 2px; }
        .hero { font-size: 28px; font-weight: 750; }
        .muted { color: #afbec5; font-size: 15px; }
        .card { background: #1b2a34; border-radius: 20px; padding: 22px; }
        .sensor-title { font-size: 20px; font-weight: 700; }
        .sensor-icon { color: #91e1ce; margin: 10px; }
        button.check { background: #91e1ce; color: #102720; border-radius: 14px; min-height: 50px; font-weight: 700; }
        button.check:disabled { background: #42665f; }
        .passed { color: #91e1ce; } .failed { color: #ffb5a7; }
        textview, textview text { background: #15212a; color: #bccbd2; font-family: monospace; font-size: 11px; }
    `);
    Gtk.StyleContext.add_provider_for_display(Gdk.Display.get_default(), css, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION);
    window = new Gtk.ApplicationWindow({application: app, title: 'Fingerprint Lab', default_width: 360, default_height: 560});
    const header = new Gtk.HeaderBar();
    header.set_title_widget(new Gtk.Label({label: 'Fingerprint Lab'}));
    window.set_titlebar(header);
    const scroll = new Gtk.ScrolledWindow({hscrollbar_policy: Gtk.PolicyType.NEVER, vscrollbar_policy: Gtk.PolicyType.ALWAYS, overlay_scrolling: false, kinetic_scrolling: true, vexpand: true, propagate_natural_height: false});
    const page = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL, spacing: 18});
    page.add_css_class('page');
    page.append(label('POCO F1', 'eyebrow'));
    page.append(label('Your fingerprints', 'hero'));
    page.append(label('Enroll a finger, view saved fingerprints, or check a match. Setup status appears below.', 'muted'));
    const card = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL, spacing: 14});
    card.add_css_class('card');
    const icon = new Gtk.Image({icon_name: 'fingerprint-symbolic', pixel_size: 64});
    icon.add_css_class('sensor-icon');card.append(icon);
    title = label('Ready to check', 'sensor-title');card.append(title);
    description = label('No fingerprint is recorded during this check.', 'muted');card.append(description);
    geometry = label('Goodix fingerprint sensor', 'muted');card.append(geometry);
    spinner = new Gtk.Spinner({halign: Gtk.Align.CENTER});card.append(spinner);
    button = new Gtk.Button({label: 'Check sensor'});button.add_css_class('check');button.connect('clicked', () => check('test'));actions.push(button);card.append(button);
    latest = label('No checks yet', 'muted');card.append(latest);page.append(card);
    const controls = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL, spacing: 12});
    controls.add_css_class('card');
    controls.append(label('Manage fingerprints', 'sensor-title'));
    for (const [action, caption] of [['enroll', 'Enroll fingerprint'], ['list', 'Saved fingerprints'], ['verify', 'Check enrolled fingerprint']]) {
        const control = new Gtk.Button({label: caption});
        control.add_css_class('check');
        control.connect('clicked', () => check(action));
        actions.push(control);controls.append(control);
    }
    controls.append(label('One fingerprint is supported. Wait for the touch prompt. Fingerprint Lab does not enable screen unlock.', 'muted'));
    page.append(controls);
    const expander = new Gtk.Expander({label: 'Test details'});
    details = label('', 'muted');
    expander.set_child(details);page.append(expander);
    scroll.set_child(page);
    const root = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL});
    root.append(scroll);
    const down = new Gtk.Button({label: 'Scroll down ↓', margin_start: 16, margin_end: 16, margin_top: 8, margin_bottom: 12});
    down.connect('clicked', () => {
        const adj = scroll.get_vadjustment();
        adj.value = Math.min(adj.upper - adj.page_size, adj.value + adj.page_size * 0.8);
    });
    root.append(down);window.set_child(root);window.present();
    if (uiSmoke) {
      expander.expanded = true;
      details.label = Array(50).fill('Scrolling test: hardware details remain reachable.').join('\n');
      GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1200, () => {
        const adj = scroll.get_vadjustment();
        down.emit('clicked');
        const ok = actions.length === 4 && adj.upper > adj.page_size && adj.value > 0;
        print(JSON.stringify({event: 'ui-scroll-test', ok, controls: actions.length, contentHeight: adj.upper, viewportHeight: adj.page_size, scrollPosition: adj.value}));
        exitCode = ok ? 0 : 1;app.quit();return GLib.SOURCE_REMOVE;
    });
    }
    if (checkOnStart) GLib.idle_add(GLib.PRIORITY_DEFAULT_IDLE, () => {check(); return GLib.SOURCE_REMOVE;});
});
app.run([]);
if (exitCode) imports.system.exit(exitCode);
