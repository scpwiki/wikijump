<?php

/**
*
* Includes the contents of another PHP script into the source text.
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
*
* @license LGPL
*
* @version $Id$
*
*/

/**
*
* This class implements a Text_Wiki_Parse to include the results of a
* script directly into the source at parse-time; thus, the output of the
* script will be parsed by Text_Wiki.  This differs from the 'embed'
* rule, which incorporates the results at render-time, meaning that the
* 'embed' content is not parsed by Text_Wiki.
*
* DANGER!
*
* This rule is inherently not secure; it allows cross-site scripting to
* occur if the embedded output has <script> or other similar tags.  Be
* careful.
*
* @category Text
*
* @package Text_Wiki
*
* @author Paul M. Jones <pmjones@php.net>
*
*/

class Text_Wiki_Parse_Include extends Text_Wiki_Parse {

    public $conf = array(
        'base' => '/path/to/scripts/'
    );

    public $file = null;

    public $output = null;

    public $vars = null;

    /**
    *
    * The regular expression used to find source text matching this
    * rule.
    *
    * @access public
    *
    * @var string
    *
    */

    public $regex = 	'/' . 
        				'^\[\[include\s' .          # Declare include module
        				'([a-zA-Z0-9\s\-:]+?)' .    # Name or location of component
        				'(\s+.*?)?' .               # Parameters
        				'(?:\]\])$' .               # Match but not capture closing tags? Ok
        				'/imsx';

 	function parse(){
 		$level = 0;
 		do{
 			$oldSource = $this->wiki->source;
        	$this->wiki->source = preg_replace_callback(
                $this->regex, array(&$this, 'process'), $this->wiki->source);
        	$level++;
 		}while($oldSource != $this->wiki->source && $level<5);

 	}

    function process(&$matches)
    {

     	$pageName =  WDStringUtils::toUnixName(trim($matches[1]));

     	// get page source (if exists)

     	$runData = Ozone::getRunData();
     	$site = $runData->getTemp("site");

    		$page = DB\PagePeer::instance()->selectByName($site->getSiteId(), $pageName);

    		if($page == null){
    			//$output =  $this->wiki->addToken(
            	//	$this->rule, array('fromIncludeRule' => true, 'type' => 'error', 'pageName' => $pageName)
        		$output = "\n\n".'[[div class="error-block"]]'."\n".sprintf(_('Page to be included %s can not be found!'),htmlspecialchars($pageName))."\n".'[[/div]]'."\n\n";

        		$wiki = $this->wiki;
        		if($wiki->vars['inclusionsNotExist'] == null){
					$wiki->vars['inclusionsNotExist'] = array();
				}
				$wiki->vars['inclusionsNotExist'][$pageName] = $pageName;
    		}else {

    			$output = $page->getSource();

    			// prepare entry...
    			$wiki = $this->wiki;
    			if($wiki->vars['inclusions'] == null){
					$wiki->vars['inclusions'] = array();
				}
				$wiki->vars['inclusions'][$page->getPageId()] = $page->getPageId();

    			// preprocess the output too!!!
    			// missed a few rules so far... TODO!!!

    			//process the output - make substitutions.

    			$subs = $matches[2];
    			if($subs){
    				$subsArray = explode('|', $subs);
    				foreach($subsArray as $sub){
    					if(strpos($sub, '=') !== false){
    						$pos = strpos($sub,'=');
    						$var = trim(substr($sub, 0, $pos));
    						$value = trim(substr($sub, $pos+1));
    						if($value!='' && $var != '' && preg_match('/^[a-z0-9\-\_]+$/i', $var)){
    							// substitute!!!
    							$output = str_replace('{$'.$var.'}', $value, $output);
    						}
    					}
    				}
    			}

    		}
        // done, place the script output directly in the source
        return "\n\n".$output."\n\n";
    }
}
