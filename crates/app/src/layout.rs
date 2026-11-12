//! The docking layout (D-25): every pane sits in a tab group on an `egui_tiles` tree, which the
//! owner rearranges by dragging tabs. The tree is kept in eframe's state with the window's size
//! and place; a saved tree gains the panes of a later build beside the Detail pane, so the
//! owner's arrangement survives an upgrade.

use eframe::egui;
use egui_tiles::{Container, SimplificationOptions, Tile, TileId, Tiles, Tree, UiResponse};
use serde::{Deserialize, Serialize};

use crate::board::Board;

pub const LAYOUT_KEY: &str = "layout";

/// A pane of the window. Each appears exactly once in the layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pane {
    Projects,
    Detail,
    Decisions,
    Sessions,
    Chat,
    Usage,
}

impl Pane {
    pub const ALL: [Pane; 6] = [
        Pane::Projects,
        Pane::Detail,
        Pane::Decisions,
        Pane::Sessions,
        Pane::Chat,
        Pane::Usage,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Pane::Projects => "Projects",
            Pane::Detail => "Detail",
            Pane::Decisions => "Decisions",
            Pane::Sessions => "Sessions",
            Pane::Chat => "Chat",
            Pane::Usage => "Usage",
        }
    }

    /// The pane named in a screenshot run's setup (`tile=detail`), case aside.
    #[cfg(feature = "screenshot")]
    pub fn from_name(name: &str) -> Option<Pane> {
        Pane::ALL
            .into_iter()
            .find(|p| p.title().eq_ignore_ascii_case(name))
    }
}

/// The project list left and wide, the detail in the centre, and the tiles later briefs fill
/// tabbed on the right behind Decisions.
pub fn default_layout() -> Tree<Pane> {
    let mut tiles = Tiles::default();
    let groups: [(&[Pane], f32); 3] = [
        (&[Pane::Projects], 1.75),
        (&[Pane::Detail], 1.0),
        (
            &[Pane::Decisions, Pane::Sessions, Pane::Chat, Pane::Usage],
            0.45,
        ),
    ];
    let tabs: Vec<TileId> = groups
        .iter()
        .map(|(panes, _)| {
            let panes = panes.iter().map(|&p| tiles.insert_pane(p)).collect();
            tiles.insert_tab_tile(panes)
        })
        .collect();
    let root = tiles.insert_horizontal_tile(tabs.clone());
    if let Some(Tile::Container(Container::Linear(row))) = tiles.get_mut(root) {
        for (&tab, &(_, share)) in tabs.iter().zip(&groups) {
            row.shares.set_share(tab, share);
        }
    }
    Tree::new(egui::Id::unique("layout"), root, tiles)
}

/// The saved layout, completed with any pane it lacks; the default when none is saved or the
/// saved one is not usable.
pub fn restore(storage: Option<&dyn eframe::Storage>) -> Tree<Pane> {
    storage
        .and_then(|s| eframe::get_value::<Tree<Pane>>(s, LAYOUT_KEY))
        .and_then(complete)
        .unwrap_or_else(default_layout)
}

/// A saved tree that holds the Detail pane and no pane twice, with the panes it lacks added as
/// tabs beside the Detail pane, which stays in front. `None` for anything else.
fn complete(mut tree: Tree<Pane>) -> Option<Tree<Pane>> {
    let found = panes_once(&tree)?;
    let detail = tree.tiles.find_pane(&Pane::Detail)?;
    let beside = tree.tiles.parent_of(detail)?;
    for pane in Pane::ALL.into_iter().filter(|p| !found.contains(p)) {
        let id = tree.tiles.insert_pane(pane);
        match tree.tiles.get_mut(beside) {
            Some(Tile::Container(Container::Tabs(tabs))) => tabs.add_child(id),
            _ => return None,
        }
    }
    Some(tree)
}

/// The panes reachable from the root, if each is visible and there only once.
fn panes_once(tree: &Tree<Pane>) -> Option<Vec<Pane>> {
    let mut found = Vec::new();
    let mut pending: Vec<TileId> = tree.root().into_iter().collect();
    // Each tile is visited once in a well-formed tree; more visits mean a cycle or a tile
    // shared by two containers.
    let mut budget = tree.tiles.len();
    while let Some(id) = pending.pop() {
        if budget == 0 || !tree.is_visible(id) {
            return None;
        }
        budget -= 1;
        match tree.tiles.get(id)? {
            Tile::Pane(pane) if found.contains(pane) => return None,
            Tile::Pane(pane) => found.push(*pane),
            Tile::Container(container) => pending.extend(container.children()),
        }
    }
    Some(found)
}

/// Brings the tab holding `pane` to the front of its group.
pub fn show(tree: &mut Tree<Pane>, pane: Pane) {
    tree.make_active(|_, tile| matches!(tile, Tile::Pane(p) if *p == pane));
}

