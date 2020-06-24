<?php

/**
*
* Parses for wiki freelink text.
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
* @author Michal Frackowiak
*
* @license LGPL
*
* @version $Id$
*
*/

/**
*
* Parses for freelinked page links.
*
* This class implements a Text_Wiki_Parse to find source text marked as a
* wiki freelink, and automatically create a link to that page.
*
* A freelink is any page name not conforming to the standard
* StudlyCapsStyle for a wiki page name.  For example, a page normally
* named MyHomePage can be renamed and referred to as ((My Home Page)) --
* note the spaces in the page name.  You can also make a "nice-looking"
* link without renaming the target page; e.g., ((MyHomePage|My Home
* Page)).  Finally, you can use named anchors on the target page:
* ((MyHomePage|My Home Page#Section1)).
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
* @author Michal Frackowiak
*
*/

class Text_Wiki_Parse_Freelink extends Text_Wiki_Parse {

    public $regex =     '/' . 
                        '\[\[\[' .                  # Opening brackets
                        '([^\[\]\|\#]+)' .          # Target page name
                        '\s*' . 
                        '(' .                       # Match a URL fragment after the page name
                        '\#' . 
                        '[A-Za-z]' .                # Require that first letter is alphanumeric
                        '[-A-Za-z0-9_:.]*' . 
                        ')?' .                      # URL fragment is optional
                        '\s*' . 
                        '(' . 
                        '\|' .                      # Pipe to delimit link text
                        '[^\]\|\[\#]*' .            # Link text
                        ')?' .                      # Link text is optional
                        '()' .                      # Are you kidding me
                        '\]\]\]' .                  # Closing brackets
                        '/x';

    /**
    *
    * Generates a replacement for the matched text.  Token options are:
    *
    * 'page' => the wiki page name (e.g., HomePage).
    *
    * 'text' => alternative text to be displayed in place of the wiki
    * page name.
    *
    * 'anchor' => a named anchor on the target wiki page
    *
    * @access public
    *
    * @param array &$matches The array of matches from parse().
    *
    * @return A delimited token to be used as a placeholder in
    * the source text, plus any text priot to the match.
    *
    */

    function process(&$matches)
    {
        // use nice variable names
        $page = $matches[1];
        $text = $matches[3];
        $anchor = $matches[2];

        if($page[0] == '_'){
        		$page = substr($page, 1);
        		$nonbr = true;
        }
        // check if references to another site too.
        $site = null;

        if(strpos($page, '::')){
            $site = substr($page,0,strpos($page, '::'));
            $site = WDStringUtils::toUnixName($site);
            $page = substr($page,strpos($page, '::')+2);
            if(!$page){
                $page = $site;
            }
        }

        // is the page given a new text appearance?
        if (trim($text) == '') {
            // no
            $text = $page;
            if(strpos($text, ':') != false){
				$text = substr($text, strpos($text, ':')+1);
        	}
        }elseif(trim($text) == '|'){
        	// get $text from the page title (if exists)
        	$textFromTitle = true;

        } else {
            // yes, strip the leading | character
            $text = substr($text, 1);
        }



        // MF: 'purify' the page name
        $page = WDStringUtils::toUnixName($page);

        // set the options
        $options = array(
            'site'	=> $site,
            'page'   => $page,
            'text'   => $text,
            'anchor' => $anchor,
            'textFromTitle' => $textFromTitle
        );
        if($nonbr){
        		$options['nonbr']=true;
        }

        // return a token placeholder
        return $this->wiki->addToken($this->rule, $options);
    }
}
