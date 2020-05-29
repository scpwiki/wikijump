<?php
/**
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 * 
 * @category Wikidot
 * @package Wikidot
 * @version $Id$
 * @copyright Copyright (c) 2008, Wikidot Inc.
 * @license http://www.gnu.org/licenses/agpl-3.0.html GNU Affero General Public License
 */

class SocialBookmarksModule extends SmartyModule {
	
	private $sites = Array(

	'blinkbits' => Array(
		'name' => 'blinkbits',
		'favicon' => 'blinkbits.png',
		'url' => 'http://www.blinkbits.com/bookmarklets/save.php?v=1&amp;source_url=PERMALINK&amp;title=TITLE&amp;body=TITLE',
	),

	'blinklist' => Array(
		'name' => 'BlinkList',
		'favicon' => 'blinklist.png',
		'url' => 'http://www.blinklist.com/index.php?Action=Blink/addblink.php&amp;Description=&amp;Url=PERMALINK&amp;Title=TITLE',
	),

	'blogmarks' => Array(
		'name' => 'blogmarks',
		'favicon' => 'blogmarks.png',
		'url' => 'http://blogmarks.net/my/new.php?mini=1&amp;simple=1&amp;url=PERMALINK&amp;title=TITLE',
	),

	'co.mments' => Array(
		'name' => 'co.mments',
		'favicon' => 'co.mments.gif',
		'url' => 'http://co.mments.com/track?url=PERMALINK&amp;title=TITLE',
	),

	'connotea' => Array(
		'name' => 'connotea',
		'favicon' => 'connotea.png',
		'url' => 'http://www.connotea.org/addpopup?continue=confirm&amp;uri=PERMALINK&amp;title=TITLE',
	),

	'del.icio.us' => Array(
		'name' => 'del.icio.us',
		'favicon' => 'delicious.png',
		'url' => 'http://del.icio.us/post?url=PERMALINK&amp;title=TITLE',
		'onclick' => "window.open('http://del.icio.us/post?v=4&noui&jump=close&url='+encodeURIComponent(location.href)+'&title='+encodeURIComponent(document.title), 'delicious','toolbar=no,width=700,height=400'); return false;"

	),

	'de.lirio.us' => Array(
		'name' => 'De.lirio.us',
		'favicon' => 'delirious.png',
		'url' => 'http://de.lirio.us/rubric/post?uri=PERMALINK;title=TITLE;when_done=go_back',
	),

	'digg' => Array(
		'name' => 'digg',
		'favicon' => 'digg.png',
		'url' => 'http://digg.com/submit?phase=2&amp;url=PERMALINK&amp;title=TITLE',
	),

	'fark' => Array(
		'name' => 'Fark',
		'favicon' => 'fark.png',
		'url' => 'http://cgi.fark.com/cgi/fark/edit.pl?new_url=PERMALINK&amp;new_comment=TITLE&amp;new_comment=BLOGNAME&amp;linktype=Misc',
		// To post to a different category, see the drop-down box labeled "Link Type" at
		// http://cgi.fark.com/cgi/fark/submit.pl for a complete list
	),

	'feedmelinks' => Array(
		'name' => 'feedmelinks',
		'favicon' => 'feedmelinks.png',
		'url' => 'http://feedmelinks.com/categorize?from=toolbar&amp;op=submit&amp;url=PERMALINK&amp;name=TITLE',
	),

	'furl' => Array(
		'name' => 'Furl',
		'favicon' => 'furl.png',
		'url' => 'http://www.furl.net/storeIt.jsp?u=PERMALINK&amp;t=TITLE',
	),

	'linkagogo' => Array(
		'name' => 'LinkaGoGo',
		'favicon' => 'linkagogo.png',
		'url' => 'http://www.linkagogo.com/go/AddNoPopup?url=PERMALINK&amp;title=TITLE',
	),

	'ma.gnolia' => Array(
		'name' => 'Ma.gnolia',
		'favicon' => 'magnolia.png',
		'url' => 'http://ma.gnolia.com/beta/bookmarklet/add?url=PERMALINK&amp;title=TITLE&amp;description=TITLE',
	),

	'newsvine' => Array(
		'name' => 'NewsVine',
		'favicon' => 'newsvine.png',
		'url' => 'http://www.newsvine.com/_tools/seed&amp;save?u=PERMALINK&amp;h=TITLE',
	),

	'netvouz' => Array(
		'name' => 'Netvouz',
		'favicon' => 'netvouz.png',
		'url' => 'http://www.netvouz.com/action/submitBookmark?url=PERMALINK&amp;title=TITLE&amp;description=TITLE',
	),

	'rawsugar' => Array(
		'name' => 'RawSugar',
		'favicon' => 'rawsugar.png',
		'url' => 'http://www.rawsugar.com/tagger/?turl=PERMALINK&amp;tttl=TITTLE',
	),

	'reddit' => Array(
		'name' => 'Reddit',
		'favicon' => 'reddit.png',
		'url' => 'http://reddit.com/submit?url=PERMALINK&amp;title=TITLE',
	),

	'scuttle' => Array(
		'name' => 'scuttle',
		'favicon' => 'scuttle.png',
		'url' => 'http://www.scuttle.org/bookmarks.php/maxpower?action=add&amp;address=PERMALINK&amp;title=TITLE&amp;description=TITLE',
	),

	'shadows' => Array(
		'name' => 'Shadows',
		'favicon' => 'shadows.png',
		'url' => 'http://www.shadows.com/features/tcr.htm?url=PERMALINK&amp;title=TITLE',
	),

	'simpy' => Array(
		'name' => 'Simpy',
		'favicon' => 'simpy.png',
		'url' => 'http://www.simpy.com/simpy/LinkAdd.do?href=PERMALINK&amp;title=TITLE',
	),

	'smarking' => Array(
		'name' => 'Smarking',
		'favicon' => 'smarking.png',
		'url' => 'http://smarking.com/editbookmark/?url=PERMALINK&amp;description=TITLE',
	),

	'spurl' => Array(
		'name' => 'Spurl',
		'favicon' => 'spurl.png',
		'url' => 'http://www.spurl.net/spurl.php?url=PERMALINK&amp;title=TITLE',
	),

	'tailrank' => Array(
		'name' => 'TailRank',
		'favicon' => 'tailrank.png',
		'url' => 'http://tailrank.com/share/?text=&amp;link_href=PERMALINK&amp;title=TITLE',
	),

	'wists' => Array(
		'name' => 'Wists',
		'favicon' => 'wists.png',
		'url' => 'http://wists.com/r.php?c=&amp;r=PERMALINK&amp;title=TITLE',
	),

	'yahoomyweb' => Array(
		'name' => 'YahooMyWeb',
		'favicon' => 'yahoomyweb.png',
		'url' => 'http://myweb2.search.yahoo.com/myresults/bookmarklet?u=PERMALINK&amp;=TITLE',
	),
	'facebook' => Array(
		'name' => 'Facebook',
		'url' => 'http://www.facebook.com/share.php?u=PERMALINK',
		'favicon' => 'facebook.gif',
		'onclick' => "window.open('http://www.facebook.com/sharer.php?u='+encodeURIComponent(location.href)+'&t='+encodeURIComponent(document.title),'sharer','toolbar=0,status=0,width=626,height=436');return false;"
	)
	
	);
	
