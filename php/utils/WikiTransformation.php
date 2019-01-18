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

require_once(WIKIDOT_ROOT."/lib/Text_Wiki/Text/Wiki.php");
// just for text_wiki extend the include_path
ini_set('include_path',ini_get('include_path').':'.WIKIDOT_ROOT.'/lib/Text_Wiki/');
//for SmartyPants

class WikiTransformation {
	/**
     * The array stores all internal links within a page. 
     * Each element is an array (page_id, page_unix_name)
     */
    public static $internalLinksExist;
    public static $externalLinks;
    public static $internalLinksNotExist;
    public static $inclusions;
    
    private $page; 
    private $pageUnixName;
    
    private $transformationFormat = 'xhtml'; 
    
    public $wiki;
    
    private $_tmpPage;

	public function __construct($initialize = true){
		if($initialize){
			self::$internalLinksExist = array();
			self::$internalLinksNotExist = array();
			self::$externalLinks = array();
			$this->resetWiki();
		}
	}
	
	public static function purifyHtml($code){
		//$code is not a complete page so we need to wrap it!
		$head = '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"><html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">  <head>    <title>Just A Wrapper</title><meta http-equiv="content-type" content="text/html;charset=UTF-8"/>  </head> <!--wrapdelimiter--><body>';
  		$tail=' </body><!--wrapdelimiter--></html>';
		
		$c = $head.$code.$tail;
		$config = array('indent' => false,
               'output-xhtml' => TRUE,
               'wrap' => 0);

		$c2 = tidy_parse_string($c, $config, 'UTF8');
			
		$arr = explode("<!--wrapdelimiter-->", $c2);
		$out=$arr[1];
		$out = str_replace("<body>", "", $out);
		$out = str_replace("</body>", "", $out);
		return $out;
	}
	
	public function processSource($source){
		
		$wiki = $this->wiki;
	    if(preg_match(';_template$;', $wiki->vars['pageName'])){
			$wiki->disablerule('separator');
		}
		$out = $wiki->transform($source, $this->transformationFormat);
	
		$out = $this->purifyHtml($out);
		
		return $out;	
	}
	