impl egui_tiles::Behavior<Pane> for Board {
    /// The Detail tab carries the selection's name; Decisions and Sessions their counts.
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Detail => match self.selected_name() {
                Some(name) => format!("Detail · {name}").into(),
                None => "Detail".into(),
            },
            Pane::Decisions if self.decisions_count() > 0 => {
                format!("Decisions · {}", self.decisions_count()).into()
            }
            Pane::Sessions => {
                // A conversation's run is the Chat tile's, not a tab here (EP-24).
                let runs = self
                    .sessions
                    .live
                    .iter()
                    .filter(|live| !live.is_chat())
                    .count();
                match runs + self.seen_sessions().len() {
                    0 => "Sessions".into(),
                    count => format!("Sessions · {count}").into(),
                }
            }
            pane => pane.title().into(),
        }
    }

    fn pane_ui(&mut self, ui: &mut egui::Ui, _tile_id: TileId, pane: &mut Pane) -> UiResponse {
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| match pane {
                Pane::Projects => self.projects_pane(ui),
                Pane::Detail => self.detail_pane(ui),
                Pane::Decisions => self.decisions_pane(ui),
                Pane::Sessions => self.sessions_pane(ui),
                Pane::Chat => self.chat_pane(ui),
                Pane::Usage => self.usage_pane(ui),
            });
        UiResponse::None
    }

    /// Every pane keeps a tab, so it can always be dragged somewhere else by it.
    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..SimplificationOptions::default()
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;

    use eframe::Storage as _;

    use super::*;

    /// eframe's file storage without the file, so the tests go through the same RON encoding
    /// the app writes.
    #[derive(Default)]
    pub(crate) struct MemoryStorage(HashMap<String, String>);

    impl eframe::Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.to_owned(), value);
        }
        fn remove_string(&mut self, key: &str) {
            self.0.remove(key);
        }
        fn flush(&mut self) {}
    }

    fn shown(tree: &Tree<Pane>) -> Vec<Pane> {
        let mut shown: Vec<Pane> = tree
            .active_tiles()
            .into_iter()
            .filter_map(|id| tree.tiles.get_pane(&id).copied())
            .collect();
        shown.sort_by_key(|p| Pane::ALL.iter().position(|q| q == p));
        shown
    }

    fn holds_every_pane_once(tree: &Tree<Pane>) -> bool {
        panes_once(tree).is_some_and(|found| found.len() == Pane::ALL.len())
    }

    #[test]
    fn the_default_layout_shows_the_list_the_detail_and_the_decisions() {
        let mut tree = default_layout();
        assert!(holds_every_pane_once(&tree));
        assert_eq!(
            shown(&tree),
            [Pane::Projects, Pane::Detail, Pane::Decisions]
        );
        show(&mut tree, Pane::Usage);
        assert_eq!(shown(&tree), [Pane::Projects, Pane::Detail, Pane::Usage]);
    }

    #[test]
    fn a_rearranged_layout_survives_the_round_trip_through_storage() {
        let mut tree = default_layout();
        // What dragging the Chat tab onto the list's tab bar does.
        let chat = tree.tiles.find_pane(&Pane::Chat).unwrap();
        let list_tabs = tree
            .tiles
            .parent_of(tree.tiles.find_pane(&Pane::Projects).unwrap())
            .unwrap();
        tree.move_tile_to_container(chat, list_tabs, 1, false);
        assert!(holds_every_pane_once(&tree));

        let mut storage = MemoryStorage::default();
        eframe::set_value(&mut storage, LAYOUT_KEY, &tree);
        let restored = restore(Some(&storage));
        assert_eq!(restored, tree);
        assert_ne!(restored, default_layout());
    }

    #[test]
    fn a_layout_without_a_later_pane_gains_it_beside_the_detail() {
        let id = egui::Id::unique("layout");
        let older = Tree::new_tabs(id, vec![Pane::Projects, Pane::Detail]);
        let mut storage = MemoryStorage::default();
        eframe::set_value(&mut storage, LAYOUT_KEY, &older);
        let completed = restore(Some(&storage));
        assert!(holds_every_pane_once(&completed));
        let group = |pane| {
            completed
                .tiles
                .parent_of(completed.tiles.find_pane(&pane).unwrap())
                .unwrap()
        };
        assert_eq!(group(Pane::Usage), group(Pane::Detail));
    }

    #[test]
    fn an_unusable_saved_layout_gives_way_to_the_default() {
        let mut storage = MemoryStorage::default();
        assert_eq!(restore(None), default_layout());
        assert_eq!(restore(Some(&storage)), default_layout());
        storage.set_string(LAYOUT_KEY, "not a layout".to_owned());
        assert_eq!(restore(Some(&storage)), default_layout());
        // A pane twice, or no Detail pane for the missing ones to join.
        let id = egui::Id::unique("layout");
        let mut twice = Pane::ALL.to_vec();
        twice.push(Pane::Chat);
        for tree in [
            Tree::new_tabs(id, twice),
            Tree::new_tabs(id, vec![Pane::Projects, Pane::Usage]),
        ] {
            eframe::set_value(&mut storage, LAYOUT_KEY, &tree);
            assert_eq!(restore(Some(&storage)), default_layout());
        }
    }
}
