# SpectreOS Calendar

Calendar is a native Wisp app in the launcher. Its week timeline, day, month, and agenda
workflows take their reference from the open-source Etar calendar shipped on the Galaxy S9.
The app uses its own Rust implementation, Wisp theme, and local data store.

- Tap the add button or a time slot to create an event. Choose a calendar, dates and times,
  recurrence, and a reminder. The start and end date buttons open libbhoo’s Android
  calendar-mode DatePicker. Use OK to apply the date or Cancel to keep the draft
  unchanged. The event editor slides and fades in unless reduced motion is enabled.
  Events can also span multiple days or be marked all-day.
- In Day and Week views, press and hold an empty time slot, then drag up or down to
  select a time range in 15-minute increments. Release to open the event editor.
- In Week view, drag sideways or scroll horizontally to move through dates continuously.
  Quick swipes coast to a stop. The arrows move by a week, and vertical scrolling moves through the hours. Only
  today has a tinted timeline column. The Today button eases back to the current
  date and time in Day and Week views, or jumps there with reduced motion.
- Tap a recurring event to edit or delete that occurrence or the full series.
- Open the top-left menu to switch between Day, Week, Month, and Agenda. The Material
  navigation drawer slides in from the left and highlights the current view. Select a
  view, tap outside, or use Back to close it. The three-dot button opens a bottom sheet with Search, Go to date,
  calendar management, and iCalendar import and export.
- Create and name local calendars, assign colors, and hide or show each calendar.
- Import or export iCalendar files from the overflow menu. Individual events and their
  series can be exported from the event details page. Import supports date-only events,
  named IANA time zones, recurrence rules, exception dates, and display alarms before the
  start. Imports go to the first visible calendar. Matching event identifiers update the
  existing copy.
- Wisp posts reminders while the app is closed. The Calendar launcher widget reads
  upcoming events from the same store.

Data lives in `$XDG_DATA_HOME/spectre-calendar/calendar.json`, with the standard
`~/.local/share` fallback. The launcher handles the application window and keyboard.

```sh
cargo test --locked --manifest-path codes/apps/calendar/Cargo.toml
```

Cloud-account and CalDAV synchronization, sending invitations, and arbitrary custom
iCalendar timezone definitions are outside this implementation.

[Etar source and feature reference](https://github.com/Etar-Group/Etar-Calendar)