	public function assemblyTemplate($source, $template, $page = null){
	    /* First check if it is a real "live" template. If not, return the original $source. 
		 * To be recognized as a live template it mast contain either %%content%% or
		 * %%content{X}%% tags. */
	    
	    /* Handle ListPages module inside a template -- %%content%% need to be escaped. */
	    $template = preg_replace_callback(";^\\[\\[module\\s+ListPages(.*?)\n\\[\\[/module\\]\\];ms", array($this, '_assemblyTemplateHandleListPages'), $template);
	    $template = preg_replace_callback(";^\\[\\[module\\s+NextPage(.*?)\n\\[\\[/module\\]\\];ms", array($this, '_assemblyTemplateHandleListPages'), $template);
	    $template = preg_replace_callback(";^\\[\\[module\\s+PreviousPage(.*?)\n\\[\\[/module\\]\\];ms", array($this, '_assemblyTemplateHandleListPages'), $template);
	    $template = preg_replace_callback(";^\\[\\[module\\s+Feed(.*?)\n\\[\\[/module\\]\\];ms", array($this, '_assemblyTemplateHandleListPages'), $template);
	    $template = preg_replace_callback(";^\\[\\[module\\s+FrontForum(.*?)\n\\[\\[/module\\]\\];ms", array($this, '_assemblyTemplateHandleListPages'), $template);
	    
	    if(!preg_match(';%%content({[0-9]+})?%%;', $template)) {
	        return $source;
	    }
	    $out = $source;

	    $template = preg_replace(';%%content({[0-9]+})?%%;', '%%%\\0%%%', $template);
	    $template = preg_replace(';(?<!%)%%[a-z0-9\(\)_]+%%(?!%);i', '%%%\\0%%%', $template);
	    $template = preg_replace(';(?<!%)%%date(\|.*?)?%%(?!%);i', '%%%\\0%%%', $template);
	    
	    $template = preg_replace(";%\xFA%(content({[0-9]+}))?%\xFA%;", "%%\\1%%", $template);
	    $template = preg_replace(";%\xFA%([a-z0-9\(\)_]+)%\xFA%;i", '%%\\1%%', $template);
	    $template = preg_replace(";%\xFA%(date(\|.*?)?)%\xFA%;i", '%%\\1%%', $template);
	    
		/* Check if has a ===== delimiter. */
	    $split = preg_split(';^={4,}$;sm', $template);
	    if(count($split) > 1){
	        $template = trim($split[0]);
	    }
	    
	    /* If there is $page, try substituting more tags. */
	    if($page) {
	    	$this->_tmpPage = $page;
	    	$b = $template;
	    	$title = $page->getTitle();
	    	$title = str_replace(array('[',']'), '', $title);
            //$title = str_replace('%%', "\xFD", $title);
	    	$b = str_replace('%%%%%title%%%%%', $title, $b);
            $b = preg_replace(";%%%%%((linked_title)|(title_linked))%%%%%;i", preg_quote_replacement('[[[' . $page->getUnixName() . ' | ' . $title . ']]]'), $b);
            
            
            if($page->getOwnerUserId()){
	            $user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($page->getOwnerUserId());
	            if ($user->getUserId() > 0) {
	                $userString = '[[*user ' . $user->getNickName() . ']]';
	            } else {
	                $userString = _('Anonymous user');
	            }
            } else {
                $userString = _('Anonymous user');
            }
            $b = str_ireplace("%%%%%author%%%%%", $userString, $b);
            $b = str_ireplace("%%%%%user%%%%%", $userString, $b);
            
//            if($lastRevision->getUserId()){
//	            $user = DB_OzoneUserPeer::instance()->selectByPrimaryKey($lastRevision->getUserId());
//	            if ($user->getUserId() > 0) {
//	                $userString = '[[*user ' . $user->getNickName() . ']]';
//	            } else {
//	                $userString = _('Anonymous user');
//	            }
//            } else {
//                $userString = _('Anonymous user');
//            }
            //$b = str_ireplace("%%author_edited%%", $userString, $b);
            $b = str_ireplace("%%%%%user_edited%%%%%", $userString, $b);
            
            $b = preg_replace(';%%%%%date(\|.*?)?%%%%%;', '%%%%%date|' . $page->getDateCreated()->getTimestamp() . '\\1%%%%%', $b);
            $b = preg_replace(';%%%%%date_edited(\|.*?)?%%%%%;', '%%%%%date|' . $page->getDateLastEdited()->getTimestamp() . '\\1%%%%%', $b);
            
            /* %%rating%% */
            $b = str_ireplace('%%%%%rating%%%%%', $page->getRate(), $b);
                
             /* %%comments%% */
            $b = preg_replace_callback("/%%%%%comments%%%%%/i", array(
                $this, '_handleComementsCount'), $b);
                
            /* %%page_unix_name%% */
            $b = str_ireplace('%%%%%page_unix_name%%%%%', $page->getUnixName(), $b);
            
            if(strpos( $page->getUnixName(), ":") != false){
				$tmp0 = explode(':',$page->getUnixName()); 
				$categoryName00 = $tmp0[0];
			} else {
				$categoryName00 = "_default";
			}
            
			$b = str_ireplace('%%%%%category%%%%%', $categoryName00, $b);
			
            /* %%link%% */
			$site = $page->getSite();
            $b = str_ireplace('%%%%%link%%%%%', 'http://' . $site->getDomain().'/'.$page->getUnixName(), $b);
            
            /* %%tags%% */
            $b = preg_replace_callback("/%%%%%tags%%%%%/i", array(
                $this, '_handleTags'), $b);
                
            $b = preg_replace_callback(';%%%%%date\|([0-9]+)(\|.*?)?%%%%%;', array(
            $this, '_formatDate'), $b);
            
            $template = $b;
            //$template = preg_replace(';(%%%%%([a-z0-9\(\)_]+)%%%%%;i', '%%\\1%%', $template);
	    }
	    
	    $out = str_replace('%%%%%content%%%%%', trim($out), $template);
	    /* Handle split sources. */
	    $splitSource = preg_split('/^([=]{4,})$/m', $source);
	    for ($i = 0; $i < count($splitSource); $i++) {
	        $out = str_replace('%%%%%content{'.($i+1).'}%%%%%', trim($splitSource[$i]), $out);
	    }
	    $out = preg_replace(';%%%%%content({[0-9]+})?%%%%%;', '', $out);
	    return $out;
	}
	
	private function _formatDate($m) {
        if (isset($m[2])) {
            $format = preg_replace(';^\|;', '', $m[2]);
        } else {
            $format = '%e %b %Y, %H:%M %Z|agohover';
        }
        $dateString = '[[date ' . $m[1] . ' format="'.$format.'"' . ']]';
        return $dateString;
    }
    
	private function _handleComementsCount($m){
    	$page = $this->_tmpPage;
    	$threadId = $page->getThreadId();
    	if($threadId) {
    		$thread = DB_ForumThreadPeer::instance()->selectByPrimaryKey($threadId);
    	}
    	if($thread) {
    		return $thread->getNumberPosts();
    	}
    	return 0;
    }
    