	public function render($runData){
		
		$sites = $runData->getParameterList()->getParameterValue("sites");
    	 	
    	 	// split sites by ", "
    	 	$ss = preg_split('/,\s*/', $sites);
    	 	
    	 	$out = '';
    	 	$imageUrl = "/common--images/social/";
    	 	if($sites && $sites != ""){
    	 		foreach($ss as $siteName){
	    	 		if($this->sites[strtolower($siteName)]){
    	 				$ss2[] = $this->sites[$siteName];
    	 			}
    	 		}
    	 	}else{
    	 		$ss2 = $this->sites;
    	 	}
    	 	
    	 	// render
    	 	$wikiSite = $runData->getTemp("site");
    	 	$page = $runData->getTemp("page");
    	 	$pageTitle = $wikiSite->getName();
    	 	if($page){
    	 		if($page->getTitle() != '') {$pageTitle .= ': '.$page->getTitle();}
    	 	 	$pageName = $page->getUnixName();
    	 	}

    	 	$permalink = 'http://'.$_SERVER['HTTP_HOST'].$_SERVER['REQUEST_URI'];
    	 	
    	 	$out = '';
    	 	$id = 'social'.rand(1,100000);
    	 	$out .= '<span id="'.$id.'">';
    	 	foreach($ss2 as $key => $site){
    	 		$url = $site['url'];
    	 		$url = str_replace('PERMALINK', urlencode($permalink), $url);
    	 		$url = str_replace('BLOGNAME', urlencode($wikiSite->getName()), $url);
    	 		$out .= '<a href="'.$url.'" style="margin: 0 2px" title="'.$site['name'].'" '.($site['onclick']?' onclick="'.$site['onclick'].'"':'');
    	 		if(isset($site['onclick'])){
    	 			$out .= ' onclick="'.$site['onclick'].'" ';	
    	 		}
    	 		$out .= '>';
    	 		$out .= '<img src="'.$imageUrl.$site['favicon'].'" alt="'.$site['name'].'"/></a>';
    	 		
    	 	}
    	 	$out .= '</span>';
    	 	// ok, now fix the d.... title
    	 	$out .= '<script type="text/javascript">
			var socialspan = $("'.$id.'");
			var els = socialspan.getElementsByTagName("a");	
			for(var i=0;i<els.length;i++){
				els[i].href = els[i].href.replace("TITLE", encodeURIComponent(document.title));
			}
			</script>'		
				;
				
			// purify???
			$wt = new WikiTransformation();
			$out = $wt->purifyHtml($out);
    	 	return $out;
    	 	
	}
	
	public function build($r){}
}
