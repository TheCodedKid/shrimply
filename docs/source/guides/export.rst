Export
======

Export types
------------

The export window can produce rendered video, captions, or JSON project data.

Captions and plain text
-----------------------

AppKit and GTK offer **Export Captions** with YouTube YTT, Advanced SubStation
Alpha (ASS), SubRip (SRT), WebVTT (VTT), and plain text (TXT). Choose a format,
then merge enabled caption tracks into one file or export each track separately.
YTT and merging are the defaults. Qt retains its existing YTT export interface.

Captions are ordered by start time across the full timeline; disabled tracks and
blank captions are excluded. Separate files use ``name-track-N.extension``.
Existing output files require replacement confirmation. An empty export reports
an error instead of creating empty files.

TXT contains only caption text, with a blank line between captions. It has no
timestamps, cue numbers, or formatting markup. Existing line breaks are preserved;
ruby annotations are omitted while their base text is retained.

YTT preserves the existing YouTube styling, placement, ruby, and timed spans.
ASS preserves emphasis, fonts, size, colors, opacity, placement, rotation, and
timed text reveal using successive events. Edge effects are approximated with
outlines and shadows; a background box takes precedence over an outline. ASS uses the project canvas
dimensions and a 32-pixel base font scaled by each caption's font scale.

VTT preserves emphasis, ruby, in-cue timestamps, supported CSS styling, and
horizontal or vertical placement. SRT preserves emphasis and readable text;
player support for emphasis varies. SRT omits layout and timed-span effects.
In ASS and SRT, ruby is written as ``base (annotation)``. Unsupported vertical
typography in ASS and rotated typography in VTT use horizontal text. Font
availability and subtitle player capabilities affect final appearance.

SRT and VTT use millisecond timestamps, while ASS uses centiseconds. Conversion
uses rational arithmetic and truncates to the format's precision; positive cues
remain at least one tick long. Overlapping captions remain separate cues.

Video and GIF
-------------

Current video choices are H.264, H.265, and GIF. H.264 and H.265 require NVENC;
no software encoder fallback is available. Available containers are MP4,
Matroska, and GIF.

Audio encoder choices include AAC, FDK AAC, and Opus. The compatible choices
depend on the selected container.

Before exporting, confirm the project canvas, frame rate, range, container,
video codec, and audio codec. The native Shrimply compositor renders exports
without recording the preview surface.