	private function _handleTags($m) {
        $page = $this->_tmpPage;
        /* Select tags. */
        // get the tags
        $c = new Criteria();
        $c->add("page_id", $page->getPageId());
        $c->addOrderAscending("tag");
        $tags = DB_PageTagPeer::instance()->select($c);
        $t2 = array();
        foreach ($tags as $t) {
            $t2[] = $t->getTag();
        }
        if(count($t2) == 0) {
            return _('//no tags found for this page//');
        }
        return implode(' ', $t2);
    }
	
	private function _assemblyTemplateHandleListPages($m){
		if(preg_match(';^\[\[module;sm', $m[1])){
	    	return $m[0];
	    }else{
	    	$b = preg_replace(';%%(content({[0-9]+}))?%%;', "%\xFA%\\1%\xFA%", $m[0]);
	    	$b = preg_replace(';(?<!%)%%([a-z0-9\(\)_]+)%%(?!%);i', "%\xFA%\\1%\xFA%", $b);
	    	$b = preg_replace(';(?<!%)%%(date(\|.*?)?)%%(?!%);i', "%\xFA%\\1%\xFA%", $b);
	    	//$b = preg_replace(";%\xFA%(content({[0-9]+}))?%\xFA%;", "%\xFA%\\1%\xFA%", $b);
	        return $b;
	    }
	}
	
	public function setPage($page){
		
		$this->wiki->setRenderConf($this->transformationFormat, 'image', 'base', '/local--files/'.$page->getUnixName().'/');
 		$this->wiki->setRenderConf($this->transformationFormat, 'file', 'base', '/local--files/'.$page->getUnixName().'/');
 		$this->wiki->vars['pageName'] = $page->getUnixName();
 		$this->wiki->vars['pageTitle'] = $page->getTitleOrUnixName();
 		$this->wiki->vars['page'] = $page;
		$this->page = $page;
	}	
	public function setPageUnixName($pageUnixName){
		$this->wiki->setRenderConf($this->transformationFormat, 'image', 'base', '/local--files/'.$pageUnixName.'/');
		$this->wiki->setRenderConf($this->transformationFormat, 'file', 'base', '/local--files/'.$pageUnixName.'/');
		$this->wiki->vars['pageName'] = $pageUnixName;
		$this->pageUnixName = $pageUnixName;
	}	
	
	public function resetWiki(){
		// initialize wiki engine with default values
		$wiki = new Text_Wiki();
		$viewUrl = "/%s";
		$wiki->setRenderConf($this->transformationFormat, 'freelink', 'view_url', $viewUrl);
 		$wiki->setRenderConf($this->transformationFormat, 'freelink', 'new_url', $viewUrl);
 		$wiki->setRenderConf($this->transformationFormat, 'url', 'images', false);
 		
 		$wiki->setRenderConf($this->transformationFormat, 'freelink', 'new_text', '');
 		$wiki->setRenderConf($this->transformationFormat, 'freelink', 'css_new', 'newpage');
		$wiki->setRenderConf($this->transformationFormat, 'table', 'css_table', 'wiki-content-table');
 		
 		$wiki->setRenderConf($this->transformationFormat, 'freelink', 'exists_callback', 'wikiPageExists');
 		
 		$interWikis = array(
		    'wikipedia'    => 'http://en.wikipedia.org/wiki/%s',
		    'wikipedia.pl'    => 'http://pl.wikipedia.org/wiki/%s',
		    'pl.wikipedia'    => 'http://pl.wikipedia.org/wiki/%s',
		    'google'    => 'http://www.google.com/search?q=%s',
		    'dictionary' => 'http://dictionary.reference.com/browse/%s'
		);

		// configure the interwiki rule
		$wiki->setRenderConf($this->transformationFormat, 'interwiki', 'sites', $interWikis);
 		$wiki->setParseConf('interwiki', 'sites', $interWikis);
 		
 		$wiki->disableRule('wikilink');
 		$this->wiki = $wiki;	
		
	}
	
