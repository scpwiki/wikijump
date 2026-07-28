# Embedding code from other sites syntax

- Feature ID: `syntax-embedding-code`
- Category: `wiki-syntax`
- Documentation status: `documented`
- Specification source: frozen local Wikidot documentation corpus
- Behavioral authority: documentation-derived; live Wikidot wins if tested behavior conflicts

## Purpose

Parse and render Wikidot's documented embedding code from other sites syntax, including every documented form, option, output rule, and limitation.

## Implementation contract

- The parser MUST recognize every documented spelling and structural form in the evidence below.
- The renderer MUST produce the described visible text, HTML structure, links, and context-sensitive behavior.
- Whitespace, escaping, nesting, and malformed-input behavior MUST follow explicit documentation; unspecified cases require oracle evidence before widening acceptance.

Every explicit default, accepted value, rejected value, alias, limit, interaction, output form, URL form, permission rule, and stated limitation in the evidence below is part of this specification. Examples are conformance fixtures. Text that merely describes the documentation site or presents a live demo is informative rather than normative.

If the documentation is silent or contradictory, the implementation MUST fail closed or preserve the existing literal behavior until a live Wikidot experiment supplies a stable expectation. The spec and catalog must then be updated with that evidence.


## Suggested public TDD seams

These seams are recommendations. The implementation agent must present and confirm the actual seam map before writing tests.

- FTML public parse/render interface using Wikidot layout
- Rendered HTML/DOM at the saved-page boundary for context-dependent forms

## Feature-specific implementation notes

- No feature-specific implementation note beyond the corpus contract.

## Source inventory

- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:embedding-code/source.wikidot.txt:1` through line 23 (canonical)
- `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:embedding/source.wikidot.txt:1` through line 131 (supporting)

## Documentation-derived behavioral evidence

### doc-wiki-syntax:embedding-code (canonical)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc-wiki-syntax:embedding-code/source.wikidot.txt:1` through line 23  
SHA-256 of complete source file: `bd017698ec8bfcd48c09d1f336422444f65376ce69d9785173e5ce6c3b050171`

```wikidot
L0001 Sometimes web sites (mainly social-oriented ones) allow you to paste a code block directly into other pages in order to increase your site functionality or embed some content from the original site.
L0002 
L0003 ++ {{@@[[embed]]@@}} tag
L0004 
L0005 The {{@@[[embed]]@@}} block tag allows you to do the same with your wiki pages. E.g. to display del.icio.us tag cloud as generated from http://del.icio.us/help/tagrolls simply wrap the html code:
L0006 
L0007 [[code]]
L0008 [[embed]]
L0009 <script type="text/javascript" src="http://del.icio.us/feeds/js/tags/michal_frackowiak?icon;size=12-35;color=87ceeb-0000ff;title=my%20del.icio.us%20tags"></script>
L0010 [[/embed]]
L0011 [[/code]]
L0012 
L0013 For the list of supported services please see the page: [[[doc:embedding | Embedding code from other services]]].
L0014 
L0015 Please note that if the code contains {{<script type="text/javascript"...}} i.e. just JavaScript, the content will not be fully rendered when you click {{preview}} while editing the page. It should be there however when you view the page afterwards.
L0016 
L0017 ++ [[# iframe]]{{@@[[iframe]]@@}} element
L0018 
L0019 Using the //iframe// element one can embed content of any other web page. The syntax is
L0020 [[code]]
L0021 [[iframe url-source attributes]]
L0022 [[/code]]
L0023 and it translates into HTML tags {{<iframe src="url-source" attributes></iframe>}}. The allowed attributes are: frameborder (0 or 1 allowed), align (left, right, top, bottom, middle), height (number of pixels or %), width (number of pixels or %), scrolling (yes or no), class, style
```

### doc:embedding (supporting)

Source: `~/src/Rokurolize/scp-wiki-translation/corpus/www/pages/doc:embedding/source.wikidot.txt:1` through line 131  
SHA-256 of complete source file: `5c404ef01f56fa48c89f03d65ae87d69c58b3f02430aafb67e3a8aa4eb9ed03f`

