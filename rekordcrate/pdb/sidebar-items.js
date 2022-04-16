initSidebarItems({"enum":[["PageType","The type of pages found inside a `Table`."],["Row","A table row contains the actual data."]],"mod":[["string","`binrw`-based implementation for DeviceSQLStrings capable of parsing and serializing [`DeviceSQLString`]s"]],"struct":[["Album","Contains the album name, along with an ID of the corresponding artist."],["Artist","Contains the artist name and ID."],["Artwork","Contains the artwork path and ID."],["Color","Contains numeric color ID"],["Genre","Represents a musical genre."],["Header","The PDB header structure, including the list of tables."],["HistoryEntry","Represents a history playlist."],["HistoryPlaylist","Represents a history playlist."],["Key","Represents a musical key."],["Label","Represents a record label."],["Page","A table page."],["PageIndex","Points to a table page and can be used to calculate the page’s file offset by multiplying it with the page size (found in the file header)."],["PlaylistEntry","Represents a track entry in a playlist."],["PlaylistTreeNode","Represents a node in the playlist tree (either a folder or a playlist)."],["RowGroup","A group of row indices, which are built backwards from the end of the page. Holds up to sixteen row offsets, along with a bit mask that indicates whether each row is actually present in the table."],["Table","Tables are linked lists of pages containing rows of a single type, which are organized into groups."],["Track","Contains the album name, along with an ID of the corresponding artist."]]});