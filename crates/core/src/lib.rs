//! The model and the engines of clauderoadmapwrangler, without UI types (D-14). The roadmap
//! parsers, the session runner and the usage meter arrive here brief by brief; EP-1 brought the
//! version checks behind `crw doctor`, EP-2 the registry, discovery and the ledger, EP-3 the
//! roadmap README's parser.

pub mod discovery;
pub mod doctor;
pub mod files;
pub mod ledger;
pub mod paths;
pub mod process;
pub mod proposal;
pub mod registry;
pub mod roadmap;
pub mod version;
pub mod workspace;
