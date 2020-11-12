<?php
/**
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 * @license LGPL
 *
 * @version $Id$
 *
 */

/**
 *
 * Creates a conditional, tag-based block.
 *
 * @category Text
 *
 * @package Text_Wiki
 *
 * @author Michal Frackowiak
 *
 */
class Text_Wiki_Parse_Iftags extends Text_Wiki_Parse {


    public $regex = 	'/' . 
        				'\[\[iftags(\s[^\]]*)?\]\]' .  		# Opening iftags tag including parameters
        				'(' . 
        				'(?:(?R)|.)' . 	            		# Contents of the page, including other iftags
        				'*?)' .                        		# Non-greedy match to claim next closing tag
        				'\[\[\/iftags\]\]' .            	# Closing tag
        				'/msix';
    # Note regarding non-greedy match: I'm not sure how a recursive regex match
    # works entirely, so I've written that it claims the next closing tag.
    # That's unlikely to be true and definitely not intended behaviour.
    # Because the ?R is before the . in the OR group, it may be that it
    # attempts to correctly assign nested iftags modules.

    /**
    *
    * Generates a token entry for the matched text.  Token options are:
    *
    * 'text' => The full matched text, not including the <code></code> tags.
    *
    * @access public
    *
    * @param array &$matches The array of matches from parse().
    *
    * @return A delimited token number to be used as a placeholder in
    * the source text.
    *
    */

    function process(&$matches)
    {
    	$page = $this->wiki->vars['page'];
    	if(!$page){
    		$pageName = $this->wiki->vars['pageName'];
	    	$site = $GLOBALS['site'];
	    	$page = DB\PagePeer::instance()->selectByName($site->getSiteId(), $pageName);
    	}
    	if(!$page) {
    		return;
    	}

    	$tag0 = $tags0[0];
    	$tags = $page->getTagsAsArray();

    	$tags0 = preg_split('/[, ]+/', trim($matches[1]));

    	$allTags = array();
    	$noTags = array();
    	$anyTags = array();

    	foreach($tags0 as $t){
    		if (substr($t, 0, 1) == '+') {
               $allTags[] = substr($t, 1);
            } elseif (substr($t, 0, 1) == '-') {
               $noTags[] = substr($t, 1);
            } else {
            	$anyTags[] = $t;
            }
    	}

    	if(count($allTags) > 0){
    		foreach($allTags as $t){
    			/* If any of the required tags is not present, return ''. */
    			if(!in_array($t, $tags)){
    				return '';
    			}
    		}
    	}
   		if(count($noTags) > 0){
    		foreach($noTags as $t){
    			/* If any of the forbidden tags is present, return ''. */
    			if(in_array($t, $tags)){
    				return '';
    			}
    		}
    	}

    	if(count($anyTags) > 0){
    		foreach($anyTags as $t){
    			/* If any of the "any" tags is present, return the content. */
    			if(in_array($t, $tags)){
    				return $matches[2];
    			}
    		}
    		/* If not, return ''. */
    		return '';
    	}

    	/* If we are here, the content should be returned. */
    	return $matches[2];
    }

}
