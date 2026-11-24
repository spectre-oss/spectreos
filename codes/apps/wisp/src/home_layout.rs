use std::collections::HashSet;

use crate::AppId;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum HomeApp {
    Builtin(AppId),
    Desktop(String),
}

pub(super) type HomeCell = (usize, usize, HomeApp);

pub(super) struct HomeLayout {
    pub(super) grid: (usize, usize),
    pub(super) cells: Vec<HomeCell>,
    pub(super) pages: Vec<usize>,
}

impl HomeLayout {
    fn parse(contents: &str) -> Option<Self> {
        let mut lines = contents.lines();
        let [version, columns, rows] = lines
            .next()?
            .split('\t')
            .collect::<Vec<_>>()
            .try_into()
            .ok()?;
        if version != "1" {
            return None;
        }
        let grid = (columns.parse::<usize>().ok()?, rows.parse::<usize>().ok()?);
        if !(2..=5).contains(&grid.0) || !(2..=6).contains(&grid.1) {
            return None;
        }

        let mut app_ids = HashSet::new();
        let mut occupied = HashSet::new();
        let mut layout = Self {
            grid,
            cells: Vec::new(),
            pages: Vec::new(),
        };
        for line in lines {
            let [page, col, row, kind, id] =
                line.split('\t').collect::<Vec<_>>().try_into().ok()?;
            let (page, col, row) = (
                page.parse::<usize>().ok()?,
                col.parse::<usize>().ok()?,
                row.parse::<usize>().ok()?,
            );
            if page > u16::MAX as usize || col >= grid.0 || row >= grid.1 {
                return None;
            }
            let app = match kind {
                "builtin" => {
                    let Some(app) = AppId::ALL.iter().copied().find(|app| app.package() == id)
                    else {
                        continue;
                    };
                    HomeApp::Builtin(app)
                }
                "desktop" if !id.is_empty() => HomeApp::Desktop(id.into()),
                _ => return None,
            };
            if !app_ids.insert(app.clone()) || !occupied.insert((page, col, row)) {
                return None;
            }
            layout.cells.push((col, row, app));
            layout.pages.push(page);
        }
        Some(layout)
    }

    fn encode(grid: (usize, usize), cells: &[HomeCell], pages: &[usize]) -> String {
        use std::fmt::Write;

        let mut text = format!("1\t{}\t{}\n", grid.0, grid.1);
        for ((col, row, app), page) in cells.iter().zip(pages) {
            let (kind, id) = match app {
                HomeApp::Builtin(app) => ("builtin", app.package()),
                HomeApp::Desktop(id) => ("desktop", id.as_str()),
            };
            if !id.contains(['\t', '\n', '\r']) {
                let _ = writeln!(text, "{page}\t{col}\t{row}\t{kind}\t{id}");
            }
        }
        text
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn load() -> Option<Self> {
        let path = crate::settings::xdg_config_home()?.join("wisp/home-layout");
        Self::parse(&std::fs::read_to_string(path).ok()?)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn save(
        grid: (usize, usize),
        cells: &[HomeCell],
        pages: &[usize],
    ) -> std::io::Result<()> {
        let directory = crate::settings::xdg_config_home()
            .ok_or(std::io::ErrorKind::NotFound)?
            .join("wisp");
        std::fs::create_dir_all(&directory)?;
        let pending = directory.join("home-layout.new");
        std::fs::write(&pending, Self::encode(grid, cells, pages))?;
        std::fs::rename(pending, directory.join("home-layout"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_layout_persists_native_and_desktop_shortcuts_across_pages() {
        let cells = vec![
            (1, 4, HomeApp::Builtin(AppId::Clock)),
            (0, 2, HomeApp::Desktop("org.example.App.desktop".into())),
        ];
        let encoded = HomeLayout::encode((5, 6), &cells, &[0, 2]);
        let decoded = HomeLayout::parse(&encoded).unwrap();
        assert_eq!(decoded.grid, (5, 6));
        assert_eq!(decoded.pages, [0, 2]);
        assert_eq!(decoded.cells, cells);
    }

    #[test]
    fn home_layout_rejects_duplicate_slots_and_invalid_coordinates() {
        let first = "1\t5\t5\n0\t1\t4\tdesktop\tfirst.desktop\n";
        assert!(HomeLayout::parse(&format!("{first}0\t1\t4\tdesktop\tsecond.desktop\n")).is_none());
        assert!(HomeLayout::parse(&format!("{first}0\t1\t5\tdesktop\tsecond.desktop\n")).is_none());
    }
}