```wikidot
L0001 You can safely embed any external HTML code on your pages. To guarantee maximum safety, HTML code is placed in a safe environment (iframe sandbox) to limit the possibility of cross-site scripting attacks.
L0002 
L0003 E.g. to display del.icio.us tag cloud as generated from http://del.icio.us/help/tagrolls simply wrap the html code in a [[html]]...[[/html]] tags:
L0004 
L0005 [[code]]
L0006 [[html]]
L0007 <script type="text/javascript" src="http://del.icio.us/feeds/js/tags/michal_frackowiak?icon;size=12-35;color=87ceeb-0000ff;title=my%20del.icio.us%20tags"></script>
L0008 [[/html]]
L0009 [[/code]]
L0010 
L0011 Note: previously [[html]] tag functionality was partly covered by the [[embed]] tag, which is now deprecated and aliased to [[html]]. This is why you might still find [[embed]] tag here and there.
L0012 
L0013 + Examples of supported media using [[html]] tags:
L0014 [[div style="float:left;"]]
L0015 [[div style="-moz-border-radius-bottomleft:5px; -moz-border-radius-bottomright:5px; -moz-border-radius-topleft:5px; -moz-border-radius-topright:5px; border:1px solid #CCCCCC; margin-right:10px;
L0016 padding-left:10px; padding-right:10px; padding-top:0px; width:310px; padding-bottom: 10px;"]]
L0017 ++ Video & Audio
L0018 
L0019 [[image http://youtube.com/favicon.ico height="16px" width="16px"]] [*http://www.youtube.com YouTube video]
L0020 [[image http://www.google.com/favicon.ico height="16px" width="16px"]] [*http://www.video.google.com Google Video]
L0021 [[image http://vimeo.com/favicon.ico height="16px" width="16px"]] [*http://www.vimeo.com/ Vimeo] videos (HD) - [http://snippets.wikidot.com/code:vimeo more»]
L0022 [[image http://www.dailymotion.pl/favicon.ico height="16px" width="16px"]] [*http://dailymotion.com DailyMotion] videos -  [http://snippets.wikidot.com/code:dailymotion more»]
L0023 [[image http://www.gametrailers.com/favicon.ico height="16px" width="16px"]] [*http://www.gametrailers.com GameTrailers Video (HD)] - [http://snippets.wikidot.com/code:gametrailers more»].
L0024 [[image http://animoto.com/favicon.ico height="16px" width="16px"]] [*http://www.animoto.com Animoto]
L0025 [[image http://flickr.com/favicon.ico height="16px" width="16px"]] [*http://flickr.com Flickr] videos - [http://snippets.wikidot.com/code:flickr-video more»]
L0026 [[image http://www.teachertube.com/favicon.ico height="16px" width="16px"]] [*http://www.teachertube.com TeacherTube] videos - [http://snippets.wikidot.com/code:teachertube more»]
L0027 [[image http://www.schooltube.com/favicon.ico height="16px" width="16px"]] [*http://www.shooltube.com SchoolTube] videos -  [http://snippets.wikidot.com/code:schooltube more»]
L0028 [[image http://blip.tv/favicon.ico height="16px" width="16px"]] [*http://blip.tv Blip.tv] videos (HD) - [http://snippets.wikidot.com/code:bliptv more»]
L0029 [[image http://www.playlist.com/favicon.ico height="16px" width="16px"]] [*http://www.playlist.com Playlist.com] music player - [http://snippets.wikidot.com/code:playlist more»]
L0030 [[image http://finetune.com/favicon.ico height="16px" width="16px"]] [*http://www.finetune.com/ FineTune player] 
L0031 
L0032 [[/div]]
L0033 [[div style="-moz-border-radius-bottomleft:5px; -moz-border-radius-bottomright:5px; -moz-border-radius-topleft:5px; -moz-border-radius-topright:5px; border:1px solid #CCCCCC; margin-right:10px;
L0034 padding-left:10px; padding-right:10px; padding-top:0px; width:310px; padding-bottom: 10px; margin-top: 10px;"]]
L0035 
L0036 ++ Images
L0037 [[image http://photobucket.com/favicon.ico height="16px" width="16px"]] [*http://www.photobucket.com Photobucket] photo widgets - [http://snippets.wikidot.com/code:photobucket-widget more»]
L0038 [[image http://picasaweb.google.com/favicon.ico height="16px" width="16px"]] [*http://picasaweb.google.pl/home Picasa] web albums - [http://snippets.wikidot.com/code:picasaweb more»]
L0039 
L0040 [[/div]]
L0041 
L0042 [[div style="-moz-border-radius-bottomleft:5px; -moz-border-radius-bottomright:5px; -moz-border-radius-topleft:5px; -moz-border-radius-topright:5px; border:1px solid #CCCCCC; margin-right:10px;
L0043 padding-left:10px; padding-right:10px; padding-top:0px; width:310px; margin-top: 10px; padding-bottom: 10px;"]]
L0044 ++ Office Tools
L0045 
L0046 [[image http://writer.zoho.com/images/favicon.ico height="16px" width="16px"]] [*http://zohopolls.com/ Zoho Polls] - [*http://snippets.wikidot.com/code:zohopolls more»]
L0047 [[image http://writer.zoho.com/images/favicon.ico height="16px" width="16px"]] [*http://zohowriter.com Zoho Writer] - [http://snippets.wikidot.com/code:zohowriter more»]
L0048 [[image http://writer.zoho.com/images/favicon.ico height="16px" width="16px"]] [*http://zohoshow.com Zoho Show] - [http://snippets.wikidot.com/code:zohoshow more»]
L0049 [[image http://writer.zoho.com/images/favicon.ico height="16px" width="16px"]] [*http://www.zohosheet.com/ Zoho Sheet] - [http://snippets.wikidot.com/code:zohosheet more»]
L0050 [[image http://www.editgrid.com/favicon.ico height="16px" width="16px"]] [*http://www.editgrid.com EditGrid] - [http://snippets.wikidot.com/code:editgrid more»]
L0051 [[image http://instacalc.com/favicon.ico height="16px" width="16px"]] [*http://instacalc.com/ Instacalc] - [http://snippets.wikidot.com/code:instacalc more»]
L0052 [[image http://quimble.com/favicon.ico height="16px" width="16px"]] [*http://quimble.com Quimble] polls - [*http://snippets.wikidot.com/code:quimble-poll more»]
L0053 
L0054 [[/div]]
L0055 [[div style="-moz-border-radius-bottomleft:5px; -moz-border-radius-bottomright:5px; -moz-border-radius-topleft:5px; -moz-border-radius-topright:5px; border:1px solid #CCCCCC; margin-right:10px;
L0056 padding-left:10px; padding-right:10px; padding-top:0px; width:310px; margin-top: 10px; padding-bottom: 10px;"]]
L0057 
L0058 ++ Slideshows & Presentations
L0059 
L0060 [[image http://voicethread.com/favicon.ico height="16px" width="16px"]] [*http://voicethread.com/ Voicethread] slideshows - [http://snippets.wikidot.com/code:voicethread more»]
L0061 [[image http://www.slideboom.com/images/favicon.ico height="16px" width="16px"]] [*http://www.slideboom.com/ SlideBoom] slideshows and presentations - [http://snippets.wikidot.com/code:slideboom more »]
L0062 
L0063 [[/div]]
L0064 [[/div]]
L0065 
L0066 [[div style="float:left;"]]
L0067 
L0068 [[div style="-moz-border-radius-bottomleft:5px; -moz-border-radius-bottomright:5px; -moz-border-radius-topleft:5px; -moz-border-radius-topright:5px; border:1px solid #CCCCCC; margin-right:10px;
L0069 padding-left:10px; padding-right:10px; padding-top:0px; width:310px; padding-bottom: 10px;"]]
L0070 ++ Maps
L0071 
L0072 [[image http://www.google.com/favicon.ico height="16px" width="16px"]] [*http://maps.google.com Google Maps] - [http://snippets.wikidot.com/code:google-maps more»]
L0073 [[image http://wikimapia.org/favicon.ico height="16px" width="16px"]] [*http://wikimapia.org Wikimapia] 
L0074 [[image http://quikmaps.com/favicon.ico height="16px" width="16px"]] [*http://quikmaps.com Quikmaps maps] - [*http://snippets.wikidot.com/code:quikmaps-maps more»]
L0075 [[image http://www.everytrail.com/favicon.ico width="16" height="16"]] [*http://everytrail.com/ EveryTrail] 
L0076 [[image http://motionbased.com/favicon.ico height="16px" width="16px"]] [*http://motionbased.com MotionBased]
L0077 [[/div]]
L0078 
L0079 [[div style="-moz-border-radius-bottomleft:5px; -moz-border-radius-bottomright:5px; -moz-border-radius-topleft:5px; -moz-border-radius-topright:5px; border:1px solid #CCCCCC; margin-right:10px;
L0080 padding-left:10px; padding-right:10px; padding-top:0px; width:310px; padding-bottom: 10px; margin-top: 10px;"]]
L0081 ++ Social services
L0082 
L0083 [[image http://disqus.com/favicon.ico height="16px" width="16px"]] **NEW!** [*http://disqus.com Disqus]
L0084 [[image http://tweetmeme.com//images/favicon.ico height="16px" width="16px"]] [*http://tweetmeme.com/ TweetMeme]
L0085 [[image http://addthis.com/favicon.ico height="16px" width="16px"]] [*http://addthis.com AddThis]
L0086 [[image http://delicious.com/favicon.ico height="16px" width="16px"]] [*http://del.icio.us/help/tagrolls del.icio.us tagrolls]
L0087 [[image http://delicious.com/favicon.ico height="16px" width="16px"]] [*http://del.icio.us/help/linkrolls del.icio.us linkrolls]
L0088 [[image http://delicious.com/favicon.ico height="16px" width="16px"]] [*http://del.icio.us/help/tagometer del.icio.us tagometer]
L0089 [[image http://delicious.com/favicon.ico height="16px" width="16px"]] [*http://del.icio.us/help/forpublishers del.icio.us "Save this page"] - [*http://snippets.wikidot.com/code:social-bookmarking more»]
L0090 [[image http://digg.com/favicon.ico height="16px" width="16px"]] [*http://www.digg.com/add-digg Digg news] - [*http://snippets.wikidot.com/code:import-the-digg-feed more»]
L0091 [[image http://www.spreadfirefox.com/files/spreadfirefox_RCS_favicon.png height="16px" width="16px"]] [*http://www.spreadfirefox.com/?q=affiliates/homepage spreadfirefox.com] affiliate buttons
L0092 [[image http://www.ohloh.net/favicon.ico height="16px" width="16px"]] [*http://ohloh.net Ohloh] snippets -  [http://snippets.wikidot.com/code:ohloh more»]
L0093 [[image http://www.meebo.com/favicon.ico height="16px" width="16px"]] [*http://www.meebome.com Meebo Me] IM chat window - [http://snippets.wikidot.com/code:meebome more»]
L0094 [[image http://home.gabbly.com/images/favicon.ico height="16px" width="16px"]] [*http://gabbly.com/ Gabbly] multi-user chat - [http://snippets.wikidot.com/code:gabbly more»]
L0095 [[image http://www.wowdb.com/favicon.png height="16px" width="16px"]] [*http://www.wowdb.com/ WOWDB], [*http://www.wowhead.com/ Wowhead], [*http://thottbot.com/ Thottbot] tooltips - [http://snippets.wikidot.com/code:wow-tooltips more»]
L0096 [[image http://www.mybloglog.com/favicon.ico height="16px" width="16px"]] [*http://mybloglog.com MyBlogLog] widget - [http://snippets.wikidot.com/code:mybloglog more»]
L0097 
L0098 [[/div]]
L0099 
L0100 [[div style="-moz-border-radius-bottomleft:5px; -moz-border-radius-bottomright:5px; -moz-border-radius-topleft:5px; -moz-border-radius-topright:5px; border:1px solid #CCCCCC; margin-right:10px;
L0101 padding-left:10px; padding-right:10px; padding-top:0px; width:310px; margin-top: 10px; padding-bottom: 10px;"]]
L0102 ++ Widgets
L0103 
L0104 [[image http://www.google.com/favicon.ico height="16px" width="16px"]] **[*http://www.google.com/ig/directory?synd=open Google Gadgets]** - [http://snippets.wikidot.com/code:google-gadgets more»]
L0105 [[image http://www.google.com/favicon.ico height="16px" width="16px"]] [*http://www.google.com/calendar Google Calendar] - [*http://www.google.com/support/calendar/bin/answer.py?answer=41207 more»]
L0106 [[image http://www.widgetbox.com/favicon.ico height="16px" width="16px"]] [*http://www.widgetbox.com Widgetbox] Widgets - [http://snippets.wikidot.com/code:widgetbox-panel more»]
L0107 [[image http://js-kit.com/favicon.ico height="16px" width="16px"]] [*http://js-kit.com/ratings/ JS-Kit ratings] - [http://snippets.wikidot.com/code:js-kit-ratings more»]
L0108 [[image http://www.labpixies.com/favicon.ico height="16px" width="16px"]] [*http://www.labpixies.com/  Labpixies] gadgets - [http://snippets.wikidot.com/code:pabpixies-gadgets more»]
L0109 
L0110 [[/div]]
L0111 
L0112 
L0113 [[div style="-moz-border-radius-bottomleft:5px; -moz-border-radius-bottomright:5px; -moz-border-radius-topleft:5px; -moz-border-radius-topright:5px; border:1px solid #CCCCCC; margin-right:10px;
L0114 padding-left:10px; padding-right:10px; padding-top:0px; padding-bottom: 10px; width:310px; margin-top: 10px;"]]
L0115 ++ Web Tools
L0116 
L0117 [[image http://www.statcounter.com/favicon.ico height="16px" width="16px"]] [*http://www.statcounter.com/ StatCounter] tracking code - [*http://community.wikidot.com/howto:site-statistics more»]
L0118 [[image http://www.alexa.com/favicon.ico height="16px" width="16px"]] [*http://www.alexa.com Alexa.com] traffic ratings - [http://snippets.wikidot.com/code:alexa-traffic-ratings more»]
L0119 [[image http://www.feedburner.com/fb/images/favicon.ico height="16px" width="16px"]] [*http://www.feedburner.com Feedburner] - [http://snippets.wikidot.com/code:feedburner more»]
L0120 [[image http://feedblitz.com/favicon.ico height="16px" width="16px"]] [*http://www.feedblitz.com/ FeedBlitz]
L0121 [[image http://babelfish.yahoo.com/favicon.ico height="16px" width="16px"]] [*http://babelfish.yahoo.com/free_trans_service Babelfish translation] - [*http://snippets.wikidot.com/code:babelfish-translation more»]
L0122 [[image http://skype.com/favicon.ico height="16px" width="16px"]] [*http://www.skype.com/share/buttons/index.html Skype] - "call me" buttons
L0123 [[image http://www.brainyquote.com/favicon.ico width="16" height="16"]] [*http://www.brainyquote.com/link/index.html Brainy Quote] - quote of the day
L0124 [[image http://cornify.com/favicon.ico width="16" height="16"]][*http://www.cornify.com Cornify] - don't even ask :) [http://snippets.wikidot.com/code:cornify more»]
L0125 
L0126 [[/div]]
L0127 [[/div]]
L0128 ~~~~
L0129 
L0130 
L0131 If you want a new service enabled, please write to support@wikidot.com or put your suggestion in our [http://community.wikidot.com/forum/c-11/new-features-and-ideas Community Forum].
```
