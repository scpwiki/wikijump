<?php

/**
*
*
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
* This class implements a Text_Wiki_Parse to find math invocation code,
* i.e. [[footnote]] text [[/footnote]]
*
* @category Text
*
* @package Text_Wiki
*
* @author Michal Frackowiak
*
*/

class Text_Wiki_Parse_Footnote extends Text_Wiki_Parse {

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

    public $regex =     '/' . 
                        '\s*' . 
                        '\[\[footnote\]\]' . 
                        '(.*?)' . 
                        '\[\[\/footnote\]\]' . 
                        '/sx';

    function process(&$matches)
    {
    		if($this->wiki->vars['footnotes'] == null){
    			$this->wiki->vars['footnotes'] = array();
    		}
    	 	$id = count($this->wiki->vars['footnotes'])+1;
   	 	//echo "dup[a]"

	    	$content = trim($matches[1]);

	    $this->wiki->vars['footnotes'][$id-1] = $content;
        $options = array('id'=>$id);

        return $this->wiki->addToken($this->rule, $options);
    }
}
