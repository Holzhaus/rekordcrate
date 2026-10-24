CREATE TABLE content(
    content_id integer primary key,
    title varchar,
    titleForSearch varchar,
    subtitle varchar,
    bpmx100 integer,
    length integer,
    trackNo integer,
    discNo integer,
    artist_id_artist integer,
    artist_id_remixer integer,
    artist_id_originalArtist integer,
    artist_id_composer integer,
    artist_id_lyricist integer,
    album_id integer,
    genre_id integer,
    label_id integer,
    key_id integer,
    color_id integer,
    image_id integer,
    djComment varchar,
    rating integer,
    releaseYear integer,
    releaseDate varchar,
    dateCreated varchar,
    dateAdded varchar,
    path varchar,
    fileName varchar,
    fileSize integer,
    fileType integer,
    bitrate integer,
    bitDepth integer,
    samplingRate integer,
    isrc varchar,
    djPlayCount integer,
    isHotCueAutoLoadOn integer,
    isKuvoDeliverStatusOn integer,
    kuvoDeliveryComment varchar,
    masterDbId integer,
    masterContentId integer,
    analysisDataFilePath varchar,
    analysedBits integer,
    contentLink integer,
    hasModified integer,
    cueUpdateCount integer,
    analysisDataUpdateCount integer,
    informationUpdateCount integer
);
CREATE TABLE genre(genre_id integer primary key, name varchar);
CREATE TABLE artist(artist_id integer primary key, name varchar, nameForSearch varchar);
CREATE TABLE album(
    album_id integer primary key,
    name varchar,
    artist_id integer,
    image_id integer,
    isComplation integer,
    nameForSearch varchar
);
CREATE TABLE label(label_id integer primary key, name varchar);
CREATE TABLE key(key_id integer primary key, name varchar);
CREATE TABLE color(color_id integer primary key, name varchar);
CREATE TABLE playlist(
    playlist_id integer primary key,
    sequenceNo integer,
    name varchar,
    image_id integer,
    attribute integer,
    playlist_id_parent integer
);
CREATE TABLE playlist_content(playlist_id integer, content_id integer, sequenceNo integer);
CREATE TABLE hotCueBankList(
    hotCueBankList_id integer primary key,
    sequenceNo integer,
    name varchar,
    image_id integer,
    attribute integer,
    hotCueBankList_id_parent integer
);
CREATE TABLE hotCueBankList_cue(hotCueBankList_id integer, cue_id integer, sequenceNo integer);
CREATE TABLE history(
    history_id integer primary key,
    sequenceNo integer,
    name varchar,
    attribute integer,
    history_id_parent integer
);
CREATE TABLE history_content(history_id integer, content_id integer, sequenceNo integer);
CREATE TABLE image(image_id integer primary key, path varchar);
CREATE TABLE cue(
    cue_id integer primary key,
    content_id integer,
    kind integer,
    colorTableIndex integer,
    cueComment varchar,
    isActiveLoop integer,
    beatLoopNumerator integer,
    beatLoopDenominator integer,
    inUsec integer,
    outUsec integer,
    in150FramePerSec integer,
    out150FramePerSec integer,
    inMpegFrameNumber integer,
    outMpegFrameNumber integer,
    inMpegAbs integer,
    outMpegAbs integer,
    inDecodingStartFramePosition integer,
    outDecodingStartFramePosition integer,
    inFileOffsetInBlock integer,
    OutFileOffsetInBlock integer,
    inNumberOfSampleInBlock integer,
    outNumberOfSampleInBlock integer
);
CREATE TABLE menuItem(menuItem_id integer primary key, kind integer, name varchar);
CREATE TABLE category(category_id integer primary key, menuItem_id integer, sequenceNo integer, isVisible integer);
CREATE TABLE sort(
    sort_id integer primary key,
    menuItem_id integer,
    sequenceNo integer,
    isVisible integer,
    isSelectedAsSubColumn integer
);
CREATE TABLE property(
    deviceName varchar,
    dbVersion varchar,
    numberOfContents integer,
    createdDate varchar,
    backGroundColorType integer,
    myTagMasterDBID integer
);
CREATE TABLE recommendedLike(content_id_1 integer, content_id_2 integer, rating integer, createdDate integer);
CREATE TABLE myTag(myTag_id integer primary key, sequenceNo integer, name varchar, attribute integer, myTag_id_parent integer);
CREATE TABLE myTag_content(myTag_id integer, content_id integer);
CREATE INDEX index_playlist_content_playlist_id on playlist_content(playlist_id);
CREATE INDEX index_myTag_content_myTag_id on myTag_content(myTag_id);
CREATE INDEX index_myTag_content_content_id on myTag_content(content_id);
CREATE INDEX index_hotCueBankList_cue_hotCueBankList_id on hotCueBankList_cue(hotCueBankList_id);
