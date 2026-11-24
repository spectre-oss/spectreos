# Writing responsive SpectreOS apps

A user should see a response to a tap even when the disk is busy or a system service is slow. Draw the page from the state already in memory. Load missing information separately and update the part of the page that needs it. In an app embedded in Wisp, a blocked frame can also delay the launcher animation and input for the whole shell.

## Draw the first frame

1. On a tap, update navigation immediately. Draw the title, back affordance, and any known rows on the first frame of the transition.
2. Use a placeholder only where a value is missing. An Apps row can show its label while its icon loads. A storage row can show “Loading…” in the space reserved for its subtitle. Keep row height and touch targets stable when the real value arrives.
3. Start slow work once, on entry or when the underlying data changes. Directory scans, subprocess output, blocking service calls, large file parsing, and image decoding belong on a worker. Check constructors and first-use cache misses as well as the draw function.
4. Deliver the result to the UI and request a redraw. With a Rust channel, read it with `try_recv` on the UI thread. Use an event-loop wakeup or a timed wake while a result is pending. Stop scheduling wakes when the request finishes. Keep back, scrolling, and unrelated controls responsive throughout.

Distinguish **loading**, **ready but empty**, and **failed**. For example, show “No files in this folder” only after a successful listing. On refresh, keep the previous listing visible until the replacement is ready. If a result belongs to a folder the user has already left, discard it. Owning the result receiver in the page or cache entry is one way to do that.

## Choose what to cache and when to refresh

Cache the data used to build the page across frames. Identify the inputs that can change it. For an installed-app list, a desktop-entry install or removal changes list membership, and a package update can change the displayed version. A scroll, animation tick, or battery update uses the existing list. Resolve rows by a stable identifier such as a desktop ID when applying delayed results.

For periodic system status, use a worker with a bounded request queue. If an equivalent refresh is already queued, coalesce the request. Publish the latest complete snapshot and redraw if the visible values changed. Keep periodic reads off the compositor thread even when an individual read usually finishes quickly. [Shiraz status worker](../../codes/apps/wisp/src/backends/linux/shiraz.rs)

## What caused concern in Settings

The embedded Settings render path rebuilt its installed-app catalog on every frame, including frames that only displayed the Settings home page. It matched desktop entries to built-in apps and requested each icon. A cold icon lookup could search the filesystem and decode an image during the launch animation. Independently, Wisp polled Shiraz status on the compositor thread. Reading that status also scanned installed Android desktop entries, so the scan could coincide with a tap.

Wisp now reuses the catalog, resolves desktop icons on a worker, and collects completed status snapshots without waiting. Four post-change launches on the Poco F1 reached the fully drawn Settings page in 0.85–0.94 seconds as observed over the remote stream. That measurement includes the tap, animation, and streaming delay. We did not capture a trace of the original 1–2 second stall, so its exact breakdown remains unknown. [Settings app catalog](../../codes/apps/wisp/src/settings_apps.rs) · [Wisp integration](../../codes/apps/wisp/src/lib.rs) · [Shiraz discovery](../../codes/apps/settings/src/state.rs)

## Verify a change on the phone

- Measure **tap to first visible response** and **tap to settled page** separately. Record whether timings came from device timestamps, a camera, or a remote stream. If diagnosing a stall, capture a baseline before changing the code and record how long the suspected operation takes.
- Exercise the first launch after restarting the app, several warm launches, and a launch during a refresh. Check on-device frame timings around the tap. At 60 Hz the entire frame has about 16.7 ms, including drawing and presentation.
- Delay or fail a data request in a test. Verify that the page opens, back and scrolling work, the placeholder resolves on success, and a late result cannot replace a newer one. Check the empty and failure states separately.
- After an unattended Poco test, end the remote session and verify the display is off.
