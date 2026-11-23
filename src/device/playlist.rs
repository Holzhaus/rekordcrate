// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! The playlist tree of a device export.

use crate::pdb::io::Database;
use crate::pdb::{PlaylistTreeNode, PlaylistTreeNodeId};
use fallible_iterator::FallibleIterator;
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Either a playlist folder or a playlist.
#[derive(Debug, PartialEq)]
pub enum PlaylistNode {
    /// A folder containing child [`PlaylistNode`]s.
    Folder(PlaylistFolder),
    /// A playlist (leaf).
    Playlist(Playlist),
}

/// A playlist folder.
#[derive(Debug, PartialEq)]
pub struct PlaylistFolder {
    /// ID of this node in the playlist tree.
    pub id: PlaylistTreeNodeId,
    /// Name of the playlist folder.
    pub name: String,
    /// Child nodes of the playlist folder.
    pub children: Vec<PlaylistNode>,
}

/// A playlist.
#[derive(Debug, PartialEq, Eq)]
pub struct Playlist {
    /// ID of this node in the playlist tree.
    pub id: PlaylistTreeNodeId,
    /// Name of the playlist.
    pub name: String,
}

/// Build the playlist tree from the PDB database, each level in display order.
pub fn get_playlists<R: Read + Seek>(db: &mut Database<R>) -> crate::Result<Vec<PlaylistNode>> {
    let mut playlists: HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>> = HashMap::new();

    db.iter_rows::<PlaylistTreeNode>()?.for_each(|node| {
        playlists
            .entry(node.parent_id)
            .or_default()
            .push(node.clone());
        Ok(())
    })?;
    // Rows are stored in no particular order; `sort_order` is the display order among siblings.
    for siblings in playlists.values_mut() {
        siblings.sort_by_key(|node| node.sort_order);
    }

    fn get_child_nodes(
        playlists: &HashMap<PlaylistTreeNodeId, Vec<PlaylistTreeNode>>,
        id: PlaylistTreeNodeId,
    ) -> impl Iterator<Item = crate::Result<PlaylistNode>> + '_ {
        playlists
            .get(&id)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
            .map(|node| -> crate::Result<PlaylistNode> {
                let child_node = if node.is_folder() {
                    let folder = PlaylistFolder {
                        id: node.id,
                        name: node.name.clone().into_string()?,
                        children: get_child_nodes(playlists, node.id)
                            .collect::<crate::Result<Vec<PlaylistNode>>>()?,
                    };
                    PlaylistNode::Folder(folder)
                } else {
                    let playlist = Playlist {
                        id: node.id,
                        name: node.name.clone().into_string()?,
                    };
                    PlaylistNode::Playlist(playlist)
                };
                Ok(child_node)
            })
    }

    get_child_nodes(&playlists, PlaylistTreeNodeId(0)).collect::<crate::Result<Vec<PlaylistNode>>>()
}
