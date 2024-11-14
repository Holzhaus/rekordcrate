(function() {
    var implementors = Object.fromEntries([["rekordcrate",[["impl BinRead for <a class=\"enum\" href=\"rekordcrate/anlz/enum.Bank.html\" title=\"enum rekordcrate::anlz::Bank\">Bank</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/anlz/enum.Content.html\" title=\"enum rekordcrate::anlz::Content\">Content</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/anlz/enum.ContentKind.html\" title=\"enum rekordcrate::anlz::ContentKind\">ContentKind</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/anlz/enum.CueListType.html\" title=\"enum rekordcrate::anlz::CueListType\">CueListType</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/anlz/enum.CueType.html\" title=\"enum rekordcrate::anlz::CueType\">CueType</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/anlz/enum.Mood.html\" title=\"enum rekordcrate::anlz::Mood\">Mood</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/pdb/enum.PageType.html\" title=\"enum rekordcrate::pdb::PageType\">PageType</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/pdb/enum.Row.html\" title=\"enum rekordcrate::pdb::Row\">Row</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.AutoCue.html\" title=\"enum rekordcrate::setting::AutoCue\">AutoCue</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.AutoCueLevel.html\" title=\"enum rekordcrate::setting::AutoCueLevel\">AutoCueLevel</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.BeatFXQuantize.html\" title=\"enum rekordcrate::setting::BeatFXQuantize\">BeatFXQuantize</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.BeatJumpBeatValue.html\" title=\"enum rekordcrate::setting::BeatJumpBeatValue\">BeatJumpBeatValue</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.ChannelFaderCurve.html\" title=\"enum rekordcrate::setting::ChannelFaderCurve\">ChannelFaderCurve</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.ChannelFaderCurveLongFader.html\" title=\"enum rekordcrate::setting::ChannelFaderCurveLongFader\">ChannelFaderCurveLongFader</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.CrossfaderCurve.html\" title=\"enum rekordcrate::setting::CrossfaderCurve\">CrossfaderCurve</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.DiscSlotIllumination.html\" title=\"enum rekordcrate::setting::DiscSlotIllumination\">DiscSlotIllumination</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.EjectLock.html\" title=\"enum rekordcrate::setting::EjectLock\">EjectLock</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.HeadphonesMonoSplit.html\" title=\"enum rekordcrate::setting::HeadphonesMonoSplit\">HeadphonesMonoSplit</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.HeadphonesPreEQ.html\" title=\"enum rekordcrate::setting::HeadphonesPreEQ\">HeadphonesPreEQ</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.HotCueAutoLoad.html\" title=\"enum rekordcrate::setting::HotCueAutoLoad\">HotCueAutoLoad</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.HotCueColor.html\" title=\"enum rekordcrate::setting::HotCueColor\">HotCueColor</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.JogDisplayMode.html\" title=\"enum rekordcrate::setting::JogDisplayMode\">JogDisplayMode</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.JogLCDBrightness.html\" title=\"enum rekordcrate::setting::JogLCDBrightness\">JogLCDBrightness</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.JogMode.html\" title=\"enum rekordcrate::setting::JogMode\">JogMode</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.JogRingBrightness.html\" title=\"enum rekordcrate::setting::JogRingBrightness\">JogRingBrightness</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.JogRingIndicator.html\" title=\"enum rekordcrate::setting::JogRingIndicator\">JogRingIndicator</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.KeyDisplayFormat.html\" title=\"enum rekordcrate::setting::KeyDisplayFormat\">KeyDisplayFormat</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.LCDBrightness.html\" title=\"enum rekordcrate::setting::LCDBrightness\">LCDBrightness</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.Language.html\" title=\"enum rekordcrate::setting::Language\">Language</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.MasterTempo.html\" title=\"enum rekordcrate::setting::MasterTempo\">MasterTempo</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.MicLowCut.html\" title=\"enum rekordcrate::setting::MicLowCut\">MicLowCut</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.MidiButtonType.html\" title=\"enum rekordcrate::setting::MidiButtonType\">MidiButtonType</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.MidiChannel.html\" title=\"enum rekordcrate::setting::MidiChannel\">MidiChannel</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.MixerDisplayBrightness.html\" title=\"enum rekordcrate::setting::MixerDisplayBrightness\">MixerDisplayBrightness</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.MixerIndicatorBrightness.html\" title=\"enum rekordcrate::setting::MixerIndicatorBrightness\">MixerIndicatorBrightness</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.NeedleLock.html\" title=\"enum rekordcrate::setting::NeedleLock\">NeedleLock</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.OnAirDisplay.html\" title=\"enum rekordcrate::setting::OnAirDisplay\">OnAirDisplay</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.OverviewWaveformType.html\" title=\"enum rekordcrate::setting::OverviewWaveformType\">OverviewWaveformType</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.PadButtonBrightness.html\" title=\"enum rekordcrate::setting::PadButtonBrightness\">PadButtonBrightness</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.PhaseMeter.html\" title=\"enum rekordcrate::setting::PhaseMeter\">PhaseMeter</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.PlayMode.html\" title=\"enum rekordcrate::setting::PlayMode\">PlayMode</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.Quantize.html\" title=\"enum rekordcrate::setting::Quantize\">Quantize</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.QuantizeBeatValue.html\" title=\"enum rekordcrate::setting::QuantizeBeatValue\">QuantizeBeatValue</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.SettingData.html\" title=\"enum rekordcrate::setting::SettingData\">SettingData</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.SlipFlashing.html\" title=\"enum rekordcrate::setting::SlipFlashing\">SlipFlashing</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.Sync.html\" title=\"enum rekordcrate::setting::Sync\">Sync</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.TalkOverLevel.html\" title=\"enum rekordcrate::setting::TalkOverLevel\">TalkOverLevel</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.TalkOverMode.html\" title=\"enum rekordcrate::setting::TalkOverMode\">TalkOverMode</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.TempoRange.html\" title=\"enum rekordcrate::setting::TempoRange\">TempoRange</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.TimeMode.html\" title=\"enum rekordcrate::setting::TimeMode\">TimeMode</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.VinylSpeedAdjust.html\" title=\"enum rekordcrate::setting::VinylSpeedAdjust\">VinylSpeedAdjust</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.Waveform.html\" title=\"enum rekordcrate::setting::Waveform\">Waveform</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.WaveformColor.html\" title=\"enum rekordcrate::setting::WaveformColor\">WaveformColor</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.WaveformCurrentPosition.html\" title=\"enum rekordcrate::setting::WaveformCurrentPosition\">WaveformCurrentPosition</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/setting/enum.WaveformDivisions.html\" title=\"enum rekordcrate::setting::WaveformDivisions\">WaveformDivisions</a>"],["impl BinRead for <a class=\"enum\" href=\"rekordcrate/util/enum.ColorIndex.html\" title=\"enum rekordcrate::util::ColorIndex\">ColorIndex</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.ANLZ.html\" title=\"struct rekordcrate::anlz::ANLZ\">ANLZ</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.Beat.html\" title=\"struct rekordcrate::anlz::Beat\">Beat</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.BeatGrid.html\" title=\"struct rekordcrate::anlz::BeatGrid\">BeatGrid</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.Cue.html\" title=\"struct rekordcrate::anlz::Cue\">Cue</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.CueList.html\" title=\"struct rekordcrate::anlz::CueList\">CueList</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.ExtendedCue.html\" title=\"struct rekordcrate::anlz::ExtendedCue\">ExtendedCue</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.ExtendedCueList.html\" title=\"struct rekordcrate::anlz::ExtendedCueList\">ExtendedCueList</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.Header.html\" title=\"struct rekordcrate::anlz::Header\">Header</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.Path.html\" title=\"struct rekordcrate::anlz::Path\">Path</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.Phrase.html\" title=\"struct rekordcrate::anlz::Phrase\">Phrase</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.Section.html\" title=\"struct rekordcrate::anlz::Section\">Section</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.SongStructure.html\" title=\"struct rekordcrate::anlz::SongStructure\">SongStructure</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.SongStructureData.html\" title=\"struct rekordcrate::anlz::SongStructureData\">SongStructureData</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.TinyWaveformPreview.html\" title=\"struct rekordcrate::anlz::TinyWaveformPreview\">TinyWaveformPreview</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.TinyWaveformPreviewColumn.html\" title=\"struct rekordcrate::anlz::TinyWaveformPreviewColumn\">TinyWaveformPreviewColumn</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.Unknown.html\" title=\"struct rekordcrate::anlz::Unknown\">Unknown</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.VBR.html\" title=\"struct rekordcrate::anlz::VBR\">VBR</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.WaveformColorDetail.html\" title=\"struct rekordcrate::anlz::WaveformColorDetail\">WaveformColorDetail</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.WaveformColorDetailColumn.html\" title=\"struct rekordcrate::anlz::WaveformColorDetailColumn\">WaveformColorDetailColumn</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.WaveformColorPreview.html\" title=\"struct rekordcrate::anlz::WaveformColorPreview\">WaveformColorPreview</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.WaveformColorPreviewColumn.html\" title=\"struct rekordcrate::anlz::WaveformColorPreviewColumn\">WaveformColorPreviewColumn</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.WaveformDetail.html\" title=\"struct rekordcrate::anlz::WaveformDetail\">WaveformDetail</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.WaveformPreview.html\" title=\"struct rekordcrate::anlz::WaveformPreview\">WaveformPreview</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/anlz/struct.WaveformPreviewColumn.html\" title=\"struct rekordcrate::anlz::WaveformPreviewColumn\">WaveformPreviewColumn</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/string/struct.DeviceSQLString.html\" title=\"struct rekordcrate::pdb::string::DeviceSQLString\">DeviceSQLString</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Album.html\" title=\"struct rekordcrate::pdb::Album\">Album</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.AlbumId.html\" title=\"struct rekordcrate::pdb::AlbumId\">AlbumId</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Artist.html\" title=\"struct rekordcrate::pdb::Artist\">Artist</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.ArtistId.html\" title=\"struct rekordcrate::pdb::ArtistId\">ArtistId</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Artwork.html\" title=\"struct rekordcrate::pdb::Artwork\">Artwork</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.ArtworkId.html\" title=\"struct rekordcrate::pdb::ArtworkId\">ArtworkId</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Color.html\" title=\"struct rekordcrate::pdb::Color\">Color</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.ColumnEntry.html\" title=\"struct rekordcrate::pdb::ColumnEntry\">ColumnEntry</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Genre.html\" title=\"struct rekordcrate::pdb::Genre\">Genre</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.GenreId.html\" title=\"struct rekordcrate::pdb::GenreId\">GenreId</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Header.html\" title=\"struct rekordcrate::pdb::Header\">Header</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.HistoryEntry.html\" title=\"struct rekordcrate::pdb::HistoryEntry\">HistoryEntry</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.HistoryPlaylist.html\" title=\"struct rekordcrate::pdb::HistoryPlaylist\">HistoryPlaylist</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.HistoryPlaylistId.html\" title=\"struct rekordcrate::pdb::HistoryPlaylistId\">HistoryPlaylistId</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Key.html\" title=\"struct rekordcrate::pdb::Key\">Key</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.KeyId.html\" title=\"struct rekordcrate::pdb::KeyId\">KeyId</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Label.html\" title=\"struct rekordcrate::pdb::Label\">Label</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.LabelId.html\" title=\"struct rekordcrate::pdb::LabelId\">LabelId</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Page.html\" title=\"struct rekordcrate::pdb::Page\">Page</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.PageIndex.html\" title=\"struct rekordcrate::pdb::PageIndex\">PageIndex</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.PlaylistEntry.html\" title=\"struct rekordcrate::pdb::PlaylistEntry\">PlaylistEntry</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.PlaylistTreeNode.html\" title=\"struct rekordcrate::pdb::PlaylistTreeNode\">PlaylistTreeNode</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.PlaylistTreeNodeId.html\" title=\"struct rekordcrate::pdb::PlaylistTreeNodeId\">PlaylistTreeNodeId</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.RowGroup.html\" title=\"struct rekordcrate::pdb::RowGroup\">RowGroup</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Table.html\" title=\"struct rekordcrate::pdb::Table\">Table</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.Track.html\" title=\"struct rekordcrate::pdb::Track\">Track</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/pdb/struct.TrackId.html\" title=\"struct rekordcrate::pdb::TrackId\">TrackId</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/setting/struct.DJMMySetting.html\" title=\"struct rekordcrate::setting::DJMMySetting\">DJMMySetting</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/setting/struct.DevSetting.html\" title=\"struct rekordcrate::setting::DevSetting\">DevSetting</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/setting/struct.MySetting.html\" title=\"struct rekordcrate::setting::MySetting\">MySetting</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/setting/struct.MySetting2.html\" title=\"struct rekordcrate::setting::MySetting2\">MySetting2</a>"],["impl BinRead for <a class=\"struct\" href=\"rekordcrate/setting/struct.Setting.html\" title=\"struct rekordcrate::setting::Setting\">Setting</a>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[17625]}