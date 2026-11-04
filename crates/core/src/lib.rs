//! The model and the engines of clauderoadmapwrangler, without UI types (D-14). The roadmap
//! parsers, the session runner and the usage meter arrive here brief by brief; EP-1 brought the
//! version checks behind `crw doctor`, EP-2 the registry, discovery and the ledger, EP-3 the
//! roadmap README's parser, EP-4 the briefs, the decision log and the snapshot, EP-5 the owner
//! tasks, the inbox and the older forms of the owner's to-dos, EP-7 the badges and the git
//! plumbing they share with the snapshot, EP-9 the session runner, EP-10 the launch prompt, the
//! rule table and the model catalog, EP-11 the visual launch, EP-12 the signals (the hook and
//! status-line relays and their installation) and the sessions the tool sees, EP-15 the
//! notifications' rules, EP-15b a session's start and its revert.

pub mod badges;
pub mod catalog;
pub mod checklist;
pub mod decisions;
pub mod discovery;
pub mod doctor;
pub mod files;
pub mod git;
pub mod inbox;
pub mod ledger;
pub mod notify;
pub mod paths;
pub mod process;
pub mod prompt;
pub mod proposal;
pub mod registry;
pub mod revert;
pub mod roadmap;
pub mod rules;
pub mod session;
pub mod sessions;
pub mod signals;
pub mod tasks;
pub mod version;
pub mod visual;
pub mod workspace;