	public function setMode($mode){
		$wiki = $this->wiki;
		switch($mode){
			case 'post':
				// disable a few rules
				$wiki->disableRule("include");
				$wiki->disableRule("modulepre");
				$wiki->disableRule("module");
				$wiki->disableRule("module654");
				$wiki->disableRule("toc");
				$wiki->disableRule("Social");
				$wiki->disableRule("button");
				
				//configure
			
				$wiki->setRenderConf($this->transformationFormat, 'heading', 'use_id', false);
				$wiki->setRenderConf($this->transformationFormat, 'footnote', 'id_prefix', rand(0,1000000).'-');
				$wiki->setRenderConf($this->transformationFormat, 'bibitem', 'id_prefix', rand(0,1000000).'-');
				$wiki->setRenderConf($this->transformationFormat, 'math', 'id_prefix', rand(0,1000000).'-');
				
				$wiki->setRenderConf($this->transformationFormat, 'file', 'no_local', true);
				$wiki->setRenderConf($this->transformationFormat, 'image', 'no_local', true);
				$wiki->setRenderConf($this->transformationFormat, 'gallery', 'no_local', true);
				break;
			case 'list':
			    $wiki->setRenderConf($this->transformationFormat, 'heading', 'use_id', false);
				$wiki->setRenderConf($this->transformationFormat, 'footnote', 'id_prefix', rand(0,1000000).'-');
				$wiki->setRenderConf($this->transformationFormat, 'bibitem', 'id_prefix', rand(0,1000000).'-');
				$wiki->setRenderConf($this->transformationFormat, 'math', 'id_prefix', rand(0,1000000).'-');
				
			    break;
			case 'pm':
				// disable a few rules
				$wiki->disableRule("include");
				$wiki->disableRule("modulepre");
				$wiki->disableRule("module");
				$wiki->disableRule("module654");
				$wiki->disableRule("toc");
				$wiki->disableRule("Social");
				$wiki->disableRule("button");
				
				//configure
			
				$wiki->setRenderConf($this->transformationFormat, 'heading', 'use_id', false);
				$wiki->setRenderConf($this->transformationFormat, 'footnote', 'id_prefix', rand(0,1000000).'-');
				$wiki->setRenderConf($this->transformationFormat, 'bibitem', 'id_prefix', rand(0,1000000).'-');
				$wiki->setRenderConf($this->transformationFormat, 'math', 'id_prefix', rand(0,1000000).'-');
				
				$wiki->setRenderConf($this->transformationFormat, 'file', 'no_local', true);
				$wiki->setRenderConf($this->transformationFormat, 'image', 'no_local', true);
				$wiki->setRenderConf($this->transformationFormat, 'gallery', 'no_local', true);
				break;	
				
			case 'feed':
				// disable a few rules
				$wiki->disableRule("include");
				$wiki->disableRule("modulepre");
				$wiki->disableRule("module");
				$wiki->disableRule("module654");
				$wiki->disableRule("toc");
				
				$wiki->disableRule("footnote");
				$wiki->disableRule("math");
				$wiki->disableRule("equationreference");
				$wiki->disableRule("Footnoteitem");
				$wiki->disableRule("Footnoteblock");
				$wiki->disableRule("Bibitem");
				$wiki->disableRule("Bibliography");
				$wiki->disableRule("Bibcite");
				$wiki->disableRule("Gallery");
				$wiki->disableRule("File");
				$wiki->disableRule("Social");

				// configure
				
				$wiki->setRenderConf($this->transformationFormat, 'heading', 'use_id', false);
				
				$wiki->setRenderConf($this->transformationFormat, 'file', 'no_local', true);
				$wiki->setRenderConf($this->transformationFormat, 'image', 'no_local', true);
				$wiki->setRenderConf($this->transformationFormat, 'image', 'post_vars', true);
				$wiki->setParseConf( 'url', 'post_vars', true);

				break;
				
			case 'awiki':
				break;
			default:
				throw Exception("Invalid wiki engine mode.");	
				break;
		}	
	}
	
	public function setTransformationFormat($format){
		$this->transformationFormat = $format;
		$this->resetWiki();
	}
	
	public function processHtml($doc){
		require_once(WIKIDOT_ROOT."/lib/Text_Antiwiki/Text/Antiwiki.php");
		// just for text_wiki extend the include_path
		ini_set('include_path',ini_get('include_path').':'.WIKIDOT_ROOT.'/lib/Text_Antiwiki/');
		
		// clean the code!!!
		$doc = $this->purifyHtml($doc);
		
		// no extra parameters, just GO GO GO
		$wiki = new Text_Antiwiki();
		$out = $wiki->transform($doc, 'Wiki');
		
		return $out;
		
	}
	
}

// quick checkup if page exists

function wikiPageExists($pageName){
	
	if($GLOBALS['site'] == null){
		$runData = Ozone::getRunData();
		$siteId = $runData->getTemp("site")->getSiteId();
	}else{
		$siteId = $GLOBALS['site']->getSiteId();
	}
	$q = "SELECT page_id FROM page WHERE unix_name='".db_escape_string($pageName)."' AND site_id='".db_escape_string($siteId)."' LIMIT 1";
	$db = Database::connection();
	$r = $db->query($q);
	if($row = $r->nextRow()){
		return $row['page_id'];
	} else {
		return false;
	}	
}
